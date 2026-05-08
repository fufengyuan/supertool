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
const logger_1 = require("../logger");
const ssh2_1 = require("ssh2");
const EventEmitter = require("events");
const fs = __importStar(require("fs"));
class ServerService extends EventEmitter {
    constructor() {
        super(...arguments);
        this.connections = new Map(); // serverId -> Client
        this.sftpSessions = new Map(); // serverId -> SFTP session
        this.terminals = new Map(); // terminalId -> stream
    }
    // 连接服务器
    async connect(server) {
        // 如果已经连接，直接返回成功
        if (this.connections.has(server.id)) {
            (0, logger_1.info)(`[ServerService] Already connected to ${server.id} ${server.name}`);
            return { success: true, serverId: server.id };
        }
        (0, logger_1.info)(`[ServerService] Connecting to ${server.id} ${server.host}:${server.port || 22} as ${server.username} ${server.sshKeyPath ? `(key: ${server.sshKeyPath})` : '(password)'}`);
        return new Promise((resolve, reject) => {
            const conn = new ssh2_1.Client();
            conn.on('ready', () => {
                (0, logger_1.info)('[ServerService] SSH ready for', server.id, server.name);
                this.connections.set(server.id, conn);
                this.emit('connected', { serverId: server.id, server });
                resolve({ success: true, serverId: server.id });
            });
            conn.on('error', (err) => {
                console.error('[ServerService] SSH error for', server.id, err.message);
                this.emit('connection-error', { serverId: server.id, error: err.message });
                reject(err);
            });
            conn.on('close', () => {
                (0, logger_1.info)('[ServerService] SSH closed for', server.id);
                this.connections.delete(server.id);
                this.sftpSessions.delete(server.id);
                this.emit('disconnected', { serverId: server.id });
            });
            const config = {
                host: server.host,
                port: server.port || 22,
                username: server.username,
                readyTimeout: 20000, // 20s timeout for SSH handshake
                keepaliveInterval: 30000,
                keepaliveCountMax: 3,
            };
            if (server.sshKeyPath) {
                try {
                    config.privateKey = fs.readFileSync(server.sshKeyPath);
                }
                catch (e) {
                    return reject(new Error(`读取 SSH 密钥失败: ${e.message}`));
                }
            }
            else if (server.password) {
                config.password = server.password;
            }
            (0, logger_1.info)('[ServerService] Calling conn.connect() with config:', JSON.stringify({ host: config.host, port: config.port, username: config.username, readyTimeout: config.readyTimeout }));
            conn.connect(config);
        });
    }
    // 断开连接
    disconnect(serverId) {
        const conn = this.connections.get(serverId);
        if (conn) {
            conn.end();
            this.connections.delete(serverId);
            this.sftpSessions.delete(serverId);
        }
        return { success: true };
    }
    // 测试连接
    async testConnection(server) {
        return new Promise((resolve, reject) => {
            const conn = new ssh2_1.Client();
            conn.on('ready', () => {
                conn.end();
                resolve({ success: true });
            });
            conn.on('error', (err) => {
                reject(err);
            });
            const config = {
                host: server.host,
                port: server.port || 22,
                username: server.username,
                readyTimeout: 10000
            };
            if (server.sshKeyPath) {
                try {
                    config.privateKey = fs.readFileSync(server.sshKeyPath);
                }
                catch (e) {
                    return reject(new Error(`读取 SSH 密钥失败: ${e.message}`));
                }
            }
            else if (server.password) {
                config.password = server.password;
            }
            conn.connect(config);
        });
    }
    // 执行命令
    async execCommand(serverId, command) {
        const conn = this.connections.get(serverId);
        if (!conn) {
            throw new Error('服务器未连接');
        }
        return new Promise((resolve, reject) => {
            conn.exec(command, (err, stream) => {
                if (err)
                    return reject(err);
                let output = '';
                let errorOutput = '';
                stream.on('data', (data) => {
                    output += data.toString();
                    this.emit('command-output', { serverId, output: data.toString() });
                });
                stream.stderr?.on('data', (data) => {
                    errorOutput += data.toString();
                });
                stream.on('error', (err) => {
                    reject(err);
                });
                stream.on('close', (code) => {
                    resolve({
                        success: code === 0,
                        output,
                        errorOutput,
                        exitCode: code
                    });
                });
            });
        });
    }
    // 流式执行命令（用于日志查看等长输出场景）
    async streamCommand(serverId, command, onLine, onEnd, onError) {
        const conn = this.connections.get(serverId);
        if (!conn) {
            throw new Error('服务器未连接');
        }
        let buffer = '';
        let stopped = false;
        const stream = await new Promise((resolve, reject) => {
            conn.exec(command, (err, s) => {
                if (err) {
                    onError?.(err.message);
                    reject(err);
                }
                else {
                    resolve(s);
                }
            });
        });
        stream.on('data', (data) => {
            if (stopped)
                return;
            buffer += data.toString();
            const lines = buffer.split('\n');
            buffer = lines.pop() || '';
            lines.forEach(line => {
                if (line.trim())
                    onLine(line);
            });
        });
        stream.stderr?.on('data', (data) => {
            if (stopped)
                return;
            onError?.(data.toString());
        });
        stream.on('close', () => {
            if (!stopped) {
                onEnd?.();
            }
        });
        stream.on('error', (err) => {
            if (!stopped) {
                onError?.(err.message);
            }
        });
        return {
            stop: () => {
                stopped = true;
                // Try to kill the remote process first (SIGTERM), then close channel
                try {
                    stream.kill?.('SIGTERM');
                }
                catch { }
                try {
                    stream.end();
                }
                catch { }
                try {
                    stream.destroy();
                }
                catch { }
            }
        };
    }
    // 创建终端会话（请求 PTY 以支持交互式程序）
    async createTerminal(serverId, terminalId, rows = 24, cols = 80) {
        const conn = this.connections.get(serverId);
        (0, logger_1.info)('[ServerService] createTerminal: serverId=', serverId, 'terminalId=', terminalId, 'conn=', conn ? 'exists' : 'MISSING', `pty=${cols}x${rows}`);
        if (!conn) {
            throw new Error('服务器未连接');
        }
        return new Promise((resolve, reject) => {
            (0, logger_1.info)('[ServerService] Calling conn.shell() for', serverId, `pty=${cols}x${rows}`);
            conn.shell({ term: 'xterm-256color', rows, cols }, (err, stream) => {
                if (err) {
                    console.error('[ServerService] conn.shell() error for', serverId, err.message);
                    return reject(err);
                }
                (0, logger_1.info)('[ServerService] conn.shell() success for', serverId, 'terminalId=', terminalId);
                this.terminals.set(terminalId, stream);
                stream.on('data', (data) => {
                    this.emit('terminal-data', { terminalId, data: data.toString() });
                });
                stream.on('close', () => {
                    (0, logger_1.info)('[ServerService] terminal stream closed for', terminalId);
                    this.terminals.delete(terminalId);
                    this.emit('terminal-close', { terminalId });
                });
                stream.on('error', (err) => {
                    console.error('[ServerService] terminal stream error for', terminalId, err.message);
                    this.terminals.delete(terminalId);
                    this.emit('terminal-close', { terminalId });
                });
                resolve({ success: true, terminalId });
            });
        });
    }
    // 调整终端 PTY 尺寸（发送 SIGWINCH 信号）
    resizeTerminal(terminalId, rows, cols) {
        const stream = this.terminals.get(terminalId);
        if (!stream) {
            return { success: false };
        }
        try {
            stream.setWindow(rows, cols);
            return { success: true };
        }
        catch (e) {
            console.error('[ServerService] resizeTerminal failed:', e.message);
            return { success: false };
        }
    }
    // 向终端写入数据
    writeToTerminal(terminalId, data) {
        const stream = this.terminals.get(terminalId);
        if (!stream) {
            throw new Error('终端不存在');
        }
        // Check if stream is still writable
        if (stream.writable === false) {
            this.terminals.delete(terminalId);
            throw new Error('终端连接已断开');
        }
        try {
            stream.write(data);
        }
        catch (e) {
            this.terminals.delete(terminalId);
            throw new Error(`写入终端失败: ${e.message}`);
        }
        return { success: true };
    }
    // 关闭终端
    closeTerminal(terminalId) {
        const stream = this.terminals.get(terminalId);
        if (stream) {
            try {
                stream.end();
            }
            catch { }
            this.terminals.delete(terminalId);
        }
        return { success: true };
    }
    // 创建 SFTP 会话
    async createSftp(serverId) {
        const conn = this.connections.get(serverId);
        if (!conn) {
            throw new Error('服务器未连接');
        }
        return new Promise((resolve, reject) => {
            conn.sftp((err, sftp) => {
                if (err)
                    return reject(err);
                this.sftpSessions.set(serverId, sftp);
                resolve({ success: true, serverId });
            });
        });
    }
    // 列出远程目录
    async listRemoteDir(serverId, remotePath) {
        let sftp = this.sftpSessions.get(serverId);
        if (!sftp) {
            await this.createSftp(serverId);
            sftp = this.sftpSessions.get(serverId);
        }
        return new Promise((resolve, reject) => {
            sftp.readdir(remotePath, (err, list) => {
                if (err)
                    return reject(err);
                const files = (list || []).map(item => ({
                    name: item.filename,
                    type: item.longname.startsWith('d') ? 'directory' : 'file',
                    size: item.attrs.size,
                    modifyTime: new Date(item.attrs.mtime * 1000).toISOString(),
                    permissions: item.attrs.mode
                }));
                resolve(files);
            });
        });
    }
    // 下载文件（带进度回调）
    async downloadFile(serverId, remotePath, localPath, progress) {
        let sftp = this.sftpSessions.get(serverId);
        if (!sftp) {
            await this.createSftp(serverId);
            sftp = this.sftpSessions.get(serverId);
        }
        return new Promise((resolve, reject) => {
            sftp.fastGet(remotePath, localPath, { step: progress }, (err) => {
                if (err)
                    return reject(err);
                resolve({ success: true, localPath });
            });
        });
    }
    // 上传文件（带进度回调）
    async uploadFile(serverId, localPath, remotePath, progress) {
        let sftp = this.sftpSessions.get(serverId);
        if (!sftp) {
            await this.createSftp(serverId);
            sftp = this.sftpSessions.get(serverId);
        }
        return new Promise((resolve, reject) => {
            sftp.fastPut(localPath, remotePath, { step: progress }, (err) => {
                if (err)
                    return reject(err);
                resolve({ success: true, remotePath });
            });
        });
    }
    // 创建远程目录
    async createRemoteDir(serverId, remotePath) {
        let sftp = this.sftpSessions.get(serverId);
        if (!sftp) {
            await this.createSftp(serverId);
            sftp = this.sftpSessions.get(serverId);
        }
        return new Promise((resolve, reject) => {
            sftp.mkdir(remotePath, (err) => {
                if (err)
                    return reject(err);
                resolve({ success: true });
            });
        });
    }
    // 删除远程文件
    async deleteRemoteFile(serverId, remotePath) {
        let sftp = this.sftpSessions.get(serverId);
        if (!sftp) {
            await this.createSftp(serverId);
            sftp = this.sftpSessions.get(serverId);
        }
        return new Promise((resolve, reject) => {
            sftp.unlink(remotePath, (err) => {
                if (err)
                    return reject(err);
                resolve({ success: true });
            });
        });
    }
    // 删除远程目录
    async deleteRemoteDir(serverId, remotePath) {
        let sftp = this.sftpSessions.get(serverId);
        if (!sftp) {
            await this.createSftp(serverId);
            sftp = this.sftpSessions.get(serverId);
        }
        return new Promise((resolve, reject) => {
            sftp.rmdir(remotePath, (err) => {
                if (err)
                    return reject(err);
                resolve({ success: true });
            });
        });
    }
    // 获取连接状态
    isConnected(serverId) {
        return this.connections.has(serverId);
    }
    // 获取连接的 SSH Client（给主进程用）
    getConnection(serverId) {
        return this.connections.get(serverId);
    }
    // 获取所有连接
    getActiveConnections() {
        return Array.from(this.connections.keys());
    }
}
module.exports = ServerService;
//# sourceMappingURL=server-service.js.map