import EventEmitter = require('events');
interface Peer {
    id: string;
    name: string;
    avatar?: string;
    address: string;
    messagePort: number;
    version?: string;
    lastSeen: number;
    online: boolean;
    status?: 'online' | 'busy' | 'away';
}
interface LanMessage {
    type: string;
    from?: string;
    fromName?: string;
    to?: string;
    toName?: string;
    content?: string;
    timestamp?: number;
    task?: Record<string, unknown>;
    project?: Record<string, unknown>;
    todo?: Record<string, unknown>;
    todoId?: string;
    comment?: string;
    editorName?: string;
    [key: string]: unknown;
}
interface FileTransfer {
    id: string;
    fromUserId: string;
    fromUserName: string;
    toUserId: string;
    toUserName: string;
    fileName: string;
    fileSize: number;
    filePath?: string;
    status: string;
    progress: number;
    createdAt: string;
    completedAt?: string;
    _completing?: boolean;
}
interface Callbacks {
    onPeerDiscovered: ((peer: Peer) => void) | null;
    onPeerLost: ((peer: Peer) => void) | null;
    onMessage: ((data: LanMessage) => void) | null;
    onTaskAssigned: ((data: LanMessage) => void) | null;
    onProjectSync: ((data: LanMessage) => void) | null;
    onTaskUpdated: ((data: LanMessage) => void) | null;
    onTaskStatusChanged: ((data: LanMessage) => void) | null;
    onTaskCommentAdded: ((data: LanMessage) => void) | null;
    onCollaborationStarted: ((data: LanMessage) => void) | null;
    onCollaborationEnded: ((data: LanMessage) => void) | null;
    onFileTransferStarted: ((data: FileTransfer) => void) | null;
    onFileTransferProgress: ((data: FileTransfer) => void) | null;
    onFileTransferCompleted: ((data: FileTransfer) => void) | null;
    onFileTransferError: ((data: FileTransfer) => void) | null;
    onFileReceived: ((data: FileTransfer) => void) | null;
}
declare class LanService extends EventEmitter {
    private udpSocket;
    private peers;
    private messagePort;
    private isRunning;
    private userDataPath;
    private incomingFiles;
    private udx;
    private udxSocket;
    private udxPort;
    private outgoingTransfers;
    private messageAckWaiters;
    private offlineMessages;
    private _logBuffer;
    private readonly MAX_LOG_ENTRIES;
    private _logStream;
    private readonly LOG_RETENTION_DAYS;
    private _networkPermissionGranted;
    private _networkPermissionDetails;
    /** Check if network permission was granted (null = not yet checked) */
    get networkPermissionGranted(): boolean | null;
    get networkPermissionDetails(): string | null;
    /**
     * 检查并请求 macOS 局域网权限
     */
    checkNetworkPermission(): Promise<{
        granted: boolean;
        details: string;
    }>;
    /** Get log directory for current platform — unified under ~/.supertool/logs/ */
    getLogDir(): string;
    /** Initialize log file */
    private _initLogFile;
    /** Delete log files older than LOG_RETENTION_DAYS */
    private _cleanupOldLogs;
    /** Get recent logs */
    getLogs(limit?: number): Array<{
        time: string;
        level: string;
        message: string;
    }>;
    /** Get log file path */
    get logFilePath(): string | null;
    /** Internal log method */
    private _log;
    private log;
    private warn;
    private error;
    private callbacks;
    get userId(): string;
    get userName(): string;
    get avatar(): string;
    get myStatusDisplay(): string;
    private _networkInfoCache;
    private _networkInfoCachedAt;
    get networkInfo(): {
        address: string;
        ports: string;
    };
    private _userId;
    private _userName;
    private _nickName;
    private _avatar;
    private _receivePath;
    private myStatus;
    private readonly VERSION;
    private db;
    constructor(userDataPath: string);
    private loadReceivePath;
    getReceivePath(): string;
    setReceivePath(dirPath: string): void;
    private loadProfile;
    setNickName(name: string): void;
    setAvatar(emoji: string): void;
    /**
     * 设置在线状态
     */
    setStatus(status: 'online' | 'busy' | 'away'): void;
    /**
     * 加载用户状态
     */
    private loadStatus;
    private initDatabase;
    /**
     * 启动局域网服务
     */
    start(): Promise<void>;
    /**
     * 停止局域网服务
     */
    stop(): void;
    private broadcastTimers;
    private heartbeatTimer;
    private offlineCheckTimer;
    /**
     * 启动单一 UDP socket — 处理发现、消息、文件传输
     */
    private startUdpSocket;
    /**
     * 自动重启 UDP socket
     */
    private restartUdpSocket;
    private _broadcastAddrsCache;
    private _broadcastAddrsCachedAt;
    /**
     * 获取所有子网广播地址
     */
    private getBroadcastAddresses;
    /**
     * 广播存在
     */
    private broadcastPresence;
    /**
     * 广播心跳
     */
    private broadcastHeartbeat;
    /**
     * 回复发现请求
     */
    private replyToDiscovery;
    /**
     * 手动触发重新扫描
     */
    refreshDiscovery(): void;
    /**
     * 处理发现新节点
     */
    private handlePeerDiscovered;
    /**
     * 处理心跳
     */
    private handleHeartbeat;
    /**
     * 检查离线节点
     */
    private checkOfflinePeers;
    /**
     * 发送 UDP 消息到指定 peer
     */
    private sendUdpToPeer;
    /** 发送原始 Buffer（文件传输用，不经过 JSON） */
    private sendUdpBuffer;
    /**
     * Handle incoming chat message
     */
    private handleIncomingMessage;
    /**
     * Handle message ACK
     */
    private handleMessageAck;
    /**
     * 发送消息给指定用户（UDP + ACK 机制）
     */
    sendMessage(peerId: string, message: string): {
        success: boolean;
        messageId: string;
        queued?: boolean;
        incompatibleVersion?: string;
    };
    /**
     * Send a message with ACK mechanism (retry up to 3 times, 2s timeout)
     */
    private sendWithAck;
    /**
     * 广播消息给所有在线用户
     */
    broadcastMessage(message: string): void;
    /**
     * 发送文件给指定用户（UDP sliding window with ACK）
     */
    /**
     * 初始化 UDX socket 用于可靠文件传输
     */
    private initUdxSocket;
    /**
     * 发送文件给指定用户（UDX Stream — 内置可靠传输+拥塞控制）
     */
    sendFile(peerId: string, filePath: string, fileName: string, _resumeOffset?: number, existingId?: string): boolean;
    /**
     * Start UDX stream to send file — called AFTER receiving file_start_ack from receiver.
     * At this point the receiver has created its stream and is ready to accept connections.
     */
    private _startUdxSend;
    /**
     * Handle incoming file_start
     */
    private handleFileStart;
    /**
     * Prepare UDX receive: create stream, connect() back to sender, reply ACK.
     * UDX requires both sides to connect() for the uTP handshake to complete.
     * Receiver connect()s to senderStreamId, sender connect()s to receiverStreamId.
     */
    private _prepareUdxReceive;
    /**
     * Handle file_start_ack from receiver — now we can connect our stream.
     */
    private handleFileStartAck;
    /**
     * 保存消息到数据库
     */
    private saveMessage;
    /**
     * 获取消息历史记录
     */
    getMessageHistory(limit?: number, offset?: number): any[];
    /**
     * 获取与指定用户的聊天记录
     */
    getMessagesWithPeer(peerId: string, limit?: number, offset?: number): any[];
    /**
     * 获取两个用户之间的聊天记录（与指定用户的对话历史）
     */
    getMessagesBetween(userId1: string, userId2: string, limit: number, offset: number): any[];
    /**
     * 标记消息为已读
     */
    markMessagesRead(myUserId: string, peerId: string): void;
    /**
     * 获取与指定用户的未读消息数
     */
    getUnreadCount(myUserId: string, peerId: string): number;
    /**
     * 获取所有对话的未读数统计
     */
    getAllUnreadCounts(myUserId: string): Record<string, number>;
    /**
     * 投递离线消息（当 peer 上线时自动发送缓存的消息）
     */
    private deliverOfflineMessages;
    /**
     * 获取文件传输历史记录
     */
    getFileTransferHistory(limit?: number, offset?: number): any[];
    /**
     * 分配任务给用户
     */
    assignTask(peerId: string, task: Record<string, unknown>): boolean;
    /**
     * 同步项目信息给指定用户
     */
    syncProject(peerId: string, project: Record<string, unknown>): boolean;
    /**
     * 广播项目信息给所有在线用户
     */
    broadcastProject(project: Record<string, unknown>): void;
    /**
     * 广播任务更新
     */
    broadcastTaskUpdate(todo: Record<string, unknown>): void;
    /**
     * 广播任务状态变更
     */
    broadcastTaskStatusChange(todo: Record<string, unknown>): void;
    /**
     * 广播任务评论
     */
    broadcastTaskComment(todoId: string, comment: string): void;
    /**
     * 广播协作开始
     */
    broadcastCollaborationStarted(todoId: string, editorName: string): void;
    /**
     * 广播协作结束
     */
    broadcastCollaborationEnded(todoId: string, editorName: string): void;
    /**
     * 获取在线用户列表
     */
    getOnlinePeers(): {
        id: string;
        name: string;
        address: string;
        status: string;
        avatar: string;
        version?: string;
    }[];
    /**
     * 设置回调
     */
    setCallbacks(callbacks: Partial<Callbacks>): void;
}
export = LanService;
