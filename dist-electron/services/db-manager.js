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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const logger_1 = require("../logger");
const promise_1 = __importDefault(require("mysql2/promise"));
const pg_1 = require("pg");
const ioredis_1 = __importDefault(require("ioredis"));
class DBManager {
    constructor() {
        this.connections = new Map();
        this.scanStates = new Map();
        this.streamScanStates = new Map();
        // Subscribe to a topic (returns a subscription ID for later unsubscribe)
        this.topicSubscriptions = new Map();
    }
    async connect(id, config) {
        if (this.connections.has(id)) {
            await this.disconnect(id);
        }
        // Password is already decrypted at the storage layer (settings:get / getDBConnections)
        const client = await this.createClient(config);
        this.connections.set(id, { client, config });
    }
    async disconnect(id) {
        const entry = this.connections.get(id);
        if (!entry)
            return;
        try {
            const { client, config } = entry;
            switch (config.type) {
                case 'mysql':
                    await client.end();
                    break;
                case 'postgresql':
                    await client.end();
                    break;
                case 'redis':
                    client.quit();
                    break;
                case 'sqlite':
                    if (client.close)
                        client.close();
                    break;
            }
        }
        catch (e) {
            console.error(`Error disconnecting ${id}:`, e);
        }
        finally {
            this.connections.delete(id);
        }
    }
    async query(id, sql) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        switch (config.type) {
            case 'mysql': {
                // Use .query() not .execute() — typeCast is only applied to .query()
                const [rows] = await client.query(sql);
                return Array.isArray(rows) ? rows : [];
            }
            case 'postgresql': {
                const res = await client.query(sql);
                return res.rows;
            }
            case 'sqlite': {
                const stmt = client.prepare(sql);
                if (stmt.reader)
                    return stmt.all();
                return [];
            }
            case 'redis':
                throw new Error('Use dbRedisKeys/dbRedisGet for Redis operations');
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
    }
    async getTables(id, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        switch (config.type) {
            case 'mysql': {
                if (dbName) {
                    const [rows] = await client.query(`SELECT TABLE_NAME FROM information_schema.tables WHERE TABLE_SCHEMA = ? AND TABLE_TYPE = 'BASE TABLE'`, [dbName]);
                    return rows.map((r) => r.TABLE_NAME);
                }
                const [rows] = await client.query('SHOW TABLES');
                return rows.map((r) => Object.values(r)[0]);
            }
            case 'postgresql': {
                const schema = dbName || 'public';
                const res = await client.query(`SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE'`, [schema]);
                return res.rows.map((r) => r.table_name);
            }
            case 'sqlite': {
                const rows = client.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'").all();
                return rows.map((r) => r.name);
            }
            case 'redis':
                return [];
            default:
                return [];
        }
    }
    async getDatabases(id) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const SYSTEM_DB_MYSQL = ['information_schema', 'performance_schema', 'mysql', 'sys'];
        const SYSTEM_SCHEMAS_PG = ['information_schema', 'pg_catalog', 'pg_toast'];
        switch (config.type) {
            case 'mysql': {
                const [rows] = await client.query('SHOW DATABASES');
                return rows
                    .map((r) => Object.values(r)[0])
                    .filter((db) => !SYSTEM_DB_MYSQL.includes(db));
            }
            case 'postgresql': {
                const res = await client.query(`SELECT schema_name FROM information_schema.schemata 
           WHERE schema_name NOT IN ($1, $2, $3) AND schema_name NOT LIKE 'pg_%' 
           ORDER BY schema_name`, SYSTEM_SCHEMAS_PG);
                return res.rows.map((r) => r.schema_name);
            }
            case 'sqlite': {
                // SQLite: return the file path as the single "database"
                return [config.path || 'main'];
            }
            case 'redis':
                return [];
            default:
                return [];
        }
    }
    async getViews(id, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        switch (config.type) {
            case 'mysql': {
                const [rows] = await client.query('SELECT TABLE_NAME FROM information_schema.views WHERE TABLE_SCHEMA = ?', [dbName]);
                return rows.map((r) => r.TABLE_NAME);
            }
            case 'postgresql': {
                const res = await client.query(`SELECT table_name FROM information_schema.views WHERE table_schema = $1`, [dbName]);
                return res.rows.map((r) => r.table_name);
            }
            case 'sqlite': {
                const rows = client.prepare("SELECT name FROM sqlite_master WHERE type='view'").all();
                return rows.map((r) => r.name);
            }
            case 'redis':
                return [];
            default:
                return [];
        }
    }
    async getTableData(id, table, limit, offset, dbName, orderBy, orderDir) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        // Sanitize orderBy to prevent SQL injection
        const safeOrderBy = orderBy && /^[a-zA-Z_][a-zA-Z0-9_]*$/.test(orderBy) ? this.quoteIdentifier(orderBy, config.type) : null;
        const safeDir = orderDir === 'desc' ? 'DESC' : 'ASC';
        const orderClause = safeOrderBy ? ` ORDER BY ${safeOrderBy} ${safeDir}` : '';
        switch (config.type) {
            case 'mysql': {
                const escDb = dbName ? dbName.replace(/`/g, '``') : '';
                const escTable = table.replace(/`/g, '``');
                const tableRef = dbName ? `\`${escDb}\`.\`${escTable}\`` : `\`${escTable}\``;
                const [countRes] = await client.query(`SELECT COUNT(*) as total FROM ${tableRef}`);
                const total = countRes[0]?.total ?? 0;
                const [rows] = await client.query(`SELECT * FROM ${tableRef}${orderClause} LIMIT ? OFFSET ?`, [limit, offset]);
                return { rows, total };
            }
            case 'postgresql': {
                const schema = dbName || 'public';
                const escSchema = schema.replace(/"/g, '""');
                const escTable = table.replace(/"/g, '""');
                const tableRef = `"${escSchema}"."${escTable}"`;
                const countRes = await client.query(`SELECT COUNT(*) as total FROM ${tableRef}`);
                const total = parseInt(countRes.rows[0]?.total, 10) || 0;
                const res = await client.query(`SELECT * FROM ${tableRef}${orderClause} LIMIT $1 OFFSET $2`, [limit, offset]);
                return { rows: res.rows, total };
            }
            case 'sqlite': {
                const escTable = table.replace(/"/g, '""');
                const totalRes = client.prepare(`SELECT COUNT(*) as total FROM "${escTable}"`).get();
                const total = totalRes?.total ?? 0;
                const rows = client.prepare(`SELECT * FROM "${escTable}"${orderClause} LIMIT ? OFFSET ?`).all(limit, offset);
                return { rows, total };
            }
            case 'redis':
                return { rows: [], total: 0 };
            default:
                return { rows: [], total: 0 };
        }
    }
    async testConnection(config) {
        let client;
        try {
            client = await this.createClient(config);
            switch (config.type) {
                case 'mysql':
                    await client.ping();
                    await client.end();
                    break;
                case 'postgresql':
                    await client.end();
                    break;
                case 'redis':
                    await client.ping();
                    client.quit();
                    break;
                case 'sqlite':
                    if (client.close)
                        client.close();
                    break;
            }
            return { success: true };
        }
        catch (error) {
            // Ensure cleanup on failure
            try {
                if (client) {
                    switch (config.type) {
                        case 'mysql':
                            await client.end();
                            break;
                        case 'postgresql':
                            await client.end();
                            break;
                        case 'redis':
                            client.quit();
                            break;
                        case 'sqlite':
                            if (client.close)
                                client.close();
                            break;
                    }
                }
            }
            catch { /* ignore cleanup errors */ }
            return { success: false, error: error.message };
        }
    }
    async getRedisKeys(id, pattern, count = 5000) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('getRedisKeys is only available for Redis connections');
        }
        const redisClient = client;
        const result = await redisClient.scan('0', 'MATCH', pattern || '*', 'COUNT', count);
        return { cursor: result[0], keys: result[1] };
    }
    async getRedisKeyInfo(id, key) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('getRedisKeyInfo is only available for Redis connections');
        }
        const redisClient = client;
        const type = await redisClient.type(key);
        const ttl = await redisClient.ttl(key);
        let length = 0;
        switch (type) {
            case 'string': {
                const val = await redisClient.get(key);
                length = val ? val.length : 0;
                break;
            }
            case 'hash':
                length = await redisClient.hlen(key);
                break;
            case 'list':
                length = await redisClient.llen(key);
                break;
            case 'set':
                length = await redisClient.scard(key);
                break;
            case 'zset':
                length = await redisClient.zcard(key);
                break;
        }
        return { type, ttl, length };
    }
    async getRedisKeyValue(id, key) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('getRedisKeyValue is only available for Redis connections');
        }
        const redisClient = client;
        const type = await redisClient.type(key);
        switch (type) {
            case 'string': {
                const val = await redisClient.get(key);
                // Try to parse JSON
                try {
                    return val !== null ? JSON.parse(val) : val;
                }
                catch {
                    return val;
                }
            }
            case 'hash': {
                const pairs = await redisClient.hgetall(key);
                return pairs;
            }
            case 'list': {
                const items = await redisClient.lrange(key, 0, -1);
                return items;
            }
            case 'set': {
                const members = await redisClient.smembers(key);
                return members;
            }
            case 'zset': {
                const members = await redisClient.zrange(key, 0, -1, 'WITHSCORES');
                const result = [];
                for (let i = 0; i < members.length; i += 2) {
                    result.push({ member: members[i], score: parseFloat(members[i + 1]) });
                }
                return result;
            }
            case 'stream': {
                const entries = await redisClient.xrange(key, '-', '+');
                return entries.map((e) => {
                    const fields = {};
                    for (let i = 0; i < e[1].length; i += 2) {
                        fields[e[1][i]] = e[1][i + 1];
                    }
                    return { id: e[0], fields };
                });
            }
            case 'none':
                return null;
            default:
                throw new Error(`Unsupported Redis type: ${type}`);
        }
    }
    async setRedisKey(id, key, type, value) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('setRedisKey is only available for Redis connections');
        }
        const redisClient = client;
        switch (type.toLowerCase()) {
            case 'string': {
                const strVal = typeof value === 'string' ? value : JSON.stringify(value);
                await redisClient.set(key, strVal);
                return true;
            }
            case 'hash': {
                if (typeof value !== 'object' || value === null || Array.isArray(value)) {
                    throw new Error('Hash value must be an object');
                }
                // Replace entire hash
                await redisClient.del(key);
                const fields = [];
                for (const [field, val] of Object.entries(value)) {
                    fields.push(field, typeof val === 'string' ? val : JSON.stringify(val));
                }
                if (fields.length > 0) {
                    await redisClient.hset(key, ...fields);
                }
                return true;
            }
            case 'list': {
                if (!Array.isArray(value)) {
                    throw new Error('List value must be an array');
                }
                await redisClient.del(key);
                if (value.length > 0) {
                    const strValues = value.map(v => typeof v === 'string' ? v : JSON.stringify(v));
                    await redisClient.rpush(key, ...strValues);
                }
                return true;
            }
            case 'set': {
                if (!Array.isArray(value)) {
                    throw new Error('Set value must be an array');
                }
                await redisClient.del(key);
                if (value.length > 0) {
                    const strValues = value.map(v => typeof v === 'string' ? v : JSON.stringify(v));
                    await redisClient.sadd(key, ...strValues);
                }
                return true;
            }
            case 'zset': {
                if (!Array.isArray(value)) {
                    throw new Error('ZSet value must be an array of { member, score }');
                }
                await redisClient.del(key);
                if (value.length > 0) {
                    const entries = [];
                    for (const item of value) {
                        entries.push(item.score, typeof item.member === 'string' ? item.member : JSON.stringify(item.member));
                    }
                    await redisClient.zadd(key, ...entries);
                }
                return true;
            }
            default:
                throw new Error(`Unsupported Redis type for set: ${type}`);
        }
    }
    async addRedisKey(id, key, type, value) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('addRedisKey is only available for Redis connections');
        }
        const redisClient = client;
        switch (type.toLowerCase()) {
            case 'string': {
                const strVal = typeof value === 'string' ? value : (value ? JSON.stringify(value) : '');
                await redisClient.set(key, strVal);
                return true;
            }
            case 'hash': {
                await redisClient.hset(key, String(value?.field ?? ''), String(value?.value ?? ''));
                return true;
            }
            case 'list': {
                await redisClient.rpush(key, String(value ?? ''));
                return true;
            }
            case 'set': {
                await redisClient.sadd(key, String(value ?? ''));
                return true;
            }
            case 'zset': {
                await redisClient.zadd(key, value?.score ?? 0, String(value?.member ?? ''));
                return true;
            }
            default:
                throw new Error(`Unsupported Redis type for add: ${type}`);
        }
    }
    async deleteRedisKey(id, key) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('deleteRedisKey is only available for Redis connections');
        }
        const redisClient = client;
        const result = await redisClient.del(key);
        return result > 0;
    }
    async execRedisCommand(id, command) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('execRedisCommand is only available for Redis connections');
        }
        const redisClient = client;
        const parts = [];
        let current = '';
        let inQuotes = false;
        for (const char of command.trim()) {
            if (char === '"') {
                inQuotes = !inQuotes;
                continue;
            }
            if (char === ' ' && !inQuotes) {
                if (current) {
                    parts.push(current);
                    current = '';
                }
            }
            else {
                current += char;
            }
        }
        if (current)
            parts.push(current);
        const cmd = parts[0].toUpperCase();
        const args = parts.slice(1);
        // SECURITY: Block dangerous Redis commands
        const dangerousCommands = ['FLUSHALL', 'FLUSHDB', 'SHUTDOWN', 'DEBUG', 'SLAVEOF', 'REPLCONF', 'CONFIG SET', 'MODULE', 'SCRIPT'];
        if (dangerousCommands.includes(cmd)) {
            throw new Error(`Blocked dangerous Redis command: ${cmd}`);
        }
        // Allowlist for read-only and safe commands
        // NOTE: EVAL/EVALSHA excluded — arbitrary Lua can bypass dangerous command blocklist
        const allowedCommands = ['GET', 'KEYS', 'SCAN', 'TYPE', 'TTL', 'HLEN', 'LLEN', 'SMEMBERS', 'ZRANGE', 'INFO', 'DBSIZE', 'PING', 'ECHO', 'HGET', 'HGETALL', 'LRANGE', 'SCARD', 'ZCARD', 'HVALS', 'HKEYS', 'EXISTS', 'STRLEN', 'PTTL', 'DUMP', 'OBJECT', 'MEMORY', 'SLOWLOG', 'CLIENT', 'CLUSTER', 'SELECT', 'SET', 'HSET', 'LPUSH', 'RPUSH', 'SADD', 'ZADD', 'DEL', 'EXPIRE', 'PEXPIRE', 'RENAME', 'MIGRATE', 'MOVE', 'RESTORE', 'SETNX', 'SETEX', 'PSETEX', 'APPEND', 'INCR', 'DECR', 'INCRBY', 'DECRBY', 'HINCRBY', 'ZINCRBY', 'LTRIM', 'LPOP', 'RPOP', 'SREM', 'ZREM', 'HDEL', 'BRPOP', 'BLPOP', 'SUBSCRIBE', 'PUBLISH', 'UNSUBSCRIBE', 'CONFIG GET'];
        if (!allowedCommands.includes(cmd)) {
            throw new Error(`Redis command not allowed: ${cmd}. Use read-only or safe write commands only.`);
        }
        // Call the Redis command dynamically
        const method = redisClient[cmd.toLowerCase()];
        if (typeof method === 'function') {
            return await method.apply(redisClient, args);
        }
        throw new Error(`Unknown Redis command: ${cmd}`);
    }
    async getRedisDatabases(id) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('getRedisDatabases is only available for Redis connections');
        }
        const redisClient = client;
        // 1. Get total database count from CONFIG
        let totalDbs = 16; // default
        try {
            const configResult = await redisClient.config('GET', 'databases');
            // ioredis returns: ['databases', '16'] (flat array) or { databases: '16' } (record)
            if (Array.isArray(configResult)) {
                // Find the value after 'databases' key
                const idx = configResult.indexOf('databases');
                if (idx >= 0 && idx + 1 < configResult.length) {
                    totalDbs = parseInt(configResult[idx + 1], 10) || 16;
                }
            }
            else if (typeof configResult === 'object' && configResult !== null) {
                const val = Object.values(configResult)[0];
                totalDbs = parseInt(String(val), 10) || 16;
            }
        }
        catch {
            // fallback to default 16
        }
        // 2. Parse INFO keyspace for databases that have keys (doesn't require SELECT)
        const info = await redisClient.info('keyspace');
        const lines = info.split('\n');
        const dbKeysMap = new Map();
        for (const line of lines) {
            const match = line.match(/^db(\d+):keys=(\d+)/);
            if (match) {
                dbKeysMap.set(parseInt(match[1], 10), parseInt(match[2], 10));
            }
        }
        // 3. Build full list: all databases 0..totalDbs-1, with actual key counts
        const databases = [];
        // Create a temp client for DBSIZE calls to avoid SELECT race conditions on the shared client
        const { Redis: RedisClass } = await Promise.resolve().then(() => __importStar(require('ioredis')));
        const tempClient = new RedisClass({
            host: config.host,
            port: config.port,
            password: config.password,
            db: 0,
            connectTimeout: 5000,
            retryStrategy: () => null,
        });
        try {
            for (let i = 0; i < totalDbs; i++) {
                if (dbKeysMap.has(i)) {
                    databases.push({ db: i, keys: dbKeysMap.get(i) });
                }
                else {
                    // DBSIZE on temp client (safe, no shared SELECT)
                    try {
                        await tempClient.select(i);
                        const size = await tempClient.dbsize();
                        databases.push({ db: i, keys: size });
                    }
                    catch {
                        databases.push({ db: i, keys: 0 });
                    }
                }
            }
        }
        finally {
            tempClient.disconnect();
        }
        // Fallback: if still empty (cluster mode?), just return db0
        if (databases.length === 0) {
            try {
                const size = await redisClient.dbsize();
                databases.push({ db: 0, keys: size });
            }
            catch { }
        }
        return databases;
    }
    async getRedisKeysByType(id, dbIndex, pattern = '*', maxKeysPerType = 200) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { config } = entry;
        if (config.type !== 'redis') {
            throw new Error('getRedisKeysByType is only available for Redis connections');
        }
        (0, logger_1.info)(`[RedisManager][db-manager] getRedisKeysByType: id=${id}, dbIndex=${dbIndex}, pattern=${pattern}`);
        // Use a temp client with explicit dbIndex to avoid SELECT race conditions on shared connection
        const { Redis: RedisClass } = await Promise.resolve().then(() => __importStar(require('ioredis')));
        const tempClient = new RedisClass({
            host: config.host,
            port: config.port,
            password: config.password,
            db: dbIndex,
            connectTimeout: 5000,
            retryStrategy: () => null,
        });
        try {
            const keysByType = {
                string: [],
                hash: [],
                list: [],
                set: [],
                zset: [],
            };
            let cursor = '0';
            let totalScanned = 0;
            const maxTotalScan = maxKeysPerType * 5; // Safety limit
            do {
                const result = await tempClient.scan(cursor, 'MATCH', pattern, 'COUNT', 5000);
                cursor = result[0];
                const keys = result[1];
                (0, logger_1.info)(`[RedisManager][db-manager] SCAN: cursor=${cursor}, keysFound=${keys?.length || 0}`);
                for (const key of keys) {
                    totalScanned++;
                    if (totalScanned > maxTotalScan)
                        break;
                    const type = await tempClient.type(key);
                    if (keysByType[type] && keysByType[type].length < maxKeysPerType) {
                        keysByType[type].push(key);
                    }
                }
            } while (cursor !== '0' && totalScanned <= maxTotalScan);
            (0, logger_1.info)(`[RedisManager][db-manager] getRedisKeysByType result: totalScanned=${totalScanned}, types=${JSON.stringify(Object.fromEntries(Object.entries(keysByType).map(([k, v]) => [k, v.length])))}`);
            return keysByType;
        }
        finally {
            try {
                await tempClient.quit();
            }
            catch { }
        }
    }
    async getRedisKeysTree(id, dbIndex, prefix = '', loadMore = false) {
        const stateKey = `${id}:${dbIndex}:${prefix}`;
        (0, logger_1.info)(`[getRedisKeysTree] stateKey=${stateKey}, loadMore=${loadMore}`);
        if (!loadMore) {
            this.scanStates.delete(stateKey);
        }
        let state = this.scanStates.get(stateKey);
        if (!state) {
            (0, logger_1.info)(`[getRedisKeysTree] Creating new scan state for ${stateKey}`);
            state = {
                cursor: '0',
                folders: new Map(),
                leaves: new Set(),
                finished: false,
                timer: null
            };
            this.scanStates.set(stateKey, state);
        }
        if (state.finished) {
            (0, logger_1.info)(`[getRedisKeysTree] Already finished, returning empty`);
            return { folders: [], leaves: [], hasMore: false };
        }
        // Create a temporary connection for this specific db to avoid SELECT state corruption
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { config } = entry;
        if (config.type !== 'redis')
            throw new Error('getRedisKeysTree is only available for Redis connections');
        (0, logger_1.info)(`[getRedisKeysTree] Connecting to Redis ${config.host}:${config.port} db=${dbIndex}...`);
        const { Redis: RedisClass } = await Promise.resolve().then(() => __importStar(require('ioredis')));
        const tempClient = new RedisClass({
            host: config.host,
            port: config.port,
            password: config.password,
            db: dbIndex,
            connectTimeout: 5000,
            retryStrategy: () => null,
        });
        const scanPattern = prefix ? `${prefix}*` : '*';
        const MAX_ITERATIONS = 10;
        const BATCH_SIZE = 2000;
        let iterations = 0;
        (0, logger_1.info)(`[getRedisKeysTree] Scanning pattern="${scanPattern}", cursor=${state.cursor}`);
        try {
            // Ping test
            await tempClient.ping();
            (0, logger_1.info)(`[getRedisKeysTree] Ping OK, scanning...`);
            do {
                const res = await tempClient.scan(state.cursor, 'MATCH', scanPattern, 'COUNT', BATCH_SIZE);
                if (!res)
                    break;
                state.cursor = res[0];
                const keys = res[1] || [];
                (0, logger_1.info)(`[getRedisKeysTree] iter=${iterations + 1}, nextCursor=${state.cursor}, keysFound=${keys.length}`);
                iterations++;
                if (keys.length === 0)
                    continue;
                const prefixLen = prefix.length;
                for (const fullKey of keys) {
                    const relative = fullKey.substring(prefixLen);
                    const idx = relative.indexOf(':');
                    const name = idx > -1 ? relative.substring(0, idx) : relative;
                    const isFolder = idx > -1;
                    if (isFolder) {
                        if (!state.folders.has(name)) {
                            state.folders.set(name, 1);
                        }
                        else {
                            state.folders.set(name, state.folders.get(name) + 1);
                        }
                    }
                    else {
                        if (!state.folders.has(name) && !state.leaves.has(name)) {
                            state.leaves.add(name);
                        }
                    }
                }
            } while (iterations < MAX_ITERATIONS);
            if (state.cursor === '0') {
                state.finished = true;
                (0, logger_1.info)(`[getRedisKeysTree] Scan complete (cursor=0)`);
            }
        }
        catch (err) {
            console.error(`[getRedisKeysTree] ERROR during scan: ${err.message}`, err.stack);
            throw err;
        }
        finally {
            try {
                await tempClient.quit();
            }
            catch { }
        }
        (0, logger_1.info)(`[getRedisKeysTree] Scan done: folders=${state.folders.size}, leaves=${state.leaves.size}, hasMore=${state.cursor !== '0'}`);
        // Resolve leaf types using a fresh temp client
        const typeClient = new RedisClass({
            host: config.host,
            port: config.port,
            password: config.password,
            db: dbIndex,
            connectTimeout: 5000,
            retryStrategy: () => null,
        });
        const addedFolders = [];
        const typedLeaves = [];
        for (const [name, count] of state.folders.entries()) {
            addedFolders.push({ name, isFolder: true, count });
        }
        if (state.leaves.size > 0) {
            const pipe = typeClient.pipeline();
            const leafNames = Array.from(state.leaves);
            (0, logger_1.info)(`[getRedisKeysTree] Resolving types for ${leafNames.length} leaves via pipeline`);
            leafNames.forEach(name => pipe.type(prefix + name));
            const res = await pipe.exec();
            leafNames.forEach((name, i) => {
                const type = String(res[i][0] ? 'unknown' : res[i][1]);
                typedLeaves.push({ name, isFolder: false, type });
            });
        }
        try {
            await typeClient.quit();
        }
        catch { }
        addedFolders.sort((a, b) => a.name.localeCompare(b.name));
        typedLeaves.sort((a, b) => a.name.localeCompare(b.name));
        (0, logger_1.info)(`[getRedisKeysTree] Returning: folders=${addedFolders.length}, leaves=${typedLeaves.length}, hasMore=${state.cursor !== '0'}`);
        return {
            folders: addedFolders,
            leaves: typedLeaves,
            hasMore: state.cursor !== '0'
        };
    }
    // Deprecated: scanChunk replaced by inline scanning in getRedisKeysTree using shared connection
    async scanChunk(id, dbIndex, prefix, state) {
        return { addedFolders: [], addedLeaves: [] };
    }
    // Helper: Reset state timer
    resetStateTimer(key, state) {
        if (state.timer)
            clearTimeout(state.timer);
        state.timer = setTimeout(() => {
            if (state instanceof Map)
                state.delete(key);
            // Check if it's streamScanStates
            else if (this.streamScanStates.has(key))
                this.streamScanStates.delete(key);
        }, 60000);
    }
    // Incremental Stream Scanning
    async getRedisStreamsIncremental(id, dbIndex, pattern = '*', loadMore = false) {
        const stateKey = `streams:${id}:${dbIndex ?? 0}:${pattern}`;
        (0, logger_1.info)(`[RedisStreams] getRedisStreamsIncremental called: id=${id}, dbIndex=${dbIndex ?? 0}, pattern=${pattern}, loadMore=${loadMore}, stateKey=${stateKey}`);
        // If not loading more, reset state (fresh load)
        if (!loadMore) {
            (0, logger_1.info)(`[RedisStreams] Resetting state for ${stateKey}`);
            this.streamScanStates.delete(stateKey);
        }
        let state = this.streamScanStates.get(stateKey);
        if (!state) {
            (0, logger_1.info)(`[RedisStreams] Creating new state for ${stateKey}`);
            state = { cursor: '0', foundStreams: [], finished: false, timer: null };
            this.streamScanStates.set(stateKey, state);
        }
        if (state.finished) {
            (0, logger_1.info)(`[RedisStreams] State already finished for ${stateKey}, returning empty`);
            return { streams: [], hasMore: false };
        }
        const BATCH_LIMIT = 50; // Return 50 streams per request
        const SCAN_COUNT = 2000; // Scan 2000 keys at a time
        // Get config for temp client creation
        const entry = this.connections.get(id);
        if (!entry) {
            console.error(`[RedisStreams] Connection '${id}' not found`);
            throw new Error(`Connection '${id}' not found`);
        }
        const { config } = entry;
        (0, logger_1.info)(`[RedisStreams] Creating temp client: ${config.host}:${config.port}, db=${dbIndex ?? 0}`);
        // Use a temp client to avoid SELECT race conditions on shared connection
        const { Redis: RedisClass } = await Promise.resolve().then(() => __importStar(require('ioredis')));
        const client = new RedisClass({
            host: config.host,
            port: config.port,
            password: config.password,
            db: dbIndex ?? 0,
            connectTimeout: 5000,
            retryStrategy: () => null,
        });
        try {
            // Ping to verify connection
            await client.ping();
            (0, logger_1.info)(`[RedisStreams] Temp client connected successfully`);
            // Scan loop
            let iterations = 0;
            const MAX_ITERATIONS = 50; // Prevent infinite loops
            (0, logger_1.info)(`[RedisStreams] Starting scan loop: cursor=${state.cursor}, foundStreams=${state.foundStreams.length}`);
            // Use a do-while so the first scan (cursor='0') actually executes
            do {
                iterations++;
                if (state.foundStreams.length >= BATCH_LIMIT) {
                    (0, logger_1.info)(`[RedisStreams] Already have enough (${state.foundStreams.length}), skipping scan`);
                    break;
                }
                const res = await client.scan(state.cursor, 'MATCH', pattern, 'COUNT', SCAN_COUNT);
                if (!res) {
                    (0, logger_1.info)(`[RedisStreams] Scan returned null, breaking`);
                    break;
                }
                state.cursor = res[0];
                const keys = res[1] || [];
                (0, logger_1.info)(`[RedisStreams] Iteration ${iterations}: cursor=${state.cursor}, keysFound=${keys.length}, streamsFound=${state.foundStreams.length}`);
                if (keys.length > 0) {
                    // Pipeline to check types
                    const pipe = client.pipeline();
                    keys.forEach((k) => pipe.type(k));
                    const types = await pipe.exec();
                    // Filter for streams and get info
                    for (let i = 0; i < keys.length; i++) {
                        const typeRes = (types || [])[i];
                        if (!typeRes[0] && typeRes[1] === 'stream') {
                            try {
                                const key = keys[i];
                                const info = await client.xinfo('STREAM', key);
                                const infoObj = {};
                                // Flatten xinfo array [key, val, key, val...]
                                for (let j = 0; j < info.length; j += 2) {
                                    infoObj[info[j]] = info[j + 1];
                                }
                                (0, logger_1.info)(`[RedisStreams] Found stream: ${key}, length=${infoObj.length}`);
                                state.foundStreams.push({
                                    name: key,
                                    length: infoObj.length || 0,
                                    groups: infoObj.groups || 0,
                                    pendingCount: 0
                                });
                            }
                            catch (e) {
                                console.warn(`[RedisStreams] Error getting xinfo for ${keys[i]}:`, e.message);
                            }
                        }
                    }
                }
                // If we have enough, break loop but keep cursor for next time
                if (state.foundStreams.length >= BATCH_LIMIT) {
                    (0, logger_1.info)(`[RedisStreams] Reached BATCH_LIMIT (${BATCH_LIMIT}), breaking`);
                    break;
                }
            } while (state.cursor !== '0' && iterations < MAX_ITERATIONS);
            if (iterations >= MAX_ITERATIONS) {
                console.warn(`[RedisStreams] Hit MAX_ITERATIONS (${MAX_ITERATIONS}), stopping`);
                state.finished = true;
            }
            if (state.cursor === '0') {
                state.finished = true;
                (0, logger_1.info)(`[RedisStreams] Scan completed (cursor=0), total streams=${state.foundStreams.length}`);
            }
            // Return batch
            const resultStreams = state.foundStreams.splice(0, BATCH_LIMIT);
            (0, logger_1.info)(`[RedisStreams] Returning ${resultStreams.length} streams, hasMore=${!state.finished}, remaining=${state.foundStreams.length}`);
            this.resetStateTimer(stateKey, state);
            return { streams: resultStreams, hasMore: !state.finished };
        }
        catch (error) {
            console.error(`[RedisStreams] Scan error:`, error.message);
            throw error;
        }
        finally {
            try {
                await client.quit();
            }
            catch { }
        }
    }
    async getKeys(id, pattern) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('getKeys is only available for Redis connections');
        }
        return this.scanKeys(client, pattern || '*');
    }
    async scanKeys(redisClient, pattern) {
        const keys = [];
        let cursor = '0';
        do {
            const result = await redisClient.scan(cursor, 'MATCH', pattern, 'COUNT', 5000);
            cursor = result[0];
            keys.push(...result[1]);
        } while (cursor !== '0');
        return keys;
    }
    async getValue(id, key) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis') {
            throw new Error('getValue is only available for Redis connections');
        }
        const value = await client.get(key);
        // Try to parse JSON
        try {
            return value !== null ? JSON.parse(value) : value;
        }
        catch {
            return value;
        }
    }
    async createClient(config) {
        switch (config.type) {
            case 'mysql':
                return promise_1.default.createConnection({
                    host: config.host,
                    port: config.port,
                    user: config.user,
                    password: config.password,
                    database: config.database,
                    connectTimeout: 5000,
                    // Type cast: convert BIT/TINYINT(1) booleans and parse JSON columns
                    typeCast: (field, next) => {
                        // BIT(1) → boolean (0/1)
                        if (field.type === 'BIT' && field.length === 1) {
                            const val = field.buffer();
                            return val ? val[0] : 0;
                        }
                        // TINYINT(1) → number (keep as 0/1 for display)
                        if (field.type === 'TINY' && field.length === 1) {
                            return field.string();
                        }
                        return next();
                    },
                });
            case 'postgresql': {
                const pgClient = new pg_1.Client({
                    host: config.host,
                    port: config.port,
                    user: config.user,
                    password: config.password,
                    database: config.database,
                    connectionTimeoutMillis: 5000,
                });
                await pgClient.connect();
                return pgClient;
            }
            case 'redis': {
                const redisClient = new ioredis_1.default({
                    host: config.host,
                    port: config.port,
                    password: config.password,
                    db: config.dbIndex || 0,
                    connectTimeout: 5000,
                    retryStrategy: () => null, // Don't retry on connection loss for manual connections
                });
                await redisClient.ping();
                return redisClient;
            }
            case 'sqlite': {
                const Database = require('better-sqlite3');
                const dbPath = config.path || config.host;
                return new Database(dbPath);
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
    }
    getConfig(id) {
        return this.connections.get(id)?.config;
    }
    isConnected(id) {
        return this.connections.has(id);
    }
    /** Get Redis client from connection pool with type checking */
    getRedisClient(id) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        if (entry.config.type !== 'redis')
            throw new Error(`Connection '${id}' is not a Redis connection`);
        return entry.client;
    }
    /** Check if Redis connection is alive (responds to PING) */
    async isRedisConnected(id) {
        try {
            const entry = this.connections.get(id);
            if (!entry || entry.config.type !== 'redis')
                return false;
            const redis = entry.client;
            const result = await redis.ping();
            return result === 'PONG';
        }
        catch {
            return false;
        }
    }
    // ============ Structure Synchronization ============
    async getTableStructure(id, tableName, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        switch (config.type) {
            case 'mysql': {
                const conn = client;
                // Use dbName if provided, otherwise fall back to DATABASE()
                const dbCondition = dbName ? `TABLE_SCHEMA = ?` : `TABLE_SCHEMA = DATABASE()`;
                const dbParams = dbName ? [dbName, tableName] : [tableName];
                // Get columns
                const [colRows] = await conn.query(`SELECT COLUMN_NAME, DATA_TYPE, CHARACTER_MAXIMUM_LENGTH, NUMERIC_PRECISION, NUMERIC_SCALE,
                  IS_NULLABLE, COLUMN_DEFAULT, COLUMN_KEY, EXTRA, COLUMN_COMMENT, ORDINAL_POSITION
           FROM INFORMATION_SCHEMA.COLUMNS
           WHERE ${dbCondition} AND TABLE_NAME = ?
           ORDER BY ORDINAL_POSITION`, dbParams);
                const columns = colRows.map((r) => ({
                    name: r.COLUMN_NAME,
                    type: r.DATA_TYPE.toUpperCase(),
                    length: r.CHARACTER_MAXIMUM_LENGTH != null ? r.CHARACTER_MAXIMUM_LENGTH : (r.NUMERIC_PRECISION != null ? r.NUMERIC_PRECISION : null),
                    decimals: r.NUMERIC_SCALE != null ? r.NUMERIC_SCALE : null,
                    nullable: r.IS_NULLABLE === 'YES',
                    defaultValue: r.COLUMN_DEFAULT,
                    isPrimaryKey: r.COLUMN_KEY === 'PRI',
                    isAutoIncrement: r.EXTRA?.includes('auto_increment') ?? false,
                    comment: r.COLUMN_COMMENT || undefined,
                    ordinalPosition: r.ORDINAL_POSITION,
                }));
                // Get indexes — use same dbName condition as columns query
                const [idxRows] = await conn.query(`SELECT INDEX_NAME, COLUMN_NAME, NON_UNIQUE
           FROM INFORMATION_SCHEMA.STATISTICS
           WHERE ${dbCondition} AND TABLE_NAME = ?
           ORDER BY INDEX_NAME, SEQ_IN_INDEX`, dbParams);
                const indexMap = new Map();
                for (const r of idxRows) {
                    if (!indexMap.has(r.INDEX_NAME)) {
                        indexMap.set(r.INDEX_NAME, {
                            name: r.INDEX_NAME,
                            columns: [],
                            isUnique: r.NON_UNIQUE === 0,
                            isPrimary: r.INDEX_NAME === 'PRIMARY',
                        });
                    }
                    indexMap.get(r.INDEX_NAME).columns.push(r.COLUMN_NAME);
                }
                const indexes = Array.from(indexMap.values());
                const primaryKey = columns.filter(c => c.isPrimaryKey).map(c => c.name);
                return { tableName, columns, indexes, primaryKey };
            }
            case 'postgresql': {
                const pgClient = client;
                const schema = dbName || 'public';
                const colRes = await pgClient.query(`SELECT column_name, data_type, character_maximum_length, numeric_precision, numeric_scale,
                  is_nullable, column_default, ordinal_position
           FROM information_schema.columns
           WHERE table_schema = $1 AND table_name = $2
           ORDER BY ordinal_position`, [schema, tableName]);
                const pkRes = await pgClient.query(`SELECT kcu.column_name
           FROM information_schema.table_constraints tc
           JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
           WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_schema = $1 AND tc.table_name = $2`, [schema, tableName]);
                const pkColumns = pkRes.rows.map(r => r.column_name);
                const columns = colRes.rows.map((r) => ({
                    name: r.column_name,
                    type: r.data_type.toUpperCase(),
                    length: r.character_maximum_length != null ? r.character_maximum_length : (r.numeric_precision != null ? r.numeric_precision : null),
                    decimals: r.numeric_scale != null ? r.numeric_scale : null,
                    nullable: r.is_nullable === 'YES',
                    defaultValue: r.column_default,
                    isPrimaryKey: pkColumns.includes(r.column_name),
                    isAutoIncrement: r.column_default?.includes('nextval') || false,
                    ordinalPosition: r.ordinal_position,
                }));
                const idxRes = await pgClient.query(`SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = $1 AND tablename = $2`, [schema, tableName]);
                const indexes = idxRes.rows.map((r) => {
                    // Parse column names from indexdef like:
                    //   CREATE INDEX name ON table USING btree (col1, col2)
                    //   CREATE UNIQUE INDEX name ON table (lower(name))
                    // Match the last (...) which contains column list
                    const matches = r.indexdef.match(/\((.+)\)/);
                    const cols = matches
                        ? matches[1].split(',').map((c) => c.trim().replace(/["()]/g, ''))
                        : [];
                    return {
                        name: r.indexname,
                        columns: cols,
                        isUnique: r.indexdef.includes('UNIQUE'),
                        isPrimary: r.indexname.includes('pkey'),
                    };
                });
                return { tableName, columns, indexes, primaryKey: pkColumns };
            }
            case 'sqlite': {
                const sqliteClient = client;
                const escTable = tableName.replace(/"/g, '""');
                const columns = sqliteClient.prepare(`PRAGMA table_info("${escTable}")`).all().map((r) => {
                    // SQLite stores type as string like "VARCHAR(255)" or "TEXT"
                    const rawType = r.type || '';
                    const m = rawType.match(/^([a-zA-Z]+)(?:\((\d+)(?:,(\d+))?\))?$/);
                    const baseType = m ? m[1].toUpperCase() : rawType.toUpperCase() || 'TEXT';
                    const length = m && m[2] ? parseInt(m[2], 10) : null;
                    const decimals = m && m[3] ? parseInt(m[3], 10) : null;
                    return {
                        name: r.name,
                        type: baseType,
                        length,
                        decimals,
                        nullable: r.notnull === 0,
                        defaultValue: r.dflt_value,
                        isPrimaryKey: r.pk > 0,
                        isAutoIncrement: false,
                        ordinalPosition: r.cid,
                    };
                });
                const indexRows = sqliteClient.prepare(`PRAGMA index_list("${escTable}")`).all();
                const indexes = indexRows.map((idx) => {
                    const escIdxName = idx.name.replace(/"/g, '""');
                    const cols = sqliteClient.prepare(`PRAGMA index_info("${escIdxName}")`).all();
                    return {
                        name: idx.name,
                        columns: cols.map((c) => c.name),
                        isUnique: idx.unique === 1,
                        isPrimary: idx.name.startsWith('sqlite_autoindex'),
                    };
                });
                const primaryKey = columns.filter(c => c.isPrimaryKey).map(c => c.name);
                return { tableName, columns, indexes, primaryKey };
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
    }
    async getCreateTableSql(id, tableName, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        switch (config.type) {
            case 'mysql': {
                const conn = client;
                const tableRef = dbName ? `\`${dbName.replace(/`/g, '``')}\`.\`${tableName.replace(/`/g, '``')}\`` : `\`${tableName.replace(/`/g, '``')}\``;
                const [rows] = await conn.query(`SHOW CREATE TABLE ${tableRef}`);
                if (rows.length > 0) {
                    return rows[0]['Create Table'] || rows[0]['Create View'] || '';
                }
                throw new Error(`Table '${tableName}' not found`);
            }
            case 'postgresql': {
                const pgClient = client;
                const schema = dbName || 'public';
                // Reconstruct CREATE TABLE from pg_catalog
                const colRes = await pgClient.query(`SELECT a.attname,
                  pg_catalog.format_type(a.atttypid, a.atttypmod) as data_type,
                  a.attnotnull,
                  pg_catalog.pg_get_expr(d.adbin, d.adrelid) as default_value,
                  col_description(a.attrelid, a.attnum) as comment
           FROM pg_catalog.pg_attribute a
           LEFT JOIN pg_catalog.pg_attrdef d ON (a.attrelid, a.attnum) = (d.adrelid, d.adnum)
           WHERE a.attrelid = (
             SELECT c.oid FROM pg_catalog.pg_class c
             JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = $1 AND c.relname = $2
           )
           AND a.attnum > 0 AND NOT a.attisdropped
           ORDER BY a.attnum`, [schema, tableName]);
                if (colRes.rows.length === 0)
                    throw new Error(`Table '${tableName}' not found`);
                const colDefs = colRes.rows.map((r) => {
                    let def = `  "${r.attname}" ${r.data_type}`;
                    if (r.default_value)
                        def += ` DEFAULT ${r.default_value}`;
                    if (r.attnotnull)
                        def += ' NOT NULL';
                    return def;
                });
                // Get primary key
                const pkRes = await pgClient.query(`SELECT a.attname
           FROM pg_index i
           JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
           WHERE i.indrelid = (
             SELECT c.oid FROM pg_catalog.pg_class c
             JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = $1 AND c.relname = $2
           )
           AND i.indisprimary
           ORDER BY array_position(i.indkey, a.attnum)`, [schema, tableName]);
                if (pkRes.rows.length > 0) {
                    const pkCols = pkRes.rows.map((r) => `"${r.attname}"`).join(', ');
                    colDefs.push(`  PRIMARY KEY (${pkCols})`);
                }
                let sql = `CREATE TABLE "${schema}"."${tableName}" (\n${colDefs.join(',\n')}\n);`;
                // Add comments
                const comments = colRes.rows.filter((r) => r.comment);
                if (comments.length > 0) {
                    sql += '\n' + comments.map((r) => `COMMENT ON COLUMN "${schema}"."${tableName}"."${r.attname}" IS '${r.comment.replace(/'/g, "''")}';`).join('\n');
                }
                return sql;
            }
            case 'sqlite': {
                const sqliteClient = client;
                const escTable = tableName.replace(/"/g, '""');
                const row = sqliteClient.prepare(`SELECT sql FROM sqlite_master WHERE type='table' AND name="${escTable}"`).get();
                if (row && row.sql)
                    return row.sql;
                throw new Error(`Table '${tableName}' not found`);
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
    }
    async compareStructures(sourceId, targetId, tableName, sourceDbName, targetDbName) {
        // If a specific table is requested, only compare that table
        if (tableName && tableName.trim() !== '') {
            const sourceTables = await this.getTables(sourceId, sourceDbName);
            const targetTables = await this.getTables(targetId, targetDbName);
            const sourceSetLower = new Set(sourceTables.map(t => t.toLowerCase()));
            const targetSetLower = new Set(targetTables.map(t => t.toLowerCase()));
            const diffs = [];
            const commonTables = [];
            const sourceOnly = [];
            const targetOnly = [];
            const inSource = sourceSetLower.has(tableName.toLowerCase());
            const inTarget = targetSetLower.has(tableName.toLowerCase());
            if (inSource && !inTarget) {
                sourceOnly.push(tableName);
                const structure = await this.getTableStructure(sourceId, tableName, sourceDbName);
                const targetDbType = this.connections.get(targetId)?.config.type || 'mysql';
                const createSql = this.generateCreateTableSql(structure, targetDbType);
                diffs.push({
                    tableName,
                    diffType: 'table_only_in_source',
                    sourceValue: structure,
                    sql: createSql,
                });
            }
            else if (!inSource && inTarget) {
                targetOnly.push(tableName);
                diffs.push({
                    tableName,
                    diffType: 'table_only_in_target',
                    targetValue: await this.getTableStructure(targetId, tableName, targetDbName),
                    sql: `-- Table '${tableName}' only exists in target`,
                });
            }
            else if (inSource && inTarget) {
                // Resolve actual table names from each DB
                const sourceActualTable = sourceTables.find(t => t.toLowerCase() === tableName.toLowerCase()) || tableName;
                const targetActualTable = targetTables.find(t => t.toLowerCase() === tableName.toLowerCase()) || tableName;
                commonTables.push(tableName);
                const sourceStructure = await this.getTableStructure(sourceId, sourceActualTable, sourceDbName);
                const targetStructure = await this.getTableStructure(targetId, targetActualTable, targetDbName);
                const targetDbType = this.connections.get(targetId)?.config.type || 'mysql';
                this.compareTableStructure(diffs, tableName, targetActualTable, sourceStructure, targetStructure, targetDbType);
            }
            return { diffs, sourceTables: inSource ? [tableName] : [], targetTables: inTarget ? [tableName] : [], commonTables };
        }
        // Full database comparison (no specific table)
        const sourceTables = await this.getTables(sourceId, sourceDbName);
        const targetTables = await this.getTables(targetId, targetDbName);
        const sourceSetLower = new Set(sourceTables.map(t => t.toLowerCase()));
        const targetSetLower = new Set(targetTables.map(t => t.toLowerCase()));
        // Build lowercase → actual case maps for resolving table names across databases
        const targetLowerToActual = new Map();
        for (const t of targetTables)
            targetLowerToActual.set(t.toLowerCase(), t);
        const sourceLowerToActual = new Map();
        for (const t of sourceTables)
            sourceLowerToActual.set(t.toLowerCase(), t);
        const commonTables = sourceTables.filter(t => targetSetLower.has(t.toLowerCase()));
        const sourceOnly = sourceTables.filter(t => !targetSetLower.has(t.toLowerCase()));
        const targetOnly = targetTables.filter(t => !sourceSetLower.has(t.toLowerCase()));
        const diffs = [];
        // Tables only in source (need to create in target)
        const targetDbType = this.connections.get(targetId)?.config.type || 'mysql';
        for (const t of sourceOnly) {
            const structure = await this.getTableStructure(sourceId, t, sourceDbName);
            const createSql = this.generateCreateTableSql(structure, targetDbType);
            diffs.push({
                tableName: t,
                diffType: 'table_only_in_source',
                sourceValue: structure,
                sql: createSql,
            });
        }
        // Tables only in target (optionally drop)
        for (const t of targetOnly) {
            diffs.push({
                tableName: t,
                diffType: 'table_only_in_target',
                targetValue: await this.getTableStructure(targetId, t, targetDbName),
                sql: `-- Table '${t}' only exists in target`,
            });
        }
        // Common tables - compare structure
        // ⚠️ Use ACTUAL table names from each DB — casing may differ (e.g. users vs USERS)
        for (const t of commonTables) {
            const sourceTable = sourceLowerToActual.get(t.toLowerCase()) || t;
            const targetTable = targetLowerToActual.get(t.toLowerCase()) || t;
            const sourceStructure = await this.getTableStructure(sourceId, sourceTable, sourceDbName);
            const targetStructure = await this.getTableStructure(targetId, targetTable, targetDbName);
            this.compareTableStructure(diffs, t, targetTable, sourceStructure, targetStructure, targetDbType);
        }
        return { diffs, sourceTables, targetTables, commonTables };
    }
    /** Compare structure of a single table and push diffs */
    compareTableStructure(diffs, tableName, targetTableName, sourceStructure, targetStructure, targetDbType) {
        // Compare columns — case-insensitive matching (MySQL column names may differ in case)
        const sourceCols = new Map(sourceStructure.columns.map(c => [c.name.toLowerCase(), c]));
        const targetCols = new Map(targetStructure.columns.map(c => [c.name.toLowerCase(), c]));
        for (const col of sourceStructure.columns) {
            const targetCol = targetCols.get(col.name.toLowerCase());
            if (!targetCol) {
                const addSql = this.generateAddColumnSql(targetTableName, col, targetDbType);
                diffs.push({
                    tableName,
                    diffType: 'column_added',
                    sourceValue: col,
                    sql: addSql,
                });
            }
            else {
                const differences = this.compareColumns(col, targetCol);
                if (differences.length > 0) {
                    const modifySql = this.generateModifyColumnSql(targetTableName, col, targetCol, targetDbType, differences);
                    diffs.push({
                        tableName,
                        diffType: 'column_modified',
                        sourceValue: col,
                        targetValue: targetCol,
                        sql: modifySql,
                    });
                }
            }
        }
        // Columns only in target
        for (const col of targetStructure.columns) {
            if (!sourceCols.has(col.name.toLowerCase())) {
                diffs.push({
                    tableName,
                    diffType: 'column_removed',
                    targetValue: col,
                    sql: `-- Column '${col.name}' only exists in target`,
                });
            }
        }
        // Compare indexes — case-insensitive matching
        const sourceIdx = new Map(sourceStructure.indexes.map(i => [i.name.toLowerCase(), i]));
        const targetIdx = new Map(targetStructure.indexes.map(i => [i.name.toLowerCase(), i]));
        for (const [name, idx] of sourceIdx) {
            if (idx.isPrimary)
                continue;
            const targetIdxInfo = targetIdx.get(name);
            if (!targetIdxInfo) {
                const createIdxSql = this.generateCreateIndexSql(targetTableName, idx, targetDbType);
                diffs.push({
                    tableName,
                    diffType: 'index_added',
                    sourceValue: idx,
                    sql: createIdxSql,
                });
            }
            else {
                const srcCols = [...idx.columns].sort().join(',');
                const tgtCols = [...targetIdxInfo.columns].sort().join(',');
                if (srcCols !== tgtCols || idx.isUnique !== targetIdxInfo.isUnique) {
                    diffs.push({
                        tableName,
                        diffType: 'index_modified',
                        sourceValue: idx,
                        targetValue: targetIdxInfo,
                        sql: `DROP INDEX IF EXISTS ${this.quoteIdentifier(idx.name, targetDbType)}; ${this.generateCreateIndexSql(targetTableName, idx, targetDbType)}`,
                    });
                }
            }
        }
        for (const [name, idx] of targetIdx) {
            if (idx.isPrimary)
                continue;
            if (!sourceIdx.has(name)) {
                diffs.push({
                    tableName,
                    diffType: 'index_removed',
                    targetValue: idx,
                    sql: `DROP INDEX IF EXISTS ${this.quoteIdentifier(idx.name, targetDbType)}`,
                });
            }
        }
        // Compare primary keys — case-insensitive
        const srcPK = [...sourceStructure.primaryKey].map(c => c.toLowerCase()).sort().join(',');
        const tgtPK = [...targetStructure.primaryKey].map(c => c.toLowerCase()).sort().join(',');
        if (srcPK !== tgtPK) {
            diffs.push({
                tableName,
                diffType: 'primary_key_changed',
                sourceValue: sourceStructure.primaryKey,
                targetValue: targetStructure.primaryKey,
                sql: `-- Primary key differs: source(${srcPK}) vs target(${tgtPK}) — manual intervention required`,
            });
        }
    }
    async executeStructureSync(targetId, sqls, targetDbName) {
        const entry = this.connections.get(targetId);
        if (!entry)
            throw new Error(`Connection '${targetId}' not found`);
        const { client, config } = entry;
        const errors = [];
        let executed = 0;
        // For MySQL, ensure a database is selected before executing DDL
        const dbName = targetDbName || config.database;
        if (config.type === 'mysql' && dbName) {
            try {
                const escDb = dbName.replace(/`/g, '``');
                await client.query(`USE \`${escDb}\``);
            }
            catch (e) {
                return { success: false, executed: 0, errors: [`Failed to select database '${dbName}': ${e.message}`] };
            }
        }
        // Start transaction
        try {
            switch (config.type) {
                case 'mysql':
                    await client.beginTransaction();
                    break;
                case 'postgresql':
                    await client.query('BEGIN');
                    break;
                case 'sqlite':
                    client.exec('BEGIN');
                    break;
            }
        }
        catch (e) {
            return { success: false, executed: 0, errors: [`Failed to start transaction: ${e.message}`] };
        }
        for (const sql of sqls) {
            const trimmed = sql.trim();
            if (!trimmed || trimmed.startsWith('--'))
                continue;
            try {
                switch (config.type) {
                    case 'mysql':
                        await client.query(trimmed);
                        executed++;
                        break;
                    case 'postgresql':
                        await client.query(trimmed);
                        executed++;
                        break;
                    case 'sqlite':
                        client.exec(trimmed);
                        executed++;
                        break;
                }
            }
            catch (e) {
                errors.push(`Error executing "${trimmed.substring(0, 100)}...": ${e.message}`);
                // Rollback on error
                try {
                    switch (config.type) {
                        case 'mysql':
                            await client.rollback();
                            break;
                        case 'postgresql':
                            await client.query('ROLLBACK');
                            break;
                        case 'sqlite':
                            client.exec('ROLLBACK');
                            break;
                    }
                }
                catch (rollbackError) {
                    errors.push(`Failed to rollback: ${rollbackError.message}`);
                }
                return { success: false, executed, errors };
            }
        }
        // Commit transaction
        try {
            switch (config.type) {
                case 'mysql':
                    await client.commit();
                    break;
                case 'postgresql':
                    await client.query('COMMIT');
                    break;
                case 'sqlite':
                    client.exec('COMMIT');
                    break;
            }
        }
        catch (e) {
            errors.push(`Failed to commit transaction: ${e.message}`);
            return { success: false, executed, errors };
        }
        return { success: errors.length === 0, executed, errors };
    }
    // ============ Data Synchronization ============
    async compareData(sourceId, targetId, tableName, primaryKeys, columns, sourceDbName, targetDbName, tablePrimaryKeys // Table's actual PKs — excluded from diff comparison
    ) {
        const sourceRows = await this.getAllTableData(sourceId, tableName, columns, sourceDbName);
        const targetRows = await this.getAllTableData(targetId, tableName, columns, targetDbName);
        const sourceMap = new Map();
        const targetMap = new Map();
        for (const row of sourceRows) {
            const key = this.getRowKey(row, primaryKeys);
            sourceMap.set(key, row);
        }
        for (const row of targetRows) {
            const key = this.getRowKey(row, primaryKeys);
            targetMap.set(key, row);
        }
        const diffs = [];
        let totalInserts = 0;
        let totalUpdates = 0;
        let totalDeletes = 0;
        // Columns to exclude from value comparison: compare keys + table PKs
        const excludeFromCompare = new Set([...primaryKeys, ...(tablePrimaryKeys || [])]);
        const compareColumns = columns.filter(c => !excludeFromCompare.has(c));
        // Find inserts and updates
        for (const [key, sourceRow] of sourceMap) {
            if (!targetMap.has(key)) {
                diffs.push({ diffType: 'insert', primaryKey: this.extractPrimaryKey(sourceRow, primaryKeys), sourceRow });
                totalInserts++;
            }
            else {
                const targetRow = targetMap.get(key);
                if (this.rowsDiffer(sourceRow, targetRow, compareColumns)) {
                    diffs.push({
                        diffType: 'update',
                        primaryKey: this.extractPrimaryKey(sourceRow, primaryKeys),
                        sourceRow,
                        targetRow,
                    });
                    totalUpdates++;
                }
            }
        }
        // Find deletes
        for (const [key, targetRow] of targetMap) {
            if (!sourceMap.has(key)) {
                diffs.push({ diffType: 'delete', primaryKey: this.extractPrimaryKey(targetRow, primaryKeys), targetRow });
                totalDeletes++;
            }
        }
        return { diffs, totalInserts, totalUpdates, totalDeletes };
    }
    async executeDataSync(options) {
        const startTime = Date.now();
        const entry = this.connections.get(options.targetConnectionId);
        if (!entry)
            throw new Error(`Target connection '${options.targetConnectionId}' not found`);
        const { client, config } = entry;
        const errors = [];
        let inserted = 0;
        let updated = 0;
        let deleted = 0;
        const batchSize = options.batchSize || 100;
        const useTransaction = options.useTransaction !== false;
        // For MySQL, ensure the correct database is selected (prefer options.targetDbName over config.database)
        const mysqlDbName = options.targetDbName || config.database;
        if (config.type === 'mysql' && mysqlDbName) {
            const escDb = mysqlDbName.replace(/`/g, '``');
            await client.query(`USE \`${escDb}\``);
        }
        const processBatch = async (batch) => {
            // Resolve table PKs once per batch — used to exclude from INSERT/UPDATE SET clause
            const realPkCols = [];
            if (options.tablePrimaryKeys) {
                realPkCols.push(...options.tablePrimaryKeys);
            }
            for (const diff of batch) {
                try {
                    switch (diff.diffType) {
                        case 'insert': {
                            // Strip real PK columns from sourceRow for INSERT — let target DB auto-generate them
                            const insertRow = { ...diff.sourceRow };
                            for (const pk of realPkCols)
                                delete insertRow[pk];
                            const sql = this.generateInsertSql(options.tableName, insertRow, config.type, options.targetDbName, 'ignore');
                            await this.executeSql(client, config.type, sql);
                            inserted++;
                            break;
                        }
                        case 'update': {
                            // Strip real PK columns from sourceRow — target row is located by compare keys, not PKs
                            const updateRow = { ...diff.sourceRow };
                            for (const pk of realPkCols)
                                delete updateRow[pk];
                            // Exclude both compare keys (WHERE clause) AND real PKs from SET clause
                            const excludeCols = new Set([...options.primaryKeys, ...realPkCols]);
                            const nonPkCols = Object.keys(updateRow).filter(c => !excludeCols.has(c));
                            if (nonPkCols.length === 0)
                                break;
                            const sql = this.generateUpdateSql(options.tableName, updateRow, options.primaryKeys, config.type, options.targetDbName);
                            await this.executeSql(client, config.type, sql);
                            updated++;
                            break;
                        }
                        case 'delete': {
                            const sql = this.generateDeleteSql(options.tableName, diff.primaryKey, options.primaryKeys, config.type, options.targetDbName);
                            await this.executeSql(client, config.type, sql);
                            deleted++;
                            break;
                        }
                    }
                }
                catch (e) {
                    errors.push(`Error ${diff.diffType}: ${e.message}`);
                    // In transaction mode, re-throw immediately to abort the transaction
                    // and avoid cascading failures from subsequent statements
                    if (useTransaction)
                        throw e;
                }
            }
        };
        if (useTransaction) {
            try {
                // Start transaction
                if (config.type === 'mysql') {
                    await client.beginTransaction();
                }
                else if (config.type === 'postgresql') {
                    await client.query('BEGIN');
                }
                else if (config.type === 'sqlite') {
                    client.exec('BEGIN');
                }
                for (let i = 0; i < options.diffs.length; i += batchSize) {
                    if (errors.length > 0)
                        break; // Stop after first error to avoid cascading failures in aborted transaction
                    await processBatch(options.diffs.slice(i, i + batchSize));
                }
                // Check if any errors occurred during batch processing
                if (errors.length > 0) {
                    // Rollback on errors
                    if (config.type === 'mysql') {
                        await client.rollback();
                    }
                    else if (config.type === 'postgresql') {
                        await client.query('ROLLBACK');
                    }
                    else if (config.type === 'sqlite') {
                        client.exec('ROLLBACK');
                    }
                }
                else {
                    if (config.type === 'mysql') {
                        await client.commit();
                    }
                    else if (config.type === 'postgresql') {
                        await client.query('COMMIT');
                    }
                    else if (config.type === 'sqlite') {
                        client.exec('COMMIT');
                    }
                }
            }
            catch (e) {
                // Transaction-level error (e.g. connection lost, deadlock)
                // First try to rollback if transaction is still active
                if (config.type === 'mysql') {
                    try {
                        await client.rollback();
                    }
                    catch { }
                }
                else if (config.type === 'postgresql') {
                    try {
                        await client.query('ROLLBACK');
                    }
                    catch { }
                }
                else if (config.type === 'sqlite') {
                    try {
                        client.exec('ROLLBACK');
                    }
                    catch { }
                }
                errors.push(`Transaction failed: ${e.message}`);
            }
        }
        else {
            for (let i = 0; i < options.diffs.length; i += batchSize) {
                await processBatch(options.diffs.slice(i, i + batchSize));
            }
        }
        return {
            success: errors.length === 0,
            inserted,
            updated,
            deleted,
            errors,
            duration: Date.now() - startTime,
        };
    }
    // ============ Helper Methods for Sync ============
    async getAllTableData(id, tableName, columns, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const colStr = columns && columns.length > 0 ? columns.map(c => this.quoteIdentifier(c, config.type)).join(', ') : '*';
        const tableRef = this.quoteTableName(tableName, config.type, dbName);
        const sql = `SELECT ${colStr} FROM ${tableRef}`;
        switch (config.type) {
            case 'mysql': {
                const [rows] = await client.query(sql);
                return rows;
            }
            case 'postgresql': {
                const res = await client.query(sql);
                return res.rows;
            }
            case 'sqlite': {
                return client.prepare(sql).all();
            }
            default:
                return [];
        }
    }
    getRowKey(row, primaryKeys) {
        return primaryKeys.map(pk => {
            const val = row[pk];
            if (val === null)
                return '<NULL>';
            if (val === undefined)
                return '<UNDEF>';
            return String(val);
        }).join('|||');
    }
    extractPrimaryKey(row, primaryKeys) {
        const result = {};
        for (const pk of primaryKeys) {
            result[pk] = row[pk];
        }
        return result;
    }
    rowsDiffer(source, target, columns) {
        for (const col of columns) {
            const s = source[col];
            const t = target[col];
            if (s === null && t === null)
                continue;
            if (s === null || t === null)
                return true;
            // Deep compare for objects/arrays (JSON columns)
            if (typeof s === 'object' || typeof t === 'object') {
                if (JSON.stringify(s) !== JSON.stringify(t))
                    return true;
            }
            else if (String(s) !== String(t)) {
                return true;
            }
        }
        return false;
    }
    compareColumns(source, target) {
        const differences = [];
        // Normalize type names for comparison (MySQL 'varchar' vs PG 'character varying')
        const normalizeType = (t) => t.toLowerCase()
            .replace(/character varying/g, 'varchar')
            .replace(/integer/g, 'int')
            .replace(/boolean/g, 'bool')
            .replace(/\s+/g, '')
            .replace(/\(\d+(?:,\d+)?\)/g, '');
        if (normalizeType(source.type) !== normalizeType(target.type))
            differences.push('type');
        if (source.nullable !== target.nullable)
            differences.push('nullable');
        if (String(source.defaultValue) !== String(target.defaultValue))
            differences.push('default');
        return differences;
    }
    /** Build column type string with length/precision/scale for MySQL */
    buildColumnType(col, targetType) {
        let type = col.type.toUpperCase();
        if (targetType === 'mysql' && col.length !== null && col.length !== undefined) {
            const lenType = /^(varchar|char|nvarchar|nchar|text|mediumtext|longtext)$/i;
            if (lenType.test(type)) {
                type += `(${col.length})`;
            }
            // Decimal/numeric types with precision
            const decType = /^(decimal|numeric|float|double)$/i;
            if (decType.test(type)) {
                if (col.decimals !== null && col.decimals !== undefined) {
                    type += `(${col.length},${col.decimals})`;
                }
                else {
                    type += `(${col.length})`;
                }
            }
        }
        // PostgreSQL: handle varchar/char length
        if (targetType === 'postgresql' && col.length !== null && col.length !== undefined) {
            const lenType = /^(varchar|char|character varying)$/i;
            if (lenType.test(type)) {
                type += `(${col.length})`;
            }
        }
        return type;
    }
    /** Escape a default value for SQL */
    formatDefaultValue(dv) {
        // SQL expressions and function calls — no quotes
        if (/^\d+(\.\d+)?$/.test(dv) || /^(null|true|false|current_timestamp|current_date|current_time|now\(\))$/i.test(dv)) {
            return dv;
        }
        // CURRENT_TIMESTAMP with optional precision: CURRENT_TIMESTAMP(3)
        if (/^current_timestamp(\(\d+\))?$/i.test(dv)) {
            return dv;
        }
        // String value — quote and escape
        return `'${dv.replace(/'/g, "''")}'`;
    }
    generateCreateTableSql(structure, targetType) {
        const colDefs = [];
        for (const col of structure.columns) {
            let def = `  ${this.quoteIdentifier(col.name, targetType)} ${this.buildColumnType(col, targetType)}`;
            // NOT NULL
            if (!col.nullable && !col.isAutoIncrement) {
                def += ' NOT NULL';
            }
            // AUTO_INCREMENT (MySQL)
            if (col.isAutoIncrement && targetType === 'mysql') {
                def += ' AUTO_INCREMENT';
            }
            // DEFAULT value
            if (col.defaultValue !== null && col.defaultValue !== undefined && !col.isAutoIncrement) {
                def += ` DEFAULT ${this.formatDefaultValue(String(col.defaultValue))}`;
            }
            // COMMENT (MySQL inline)
            if (col.comment && targetType === 'mysql') {
                def += ` COMMENT '${col.comment.replace(/'/g, "''")}'`;
            }
            // PRIMARY KEY inline (MySQL single-column)
            if (col.isPrimaryKey && structure.primaryKey.length === 1) {
                def += ' PRIMARY KEY';
            }
            colDefs.push(def);
        }
        // Composite PRIMARY KEY
        if (structure.primaryKey.length > 1) {
            colDefs.push(`  PRIMARY KEY (${structure.primaryKey.map(k => this.quoteIdentifier(k, targetType)).join(', ')})`);
        }
        let sql = `CREATE TABLE ${this.quoteIdentifier(structure.tableName, targetType)} (\n${colDefs.join(',\n')}\n);`;
        // Indexes (non-primary) — separate CREATE INDEX statements
        for (const idx of structure.indexes) {
            if (idx.isPrimary)
                continue;
            const idxSql = this.generateCreateIndexSql(structure.tableName, idx, targetType);
            sql += `\n${idxSql}`;
        }
        // PostgreSQL: COMMENT ON COLUMN
        if (targetType === 'postgresql') {
            const tableRef = this.quoteIdentifier(structure.tableName, targetType);
            const comments = structure.columns
                .filter(col => col.comment)
                .map(col => `COMMENT ON COLUMN ${tableRef}.${this.quoteIdentifier(col.name, targetType)} IS '${col.comment.replace(/'/g, "''")}';`);
            if (comments.length > 0) {
                sql += '\n' + comments.join('\n');
            }
        }
        return sql;
    }
    generateAddColumnSql(tableName, column, targetType) {
        let sql = `ALTER TABLE ${this.quoteIdentifier(tableName, targetType)} ADD COLUMN ${this.quoteIdentifier(column.name, targetType)} ${this.buildColumnType(column, targetType)}`;
        if (!column.nullable && !column.isAutoIncrement) {
            sql += ' NOT NULL';
        }
        if (column.isAutoIncrement && targetType === 'mysql') {
            sql += ' AUTO_INCREMENT';
        }
        if (column.defaultValue !== null && column.defaultValue !== undefined && !column.isAutoIncrement) {
            sql += ` DEFAULT ${this.formatDefaultValue(String(column.defaultValue))}`;
        }
        else if (column.defaultValue === null && !column.nullable && !column.isAutoIncrement) {
            sql = `-- ${sql} NOT NULL (may fail on non-empty table without DEFAULT value)`;
        }
        if (column.comment && targetType === 'mysql') {
            sql += ` COMMENT '${column.comment.replace(/'/g, "''")}'`;
        }
        return sql;
    }
    generateModifyColumnSql(tableName, source, target, targetType, differences) {
        const colName = this.quoteIdentifier(source.name, targetType);
        if (targetType === 'postgresql') {
            // PostgreSQL doesn't support MODIFY COLUMN, needs separate ALTER statements
            const tableRef = this.quoteIdentifier(tableName, targetType);
            const stmts = [];
            if (differences.includes('type')) {
                stmts.push(`ALTER TABLE ${tableRef} ALTER COLUMN ${colName} TYPE ${this.buildColumnType(source, targetType)}`);
            }
            if (differences.includes('nullable')) {
                stmts.push(source.nullable
                    ? `ALTER TABLE ${tableRef} ALTER COLUMN ${colName} DROP NOT NULL`
                    : `ALTER TABLE ${tableRef} ALTER COLUMN ${colName} SET NOT NULL`);
            }
            if (differences.includes('default')) {
                if (source.defaultValue !== null && source.defaultValue !== undefined) {
                    stmts.push(`ALTER TABLE ${tableRef} ALTER COLUMN ${colName} SET DEFAULT ${this.formatDefaultValue(String(source.defaultValue))}`);
                }
                else {
                    stmts.push(`ALTER TABLE ${tableRef} ALTER COLUMN ${colName} DROP DEFAULT`);
                }
            }
            return stmts.join('; ');
        }
        if (targetType === 'sqlite') {
            return `-- ALTER TABLE not fully supported in SQLite for MODIFY COLUMN`;
        }
        // MySQL: MODIFY COLUMN
        const colType = this.buildColumnType(source, targetType);
        const stmts = [];
        // When adding NOT NULL to a column that was previously nullable, prepend UPDATE to clear existing NULLs
        // This prevents "Invalid use of NULL value" error during ALTER TABLE
        if (differences.includes('nullable') && !source.nullable && target.nullable) {
            if (source.defaultValue !== null && source.defaultValue !== undefined) {
                stmts.push(`UPDATE ${this.quoteIdentifier(tableName, targetType)} SET ${colName} = ${this.formatDefaultValue(String(source.defaultValue))} WHERE ${colName} IS NULL`);
            }
            else {
                return `-- ALTER TABLE ${this.quoteIdentifier(tableName, targetType)} MODIFY COLUMN ${colName} ${colType} NOT NULL (may fail — existing NULL values found, no DEFAULT value available)`;
            }
        }
        let sql = `ALTER TABLE ${this.quoteIdentifier(tableName, targetType)} MODIFY COLUMN ${colName} ${colType}${source.nullable ? '' : ' NOT NULL'}`;
        if (source.isAutoIncrement) {
            sql += ' AUTO_INCREMENT';
        }
        if (differences.includes('default') && source.defaultValue !== null && source.defaultValue !== undefined) {
            sql += ` DEFAULT ${this.formatDefaultValue(String(source.defaultValue))}`;
        }
        else if (differences.includes('default') && (source.defaultValue === null || source.defaultValue === undefined)) {
            sql += ` DROP DEFAULT`;
        }
        if (source.comment) {
            sql += ` COMMENT '${source.comment.replace(/'/g, "''")}'`;
        }
        stmts.push(sql);
        return stmts.join('; ');
    }
    generateCreateIndexSql(tableName, index, targetType) {
        const unique = index.isUnique ? 'UNIQUE ' : '';
        const cols = index.columns.map(c => this.quoteIdentifier(c, targetType)).join(', ');
        return `CREATE ${unique}INDEX ${this.quoteIdentifier(index.name, targetType)} ON ${this.quoteIdentifier(tableName, targetType)} (${cols});`;
    }
    escapeSqlValue(val) {
        if (val === null || val === undefined)
            return 'NULL';
        if (typeof val === 'number') {
            // Large numbers that exceed JS safe integer range — stringify and quote
            if (!Number.isSafeInteger(val) && Number.isFinite(val)) {
                const escaped = String(val).replace(/\\/g, '\\\\').replace(/'/g, "''");
                return `'${escaped}'`;
            }
            return String(val);
        }
        // Arrays: JSON-encode to preserve structure, don't flatten into comma-separated bare values
        if (Array.isArray(val)) {
            const json = JSON.stringify(val).replace(/\\/g, '\\\\').replace(/'/g, "''");
            return `'${json}'`;
        }
        // Objects: JSON-encode
        if (typeof val === 'object' && val !== null) {
            const json = JSON.stringify(val).replace(/\\/g, '\\\\').replace(/'/g, "''");
            return `'${json}'`;
        }
        const escaped = String(val)
            .replace(/\\/g, '\\\\')
            .replace(/'/g, "''")
            .replace(/\n/g, '\\n')
            .replace(/\r/g, '\\r');
        return `'${escaped}'`;
    }
    generateInsertSql(tableName, row, dbType, dbName, onConflict = 'error') {
        const columns = Object.keys(row);
        const values = columns.map(col => this.escapeSqlValue(row[col]));
        const tableRef = this.quoteTableName(tableName, dbType, dbName);
        const colList = columns.map(c => this.quoteIdentifier(c, dbType)).join(', ');
        let keyword = 'INSERT';
        switch (dbType) {
            case 'mysql':
                if (onConflict === 'ignore')
                    keyword = 'INSERT IGNORE';
                else if (onConflict === 'replace')
                    keyword = 'REPLACE';
                break;
            case 'sqlite':
                if (onConflict === 'ignore')
                    keyword = 'INSERT OR IGNORE';
                else if (onConflict === 'replace')
                    keyword = 'INSERT OR REPLACE';
                break;
            case 'postgresql':
                // PostgreSQL uses ON CONFLICT syntax, handle below
                break;
        }
        let sql = `${keyword} INTO ${tableRef} (${colList}) VALUES (${values.join(', ')});`;
        // PostgreSQL ON CONFLICT
        if (dbType === 'postgresql' && onConflict === 'ignore') {
            sql = `INSERT INTO ${tableRef} (${colList}) VALUES (${values.join(', ')}) ON CONFLICT DO NOTHING;`;
        }
        return sql;
    }
    generateUpdateSql(tableName, row, primaryKeys, dbType, dbName) {
        const columns = Object.keys(row).filter(c => !primaryKeys.includes(c));
        if (columns.length === 0) {
            throw new Error(`No non-primary-key columns found to update in row for table '${tableName}'`);
        }
        const setClause = columns.map(col => {
            const valStr = this.escapeSqlValue(row[col]);
            return `${this.quoteIdentifier(col, dbType)} = ${valStr}`;
        }).join(', ');
        const whereClause = primaryKeys.map(pk => {
            const val = row[pk];
            if (val === null || val === undefined) {
                return `${this.quoteIdentifier(pk, dbType)} IS NULL`;
            }
            // Large integers (snowflake/bigint IDs) exceed JS safe integer range — treat as strings
            const isSafeNumber = typeof val === 'number' && Number.isSafeInteger(val);
            if (isSafeNumber) {
                return `${this.quoteIdentifier(pk, dbType)} = ${val}`;
            }
            const escaped = String(val).replace(/\\/g, '\\\\').replace(/'/g, "''").replace(/\n/g, '\\n').replace(/\r/g, '\\r');
            return `${this.quoteIdentifier(pk, dbType)} = '${escaped}'`;
        }).join(' AND ');
        const tableRef = this.quoteTableName(tableName, dbType, dbName);
        return `UPDATE ${tableRef} SET ${setClause} WHERE ${whereClause};`;
    }
    generateDeleteSql(tableName, primaryKey, primaryKeys, dbType, dbName) {
        const whereClause = primaryKeys.map(pk => {
            const val = primaryKey[pk];
            if (val === null || val === undefined) {
                return `${this.quoteIdentifier(pk, dbType)} IS NULL`;
            }
            // Large integers (snowflake/bigint IDs) exceed JS safe integer range — treat as strings
            const isSafeNumber = typeof val === 'number' && Number.isSafeInteger(val);
            if (isSafeNumber) {
                return `${this.quoteIdentifier(pk, dbType)} = ${val}`;
            }
            return `${this.quoteIdentifier(pk, dbType)} = ${this.escapeSqlValue(val)}`;
        }).join(' AND ');
        const tableRef = this.quoteTableName(tableName, dbType, dbName);
        return `DELETE FROM ${tableRef} WHERE ${whereClause};`;
    }
    quoteIdentifier(identifier, dbType) {
        switch (dbType) {
            case 'mysql':
                return `\`${identifier.replace(/`/g, '``')}\``;
            case 'postgresql':
            case 'sqlite':
                return `"${identifier.replace(/"/g, '""')}"`;
            default:
                return `"${identifier.replace(/"/g, '""')}"`;
        }
    }
    quoteTableName(tableName, dbType, dbName) {
        if (dbName) {
            switch (dbType) {
                case 'mysql':
                    return `\`${dbName.replace(/`/g, '``')}\`.\`${tableName.replace(/`/g, '``')}\``;
                case 'postgresql':
                    return `"${dbName.replace(/"/g, '""')}"."${tableName.replace(/"/g, '""')}"`;
                default:
                    return `"${tableName.replace(/"/g, '""')}"`;
            }
        }
        return this.quoteIdentifier(tableName, dbType);
    }
    async executeSql(client, dbType, sql) {
        switch (dbType) {
            case 'mysql':
                await client.query(sql);
                break;
            case 'postgresql':
                await client.query(sql);
                break;
            case 'sqlite':
                client.prepare(sql).run();
                break;
            default:
                throw new Error(`Unsupported database type: ${dbType}`);
        }
    }
    // ============ Table Row CRUD Operations ============
    async getTablePrimaryKeys(id, table, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        switch (config.type) {
            case 'mysql': {
                const escDb = dbName ? dbName.replace(/`/g, '``') : '';
                const escTable = table.replace(/`/g, '``');
                const tableRef = dbName ? `\`${escDb}\`.\`${escTable}\`` : `\`${escTable}\``;
                const [rows] = await client.query(`SHOW INDEX FROM ${tableRef}`);
                return rows
                    .filter((r) => r.Key_name === 'PRIMARY')
                    .sort((a, b) => a.Seq_in_index - b.Seq_in_index)
                    .map((r) => r.Column_name);
            }
            case 'postgresql': {
                const schema = dbName || 'public';
                const res = await client.query(`SELECT kcu.column_name
           FROM information_schema.table_constraints tc
           JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
             AND tc.table_schema = kcu.table_schema
           WHERE tc.constraint_type = 'PRIMARY KEY'
             AND tc.table_schema = $1 AND tc.table_name = $2
           ORDER BY kcu.ordinal_position`, [schema, table]);
                return res.rows.map((r) => r.column_name);
            }
            case 'sqlite': {
                const escTable = table.replace(/"/g, '""');
                const rows = client.prepare(`PRAGMA table_info("${escTable}")`).all();
                return rows.filter((r) => r.pk > 0).sort((a, b) => a.pk - b.pk).map((r) => r.name);
            }
            default:
                return [];
        }
    }
    async updateTableRow(id, table, oldRow, newRow, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const pks = await this.getTablePrimaryKeys(id, table, dbName);
        if (pks.length === 0)
            throw new Error(`Table '${table}' has no primary key — cannot update without one`);
        // Build SET clause (only changed columns)
        const setColumns = [];
        const setParams = [];
        for (const col of Object.keys(newRow)) {
            if (pks.includes(col))
                continue; // Don't update primary key columns
            const oldVal = oldRow[col];
            const newVal = newRow[col];
            // Skip if values are identical (handle null === null correctly)
            if (oldVal === newVal)
                continue;
            if (oldVal === null && newVal === null)
                continue;
            setColumns.push(`${this.quoteIdentifier(col, config.type)} = ?`);
            setParams.push(newVal);
        }
        if (setColumns.length === 0)
            return true; // No changes detected
        // Build WHERE clause using primary key values from OLD row
        const whereClause = [];
        const whereParams = [];
        for (const pk of pks) {
            const val = oldRow[pk];
            if (val === null) {
                whereClause.push(`${this.quoteIdentifier(pk, config.type)} IS NULL`);
            }
            else {
                whereClause.push(`${this.quoteIdentifier(pk, config.type)} = ?`);
                whereParams.push(val);
            }
        }
        switch (config.type) {
            case 'mysql': {
                const escDb = dbName ? dbName.replace(/`/g, '``') : '';
                const escTable = table.replace(/`/g, '``');
                const tableRef = dbName ? `\`${escDb}\`.\`${escTable}\`` : `\`${escTable}\``;
                // MySQL JSON/BLOB columns require string values — serialize objects/arrays to JSON strings
                const mysqlParams = setParams.map(v => {
                    if (v !== null && typeof v === 'object')
                        return JSON.stringify(v);
                    return v;
                });
                const sql = `UPDATE ${tableRef} SET ${setColumns.join(', ')} WHERE ${whereClause.join(' AND ')}`;
                await client.query(sql, [...mysqlParams, ...whereParams]);
                return true;
            }
            case 'postgresql': {
                const schema = dbName || 'public';
                const escSchema = schema.replace(/"/g, '""');
                const escTable = table.replace(/"/g, '""');
                const tableRef = `"${escSchema}"."${escTable}"`;
                const allParams = [...setParams, ...whereParams];
                const setWithPlaceholders = setColumns.map((_, i) => {
                    const col = setColumns[i].split(' = ')[0];
                    return `${col} = $${i + 1}`;
                }).join(', ');
                const whereWithPlaceholders = whereClause.map((_, i) => {
                    const pk = pks[i];
                    if (oldRow[pk] === null) {
                        return `${this.quoteIdentifier(pk, config.type)} IS NULL`;
                    }
                    return `${this.quoteIdentifier(pk, config.type)} = $${setParams.length + i + 1}`;
                }).join(' AND ');
                const sql = `UPDATE ${tableRef} SET ${setWithPlaceholders} WHERE ${whereWithPlaceholders}`;
                await client.query(sql, allParams);
                return true;
            }
            case 'sqlite': {
                const escTable = table.replace(/"/g, '""');
                const sql = `UPDATE "${escTable}" SET ${setColumns.join(', ')} WHERE ${whereClause.join(' AND ')}`;
                client.prepare(sql).run(...setParams, ...whereParams);
                return true;
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
    }
    async insertTableRow(id, table, row, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const columns = Object.keys(row);
        const values = columns.map(c => row[c]);
        switch (config.type) {
            case 'mysql': {
                const escDb = dbName ? dbName.replace(/`/g, '``') : '';
                const escTable = table.replace(/`/g, '``');
                const tableRef = dbName ? `\`${escDb}\`.\`${escTable}\`` : `\`${escTable}\``;
                const placeholders = columns.map(() => '?').join(', ');
                const colList = columns.map(c => this.quoteIdentifier(c, config.type)).join(', ');
                const sql = `INSERT INTO ${tableRef} (${colList}) VALUES (${placeholders})`;
                // MySQL JSON columns require string values — serialize objects/arrays to JSON strings
                const mysqlValues = values.map(v => {
                    if (v !== null && typeof v === 'object')
                        return JSON.stringify(v);
                    return v;
                });
                await client.query(sql, mysqlValues);
                return true;
            }
            case 'postgresql': {
                const schema = dbName || 'public';
                const escSchema = schema.replace(/"/g, '""');
                const escTable = table.replace(/"/g, '""');
                const tableRef = `"${escSchema}"."${escTable}"`;
                const colList = columns.map(c => `"${c.replace(/"/g, '""')}"`).join(', ');
                const placeholders = columns.map((_, i) => `$${i + 1}`).join(', ');
                const sql = `INSERT INTO ${tableRef} (${colList}) VALUES (${placeholders})`;
                await client.query(sql, values);
                return true;
            }
            case 'sqlite': {
                const escTable = table.replace(/"/g, '""');
                const colList = columns.map(c => `"${c.replace(/"/g, '""')}"`).join(', ');
                const placeholders = columns.map(() => '?').join(', ');
                const sql = `INSERT INTO "${escTable}" (${colList}) VALUES (${placeholders})`;
                client.prepare(sql).run(...values);
                return true;
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
    }
    async deleteTableRow(id, table, row, dbName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const pks = await this.getTablePrimaryKeys(id, table, dbName);
        if (pks.length === 0)
            throw new Error(`Table '${table}' has no primary key — cannot delete without one`);
        const whereClause = [];
        const params = [];
        for (const pk of pks) {
            const val = row[pk];
            if (val === null) {
                whereClause.push(`${this.quoteIdentifier(pk, config.type)} IS NULL`);
            }
            else {
                whereClause.push(`${this.quoteIdentifier(pk, config.type)} = ?`);
                params.push(val);
            }
        }
        switch (config.type) {
            case 'mysql': {
                const escDb = dbName ? dbName.replace(/`/g, '``') : '';
                const escTable = table.replace(/`/g, '``');
                const tableRef = dbName ? `\`${escDb}\`.\`${escTable}\`` : `\`${escTable}\``;
                const sql = `DELETE FROM ${tableRef} WHERE ${whereClause.join(' AND ')}`;
                await client.query(sql, params);
                return true;
            }
            case 'postgresql': {
                const schema = dbName || 'public';
                const escSchema = schema.replace(/"/g, '""');
                const escTable = table.replace(/"/g, '""');
                const tableRef = `"${escSchema}"."${escTable}"`;
                const pgWhere = whereClause.map((_, i) => {
                    const pk = pks[i];
                    if (row[pk] === null) {
                        return `${this.quoteIdentifier(pk, config.type)} IS NULL`;
                    }
                    return `${this.quoteIdentifier(pk, config.type)} = $${i + 1}`;
                }).join(' AND ');
                const sql = `DELETE FROM ${tableRef} WHERE ${pgWhere}`;
                await client.query(sql, params);
                return true;
            }
            case 'sqlite': {
                const escTable = table.replace(/"/g, '""');
                const sql = `DELETE FROM "${escTable}" WHERE ${whereClause.join(' AND ')}`;
                client.prepare(sql).run(...params);
                return true;
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
    }
    // ============ Table Structure Alteration ============
    /**
     * Generate column definition SQL fragment for ADD COLUMN / MODIFY COLUMN
     */
    buildColumnDef(column, dbType) {
        let sql = `${this.quoteIdentifier(column.name, dbType)} ${column.type.toUpperCase()}`;
        if (column.autoIncrement) {
            if (dbType === 'mysql') {
                sql += ' AUTO_INCREMENT';
            }
            else if (dbType === 'postgresql') {
                // PostgreSQL uses GENERATED ALWAYS AS IDENTITY instead of AUTO_INCREMENT
                sql += ' GENERATED ALWAYS AS IDENTITY';
            }
        }
        if (column.primaryKey) {
            sql += ' PRIMARY KEY';
        }
        if (!column.nullable && !column.primaryKey) {
            sql += ' NOT NULL';
        }
        if (column.defaultValue !== undefined && column.defaultValue !== null) {
            // If defaultValue is already quoted (e.g., a string literal), use as-is
            // Otherwise treat as expression
            sql += ` DEFAULT ${column.defaultValue}`;
        }
        if (column.comment && dbType === 'mysql') {
            sql += ` COMMENT '${column.comment.replace(/'/g, "''")}'`;
        }
        return sql;
    }
    /**
     * Build full table reference like `db`.`table` or "schema"."table" or "table"
     */
    buildTableRef(tableName, dbName, dbType) {
        const escT = tableName.replace(/`/g, '``').replace(/"/g, '""');
        switch (dbType) {
            case 'mysql':
                return dbName
                    ? `\`${dbName.replace(/`/g, '``')}\`.\`${escT}\``
                    : `\`${escT}\``;
            case 'postgresql':
                return dbName
                    ? `"${dbName.replace(/"/g, '""')}"."${escT}"`
                    : `"${escT}"`;
            case 'sqlite':
                return `"${escT}"`;
            default:
                return `"${escT}"`;
        }
    }
    async addColumn(id, dbName, tableName, column) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const tableRef = this.buildTableRef(tableName, dbName, config.type);
        const colDef = this.buildColumnDef(column, config.type);
        let sql;
        switch (config.type) {
            case 'mysql':
                sql = `ALTER TABLE ${tableRef} ADD COLUMN ${colDef}`;
                break;
            case 'postgresql':
                // PostgreSQL doesn't support COMMENT in ADD COLUMN, use separate statement
                sql = `ALTER TABLE ${tableRef} ADD COLUMN ${colDef}`;
                if (column.comment) {
                    const commentSql = `COMMENT ON COLUMN ${tableRef}.${this.quoteIdentifier(column.name, config.type)} IS '${column.comment.replace(/'/g, "''")}'`;
                    await client.query(commentSql);
                }
                break;
            case 'sqlite': {
                // SQLite has limited ALTER TABLE support
                // SQLite 3.35.0+ supports DROP COLUMN and ALTER COLUMN but ADD COLUMN is the most compatible
                sql = `ALTER TABLE ${tableRef} ADD COLUMN ${colDef}`;
                break;
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
        await this.executeSql(client, config.type, sql);
        return { success: true, sql };
    }
    async modifyColumn(id, dbName, tableName, oldColumn, newColumn) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const tableRef = this.buildTableRef(tableName, dbName, config.type);
        const colDef = this.buildColumnDef(newColumn, config.type);
        let sql;
        switch (config.type) {
            case 'mysql':
                // MySQL uses MODIFY COLUMN or CHANGE COLUMN
                if (oldColumn !== newColumn.name) {
                    // Column name is changing, use CHANGE
                    sql = `ALTER TABLE ${tableRef} CHANGE COLUMN ${this.quoteIdentifier(oldColumn, config.type)} ${colDef}`;
                }
                else {
                    sql = `ALTER TABLE ${tableRef} MODIFY COLUMN ${colDef}`;
                }
                break;
            case 'postgresql': {
                // PostgreSQL uses ALTER COLUMN and requires separate statements for different changes
                const statements = [];
                // Rename if name changed
                if (oldColumn !== newColumn.name) {
                    statements.push(`ALTER TABLE ${tableRef} RENAME COLUMN ${this.quoteIdentifier(oldColumn, config.type)} TO ${this.quoteIdentifier(newColumn.name, config.type)}`);
                }
                // Change type
                const colName = this.quoteIdentifier(newColumn.name, config.type);
                statements.push(`ALTER TABLE ${tableRef} ALTER COLUMN ${colName} TYPE ${newColumn.type.toUpperCase()}`);
                // Change nullable
                if (newColumn.nullable) {
                    statements.push(`ALTER TABLE ${tableRef} ALTER COLUMN ${colName} DROP NOT NULL`);
                }
                else {
                    statements.push(`ALTER TABLE ${tableRef} ALTER COLUMN ${colName} SET NOT NULL`);
                }
                // Change default
                if (newColumn.defaultValue !== undefined && newColumn.defaultValue !== null) {
                    statements.push(`ALTER TABLE ${tableRef} ALTER COLUMN ${colName} SET DEFAULT ${newColumn.defaultValue}`);
                }
                else {
                    statements.push(`ALTER TABLE ${tableRef} ALTER COLUMN ${colName} DROP DEFAULT`);
                }
                // Execute all statements
                for (const stmt of statements) {
                    await client.query(stmt);
                }
                // Comment
                if (newColumn.comment) {
                    const commentSql = `COMMENT ON COLUMN ${tableRef}.${colName} IS '${newColumn.comment.replace(/'/g, "''")}'`;
                    await client.query(commentSql);
                }
                return { success: true, sql: statements.join('; ') };
            }
            case 'sqlite': {
                // SQLite does NOT support MODIFY COLUMN directly
                // Workaround: recreate the table (complex and risky)
                // For safety, we just throw an error and let the caller know
                throw new Error('SQLite does not support MODIFY COLUMN. You need to recreate the table manually.');
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
        await this.executeSql(client, config.type, sql);
        return { success: true, sql };
    }
    async dropColumn(id, dbName, tableName, columnName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const tableRef = this.buildTableRef(tableName, dbName, config.type);
        let sql;
        switch (config.type) {
            case 'mysql':
                sql = `ALTER TABLE ${tableRef} DROP COLUMN ${this.quoteIdentifier(columnName, config.type)}`;
                break;
            case 'postgresql':
                sql = `ALTER TABLE ${tableRef} DROP COLUMN ${this.quoteIdentifier(columnName, config.type)}`;
                break;
            case 'sqlite': {
                // SQLite 3.35.0+ (2021-03-12) supports DROP COLUMN
                sql = `ALTER TABLE ${tableRef} DROP COLUMN ${this.quoteIdentifier(columnName, config.type)}`;
                break;
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
        await this.executeSql(client, config.type, sql);
        return { success: true, sql };
    }
    async renameColumn(id, dbName, tableName, oldName, newName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const tableRef = this.buildTableRef(tableName, dbName, config.type);
        let sql;
        switch (config.type) {
            case 'mysql':
                // MySQL CHANGE COLUMN requires full column definition
                // Fetch current column info to preserve type/constraints
                // Use dbName if provided, otherwise DATABASE()
                const dbCond = dbName ? `TABLE_SCHEMA = ?` : `TABLE_SCHEMA = DATABASE()`;
                const dbPrms = dbName ? [dbName, tableName, oldName] : [tableName, oldName];
                const [colInfoRows] = await client.query(`SELECT COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, EXTRA, COLUMN_COMMENT
           FROM INFORMATION_SCHEMA.COLUMNS
           WHERE ${dbCond} AND TABLE_NAME = ? AND COLUMN_NAME = ?`, dbPrms);
                if (colInfoRows.length === 0) {
                    throw new Error(`Column '${oldName}' not found in table '${tableName}'`);
                }
                const ci = colInfoRows[0];
                const nullable = ci.IS_NULLABLE === 'YES' ? '' : ' NOT NULL';
                const defaultVal = ci.COLUMN_DEFAULT !== null && ci.COLUMN_DEFAULT !== undefined ? ` DEFAULT ${ci.COLUMN_DEFAULT}` : '';
                const autoInc = ci.EXTRA?.includes('auto_increment') ? ' AUTO_INCREMENT' : '';
                const comment = ci.COLUMN_COMMENT ? ` COMMENT '${ci.COLUMN_COMMENT.replace(/'/g, "''")}'` : '';
                sql = `ALTER TABLE ${tableRef} CHANGE COLUMN ${this.quoteIdentifier(oldName, config.type)} ${this.quoteIdentifier(newName, config.type)} ${ci.COLUMN_TYPE}${nullable}${defaultVal}${autoInc}${comment}`;
                break;
            case 'postgresql':
                sql = `ALTER TABLE ${tableRef} RENAME COLUMN ${this.quoteIdentifier(oldName, config.type)} TO ${this.quoteIdentifier(newName, config.type)}`;
                break;
            case 'sqlite': {
                // SQLite 3.25.0+ (2018-09-15) supports RENAME COLUMN
                sql = `ALTER TABLE ${tableRef} RENAME COLUMN ${this.quoteIdentifier(oldName, config.type)} TO ${this.quoteIdentifier(newName, config.type)}`;
                break;
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
        await this.executeSql(client, config.type, sql);
        return { success: true, sql };
    }
    async addIndex(id, dbName, tableName, indexDef) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const tableRef = this.buildTableRef(tableName, dbName, config.type);
        const cols = indexDef.columns.map(c => this.quoteIdentifier(c, config.type)).join(', ');
        const unique = indexDef.unique ? 'UNIQUE ' : '';
        let sql;
        switch (config.type) {
            case 'mysql': {
                const indexType = indexDef.type ? `${indexDef.type.toUpperCase()} ` : '';
                sql = `CREATE ${unique}${indexType}INDEX ${this.quoteIdentifier(indexDef.name, config.type)} ON ${tableRef} (${cols})`;
                break;
            }
            case 'postgresql': {
                // PostgreSQL index type: USING type
                const using = indexDef.type ? ` USING ${indexDef.type}` : '';
                sql = `CREATE ${unique}INDEX ${this.quoteIdentifier(indexDef.name, config.type)} ON ${tableRef}${using} (${cols})`;
                break;
            }
            case 'sqlite': {
                // SQLite doesn't support index type
                sql = `CREATE ${unique}INDEX ${this.quoteIdentifier(indexDef.name, config.type)} ON ${tableRef} (${cols})`;
                break;
            }
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
        await this.executeSql(client, config.type, sql);
        return { success: true, sql };
    }
    async dropIndex(id, dbName, tableName, indexName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        let sql;
        switch (config.type) {
            case 'mysql':
                sql = `DROP INDEX ${this.quoteIdentifier(indexName, config.type)} ON ${this.buildTableRef(tableName, dbName, config.type)}`;
                break;
            case 'postgresql':
                // PostgreSQL needs schema-qualified index name
                const schema = dbName || 'public';
                const escSchema = schema.replace(/"/g, '""');
                sql = `DROP INDEX IF EXISTS "${escSchema}".${this.quoteIdentifier(indexName, config.type)}`;
                break;
            case 'sqlite':
                sql = `DROP INDEX IF EXISTS ${this.quoteIdentifier(indexName, config.type)}`;
                break;
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
        await this.executeSql(client, config.type, sql);
        return { success: true, sql };
    }
    async renameTable(id, dbName, oldName, newName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        let sql;
        switch (config.type) {
            case 'mysql':
                sql = `RENAME TABLE ${this.quoteIdentifier(oldName, config.type)} TO ${this.quoteIdentifier(newName, config.type)}`;
                break;
            case 'postgresql':
                sql = `ALTER TABLE ${this.quoteIdentifier(oldName, config.type)} RENAME TO ${this.quoteIdentifier(newName, config.type)}`;
                break;
            case 'sqlite':
                sql = `ALTER TABLE ${this.quoteIdentifier(oldName, config.type)} RENAME TO ${this.quoteIdentifier(newName, config.type)}`;
                break;
            default:
                throw new Error(`Unsupported database type: ${config.type}`);
        }
        await this.executeSql(client, config.type, sql);
        return { success: true, sql };
    }
    async alterTable(id, dbName, tableName, operations) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        const sqls = [];
        const errors = [];
        // Begin transaction
        const beginTransaction = async () => {
            switch (config.type) {
                case 'mysql':
                    await client.beginTransaction();
                    break;
                case 'postgresql':
                    await client.query('BEGIN');
                    break;
                case 'sqlite':
                    client.prepare('BEGIN TRANSACTION').run();
                    break;
            }
        };
        const commitTransaction = async () => {
            switch (config.type) {
                case 'mysql':
                    await client.commit();
                    break;
                case 'postgresql':
                    await client.query('COMMIT');
                    break;
                case 'sqlite':
                    client.prepare('COMMIT').run();
                    break;
            }
        };
        const rollbackTransaction = async () => {
            try {
                switch (config.type) {
                    case 'mysql':
                        await client.rollback();
                        break;
                    case 'postgresql':
                        await client.query('ROLLBACK');
                        break;
                    case 'sqlite':
                        client.prepare('ROLLBACK').run();
                        break;
                }
            }
            catch {
                // Ignore rollback errors
            }
        };
        try {
            await beginTransaction();
            for (const op of operations) {
                try {
                    switch (op.type) {
                        case 'addColumn':
                            if (!op.column)
                                throw new Error('addColumn requires column definition');
                            const addResult = await this.addColumn(id, dbName, tableName, op.column);
                            sqls.push(addResult.sql);
                            break;
                        case 'dropColumn':
                            if (!op.columnName)
                                throw new Error('dropColumn requires columnName');
                            const dropResult = await this.dropColumn(id, dbName, tableName, op.columnName);
                            sqls.push(dropResult.sql);
                            break;
                        case 'renameColumn':
                            if (!op.oldColumn || !op.newColumn)
                                throw new Error('renameColumn requires oldColumn and newColumn');
                            const renameResult = await this.renameColumn(id, dbName, tableName, op.oldColumn, op.newColumn);
                            sqls.push(renameResult.sql);
                            break;
                        case 'modifyColumn':
                            if (!op.oldColumn || !op.column)
                                throw new Error('modifyColumn requires oldColumn and new column definition');
                            const modifyResult = await this.modifyColumn(id, dbName, tableName, op.oldColumn, op.column);
                            sqls.push(modifyResult.sql);
                            break;
                        case 'addIndex':
                            if (!op.indexDef)
                                throw new Error('addIndex requires indexDef');
                            const addIndexResult = await this.addIndex(id, dbName, tableName, op.indexDef);
                            sqls.push(addIndexResult.sql);
                            break;
                        case 'dropIndex':
                            if (!op.indexName)
                                throw new Error('dropIndex requires indexName');
                            const dropIndexResult = await this.dropIndex(id, dbName, tableName, op.indexName);
                            sqls.push(dropIndexResult.sql);
                            break;
                        case 'renameTable':
                            if (!op.newName)
                                throw new Error('renameTable requires newName');
                            const renameTableResult = await this.renameTable(id, dbName, tableName, op.newName);
                            sqls.push(renameTableResult.sql);
                            break;
                        default:
                            throw new Error(`Unknown operation type: ${op.type}`);
                    }
                }
                catch (e) {
                    errors.push(`Operation ${op.type} failed: ${e.message}`);
                    // Abort on first error for all database types to prevent partial application
                    throw e;
                }
            }
            await commitTransaction();
            return { success: errors.length === 0, sqls, errors };
        }
        catch (e) {
            await rollbackTransaction();
            if (errors.length === 0) {
                errors.push(`Transaction failed: ${e.message}`);
            }
            return { success: false, sqls, errors };
        }
    }
    // ============ Filtered Query ============
    /**
     * Generate SQL WHERE clause from filter conditions
     * Returns { clause, params } for parameterized queries
     */
    buildWhereClause(filters, dbType) {
        const clauses = [];
        const params = [];
        const isPostgres = dbType === 'postgresql';
        let paramIndex = 0;
        const placeholder = () => {
            if (isPostgres)
                return `$${++paramIndex}`;
            return '?';
        };
        for (let i = 0; i < filters.length; i++) {
            const f = filters[i];
            if (!f.column || !f.operator)
                continue;
            const col = this.quoteIdentifier(f.column, dbType);
            const logic = i > 0 ? ` ${f.logic} ` : '';
            switch (f.operator) {
                case '=':
                case '!=':
                case '>':
                case '<':
                case '>=':
                case '<=':
                    clauses.push(`${logic}${col} ${f.operator} ${placeholder()}`);
                    params.push(this.parseValue(f.value));
                    break;
                case 'LIKE':
                    clauses.push(`${logic}${col} LIKE ${placeholder()}`);
                    // Don't double-add % if user already included them
                    const likeVal = f.value.includes('%') ? f.value : `%${f.value}%`;
                    params.push(likeVal);
                    break;
                case 'NOT LIKE':
                    clauses.push(`${logic}${col} NOT LIKE ${placeholder()}`);
                    const notLikeVal = f.value.includes('%') ? f.value : `%${f.value}%`;
                    params.push(notLikeVal);
                    break;
                case 'IN': {
                    const values = f.value.split(',').map(v => v.trim()).filter(v => v);
                    if (values.length === 0)
                        continue;
                    const placeholders = values.map(() => placeholder()).join(', ');
                    clauses.push(`${logic}${col} IN (${placeholders})`);
                    params.push(...values.map(v => this.parseValue(v)));
                    break;
                }
                case 'NOT IN': {
                    const values = f.value.split(',').map(v => v.trim()).filter(v => v);
                    if (values.length === 0)
                        continue;
                    const placeholders = values.map(() => placeholder()).join(', ');
                    clauses.push(`${logic}${col} NOT IN (${placeholders})`);
                    params.push(...values.map(v => this.parseValue(v)));
                    break;
                }
                case 'BETWEEN':
                    clauses.push(`${logic}${col} BETWEEN ${placeholder()} AND ${placeholder()}`);
                    params.push(this.parseValue(f.value), this.parseValue(f.value2 || ''));
                    break;
                case 'IS NULL':
                    clauses.push(`${logic}${col} IS NULL`);
                    break;
                case 'IS NOT NULL':
                    clauses.push(`${logic}${col} IS NOT NULL`);
                    break;
            }
        }
        return { clause: clauses.join(''), params };
    }
    async getTableDataFiltered(options) {
        const entry = this.connections.get(options.connectionId);
        if (!entry)
            throw new Error(`Connection '${options.connectionId}' not found`);
        const { client, config } = entry;
        const { clause, params } = this.buildWhereClause(options.filters, config.type);
        const whereSql = clause ? ` WHERE ${clause}` : '';
        const colStr = options.columns && options.columns.length > 0
            ? options.columns.map(c => this.quoteIdentifier(c, config.type)).join(', ')
            : '*';
        const tableRef = config.type === 'mysql' && options.dbName
            ? `\`${options.dbName.replace(/`/g, '``')}\`.\`${options.tableName.replace(/`/g, '``')}\``
            : this.quoteIdentifier(options.tableName, config.type);
        switch (config.type) {
            case 'mysql': {
                const conn = client;
                // Count query
                const [countRes] = await conn.query(`SELECT COUNT(*) as total FROM ${tableRef}${whereSql}`, params);
                const total = countRes[0]?.total ?? 0;
                // Data query
                const dataParams = [...params, options.limit, options.offset];
                const [rows] = await conn.query(`SELECT ${colStr} FROM ${tableRef}${whereSql} LIMIT ? OFFSET ?`, dataParams);
                return { rows, total };
            }
            case 'postgresql': {
                const pgClient = client;
                // Count query
                const countRes = await pgClient.query(`SELECT COUNT(*) as total FROM ${tableRef}${whereSql}`, params);
                const total = parseInt(countRes.rows[0]?.total, 10) || 0;
                // Data query
                const dataParams = [...params, options.limit, options.offset];
                const res = await pgClient.query(`SELECT ${colStr} FROM ${tableRef}${whereSql} LIMIT $${dataParams.length - 1} OFFSET $${dataParams.length}`, dataParams);
                return { rows: res.rows, total };
            }
            case 'sqlite': {
                const sqliteClient = client;
                // Count query
                const countSql = `SELECT COUNT(*) as total FROM ${tableRef}${whereSql}`;
                const countStmt = sqliteClient.prepare(countSql);
                const countRes = countStmt.get(...params);
                const total = countRes?.total ?? 0;
                // Data query
                const dataSql = `SELECT ${colStr} FROM ${tableRef}${whereSql} LIMIT ? OFFSET ?`;
                const dataParams = [...params, options.limit, options.offset];
                const rows = sqliteClient.prepare(dataSql).all(...dataParams);
                return { rows, total };
            }
            default:
                return { rows: [], total: 0 };
        }
    }
    parseValue(value) {
        if (value === 'NULL')
            return null;
        if (value === '')
            return '';
        // Try to parse as number — but preserve large integers as strings to avoid precision loss
        // JavaScript Number.MAX_SAFE_INTEGER = 9007199254740991 (16 digits)
        // Snowflake/bigint IDs are 18-19 digits and will lose precision if converted to Number
        if (/^-?\d+(\.\d+)?$/.test(value)) {
            // Check if it's an integer that exceeds safe range
            const integerPart = value.replace(/^-/, '').split('.')[0];
            if (integerPart.length > 15) {
                // Too large for safe Number — keep as string to preserve precision
                return value;
            }
            const num = Number(value);
            if (!isNaN(num))
                return num;
        }
        return value;
    }
    // ============ Redis Message Queue ============
    // BlockingQueue (Redis List) operations
    async getRedisQueues(id, pattern = 'queue:*', count = 1000) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        const redisClient = client;
        const result = await redisClient.scan('0', 'MATCH', pattern, 'COUNT', count);
        const queues = [];
        for (const key of result[1]) {
            const type = await redisClient.type(key);
            if (type === 'list') {
                const len = await redisClient.llen(key);
                queues.push({ name: key, length: len });
            }
        }
        return queues;
    }
    async getRedisQueueMessages(id, queueName, start = 0, count = 100) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        const redisClient = client;
        const type = await redisClient.type(queueName);
        if (type !== 'list')
            throw new Error(`Key '${queueName}' is not a list (queue)`);
        const end = start + count - 1;
        const items = await redisClient.lrange(queueName, start, end);
        return items.map((val, i) => ({
            index: start + i,
            value: val,
        }));
    }
    async pushToRedisQueue(id, queueName, message, direction = 'right') {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        const redisClient = client;
        if (direction === 'left') {
            return await redisClient.lpush(queueName, message);
        }
        else {
            return await redisClient.rpush(queueName, message);
        }
    }
    async popFromRedisQueue(id, queueName, direction = 'left') {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        const redisClient = client;
        if (direction === 'left') {
            return await redisClient.lpop(queueName);
        }
        else {
            return await redisClient.rpop(queueName);
        }
    }
    async clearRedisQueue(id, queueName) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        const redisClient = client;
        await redisClient.del(queueName);
        return true;
    }
    // Topic (Pub/Sub) operations
    async publishToRedisTopic(id, topic, message) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        const redisClient = client;
        return await redisClient.publish(topic, message);
    }
    async getRedisTopics(id, pattern = 'topic:*') {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { client, config } = entry;
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        // Redis doesn't have a built-in way to list active topics/channels
        // We scan for keys matching the pattern that might be used as topics
        const redisClient = client;
        const result = await redisClient.scan('0', 'MATCH', pattern, 'COUNT', 1000);
        return result[1];
    }
    async subscribeToRedisTopic(id, topic) {
        const entry = this.connections.get(id);
        if (!entry)
            throw new Error(`Connection '${id}' not found`);
        const { config } = entry;
        if (config.type !== 'redis')
            throw new Error('Only available for Redis connections');
        const subId = `sub_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
        const subClient = new ioredis_1.default({
            host: config.host,
            port: config.port,
            password: config.password || undefined,
            db: config.dbIndex || 0,
        });
        const callback = (channel, message) => {
            // Notify renderer via IPC (handled by main.ts)
            if (global.mainWindow) {
                global.mainWindow.webContents.send('redis:topic-message', {
                    subscriptionId: subId,
                    topic: channel,
                    message,
                    timestamp: new Date().toISOString(),
                });
            }
        };
        subClient.on('message', callback);
        await subClient.subscribe(topic);
        this.topicSubscriptions.set(subId, { client: subClient, callback: callback });
        return subId;
    }
    async unsubscribeFromRedisTopic(subId) {
        const sub = this.topicSubscriptions.get(subId);
        if (!sub)
            return false;
        await sub.client.quit();
        this.topicSubscriptions.delete(subId);
        return true;
    }
}
exports.default = new DBManager();
//# sourceMappingURL=db-manager.js.map