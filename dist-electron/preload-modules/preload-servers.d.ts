import type { Server } from './preload-core';
declare const _default: {
    getServers: () => Promise<any>;
    getServerById: (serverId: string) => Promise<any>;
    addServer: (server: Server) => Promise<any>;
    updateServer: (server: Server) => Promise<any>;
    deleteServer: (serverId: string) => Promise<any>;
    getServerGroups: () => Promise<any>;
    addServerGroup: (group: {
        id?: string;
        name: string;
        description?: string;
        parentId?: string | null;
        color?: string;
    }) => Promise<any>;
    updateServerGroup: (groupId: string, updates: {
        name?: string;
        description?: string;
        parentId?: string | null;
        color?: string;
    }) => Promise<any>;
    deleteServerGroup: (groupId: string) => Promise<any>;
    testServerConnection: (server: Server) => Promise<any>;
    connectServer: (serverId: string) => Promise<any>;
    isServerConnected: (serverId: string) => Promise<any>;
    disconnectServer: (serverId: string) => Promise<any>;
    execServerCommand: (serverId: string, command: string) => Promise<any>;
    getServerMonitor: (serverId: string, commands: string[]) => Promise<any>;
    createTerminal: (serverId: string, terminalId: string, rows?: number, cols?: number) => Promise<any>;
    writeTerminal: (terminalId: string, data: string) => Promise<any>;
    closeTerminal: (terminalId: string) => Promise<any>;
    resizeTerminal: (terminalId: string, rows: number, cols: number) => Promise<any>;
    listSftpDir: (serverId: string, remotePath: string) => Promise<any>;
    downloadFile: (serverId: string, remotePath: string, localPath: string) => Promise<any>;
    uploadFile: (serverId: string, localPath: string, remotePath: string) => Promise<any>;
    uploadFiles: (serverId: string, remotePath: string, filePaths: string[]) => Promise<any>;
    uploadFolder: (serverId: string, localDirPath: string, remotePath: string) => Promise<any>;
    uploadDroppedItems: (serverId: string, remotePath: string, items: Array<{
        relativePath: string;
        data: number[];
    }>) => Promise<any>;
    uploadSessionStart: () => Promise<any>;
    uploadSessionAdd: (sessionId: string, items: Array<{
        relativePath: string;
        data: number[];
    }>) => Promise<any>;
    uploadSessionCheckConflicts: (sessionId: string, serverId: string, remotePath: string) => Promise<any>;
    uploadSessionCommit: (sessionId: string, serverId: string, remotePath: string, options?: {
        overwrite?: boolean;
    }) => Promise<any>;
    uploadSessionCancel: (sessionId: string) => Promise<any>;
    uploadFileFromBuffer: (serverId: string, remotePath: string, buffer: number[]) => Promise<any>;
    showOpenDialogSftp: () => Promise<any>;
    showOpenDialogForDirs: () => Promise<any>;
    getDownloadsDir: () => Promise<any>;
    readDirectory: (dirPath: string) => Promise<any>;
    createSftpDir: (serverId: string, remotePath: string) => Promise<any>;
    deleteSftpFile: (serverId: string, remotePath: string) => Promise<any>;
    openSftpFileEditor: (serverId: string, remotePath: string) => Promise<any>;
    onSftpUploadProgress: (callback: (data: {
        serverId: string;
        percent: number;
        message: string;
        speed?: number;
        speedFormatted?: string;
    }) => void) => () => Electron.IpcRenderer;
    onSftpDownloadProgress: (callback: (data: {
        serverId: string;
        percent: number;
        message: string;
        speed?: number;
        speedFormatted?: string;
    }) => void) => () => Electron.IpcRenderer;
    onSftpUploadDone: (callback: (data: {
        serverId: string;
    }) => void) => () => Electron.IpcRenderer;
    onServerConnected: (callback: (data: Record<string, unknown>) => void) => () => Electron.IpcRenderer;
    onServerDisconnected: (callback: (data: Record<string, unknown>) => void) => () => Electron.IpcRenderer;
    onServerHeartbeatFailed: (callback: (data: {
        serverId: string;
    }) => void) => () => Electron.IpcRenderer;
    onTerminalData: (callback: (data: {
        terminalId: string;
        data: string;
    }) => void) => () => Electron.IpcRenderer;
    onTerminalClose: (callback: (data: {
        terminalId: string;
    }) => void) => () => Electron.IpcRenderer;
};
export default _default;
