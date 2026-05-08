"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerRedisStreamHandlers = registerRedisStreamHandlers;
exports.getStreamListeners = getStreamListeners;
exports.getNextStreamListenerId = getNextStreamListenerId;
const electron_1 = require("electron");
const encryption_manager_1 = require("./encryption-manager");
const dbManager = require('./services/db-manager').default || require('./services/db-manager');
// Track active stream message listeners for real-time updates
const streamListeners = new Map();
let streamListenerCounter = 0;
// Helper for Redis stream IPC handlers — creates temp client with correct DB index
// to avoid race conditions on the shared persistent connection
async function handleRedisStreamOp(id, dbIndex, fn) {
    try {
        const config = dbManager.getConfig(id);
        if (!config)
            throw new Error(`Connection '${id}' not found`);
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        const { Redis: RedisClass } = await Promise.resolve().then(() => __importStar(require('ioredis')));
        const client = new RedisClass({
            host: config.host,
            port: config.port,
            password: config.password ? (0, encryption_manager_1.decryptPassword)(config.password) : undefined,
            db: dbIndex ?? config.dbIndex ?? 0,
            connectTimeout: 5000,
            retryStrategy: () => null,
        });
        await client.ping();
        try {
            const result = await fn(client);
            return { success: true, ...result };
        }
        finally {
            try {
                client.quit();
            }
            catch { }
        }
    }
    catch (error) {
        return { success: false, error: error.message };
    }
}
function registerRedisStreamHandlers() {
    // Get all Stream-type keys (Incremental scan with state management)
    electron_1.ipcMain.handle('db:redis-streams', async (_event, id, dbIndex, pattern = '*', loadMore = false) => {
        try {
            const result = await dbManager.getRedisStreamsIncremental(id, dbIndex, pattern, loadMore);
            return { success: true, streams: result.streams, hasMore: result.hasMore };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // Get stream info + consumer groups
    electron_1.ipcMain.handle('db:redis-stream-info', async (_event, id, dbIndex, key) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const info = await client.xinfo('STREAM', key);
            const groups = await client.xinfo('GROUPS', key);
            const result = {};
            for (let i = 0; i < info.length; i += 2)
                result[info[i]] = info[i + 1];
            const groupList = groups.map((g) => {
                const obj = {};
                for (let i = 0; i < g.length; i += 2)
                    obj[g[i]] = g[i + 1];
                return obj;
            });
            return { info: result, groups: groupList };
        });
    });
    // Get consumers for a group
    electron_1.ipcMain.handle('db:redis-stream-consumers', async (_event, id, dbIndex, key, group) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const consumers = await client.xinfo('CONSUMERS', key, group);
            const list = consumers.map((c) => {
                const obj = {};
                for (let i = 0; i < c.length; i += 2)
                    obj[c[i]] = c[i + 1];
                return obj;
            });
            return { consumers: list };
        });
    });
    // Get stream messages (XRANGE)
    electron_1.ipcMain.handle('db:redis-stream-messages', async (_event, id, dbIndex, key, start = '-', end = '+', count = 100) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const entries = await client.xrange(key, start, end, 'COUNT', count);
            const messages = entries.map((e) => {
                const fields = {};
                for (let i = 0; i < e[1].length; i += 2)
                    fields[e[1][i]] = e[1][i + 1];
                return { id: e[0], fields };
            });
            return { messages };
        });
    });
    // Add message (XADD)
    electron_1.ipcMain.handle('db:redis-stream-add', async (_event, id, dbIndex, key, fields, maxlen) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const args = [];
            if (maxlen)
                args.push('MAXLEN', '~', maxlen);
            args.push('*');
            for (const [k, v] of Object.entries(fields))
                args.push(k, v);
            const msgId = await client.xadd(key, ...args);
            return { id: msgId };
        });
    });
    // Delete message (XDEL)
    electron_1.ipcMain.handle('db:redis-stream-del', async (_event, id, dbIndex, key, messageId) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const count = await client.xdel(key, messageId);
            return { deleted: count };
        });
    });
    // Create consumer group (XGROUP CREATE)
    electron_1.ipcMain.handle('db:redis-stream-group-create', async (_event, id, dbIndex, key, group, startId = '0') => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            await client.xgroup('CREATE', key, group, startId, 'MKSTREAM');
            return {};
        });
    });
    // Destroy consumer group (XGROUP DESTROY)
    electron_1.ipcMain.handle('db:redis-stream-group-destroy', async (_event, id, dbIndex, key, group) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            await client.xgroup('DESTROY', key, group);
            return {};
        });
    });
    // Get pending messages (XPENDING)
    electron_1.ipcMain.handle('db:redis-stream-pending', async (_event, id, dbIndex, key, group, startId = '-', endId = '+', count = 100) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const pending = await client.xpending(key, group, startId, endId, count);
            return { pending };
        });
    });
    // Claim pending message (XCLAIM)
    electron_1.ipcMain.handle('db:redis-stream-claim', async (_event, id, dbIndex, key, group, consumer, messageId, minIdleTime = 300000) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const result = await client.xclaim(key, group, consumer, minIdleTime, messageId);
            return { claimed: result };
        });
    });
    // Ack message (XACK)
    electron_1.ipcMain.handle('db:redis-stream-ack', async (_event, id, dbIndex, key, group, messageId) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const count = await client.xack(key, group, messageId);
            return { acked: count };
        });
    });
    // Delete entire stream (DEL key)
    electron_1.ipcMain.handle('db:redis-stream-delete', async (_event, id, dbIndex, key) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            await client.del(key);
            return {};
        });
    });
    // ZSet range query (for delay queues)
    electron_1.ipcMain.handle('db:redis-zset-range', async (_event, id, dbIndex, key, minScore, maxScore, count) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const entries = await client.zrangebyscore(key, minScore, maxScore, 'WITHSCORES', 'LIMIT', 0, count);
            const result = [];
            for (let i = 0; i < entries.length; i += 2)
                result.push({ value: entries[i], score: parseFloat(entries[i + 1]) });
            return { entries: result };
        });
    });
    // ZSet remove member
    electron_1.ipcMain.handle('db:redis-zset-remove', async (_event, id, dbIndex, key, value) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const removed = await client.zrem(key, value);
            return { removed: removed > 0 };
        });
    });
    // ZSet add member (for manually adding delay messages)
    electron_1.ipcMain.handle('db:redis-zset-add', async (_event, id, dbIndex, key, value, score) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            await client.zadd(key, score, value);
            return { success: true };
        });
    });
    // XTRIM queue cleanup
    electron_1.ipcMain.handle('db:redis-stream-trim', async (_event, id, dbIndex, key, maxLen) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const trimmed = await client.xtrim(key, 'MAXLEN', '~', maxLen);
            return { trimmed };
        });
    });
    // Retry pending message (read → XACK → XADD new)
    electron_1.ipcMain.handle('db:redis-stream-retry', async (_event, id, dbIndex, key, group, messageId) => {
        return handleRedisStreamOp(id, dbIndex, async (client) => {
            const entries = await client.xrange(key, messageId, messageId);
            if (!entries || entries.length === 0)
                throw new Error('Message not found');
            const entry = entries[0];
            const fields = {};
            for (let i = 0; i < entry[1].length; i += 2)
                fields[entry[1][i]] = entry[1][i + 1];
            await client.xack(key, group, messageId);
            const args = ['*'];
            for (const [k, v] of Object.entries(fields))
                args.push(k, v);
            const newId = await client.xadd(key, ...args);
            return { success: true, newMessageId: newId };
        });
    });
}
function getStreamListeners() { return streamListeners; }
function getNextStreamListenerId() { return ++streamListenerCounter; }
//# sourceMappingURL=redis-stream-manager.js.map