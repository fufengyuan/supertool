import { Client } from 'ssh2';
import EventEmitter = require('events');
interface Server {
    id: string;
    name: string;
    host: string;
    port?: number;
    username: string;
    sshKeyPath?: string;
    password?: string;
    description?: string;
    tags?: string[];
    groupId?: string;
    createdAt: string;
    updatedAt: string;
}
interface SftpFile {
    name: string;
    type: 'directory' | 'file';
    size: number;
    modifyTime: string;
    permissions: number;
}
interface ExecResult {
    success: boolean;
    output: string;
    errorOutput: string;
    exitCode: number | null;
}
interface ConnectResult {
    success: boolean;
    serverId?: string;
    terminalId?: string;
}
declare class ServerService extends EventEmitter {
    private connections;
    private sftpSessions;
    private terminals;
    connect(server: Server): Promise<ConnectResult>;
    disconnect(serverId: string): {
        success: boolean;
    };
    testConnection(server: Server): Promise<{
        success: boolean;
    }>;
    execCommand(serverId: string, command: string): Promise<ExecResult>;
    streamCommand(serverId: string, command: string, onLine: (line: string) => void, onEnd?: () => void, onError?: (err: string) => void): Promise<{
        stop: () => void;
    }>;
    createTerminal(serverId: string, terminalId: string, rows?: number, cols?: number): Promise<ConnectResult>;
    resizeTerminal(terminalId: string, rows: number, cols: number): {
        success: boolean;
    };
    writeToTerminal(terminalId: string, data: string): {
        success: boolean;
    };
    closeTerminal(terminalId: string): {
        success: boolean;
    };
    createSftp(serverId: string): Promise<ConnectResult>;
    listRemoteDir(serverId: string, remotePath: string): Promise<SftpFile[]>;
    downloadFile(serverId: string, remotePath: string, localPath: string, progress?: (bytesTransferred: number, totalBytes: number, speed: number) => void): Promise<{
        success: boolean;
        localPath: string;
    }>;
    uploadFile(serverId: string, localPath: string, remotePath: string, progress?: (bytesTransferred: number, totalBytes: number, speed: number) => void): Promise<{
        success: boolean;
        remotePath: string;
    }>;
    createRemoteDir(serverId: string, remotePath: string): Promise<{
        success: boolean;
    }>;
    deleteRemoteFile(serverId: string, remotePath: string): Promise<{
        success: boolean;
    }>;
    deleteRemoteDir(serverId: string, remotePath: string): Promise<{
        success: boolean;
    }>;
    isConnected(serverId: string): boolean;
    getConnection(serverId: string): Client | undefined;
    getActiveConnections(): string[];
}
export = ServerService;
