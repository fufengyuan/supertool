"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** Server Management, SFTP, Terminal API + Events */
const electron_1 = require("electron");
exports.default = {
    // Server Management
    getServers: () => electron_1.ipcRenderer.invoke('servers:get-all'),
    getServerById: (serverId) => electron_1.ipcRenderer.invoke('servers:get-by-id', serverId),
    addServer: (server) => electron_1.ipcRenderer.invoke('servers:add', server),
    updateServer: (server) => electron_1.ipcRenderer.invoke('servers:update', server),
    deleteServer: (serverId) => electron_1.ipcRenderer.invoke('servers:delete', serverId),
    getServerGroups: () => electron_1.ipcRenderer.invoke('servers:groups:get-all'),
    addServerGroup: (group) => electron_1.ipcRenderer.invoke('servers:groups:add', group),
    updateServerGroup: (groupId, updates) => electron_1.ipcRenderer.invoke('servers:groups:update', groupId, updates),
    deleteServerGroup: (groupId) => electron_1.ipcRenderer.invoke('servers:groups:delete', groupId),
    testServerConnection: (server) => electron_1.ipcRenderer.invoke('servers:test-connection', server),
    connectServer: (serverId) => electron_1.ipcRenderer.invoke('servers:connect', serverId),
    isServerConnected: (serverId) => electron_1.ipcRenderer.invoke('servers:is-connected', serverId),
    disconnectServer: (serverId) => electron_1.ipcRenderer.invoke('servers:disconnect', serverId),
    execServerCommand: (serverId, command) => electron_1.ipcRenderer.invoke('servers:exec', serverId, command),
    getServerMonitor: (serverId, commands) => electron_1.ipcRenderer.invoke('servers:monitor', serverId, commands),
    // Terminal
    createTerminal: (serverId, terminalId, rows, cols) => electron_1.ipcRenderer.invoke('servers:terminal:create', serverId, terminalId, rows, cols),
    writeTerminal: (terminalId, data) => electron_1.ipcRenderer.invoke('servers:terminal:write', terminalId, data),
    closeTerminal: (terminalId) => electron_1.ipcRenderer.invoke('servers:terminal:close', terminalId),
    resizeTerminal: (terminalId, rows, cols) => electron_1.ipcRenderer.invoke('servers:terminal:resize', terminalId, rows, cols),
    // SFTP
    listSftpDir: (serverId, remotePath) => electron_1.ipcRenderer.invoke('servers:sftp:list', serverId, remotePath),
    downloadFile: (serverId, remotePath, localPath) => electron_1.ipcRenderer.invoke('servers:sftp:download', serverId, remotePath, localPath),
    uploadFile: (serverId, localPath, remotePath) => electron_1.ipcRenderer.invoke('servers:sftp:upload', serverId, localPath, remotePath),
    uploadFiles: (serverId, remotePath, filePaths) => electron_1.ipcRenderer.invoke('servers:sftp:upload-files', serverId, remotePath, filePaths),
    uploadFolder: (serverId, localDirPath, remotePath) => electron_1.ipcRenderer.invoke('servers:sftp:upload-folder', serverId, localDirPath, remotePath),
    uploadDroppedItems: (serverId, remotePath, items) => electron_1.ipcRenderer.invoke('servers:sftp:upload-dropped-items', serverId, remotePath, items),
    uploadSessionStart: () => electron_1.ipcRenderer.invoke('sftp:upload-session-start'),
    uploadSessionAdd: (sessionId, items) => electron_1.ipcRenderer.invoke('sftp:upload-session-add', sessionId, items),
    uploadSessionCheckConflicts: (sessionId, serverId, remotePath) => electron_1.ipcRenderer.invoke('sftp:upload-session-check-conflicts', sessionId, serverId, remotePath),
    uploadSessionCommit: (sessionId, serverId, remotePath, options) => electron_1.ipcRenderer.invoke('sftp:upload-session-commit', sessionId, serverId, remotePath, options),
    uploadSessionCancel: (sessionId) => electron_1.ipcRenderer.invoke('sftp:upload-session-cancel', sessionId),
    uploadFileFromBuffer: (serverId, remotePath, buffer) => electron_1.ipcRenderer.invoke('servers:sftp:upload-from-buffer', serverId, remotePath, buffer),
    showOpenDialogSftp: () => electron_1.ipcRenderer.invoke('sftp:show-open-dialog'),
    showOpenDialogForDirs: () => electron_1.ipcRenderer.invoke('sftp:show-open-dialog-dirs'),
    getDownloadsDir: () => electron_1.ipcRenderer.invoke('sftp:get-downloads-dir'),
    readDirectory: (dirPath) => electron_1.ipcRenderer.invoke('fs:readdir', dirPath),
    createSftpDir: (serverId, remotePath) => electron_1.ipcRenderer.invoke('servers:sftp:create-dir', serverId, remotePath),
    deleteSftpFile: (serverId, remotePath) => electron_1.ipcRenderer.invoke('servers:sftp:delete', serverId, remotePath),
    // SFTP Remote File Editor
    openSftpFileEditor: (serverId, remotePath) => electron_1.ipcRenderer.invoke('sftp:open-editor', serverId, remotePath),
    // SFTP Progress Events
    onSftpUploadProgress: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('sftp:upload-progress', wrapper);
        return () => electron_1.ipcRenderer.removeListener('sftp:upload-progress', wrapper);
    },
    onSftpDownloadProgress: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('sftp:download-progress', wrapper);
        return () => electron_1.ipcRenderer.removeListener('sftp:download-progress', wrapper);
    },
    onSftpUploadDone: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('sftp:upload-done', wrapper);
        return () => electron_1.ipcRenderer.removeListener('sftp:upload-done', wrapper);
    },
    // Server Events
    onServerConnected: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('server:connected', wrapper);
        return () => electron_1.ipcRenderer.removeListener('server:connected', wrapper);
    },
    onServerDisconnected: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('server:disconnected', wrapper);
        return () => electron_1.ipcRenderer.removeListener('server:disconnected', wrapper);
    },
    onServerHeartbeatFailed: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('server:heartbeat-failed', wrapper);
        return () => electron_1.ipcRenderer.removeListener('server:heartbeat-failed', wrapper);
    },
    onTerminalData: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('terminal:data', wrapper);
        return () => electron_1.ipcRenderer.removeListener('terminal:data', wrapper);
    },
    onTerminalClose: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('terminal:close', wrapper);
        return () => electron_1.ipcRenderer.removeListener('terminal:close', wrapper);
    },
};
//# sourceMappingURL=preload-servers.js.map