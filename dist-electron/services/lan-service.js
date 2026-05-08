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
const logger_1 = require("../logger");
/**
 * 局域网协作服务 — UDP + UDX 混合架构
 * dgram socket (port 49152): 发现广播、心跳、消息、文件传输协商
 * udx-native Stream: 可靠文件传输（内置拥塞控制、自动重传）
 */
const dgram = __importStar(require("dgram"));
const udx_native_1 = __importDefault(require("udx-native"));
const os = __importStar(require("os"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const crypto = __importStar(require("crypto"));
const EventEmitter = require("events");
const network_permission_1 = require("./network-permission");
/**
 * Check if a file name corresponds to a common image format.
 */
function isImageFile(fileName) {
    const ext = fileName.split('.').pop()?.toLowerCase() || '';
    return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(ext);
}
// Message ACK constants
const MSG_ACK_TIMEOUT = 2000;
const MSG_MAX_RETRIES = 3;
class LanService extends EventEmitter {
    /** Check if network permission was granted (null = not yet checked) */
    get networkPermissionGranted() { return this._networkPermissionGranted; }
    get networkPermissionDetails() { return this._networkPermissionDetails; }
    /**
     * 检查并请求 macOS 局域网权限
     */
    async checkNetworkPermission() {
        const result = await (0, network_permission_1.requestLocalNetworkAccess)(5000);
        this._networkPermissionGranted = result.granted;
        this._networkPermissionDetails = result.details || null;
        if (result.granted) {
            this.log('[LanService] ✅ Local network permission granted:', result.details);
        }
        else {
            this.error('[LanService] ❌ Local network permission DENIED:', result.details);
        }
        return { granted: result.granted, details: result.details || '' };
    }
    /** Get log directory for current platform — unified under ~/.supertool/logs/ */
    getLogDir() {
        return path.join(os.homedir(), '.supertool', 'logs');
    }
    /** Initialize log file */
    _initLogFile() {
        const logDir = this.getLogDir();
        try {
            fs.mkdirSync(logDir, { recursive: true });
            const today = new Date().toISOString().slice(0, 10);
            const logPath = path.join(logDir, `supertool-${today}.log`);
            this._logStream = fs.createWriteStream(logPath, { flags: 'a', encoding: 'utf8' });
            this._logStream.on('error', (err) => {
                console.error('[LanService] Failed to write log file:', err.message);
            });
            this._log('info', `=== LAN Service started === [PID: ${process.pid}]`);
            this._cleanupOldLogs(logDir);
        }
        catch (e) {
            console.error('[LanService] Failed to create log directory:', e.message);
        }
    }
    /** Delete log files older than LOG_RETENTION_DAYS */
    _cleanupOldLogs(logDir) {
        try {
            const cutoff = Date.now() - this.LOG_RETENTION_DAYS * 24 * 60 * 60 * 1000;
            const files = fs.readdirSync(logDir).filter(f => f.endsWith('.log'));
            let deleted = 0;
            for (const file of files) {
                const filePath = path.join(logDir, file);
                const stat = fs.statSync(filePath);
                if (stat.mtimeMs < cutoff) {
                    fs.unlinkSync(filePath);
                    deleted++;
                }
            }
            if (deleted > 0) {
                this._log('info', `Cleaned up ${deleted} old log file(s) (older than ${this.LOG_RETENTION_DAYS} days)`);
            }
        }
        catch (e) {
            this._log('warn', `Log cleanup failed: ${e.message}`);
        }
    }
    /** Get recent logs */
    getLogs(limit = 100) {
        return this._logBuffer.slice(-limit);
    }
    /** Get log file path */
    get logFilePath() {
        if (!this._logStream)
            return null;
        return this._logStream.path;
    }
    /** Internal log method */
    _log(level, ...args) {
        const message = args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ');
        const entry = { time: new Date().toLocaleTimeString(), level, message };
        this._logBuffer.push(entry);
        if (this._logBuffer.length > this.MAX_LOG_ENTRIES) {
            this._logBuffer = this._logBuffer.slice(-this.MAX_LOG_ENTRIES);
        }
        this.emit('log', entry);
        if (this._logStream) {
            const ts = new Date().toISOString().slice(0, 19).replace('T', ' ');
            this._logStream.write(`[${ts}] [${level.toUpperCase()}] ${message}\n`);
        }
        if (level === 'error')
            console.error(...args);
        else if (level === 'warn')
            console.warn(...args);
        else
            console.info(...args);
    }
    // Public getters
    get userId() { return this._userId; }
    get userName() { return this._nickName || this._userName; }
    get avatar() { return this._avatar || '😀'; }
    get myStatusDisplay() { return this.myStatus; }
    get networkInfo() {
        if (this._networkInfoCache && Date.now() - this._networkInfoCachedAt < 5000) {
            return this._networkInfoCache;
        }
        const ifaces = os.networkInterfaces();
        const addrs = [];
        for (const [, list] of Object.entries(ifaces)) {
            if (!list)
                continue;
            for (const iface of list) {
                if (iface.family === 'IPv4' && !iface.internal) {
                    addrs.push(iface.address);
                }
            }
        }
        this._networkInfoCache = {
            address: addrs.join(', ') || 'unknown',
            ports: `${this.messagePort}`
        };
        this._networkInfoCachedAt = Date.now();
        return this._networkInfoCache;
    }
    constructor(userDataPath) {
        super();
        this.udpSocket = null;
        this.peers = new Map();
        this.messagePort = 49152; // same port — all traffic multiplexed on one socket
        this.isRunning = false;
        // Incoming file transfer state
        this.incomingFiles = new Map();
        // UDX file transfer
        this.udx = null;
        this.udxSocket = null;
        this.udxPort = 0;
        // Outgoing file transfer state
        this.outgoingTransfers = new Map();
        // Message ACK tracking
        this.messageAckWaiters = new Map();
        // Offline message queue
        this.offlineMessages = new Map();
        // Log buffer for frontend visibility (ring buffer, max 500 entries)
        this._logBuffer = [];
        this.MAX_LOG_ENTRIES = 500;
        // Disk log file
        this._logStream = null;
        // Log retention: keep this many days of log files
        this.LOG_RETENTION_DAYS = 7;
        // Network permission status
        this._networkPermissionGranted = null; // null = not checked yet
        this._networkPermissionDetails = null;
        // Convenience aliases
        this.log = (...args) => this._log('info', ...args);
        this.warn = (...args) => this._log('warn', ...args);
        this.error = (...args) => this._log('error', ...args);
        // Progress throttle for file transfers
        this.callbacks = {
            onPeerDiscovered: null,
            onPeerLost: null,
            onMessage: null,
            onTaskAssigned: null,
            onProjectSync: null,
            onTaskUpdated: null,
            onTaskStatusChanged: null,
            onTaskCommentAdded: null,
            onCollaborationStarted: null,
            onCollaborationEnded: null,
            onFileTransferStarted: null,
            onFileTransferProgress: null,
            onFileTransferCompleted: null,
            onFileTransferError: null,
            onFileReceived: null
        };
        this._networkInfoCache = null;
        this._networkInfoCachedAt = 0;
        this._nickName = '';
        this._avatar = '';
        this._receivePath = '';
        this.myStatus = 'online';
        this.VERSION = '2.0';
        this.broadcastTimers = [];
        this.heartbeatTimer = null;
        this.offlineCheckTimer = null;
        this._broadcastAddrsCache = [];
        this._broadcastAddrsCachedAt = 0;
        this.userDataPath = userDataPath;
        this._userId = os.hostname();
        this._userName = os.userInfo().username;
        this.db = null;
        this.initDatabase();
        this.loadProfile();
        this.loadStatus();
        this.loadReceivePath();
    }
    // 加载文件接收路径
    loadReceivePath() {
        try {
            const row = this.db.prepare("SELECT value FROM settings WHERE key = 'lan_receive_path'").get();
            this._receivePath = row?.value || path.join(this.userDataPath, 'received_files');
        }
        catch {
            this._receivePath = path.join(this.userDataPath, 'received_files');
        }
    }
    getReceivePath() {
        return this._receivePath;
    }
    setReceivePath(dirPath) {
        this._receivePath = dirPath;
        this.db.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('lan_receive_path', ?)").run(dirPath);
    }
    // 加载用户自定义资料
    loadProfile() {
        try {
            const nick = this.db.prepare("SELECT value FROM settings WHERE key = 'lan_nickname'").get();
            const avatar = this.db.prepare("SELECT value FROM settings WHERE key = 'lan_avatar'").get();
            if (nick)
                this._nickName = nick.value;
            if (avatar)
                this._avatar = avatar.value;
        }
        catch { }
    }
    setNickName(name) {
        this._nickName = name;
        this.db.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('lan_nickname', ?)").run(name);
    }
    setAvatar(emoji) {
        this._avatar = emoji;
        this.db.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('lan_avatar', ?)").run(emoji);
    }
    /**
     * 设置在线状态
     */
    setStatus(status) {
        this.myStatus = status;
        this.db.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('lan_status', ?)").run(status);
        if (this.isRunning) {
            this.broadcastPresence();
        }
    }
    /**
     * 加载用户状态
     */
    loadStatus() {
        try {
            const status = this.db.prepare("SELECT value FROM settings WHERE key = 'lan_status'").get();
            if (status && ['online', 'busy', 'away'].includes(status.value)) {
                this.myStatus = status.value;
            }
        }
        catch { }
    }
    // 初始化数据库
    initDatabase() {
        const Database = require('better-sqlite3');
        const dbPath = path.join(this.userDataPath, 'supertool.db');
        this.db = new Database(dbPath);
        // 数据库迁移：确保 chat_messages 表存在 read 列
        const cols = this.db.pragma("table_info(chat_messages)");
        const hasRead = cols.some(c => c.name === 'read');
        if (!hasRead) {
            this.db.exec('ALTER TABLE chat_messages ADD COLUMN "read" INTEGER DEFAULT 0');
        }
        // 迁移：为 file_transfers 添加 localUserId 列
        const ftCols = this.db.pragma("table_info(file_transfers)");
        const hasLocalUserId = ftCols.some(c => c.name === 'localUserId');
        if (!hasLocalUserId) {
            this.db.exec('ALTER TABLE file_transfers ADD COLUMN localUserId TEXT');
        }
        this.db.exec(`
      CREATE TABLE IF NOT EXISTS file_transfers (
        id TEXT PRIMARY KEY,
        fromUserId TEXT NOT NULL,
        fromUserName TEXT NOT NULL,
        toUserId TEXT NOT NULL,
        toUserName TEXT NOT NULL,
        fileName TEXT NOT NULL,
        fileSize INTEGER NOT NULL,
        filePath TEXT,
        status TEXT DEFAULT 'pending',
        progress INTEGER DEFAULT 0,
        createdAt TEXT NOT NULL,
        completedAt TEXT,
        localUserId TEXT
      );

      CREATE INDEX IF NOT EXISTS idx_chat_messages_peer_pair ON chat_messages(fromUserId, toUserId, createdAt);
      CREATE INDEX IF NOT EXISTS idx_chat_messages_read ON chat_messages(toUserId, fromUserId, "read");
      CREATE INDEX IF NOT EXISTS idx_file_transfers_status ON file_transfers(status, createdAt);
      CREATE INDEX IF NOT EXISTS idx_file_transfers_lookup ON file_transfers(id, localUserId);
      CREATE INDEX IF NOT EXISTS idx_file_transfers_localUser ON file_transfers(localUserId);
    `);
    }
    /**
     * 启动局域网服务
     */
    async start() {
        if (this.isRunning)
            return;
        this._initLogFile();
        this.log('[LanService] Checking macOS Local Network permission...');
        const permResult = await this.checkNetworkPermission();
        if (!permResult.granted) {
            this.error('[LanService] ⚠️  macOS Local Network Privacy is blocking LAN access!');
            this.error('[LanService] Users will see other devices but cannot send messages.');
            this.error('[LanService] Fix: System Settings → Privacy & Security → Local Network → Enable SuperTool');
            this.error('[LanService] Or: sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off');
        }
        this.initUdxSocket();
        this.startUdpSocket();
        this.isRunning = true;
        this.log('[LanService] Started on port:', this.messagePort);
    }
    /**
     * 停止局域网服务
     */
    stop() {
        if (this.broadcastTimers.length) {
            for (const t of this.broadcastTimers)
                clearTimeout(t);
            this.broadcastTimers = [];
        }
        if (this.heartbeatTimer) {
            clearInterval(this.heartbeatTimer);
            this.heartbeatTimer = null;
        }
        if (this.offlineCheckTimer) {
            clearInterval(this.offlineCheckTimer);
            this.offlineCheckTimer = null;
        }
        if (this.udpSocket) {
            this.udpSocket.close();
            this.udpSocket = null;
        }
        // Clear message ACK waiters
        for (const [id, waiter] of this.messageAckWaiters) {
            clearTimeout(waiter.timer);
        }
        this.messageAckWaiters.clear();
        // Close UDX sockets and streams
        for (const [, t] of this.outgoingTransfers) {
            try {
                t.stream?.end();
            }
            catch { }
            try {
                t.stream?.destroy();
            }
            catch { }
            try {
                t.udxSocket?.close();
            }
            catch { }
        }
        this.outgoingTransfers.clear();
        for (const [, state] of this.incomingFiles) {
            try {
                state.stream?.end();
            }
            catch { }
            try {
                state.stream?.destroy();
            }
            catch { }
            try {
                state.udxSocket?.close();
            }
            catch { }
        }
        this.incomingFiles.clear();
        if (this.udxSocket) {
            try {
                this.udxSocket.close();
            }
            catch { }
            this.udxSocket = null;
        }
        if (this.udx) {
            this.udx = null;
        }
        this.peers.clear();
        this.offlineMessages.clear();
        this._networkInfoCache = null;
        this._broadcastAddrsCache = [];
        this._broadcastAddrsCachedAt = 0;
        try {
            this.db.close();
        }
        catch { }
        if (this._logStream) {
            this._logStream.end();
            this._logStream = null;
        }
        this.isRunning = false;
        this.log('[LanService] Stopped');
    }
    /**
     * 启动单一 UDP socket — 处理发现、消息、文件传输
     */
    startUdpSocket() {
        this.udpSocket = dgram.createSocket({ type: 'udp4', reuseAddr: true });
        this.udpSocket.on('listening', () => {
            // 扩大 UDP socket 缓冲区（支撑高吞吐文件传输）
            try {
                this.udpSocket.setSendBufferSize(4 * 1024 * 1024);
            }
            catch { } // 4MB
            try {
                this.udpSocket.setRecvBufferSize(4 * 1024 * 1024);
            }
            catch { } // 4MB
            this.udpSocket.setBroadcast(true);
            const addr = this.udpSocket.address();
            this.log(`[LanService] UDP socket listening on ${addr.address}:${addr.port}`);
            // Initial discovery burst
            this.broadcastPresence();
            const t1 = setTimeout(() => { if (this.udpSocket)
                this.broadcastPresence(); }, 500);
            const t2 = setTimeout(() => { if (this.udpSocket)
                this.broadcastPresence(); }, 1000);
            this.broadcastTimers.push(t1, t2);
            this.heartbeatTimer = setInterval(() => this.broadcastHeartbeat(), 5000);
            this.offlineCheckTimer = setInterval(() => this.checkOfflinePeers(), 10000);
        });
        this.udpSocket.on('message', (msg, rinfo) => {
            // Ignore messages from our own IP
            if (rinfo.address === '127.0.0.1')
                return;
            const myAddr = this.networkInfo.address.split(',')[0]?.trim();
            if (myAddr && rinfo.address === myAddr)
                return;
            try {
                const raw = msg.toString();
                let data;
                try {
                    data = JSON.parse(raw);
                }
                catch (parseError) {
                    if (parseError.message.includes('Unexpected end') || parseError.message.includes("Expected ',' or '}'")) {
                        data = JSON.parse(raw + '}');
                    }
                    else {
                        throw parseError;
                    }
                }
                // 文件相关日志节流（不打印心跳和文件 ACK）
                if (data.type !== 'heartbeat' && data.type !== 'discovery' && data.type !== 'file_ack') {
                    this.log(`[LanService][UDP RECV] type=${data.type}, from=${rinfo.address}:${rinfo.port}, data=${JSON.stringify(data).slice(0, 300)}`);
                }
                // Route by message type
                switch (data.type) {
                    case 'discovery':
                        if (data.userId !== this.userId) {
                            if (data.version && data.version.split('.')[0] !== this.VERSION.split('.')[0]) {
                                (0, logger_1.info)(`[LanService] Ignoring incompatible peer version: ${data.version}`);
                                return;
                            }
                            this.handlePeerDiscovered(data, rinfo.address);
                            this.replyToDiscovery(data, rinfo.address, rinfo.port);
                        }
                        break;
                    case 'heartbeat':
                        if (data.userId !== this.userId) {
                            this.handleHeartbeat(data, rinfo.address);
                        }
                        break;
                    case 'message':
                        this.handleIncomingMessage(data);
                        break;
                    case 'message_ack':
                        this.handleMessageAck(data);
                        break;
                    case 'file_start':
                        this.handleFileStart(data, rinfo);
                        break;
                    case 'file_start_ack':
                        this.handleFileStartAck(data);
                        break;
                }
            }
            catch (e) {
                this.warn(`[LanService][UDP RECV] Failed to parse message from ${rinfo.address}:${rinfo.port}: ${e}`);
            }
        });
        this.udpSocket.bind(this.messagePort, '0.0.0.0');
        this.udpSocket.on('error', (err) => {
            if (err.code === 'EADDRINUSE') {
                this.error(`[LanService] UDP port ${this.messagePort} is already in use`);
            }
            else {
                this.error('[LanService] UDP socket error:', err.message);
            }
            if (this.heartbeatTimer) {
                clearInterval(this.heartbeatTimer);
                this.heartbeatTimer = null;
            }
            if (this.offlineCheckTimer) {
                clearInterval(this.offlineCheckTimer);
                this.offlineCheckTimer = null;
            }
            this.restartUdpSocket(1); // Start from attempt 1, not 0
        });
    }
    /**
     * 自动重启 UDP socket
     */
    restartUdpSocket(attempt = 1) {
        if (attempt > 3 || !this.isRunning) {
            this.error('[LanService] UDP restart failed after 3 attempts, giving up');
            return;
        }
        this.log(`[LanService] Restarting UDP socket (attempt ${attempt}/3)...`);
        try {
            if (this.udpSocket) {
                try {
                    this.udpSocket.close();
                }
                catch { }
                this.udpSocket = null;
            }
            setTimeout(() => {
                if (this.isRunning) {
                    this.startUdpSocket();
                }
            }, 1500);
        }
        catch (e) {
            this.error('[LanService] UDP restart error:', e);
            setTimeout(() => this.restartUdpSocket(attempt + 1), 1500);
        }
    }
    /**
     * 获取所有子网广播地址
     */
    getBroadcastAddresses() {
        if (this._broadcastAddrsCache.length > 0 && Date.now() - this._broadcastAddrsCachedAt < 5000) {
            return this._broadcastAddrsCache;
        }
        const addresses = [];
        const interfaces = os.networkInterfaces();
        for (const [, ifaces] of Object.entries(interfaces)) {
            if (!ifaces)
                continue;
            for (const iface of ifaces) {
                if (iface.family !== 'IPv4' || iface.internal)
                    continue;
                const ipParts = iface.address.split('.').map(Number);
                const maskParts = (iface.netmask || '255.255.255.0').split('.').map(Number);
                const broadcast = ipParts.map((ip, i) => ip | (~maskParts[i] & 0xff)).join('.');
                addresses.push(broadcast);
            }
        }
        if (addresses.length === 0) {
            addresses.push('255.255.255.255');
        }
        this._broadcastAddrsCache = [...new Set(addresses)];
        this._broadcastAddrsCachedAt = Date.now();
        return this._broadcastAddrsCache;
    }
    /**
     * 广播存在
     */
    broadcastPresence() {
        const message = JSON.stringify({
            type: 'discovery',
            userId: this.userId,
            userName: this.userName,
            avatar: this.avatar,
            messagePort: this.messagePort,
            version: this.VERSION,
            status: this.myStatus,
            timestamp: Date.now()
        });
        const broadcastAddrs = this.getBroadcastAddresses();
        for (const addr of broadcastAddrs) {
            this.udpSocket.send(message, 0, message.length, this.messagePort, addr, (err) => {
                if (err)
                    this.warn(`[LanService] Broadcast to ${addr} failed: ${err.message}`);
            });
        }
    }
    /**
     * 广播心跳
     */
    broadcastHeartbeat() {
        const message = JSON.stringify({
            type: 'heartbeat',
            userId: this.userId,
            userName: this.userName,
            avatar: this.avatar,
            messagePort: this.messagePort,
            version: this.VERSION,
            status: this.myStatus,
            timestamp: Date.now()
        });
        const broadcastAddrs = this.getBroadcastAddresses();
        for (const addr of broadcastAddrs) {
            this.udpSocket.send(message, 0, message.length, this.messagePort, addr, (err) => {
                if (err)
                    this.warn(`[LanService] Heartbeat to ${addr} failed: ${err.message}`);
            });
        }
    }
    /**
     * 回复发现请求
     */
    replyToDiscovery(_data, address, port) {
        const reply = JSON.stringify({
            type: 'discovery',
            userId: this.userId,
            userName: this.userName,
            avatar: this.avatar,
            messagePort: this.messagePort,
            version: this.VERSION,
            status: this.myStatus,
            timestamp: Date.now()
        });
        this.udpSocket.send(reply, 0, reply.length, port, address, (err) => {
            if (err)
                this.warn('[LanService] Reply to', address, ':', port, 'failed:', err.message);
        });
    }
    /**
     * 手动触发重新扫描
     */
    refreshDiscovery() {
        if (!this.udpSocket || !this.udpSocket.address()?.port) {
            this.warn('[LanService] UDP socket is down, restarting...');
            this.restartUdpSocket();
            return;
        }
        if (!this.isRunning)
            return;
        this.log('[LanService] Manual discovery refresh');
        for (const t of this.broadcastTimers)
            clearTimeout(t);
        this.broadcastTimers = [];
        this.broadcastPresence();
        const t1 = setTimeout(() => { if (this.udpSocket)
            this.broadcastPresence(); }, 500);
        const t2 = setTimeout(() => { if (this.udpSocket)
            this.broadcastPresence(); }, 1000);
        this.broadcastTimers.push(t1, t2);
    }
    /**
     * 处理发现新节点
     */
    handlePeerDiscovered(data, address) {
        const peerId = data.userId;
        const msgPort = Number(data.messagePort);
        if (!peerId || typeof peerId !== 'string')
            return;
        if (!data.userName || typeof data.userName !== 'string')
            return;
        if (isNaN(msgPort) || msgPort < 1 || msgPort > 65535)
            return;
        if (peerId.length > 256 || data.userName.length > 256)
            return;
        if (!this.peers.has(peerId)) {
            const peer = {
                id: peerId,
                name: data.userName,
                avatar: data.avatar || '😀',
                address: address,
                messagePort: msgPort,
                version: data.version || '1.0.0',
                lastSeen: Date.now(),
                online: true,
                status: data.status || 'online',
            };
            this.peers.set(peerId, peer);
            if (this.callbacks.onPeerDiscovered) {
                this.callbacks.onPeerDiscovered(peer);
            }
            // Deliver any queued offline messages
            this.deliverOfflineMessages(peerId);
        }
        else {
            const peer = this.peers.get(peerId);
            peer.lastSeen = Date.now();
            peer.online = true;
            if (data.avatar)
                peer.avatar = data.avatar;
            if (data.status)
                peer.status = data.status;
            if (data.messagePort && peer.messagePort !== data.messagePort) {
                peer.messagePort = data.messagePort;
            }
            if (data.version && peer.version !== data.version) {
                peer.version = data.version;
            }
        }
    }
    /**
     * 处理心跳
     */
    handleHeartbeat(data, address) {
        const peerId = data.userId;
        if (this.peers.has(peerId)) {
            const peer = this.peers.get(peerId);
            peer.lastSeen = Date.now();
            peer.online = true;
            if (data.avatar)
                peer.avatar = data.avatar;
            if (data.status)
                peer.status = data.status;
            if (data.messagePort && peer.messagePort !== data.messagePort) {
                peer.messagePort = data.messagePort;
            }
            if (data.version && peer.version !== data.version) {
                peer.version = data.version;
            }
            if (this.callbacks.onPeerDiscovered) {
                this.callbacks.onPeerDiscovered(peer);
            }
        }
        else if (data.messagePort) {
            this.handlePeerDiscovered(data, address);
        }
    }
    /**
     * 检查离线节点
     */
    checkOfflinePeers() {
        const now = Date.now();
        for (const [_peerId, peer] of this.peers) {
            if (peer.online && now - peer.lastSeen > 30000) {
                peer.online = false;
                this.warn('[LanService] Peer lost:', peer.name);
                if (this.callbacks.onPeerLost) {
                    this.callbacks.onPeerLost(peer);
                }
            }
        }
    }
    /**
     * 发送 UDP 消息到指定 peer
     */
    sendUdpToPeer(peerId, data) {
        const peer = this.peers.get(peerId);
        if (!peer) {
            this.warn(`[LanService][sendUdpToPeer] Peer not found: ${peerId}`);
            return;
        }
        if (!this.udpSocket) {
            this.warn(`[LanService][sendUdpToPeer] UDP socket is null!`);
            return;
        }
        const msg = JSON.stringify(data);
        const buf = Buffer.from(msg);
        if (data.type !== 'file_ack') {
            this.log(`[LanService][sendUdpToPeer] Sending ${data.type} to ${peer.name} at ${peer.address}:${peer.messagePort} (${buf.length} bytes)`);
        }
        this.udpSocket.send(buf, 0, buf.length, peer.messagePort, peer.address, (err) => {
            if (err)
                this.error(`[LanService][sendUdpToPeer] UDP send FAILED to ${peer.name} (${peer.address}:${peer.messagePort}): ${err.message}`);
            else if (data.type !== 'file_ack') {
                this.log(`[LanService][sendUdpToPeer] UDP send OK to ${peer.name}`);
            }
        });
    }
    /** 发送原始 Buffer（文件传输用，不经过 JSON） */
    sendUdpBuffer(peerId, buf) {
        const peer = this.peers.get(peerId);
        if (!peer)
            return;
        if (!this.udpSocket)
            return;
        this.udpSocket.send(buf, 0, buf.length, peer.messagePort, peer.address, (err) => {
            if (err)
                this.error(`[LanService][sendUdpBuffer] UDP send FAILED to ${peer.name}: ${err.message}`);
        });
    }
    // ========== MESSAGE HANDLING ==========
    /**
     * Handle incoming chat message
     */
    handleIncomingMessage(data) {
        this.log(`[LanService][handleIncomingMessage] Received message from ${data.from} (${data.fromName}): messageId=${data.messageId}, content=${String(data.content).slice(0, 50)}`);
        // Save to DB
        this.saveMessage(data);
        // Send ACK back
        if (data.messageId && this.udpSocket) {
            const ack = JSON.stringify({ type: 'message_ack', messageId: data.messageId });
            const peer = this.peers.get(data.from);
            if (peer) {
                this.log(`[LanService][handleIncomingMessage] Sending ACK back to ${peer.address}:${peer.messagePort}`);
                this.udpSocket.send(ack, 0, ack.length, peer.messagePort, peer.address, (err) => {
                    if (err)
                        this.error(`[LanService][handleIncomingMessage] ACK send FAILED: ${err.message}`);
                    else
                        this.log(`[LanService][handleIncomingMessage] ACK send OK`);
                });
            }
            else {
                this.warn(`[LanService][handleIncomingMessage] Cannot send ACK — peer ${data.from} not in peers map!`);
            }
        }
        else {
            this.warn(`[LanService][handleIncomingMessage] No messageId or UDP socket, skipping ACK`);
        }
        // Fire callback
        if (this.callbacks.onMessage) {
            this.callbacks.onMessage(data);
        }
    }
    /**
     * Handle message ACK
     */
    handleMessageAck(data) {
        const messageId = data.messageId;
        if (!messageId)
            return;
        const waiter = this.messageAckWaiters.get(messageId);
        if (waiter) {
            clearTimeout(waiter.timer);
            waiter.resolve();
            this.messageAckWaiters.delete(messageId);
            this.log(`[LanService][handleMessageAck] ACK resolved for messageId=${messageId}`);
        }
        else {
            this.warn(`[LanService][handleMessageAck] No waiter found for messageId=${messageId} — ACK arrived too late?`);
        }
    }
    /**
     * 发送消息给指定用户（UDP + ACK 机制）
     */
    sendMessage(peerId, message) {
        const peer = this.peers.get(peerId);
        const messageId = crypto.randomUUID();
        // 检查版本兼容性
        if (peer && peer.version) {
            const peerMajor = peer.version.split('.')[0];
            const myMajor = this.VERSION.split('.')[0];
            if (peerMajor !== myMajor) {
                this.error(`[LanService][sendMessage] INCOMPATIBLE VERSION! Peer "${peer.name}" is v${peer.version}, we are v${this.VERSION}. Peer cannot handle UDP messages!`);
                return { success: false, messageId, incompatibleVersion: peer.version };
            }
        }
        if (!peer) {
            this.warn(`[LanService][sendMessage] === ENTRY === peerId=${peerId}, peer NOT FOUND! Known peers: [${Array.from(this.peers.keys()).join(', ')}]`);
        }
        else {
            this.log(`[LanService][sendMessage] === ENTRY === peerId=${peerId}, peer=${peer.name}, addr=${peer.address}:${peer.messagePort}, version=${peer.version || 'unknown'}`);
            this.log(`[LanService][sendMessage] Known peers: [${Array.from(this.peers.keys()).join(', ')}]`);
        }
        const data = {
            type: 'message',
            from: this.userId,
            fromName: this.userName,
            to: peer?.id || peerId,
            toName: peer?.name || peerId,
            content: message,
            messageId: messageId,
            timestamp: Date.now()
        };
        // Always save to DB
        this.saveMessage(data);
        this.log(`[LanService][sendMessage] Saved to DB`);
        if (peer) {
            // Try to send via UDP (even if marked offline — UDP is fire-and-forget, no connection needed)
            this.log(`[LanService][sendMessage] Calling sendWithAck for peer ${peer.name} at ${peer.address}:${peer.messagePort}`);
            this.sendWithAck(peerId, data, messageId).catch((e) => {
                this.warn('[LanService][sendMessage] sendWithAck rejected/failed for', messageId, e?.message);
            });
            return { success: true, messageId };
        }
        // Unknown peer — queue for offline delivery
        this.warn(`[LanService][sendMessage] Peer NOT found! peerId=${peerId}, queueing message`);
        if (!this.offlineMessages.has(peerId)) {
            this.offlineMessages.set(peerId, []);
        }
        const queue = this.offlineMessages.get(peerId);
        if (queue.length >= 100) {
            queue.shift();
        }
        queue.push(data);
        this.log(`[LanService] Peer offline — queued message (queue size: ${queue.length})`);
        return { success: false, messageId, queued: true };
    }
    /**
     * Send a message with ACK mechanism (retry up to 3 times, 2s timeout)
     */
    async sendWithAck(peerId, data, messageId) {
        for (let attempt = 0; attempt < MSG_MAX_RETRIES; attempt++) {
            this.log(`[LanService][sendWithAck] Attempt ${attempt + 1}/${MSG_MAX_RETRIES} for messageId=${messageId}`);
            const result = await new Promise((resolve) => {
                const timer = setTimeout(() => {
                    this.messageAckWaiters.delete(messageId);
                    this.warn(`[LanService][sendWithAck] Timeout after ${MSG_ACK_TIMEOUT}ms for messageId=${messageId}`);
                    resolve(false);
                }, MSG_ACK_TIMEOUT);
                this.messageAckWaiters.set(messageId, {
                    resolve: () => { resolve(true); clearTimeout(timer); },
                    reject: () => { resolve(false); clearTimeout(timer); },
                    timer
                });
                this.sendUdpToPeer(peerId, data);
            });
            if (result) {
                this.log(`[LanService][sendWithAck] ACK received for messageId=${messageId} on attempt ${attempt + 1}`);
                return; // ACK received
            }
            this.warn(`[LanService][sendWithAck] Message ${messageId} attempt ${attempt + 1}/${MSG_MAX_RETRIES} timed out`);
        }
        this.error(`[LanService][sendWithAck] ALL RETRIES FAILED for messageId=${messageId}`);
    }
    /**
     * 广播消息给所有在线用户
     */
    broadcastMessage(message) {
        const messageId = crypto.randomUUID();
        const data = {
            type: 'message',
            from: this.userId,
            fromName: this.userName,
            to: 'broadcast',
            toName: 'All',
            content: message,
            messageId: messageId,
            timestamp: Date.now()
        };
        for (const [peerId] of this.peers) {
            this.sendUdpToPeer(peerId, data);
        }
        this.saveMessage(data);
    }
    // ========== FILE TRANSFER OVER UDP ===========
    // 滑动窗口协议 — 16 块同时在飞，不逐块等 ACK
    // 二进制文件传输协议（不经过 JSON/base64）
    // [type:1B][uuid:36B][seq:4B][totalChunks:4B][chunkLen:4B][data:chunkLen] = 49B header
    /**
     * 发送文件给指定用户（UDP sliding window with ACK）
     */
    /**
     * 初始化 UDX socket 用于可靠文件传输
     */
    initUdxSocket() {
        try {
            this.udx = new udx_native_1.default();
            this.udxSocket = this.udx.createSocket();
            this.udxSocket.bind(0); // Dynamic port
            this.udxPort = this.udxSocket.address().port;
            this.log(`[LanService] UDX socket initialized on port ${this.udxPort}`);
        }
        catch (e) {
            this.error('[LanService] Failed to initialize UDX socket:', e.message);
            this.udx = null;
            this.udxSocket = null;
        }
    }
    /**
     * 发送文件给指定用户（UDX Stream — 内置可靠传输+拥塞控制）
     */
    sendFile(peerId, filePath, fileName, _resumeOffset = 0, existingId) {
        const peer = this.peers.get(peerId);
        this.log(`[LanService][sendFile] Called: peerId=${peerId}, filePath=${filePath}, fileName=${fileName}`);
        if (!peer) {
            this.error('[LanService][sendFile] Peer not found:', peerId);
            return false;
        }
        if (!peer.online) {
            this.error('[LanService][sendFile] Peer not online:', peer.name);
            return false;
        }
        if (!this.udxSocket) {
            this.error('[LanService][sendFile] UDX socket not initialized');
            return false;
        }
        let fileSize;
        try {
            const stat = fs.statSync(filePath);
            fileSize = stat.size;
        }
        catch (e) {
            this.error('[LanService] Cannot access file:', filePath, e.message);
            return false;
        }
        const fileId = existingId || crypto.randomUUID();
        const fileTransfer = {
            id: fileId,
            fromUserId: this.userId,
            fromUserName: this.userName,
            toUserId: peer.id,
            toUserName: peer.name,
            fileName: fileName || path.basename(filePath),
            fileSize,
            filePath,
            status: 'sending',
            progress: 0,
            createdAt: new Date().toISOString()
        };
        // Save to DB
        this.db.prepare(`
      INSERT OR REPLACE INTO file_transfers (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt, localUserId)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(fileTransfer.id, fileTransfer.fromUserId, fileTransfer.fromUserName, fileTransfer.toUserId, fileTransfer.toUserName, fileTransfer.fileName, fileTransfer.fileSize, fileTransfer.filePath, fileTransfer.status, fileTransfer.progress, fileTransfer.createdAt, this.userId);
        // Create chat_messages record
        try {
            this.db.prepare(`
        INSERT OR IGNORE INTO chat_messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      `).run(fileTransfer.id, fileTransfer.fromUserId, fileTransfer.fromUserName, fileTransfer.toUserId, fileTransfer.toUserName, JSON.stringify({ fileName: fileTransfer.fileName, fileSize: fileTransfer.fileSize, filePath: fileTransfer.filePath, isImage: isImageFile(fileTransfer.fileName) }), 'file', fileTransfer.createdAt, 0);
        }
        catch (e) { /* ignore */ }
        if (this.callbacks.onFileTransferStarted) {
            this.callbacks.onFileTransferStarted(fileTransfer);
        }
        this.log(`[LanService] Starting UDX file transfer: ${fileTransfer.fileName} (${fileSize} bytes)`);
        // Generate sender stream ID — used by receiver to connect() back
        const senderStreamId = Math.floor(Math.random() * 1000000);
        // Send file_start notification — receiver will create stream, connect() back, and reply file_start_ack
        const fileStart = {
            type: 'file_start',
            id: fileId,
            fromUserId: this.userId,
            fromUserName: this.userName,
            toUserId: peer.id,
            toUserName: peer.name,
            fileName: fileTransfer.fileName,
            fileSize,
            isImage: isImageFile(fileTransfer.fileName),
            status: 'sending',
            progress: 0,
            createdAt: fileTransfer.createdAt,
            udxPort: this.udxPort,
            udxStreamId: senderStreamId
        };
        this.sendUdpToPeer(peerId, fileStart);
        // Track outgoing transfer (will start when file_start_ack arrives)
        const transfer = {
            id: fileId,
            peerId,
            fileName: fileTransfer.fileName,
            filePath,
            fileSize,
            status: 'sending',
            progress: 0,
            senderStreamId // store for reuse in _startUdxSend
        };
        this.outgoingTransfers.set(fileId, transfer);
        return true;
    }
    /**
     * Start UDX stream to send file — called AFTER receiving file_start_ack from receiver.
     * At this point the receiver has created its stream and is ready to accept connections.
     */
    _startUdxSend(transfer, filePath, peerAddr, receiverStreamId, receiverUdxPort) {
        if (!this.udx || !this.udxSocket)
            return;
        const stream = this.udx.createStream(transfer.senderStreamId);
        transfer.stream = stream;
        const startTime = Date.now();
        let progressTimer = null;
        stream.on('connect', () => {
            this.log(`[LanService][UDX] Stream connected, sending ${transfer.fileName}`);
            // Pipe file to stream
            const readStream = fs.createReadStream(filePath, { highWaterMark: 64 * 1024 });
            readStream.on('data', (chunk) => {
                // Progress tracking based on bytes read
                const progress = Math.min(100, Math.round((readStream.bytesRead / transfer.fileSize) * 100));
                if (progress !== transfer.progress) {
                    transfer.progress = progress;
                    this.db.prepare(`UPDATE file_transfers SET progress = ? WHERE id = ?`).run(progress, transfer.id);
                    if (this.callbacks.onFileTransferProgress && progress > 0) {
                        const peer = this.peers.get(transfer.peerId);
                        this.callbacks.onFileTransferProgress({
                            id: transfer.id, fromUserId: this.userId, fromUserName: this.userName,
                            toUserId: transfer.peerId, toUserName: peer?.name || '',
                            fileName: transfer.fileName, fileSize: transfer.fileSize,
                            status: 'sending', progress, createdAt: new Date().toISOString()
                        });
                    }
                }
            });
            readStream.on('end', () => {
                stream.end();
            });
            readStream.on('error', (err) => {
                this.error(`[LanService][UDX] Read error: ${err.message}`);
            });
            readStream.pipe(stream);
        });
        stream.on('close', () => {
            if (progressTimer)
                clearInterval(progressTimer);
            // close fires after error too — don't overwrite error status
            if (transfer.status === 'error' || transfer.status === 'cancelled')
                return;
            const elapsed = Math.max(0.1, (Date.now() - startTime) / 1000);
            const speed = (transfer.fileSize / 1024 / 1024 / elapsed).toFixed(1);
            this.log(`[LanService][UDX] Send complete: ${transfer.fileName} (${elapsed.toFixed(1)}s, ${speed} MB/s, RTT=${stream.rtt}ms)`);
            transfer.status = 'completed';
            transfer.progress = 100;
            this.db.prepare(`UPDATE file_transfers SET status = ?, progress = ?, completedAt = ? WHERE id = ?`)
                .run('completed', 100, new Date().toISOString(), transfer.id);
            try {
                this.db.prepare(`UPDATE chat_messages SET content = ? WHERE id = ? AND type = 'file'`).run(JSON.stringify({ fileName: transfer.fileName, fileSize: transfer.fileSize, filePath: transfer.filePath, status: 'completed', isImage: isImageFile(transfer.fileName) }), transfer.id);
            }
            catch (e) { /* ignore */ }
            if (this.callbacks.onFileTransferCompleted) {
                const peer = this.peers.get(transfer.peerId);
                this.callbacks.onFileTransferCompleted({
                    id: transfer.id, fromUserId: this.userId, fromUserName: this.userName,
                    toUserId: transfer.peerId, toUserName: peer?.name || '',
                    fileName: transfer.fileName, fileSize: transfer.fileSize,
                    filePath: transfer.filePath,
                    status: 'completed', progress: 100, createdAt: new Date().toISOString(),
                    completedAt: new Date().toISOString(),
                    isImage: isImageFile(transfer.fileName)
                });
            }
            this.outgoingTransfers.delete(transfer.id);
        });
        stream.on('error', (err) => {
            if (progressTimer)
                clearInterval(progressTimer);
            this.error(`[LanService][UDX] Send error: ${err.message}`);
            transfer.status = 'error';
            this.db.prepare(`UPDATE file_transfers SET status = ? WHERE id = ?`).run('error', transfer.id);
            if (this.callbacks.onFileTransferError) {
                const peer = this.peers.get(transfer.peerId);
                this.callbacks.onFileTransferError({
                    id: transfer.id, fromUserId: this.userId, fromUserName: this.userName,
                    toUserId: transfer.peerId, toUserName: peer?.name || '',
                    fileName: transfer.fileName, fileSize: transfer.fileSize,
                    status: 'error', progress: transfer.progress, createdAt: new Date().toISOString()
                });
            }
            this.outgoingTransfers.delete(transfer.id);
        });
        // Connect to receiver's stream using the shared main UDX socket
        stream.connect(this.udxSocket, receiverStreamId, receiverUdxPort, peerAddr);
    }
    /**
     * Handle incoming file_start
     */
    handleFileStart(data, rinfo) {
        const fileId = data.id;
        if (!fileId)
            return;
        // Validate required fields
        if (!data.fromUserId || !data.fileName || !data.fileSize) {
            this.error('[LanService] Invalid file_start packet');
            return;
        }
        const ft = {
            id: fileId,
            fromUserId: data.fromUserId,
            fromUserName: data.fromUserName || '',
            toUserId: this.userId,
            toUserName: this.userName,
            fileName: data.fileName,
            fileSize: typeof data.fileSize === 'number' ? data.fileSize : parseInt(String(data.fileSize)) || 0,
            filePath: '',
            status: 'receiving',
            progress: 0,
            createdAt: data.createdAt || new Date().toISOString()
        };
        if (ft.fileSize <= 0 || ft.fileSize > 500 * 1024 * 1024) {
            this.error('[LanService] Invalid file size:', ft.fileSize);
            return;
        }
        // SECURITY: Prevent path traversal
        const safeName = path.basename(ft.fileName);
        if (!safeName || safeName.startsWith('.')) {
            this.error('[LanService] Blocked suspicious filename:', ft.fileName);
            return;
        }
        const receivedDir = path.resolve(this._receivePath || path.join(this.userDataPath, 'received_files'));
        if (!fs.existsSync(receivedDir)) {
            fs.mkdirSync(receivedDir, { recursive: true });
        }
        const safePath = path.resolve(receivedDir, safeName);
        if (!safePath.startsWith(receivedDir + path.sep) && safePath !== receivedDir) {
            this.error('[LanService] Blocked path traversal attempt:', ft.fileName);
            return;
        }
        ft.filePath = safePath;
        // Save to DB
        this.db.prepare(`
      INSERT OR REPLACE INTO file_transfers
      (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt, localUserId)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(ft.id, ft.fromUserId, ft.fromUserName, this.userId, this.userName, ft.fileName, ft.fileSize, ft.filePath, ft.status, 0, ft.createdAt, this.userId);
        // Create chat_messages record
        try {
            this.db.prepare(`
        INSERT OR IGNORE INTO chat_messages
        (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      `).run(ft.id, ft.fromUserId, ft.fromUserName, this.userId, this.userName, JSON.stringify({ fileName: ft.fileName, fileSize: ft.fileSize, filePath: ft.filePath, isImage: data.isImage ?? isImageFile(ft.fileName) }), 'file', ft.createdAt, 0);
        }
        catch (e) { /* ignore */ }
        const ftWithImage = { ...ft, isImage: data.isImage ?? isImageFile(ft.fileName) };
        if (this.callbacks.onFileTransferStarted) {
            this.callbacks.onFileTransferStarted(ftWithImage);
        }
        this.log(`[LanService] Receiving file via UDX: ${ft.fileName} (${ft.fileSize} bytes) from ${rinfo.address}`);
        // Create UDX stream to receive data — connect() back to sender, then reply ACK
        const senderUdxPort = data.udxPort;
        const senderStreamId = data.udxStreamId;
        if (senderUdxPort && senderStreamId !== undefined && this.udx) {
            this._prepareUdxReceive(ft, rinfo.address, senderStreamId, senderUdxPort);
        }
    }
    /**
     * Prepare UDX receive: create stream, connect() back to sender, reply ACK.
     * UDX requires both sides to connect() for the uTP handshake to complete.
     * Receiver connect()s to senderStreamId, sender connect()s to receiverStreamId.
     */
    _prepareUdxReceive(transfer, peerAddr, senderStreamId, senderUdxPort) {
        if (!this.udx || !this.udxSocket)
            return;
        const receiverStreamId = Math.floor(Math.random() * 1000000);
        const stream = this.udx.createStream(receiverStreamId);
        if (!transfer.filePath)
            return;
        const fd = fs.openSync(transfer.filePath, 'w');
        const startTime = Date.now();
        let fdClosed = false;
        this.incomingFiles.set(transfer.id, {
            transfer,
            bytesReceived: 0,
            lastProgressUpdate: 0,
            stream,
        });
        // CRITICAL: connect() back to sender's stream so uTP handshake completes bidirectionally
        stream.connect(this.udxSocket, senderStreamId, senderUdxPort, peerAddr);
        stream.on('connect', () => {
            this.log(`[LanService][UDX] Receive stream connected for ${transfer.fileName}`);
        });
        stream.on('data', (data) => {
            try {
                if (fdClosed)
                    return;
                fs.writeSync(fd, data);
                const state = this.incomingFiles.get(transfer.id);
                if (state) {
                    state.bytesReceived += data.length;
                    const progress = Math.min(100, Math.round((state.bytesReceived / transfer.fileSize) * 100));
                    state.transfer.progress = progress;
                    const now = Date.now();
                    if (now - state.lastProgressUpdate >= 100) {
                        state.lastProgressUpdate = now;
                        this.db.prepare(`UPDATE file_transfers SET progress = ? WHERE id = ?`).run(progress, transfer.id);
                        if (this.callbacks.onFileTransferProgress) {
                            this.callbacks.onFileTransferProgress(state.transfer);
                        }
                    }
                }
            }
            catch (e) {
                this.error('[LanService][UDX] Write error:', e.message);
            }
        });
        stream.on('end', () => {
            if (fdClosed)
                return;
            fdClosed = true;
            try {
                fs.fsyncSync(fd);
            }
            catch { }
            try {
                fs.closeSync(fd);
            }
            catch { }
            const elapsed = Math.max(0.1, (Date.now() - startTime) / 1000);
            const speed = (transfer.fileSize / 1024 / 1024 / elapsed).toFixed(1);
            this.log(`[LanService][UDX] Receive complete: ${transfer.fileName} (${elapsed.toFixed(1)}s, ${speed} MB/s, RTT=${stream.rtt}ms)`);
            const state = this.incomingFiles.get(transfer.id);
            if (state) {
                state.transfer.status = 'completed';
                state.transfer.completedAt = new Date().toISOString();
                state.transfer.progress = 100;
                this.db.prepare(`UPDATE file_transfers SET status = ?, progress = ?, completedAt = ? WHERE id = ?`)
                    .run('completed', 100, state.transfer.completedAt, transfer.id);
                try {
                    this.db.prepare(`UPDATE chat_messages SET content = ? WHERE id = ? AND type = 'file'`).run(JSON.stringify({ fileName: transfer.fileName, fileSize: transfer.fileSize, filePath: transfer.filePath, status: 'completed', isImage: isImageFile(transfer.fileName) }), transfer.id);
                }
                catch (e) { /* ignore */ }
                try {
                    const msgRow = this.db.prepare('SELECT content FROM chat_messages WHERE id = ? AND type = ?').get(transfer.id, 'file');
                    if (msgRow) {
                        const meta = JSON.parse(msgRow.content);
                        state.transfer.isImage = meta.isImage ?? isImageFile(transfer.fileName);
                    }
                }
                catch { /* ignore */ }
                if (this.callbacks.onFileTransferCompleted) {
                    this.callbacks.onFileTransferCompleted(state.transfer);
                }
                if (this.callbacks.onFileReceived) {
                    this.callbacks.onFileReceived(state.transfer);
                }
                this.incomingFiles.delete(transfer.id);
            }
            stream.end();
        });
        stream.on('error', (err) => {
            this.error(`[LanService][UDX] Receive error: ${err.message}`);
            if (!fdClosed) {
                fdClosed = true;
                try {
                    fs.closeSync(fd);
                }
                catch { }
            }
            const state = this.incomingFiles.get(transfer.id);
            if (state) {
                state.transfer.status = 'error';
                if (this.callbacks.onFileTransferError) {
                    this.callbacks.onFileTransferError(state.transfer);
                }
                this.incomingFiles.delete(transfer.id);
            }
        });
        // Reply file_start_ack to sender — sender will then connect() to this stream
        // Find the peer to get messagePort
        const peer = [...this.peers.values()].find(p => p.address === peerAddr);
        if (peer) {
            this.sendUdpToPeer(peer.id, {
                type: 'file_start_ack',
                id: transfer.id,
                udxStreamId: receiverStreamId,
                udxPort: this.udxPort
            });
            this.log(`[LanService] Sent file_start_ack for ${transfer.fileName} (streamId=${receiverStreamId}, port=${this.udxPort})`);
        }
    }
    /**
     * Handle file_start_ack from receiver — now we can connect our stream.
     */
    handleFileStartAck(data) {
        const fileId = data.id;
        const transfer = this.outgoingTransfers.get(fileId);
        if (!transfer) {
            this.warn('[LanService][UDX] file_start_ack for unknown transfer:', fileId);
            return;
        }
        const receiverStreamId = data.udxStreamId;
        const receiverUdxPort = data.udxPort;
        const peer = this.peers.get(transfer.peerId);
        if (!peer)
            return;
        this.log(`[LanService][UDX] Received file_start_ack for ${transfer.fileName}, connecting stream...`);
        this._startUdxSend(transfer, transfer.filePath, peer.address, receiverStreamId, receiverUdxPort);
    }
    // ========== DATABASE & QUERIES ==========
    /**
     * 保存消息到数据库
     */
    saveMessage(data) {
        if (!data.from || !data.content)
            return;
        const message = {
            id: data.messageId || crypto.randomUUID(),
            fromUserId: data.from,
            fromUserName: data.fromName,
            toUserId: data.to || 'broadcast',
            toUserName: data.toName || 'All',
            content: data.content,
            type: data.type || 'text',
            createdAt: new Date().toISOString(),
            read: false
        };
        this.db.prepare(`
      INSERT INTO chat_messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(message.id, message.fromUserId, message.fromUserName, message.toUserId, message.toUserName, message.content, message.type, message.createdAt, message.read ? 1 : 0);
    }
    /**
     * 获取消息历史记录
     */
    getMessageHistory(limit = 100, offset = 0) {
        const stmt = this.db.prepare(`
      SELECT * FROM chat_messages
      ORDER BY createdAt DESC
      LIMIT ? OFFSET ?
    `);
        return stmt.all(limit, offset);
    }
    /**
     * 获取与指定用户的聊天记录
     */
    getMessagesWithPeer(peerId, limit = 50, offset = 0) {
        const stmt = this.db.prepare(`
      SELECT * FROM chat_messages
      WHERE (fromUserId = ? AND toUserId = ?) OR (fromUserId = ? AND toUserId = ?)
      ORDER BY createdAt DESC
      LIMIT ? OFFSET ?
    `);
        return stmt.all(this.userId, peerId, peerId, this.userId, limit, offset);
    }
    /**
     * 获取两个用户之间的聊天记录（与指定用户的对话历史）
     */
    getMessagesBetween(userId1, userId2, limit, offset) {
        const stmt = this.db.prepare(`
      SELECT * FROM chat_messages
      WHERE (fromUserId = ? AND toUserId = ?) OR (fromUserId = ? AND toUserId = ?)
      ORDER BY createdAt DESC
      LIMIT ? OFFSET ?
    `);
        return stmt.all(userId1, userId2, userId2, userId1, limit, offset);
    }
    /**
     * 标记消息为已读
     */
    markMessagesRead(myUserId, peerId) {
        this.db.prepare(`
      UPDATE chat_messages SET read = 1
      WHERE fromUserId = ? AND toUserId = ? AND read = 0
    `).run(peerId, myUserId);
    }
    /**
     * 获取与指定用户的未读消息数
     */
    getUnreadCount(myUserId, peerId) {
        const row = this.db.prepare(`
      SELECT COUNT(*) as count FROM chat_messages
      WHERE fromUserId = ? AND toUserId = ? AND read = 0
    `).get(peerId, myUserId);
        return row.count;
    }
    /**
     * 获取所有对话的未读数统计
     */
    getAllUnreadCounts(myUserId) {
        const rows = this.db.prepare(`
      SELECT fromUserId, COUNT(*) as count FROM chat_messages
      WHERE toUserId = ? AND read = 0
      GROUP BY fromUserId
    `).all(myUserId);
        const result = {};
        for (const r of rows)
            result[r.fromUserId] = r.count;
        return result;
    }
    /**
     * 投递离线消息（当 peer 上线时自动发送缓存的消息）
     */
    deliverOfflineMessages(peerId) {
        const peer = this.peers.get(peerId);
        if (!peer || !peer.online)
            return;
        const offlineMsgs = this.offlineMessages.get(peerId) || [];
        if (offlineMsgs.length === 0)
            return;
        this.log(`[LanService] Delivering ${offlineMsgs.length} offline messages to ${peer.name}`);
        const failed = [];
        for (const msg of offlineMsgs) {
            this.sendUdpToPeer(peerId, msg);
        }
        // UDP is fire-and-forget for offline delivery; clear queue
        this.offlineMessages.delete(peerId);
    }
    /**
     * 获取文件传输历史记录
     */
    getFileTransferHistory(limit = 100, offset = 0) {
        const stmt = this.db.prepare(`
      SELECT * FROM file_transfers
      ORDER BY createdAt DESC
      LIMIT ? OFFSET ?
    `);
        return stmt.all(limit, offset);
    }
    // ========== COLLABORATION METHODS (all UDP) ==========
    /**
     * 分配任务给用户
     */
    assignTask(peerId, task) {
        const peer = this.peers.get(peerId);
        const messageId = crypto.randomUUID();
        const taskContent = JSON.stringify({
            taskId: task.id || messageId,
            taskText: task.text || '',
            priority: task.priority || 'medium',
            dueDate: task.dueDate || '',
            note: task.note || ''
        });
        // Save to chat_messages
        this.db.prepare(`
      INSERT INTO chat_messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(messageId, this.userId, this.userName, peer?.id || peerId, peer?.name || peerId, taskContent, 'task_assigned', new Date().toISOString(), 0);
        // Send via UDP
        if (peer && peer.online) {
            const data = {
                type: 'task_assigned',
                from: this.userId,
                fromName: this.userName,
                to: peer.id,
                toName: peer.name,
                task: task,
                messageId: messageId,
                timestamp: Date.now()
            };
            this.sendUdpToPeer(peerId, data);
            return true;
        }
        return false;
    }
    /**
     * 同步项目信息给指定用户
     */
    syncProject(peerId, project) {
        const peer = this.peers.get(peerId);
        if (peer && peer.online) {
            const data = {
                type: 'project_sync',
                from: this.userId,
                fromName: this.userName,
                project: project,
                timestamp: Date.now()
            };
            this.sendUdpToPeer(peerId, data);
            return true;
        }
        return false;
    }
    /**
     * 广播项目信息给所有在线用户
     */
    broadcastProject(project) {
        const data = {
            type: 'project_sync',
            from: this.userId,
            fromName: this.userName,
            project: project,
            timestamp: Date.now()
        };
        for (const [peerId] of this.peers) {
            this.sendUdpToPeer(peerId, data);
        }
    }
    /**
     * 广播任务更新
     */
    broadcastTaskUpdate(todo) {
        const data = {
            type: 'task_updated',
            from: this.userId,
            fromName: this.userName,
            todo: todo,
            timestamp: Date.now()
        };
        for (const [peerId] of this.peers) {
            this.sendUdpToPeer(peerId, data);
        }
    }
    /**
     * 广播任务状态变更
     */
    broadcastTaskStatusChange(todo) {
        const data = {
            type: 'task_status_changed',
            from: this.userId,
            fromName: this.userName,
            todo: todo,
            timestamp: Date.now()
        };
        for (const [peerId] of this.peers) {
            this.sendUdpToPeer(peerId, data);
        }
    }
    /**
     * 广播任务评论
     */
    broadcastTaskComment(todoId, comment) {
        const data = {
            type: 'task_comment_added',
            from: this.userId,
            fromName: this.userName,
            todoId: todoId,
            comment: comment,
            timestamp: Date.now()
        };
        for (const [peerId] of this.peers) {
            this.sendUdpToPeer(peerId, data);
        }
    }
    /**
     * 广播协作开始
     */
    broadcastCollaborationStarted(todoId, editorName) {
        const data = {
            type: 'collaboration_started',
            from: this.userId,
            fromName: this.userName,
            todoId: todoId,
            editorName: editorName,
            timestamp: Date.now()
        };
        for (const [peerId] of this.peers) {
            this.sendUdpToPeer(peerId, data);
        }
    }
    /**
     * 广播协作结束
     */
    broadcastCollaborationEnded(todoId, editorName) {
        const data = {
            type: 'collaboration_ended',
            from: this.userId,
            fromName: this.userName,
            todoId: todoId,
            editorName: editorName,
            timestamp: Date.now()
        };
        for (const [peerId] of this.peers) {
            this.sendUdpToPeer(peerId, data);
        }
    }
    /**
     * 获取在线用户列表
     */
    getOnlinePeers() {
        const online = [];
        for (const [_peerId, peer] of this.peers) {
            if (peer.online) {
                online.push({
                    id: peer.id,
                    name: peer.name,
                    address: peer.address,
                    status: peer.status || 'online',
                    avatar: peer.avatar || '😀',
                    version: peer.version
                });
            }
        }
        return online;
    }
    /**
     * 设置回调
     */
    setCallbacks(callbacks) {
        this.callbacks = { ...this.callbacks, ...callbacks };
    }
}
module.exports = LanService;
//# sourceMappingURL=lan-service.js.map