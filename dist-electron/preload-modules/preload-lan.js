"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** LAN, File Transfer, Collaboration API + Events */
const electron_1 = require("electron");
exports.default = {
    // LAN API
    getPeers: () => electron_1.ipcRenderer.invoke('lan:get-peers'),
    sendMessage: (peerId, message) => electron_1.ipcRenderer.invoke('lan:send-message', peerId, message),
    showOpenDialog: (options) => electron_1.ipcRenderer.invoke('lan:show-open-dialog', options),
    broadcastMessage: (message) => electron_1.ipcRenderer.invoke('lan:broadcast-message', message),
    assignTask: (peerId, task) => electron_1.ipcRenderer.invoke('lan:assign-task', peerId, task),
    syncTaskStatus: (todo) => electron_1.ipcRenderer.invoke('lan:sync-task-status', todo),
    getUserInfo: () => electron_1.ipcRenderer.invoke('lan:get-user-info'),
    setNickName: (name) => electron_1.ipcRenderer.invoke('lan:set-nickname', name),
    setAvatar: (emoji) => electron_1.ipcRenderer.invoke('lan:set-avatar', emoji),
    setStatus: (status) => electron_1.ipcRenderer.invoke('lan:set-status', status),
    getStatus: () => electron_1.ipcRenderer.invoke('lan:get-status'),
    getNetworkInfo: () => electron_1.ipcRenderer.invoke('lan:get-network-info'),
    // Network permission check (macOS Local Network Privacy)
    checkNetworkPermission: () => electron_1.ipcRenderer.invoke('lan:check-network-permission'),
    getPermissionStatus: () => electron_1.ipcRenderer.invoke('lan:get-permission-status'),
    refreshDiscovery: () => electron_1.ipcRenderer.invoke('lan:refresh-discovery'),
    getReceivePath: () => electron_1.ipcRenderer.invoke('lan:get-receive-path'),
    setReceivePath: (path) => electron_1.ipcRenderer.invoke('lan:set-receive-path', path),
    openFileFolder: (filePath) => electron_1.ipcRenderer.invoke('lan:open-file-folder', filePath),
    openFile: (filePath) => electron_1.ipcRenderer.invoke('lan:open-file', filePath),
    // LAN Collaboration
    broadcastTaskUpdate: (todo) => electron_1.ipcRenderer.invoke('lan:broadcast-task-update', todo),
    broadcastTaskStatusChange: (todo) => electron_1.ipcRenderer.invoke('lan:broadcast-task-status-change', todo),
    broadcastTaskComment: (todoId, comment) => electron_1.ipcRenderer.invoke('lan:broadcast-task-comment', todoId, comment),
    broadcastCollaborationStarted: (todoId, editorName) => electron_1.ipcRenderer.invoke('lan:broadcast-collaboration-started', todoId, editorName),
    broadcastCollaborationEnded: (todoId, editorName) => electron_1.ipcRenderer.invoke('lan:broadcast-collaboration-ended', todoId, editorName),
    // File Transfer
    sendFile: (peerId, filePath, fileName, resumeOffset, fileId) => electron_1.ipcRenderer.invoke('lan:send-file', peerId, filePath, fileName, resumeOffset || 0, fileId),
    getMessageHistory: (limit, offset) => electron_1.ipcRenderer.invoke('lan:get-message-history', limit, offset),
    getMessagesBetween: (userId1, userId2, limit, offset) => electron_1.ipcRenderer.invoke('lan:get-messages-between', userId1, userId2, limit, offset),
    markMessagesRead: (myUserId, peerId) => electron_1.ipcRenderer.invoke('lan:mark-messages-read', myUserId, peerId),
    getUnreadCount: (myUserId, peerId) => electron_1.ipcRenderer.invoke('lan:get-unread-count', myUserId, peerId),
    getAllUnreadCounts: (myUserId) => electron_1.ipcRenderer.invoke('lan:get-all-unread-counts', myUserId),
    getFileTransferHistory: (limit, offset) => electron_1.ipcRenderer.invoke('lan:get-file-transfer-history', limit, offset),
    // LAN Events
    onLanPeerDiscovered: (callback) => {
        const wrapper = (_event, peer) => callback(peer);
        electron_1.ipcRenderer.on('lan:peer-discovered', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:peer-discovered', wrapper);
    },
    onLanPeerLost: (callback) => {
        const wrapper = (_event, peer) => callback(peer);
        electron_1.ipcRenderer.on('lan:peer-lost', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:peer-lost', wrapper);
    },
    onMessage: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:message', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:message', wrapper);
    },
    onTaskAssigned: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:task-assigned', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:task-assigned', wrapper);
    },
    // Task Update Events
    onTaskUpdated: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:task-updated', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:task-updated', wrapper);
    },
    onTaskStatusChanged: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:task-status-changed', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:task-status-changed', wrapper);
    },
    onTaskCommentAdded: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:task-comment-added', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:task-comment-added', wrapper);
    },
    onCollaborationStarted: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:collaboration-started', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:collaboration-started', wrapper);
    },
    onCollaborationEnded: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:collaboration-ended', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:collaboration-ended', wrapper);
    },
    // File Transfer Events
    onFileTransferStarted: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:file-transfer-started', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:file-transfer-started', wrapper);
    },
    onFileTransferProgress: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:file-transfer-progress', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:file-transfer-progress', wrapper);
    },
    onFileTransferCompleted: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:file-transfer-completed', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:file-transfer-completed', wrapper);
    },
    onFileTransferError: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:file-transfer-error', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:file-transfer-error', wrapper);
    },
    onFileReceived: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('lan:file-received', wrapper);
        return () => electron_1.ipcRenderer.removeListener('lan:file-received', wrapper);
    },
    // Screenshot & Temp File
    screenshot: () => electron_1.ipcRenderer.invoke('lan:screenshot'),
    saveTempFile: (base64Data, fileName) => electron_1.ipcRenderer.invoke('lan:save-temp-file', base64Data, fileName),
    loadLocalFileAsBase64: (filePath) => electron_1.ipcRenderer.invoke('lan:load-local-file-as-base64', filePath),
};
//# sourceMappingURL=preload-lan.js.map