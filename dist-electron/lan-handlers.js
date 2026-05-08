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
exports.registerLanHandlers = registerLanHandlers;
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const app_bootstrap_1 = require("./app-bootstrap");
const window_manager_1 = require("./window-manager");
function registerLanHandlers(lanService, db, notifyDataChange) {
    // ============ LAN ============
    electron_1.ipcMain.handle('lan:get-peers', () => lanService ? lanService.getOnlinePeers() : []);
    electron_1.ipcMain.handle('lan:get-logs', (_event, limit) => lanService ? lanService.getLogs(limit) : []);
    electron_1.ipcMain.handle('lan:get-log-path', () => ({
        logFile: lanService?.logFilePath || null,
        logDir: lanService ? lanService.getLogDir() : null
    }));
    electron_1.ipcMain.handle('lan:send-message', (_event, peerId, message) => {
        console.error(`[LAN IPC][lan:send-message] ENTRY: peerId=${peerId}, messageLen=${message?.length}, messagePreview=${String(message).slice(0, 50)}`);
        if (!lanService) {
            console.error('[LAN IPC][lan:send-message] lanService is null!');
            return { success: false, error: 'LAN service not initialized' };
        }
        try {
            const result = lanService.sendMessage(peerId, message);
            console.error(`[LAN IPC][lan:send-message] EXIT: success=${result.success}, messageId=${result.messageId}, queued=${result.queued}, incompatibleVersion=${result.incompatibleVersion}`);
            return result;
        }
        catch (err) {
            console.error(`[LAN IPC][lan:send-message] UNCAUGHT EXCEPTION: ${err?.message || err}`, err?.stack);
            return { success: false, error: err?.message || 'Unknown error' };
        }
    });
    electron_1.ipcMain.handle('lan:broadcast-message', (_event, message) => {
        if (lanService) {
            lanService.broadcastMessage(message);
            return true;
        }
        return false;
    });
    electron_1.ipcMain.handle('lan:sync-task-status', (_event, todo) => {
        if (lanService && todo) {
            lanService.broadcastTaskStatusChange(todo);
            return true;
        }
        return false;
    });
    electron_1.ipcMain.handle('lan:assign-task', async (_event, peerId, task) => {
        if (!lanService)
            return { success: false };
        const todoId = `lan-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
        const now = new Date().toISOString();
        const myInfo = { id: lanService.userId, name: lanService.userName };
        try {
            db.addTodo({
                id: todoId, text: task.text, completed: false,
                priority: task.priority || 'medium', dueDate: task.dueDate || null,
                description: task.note || '', markdownDescription: null, tag: '',
                createdAt: now, updatedAt: now, assignedTo: peerId, assignedBy: myInfo.id,
                assignedAt: now, owner: myInfo.id, orderNum: 0,
                repeatType: null, repeatInterval: 0, repeatCount: 0, repeatEndDate: null,
                parentTodoId: null, projectId: null
            });
            const sent = lanService.assignTask(peerId, { ...task, id: todoId });
            return { success: sent, todoId };
        }
        catch (err) {
            console.error('[LAN] 分配任务失败:', err);
            return { success: false };
        }
    });
    electron_1.ipcMain.handle('lan:get-user-info', () => ({
        id: lanService ? lanService.userId : 'unknown',
        name: lanService ? lanService.userName : 'unknown',
        avatar: lanService ? lanService.avatar : '😀'
    }));
    electron_1.ipcMain.handle('lan:set-nickname', (_event, name) => {
        if (lanService)
            lanService.setNickName(name);
        notifyDataChange('lan-user-info');
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:set-avatar', (_event, emoji) => {
        if (lanService)
            lanService.setAvatar(emoji);
        notifyDataChange('lan-user-info');
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:set-status', (_event, status) => {
        if (lanService)
            lanService.setStatus(status);
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:get-status', () => ({
        status: lanService ? lanService.myStatusDisplay : 'online'
    }));
    electron_1.ipcMain.handle('lan:get-network-info', () => ({
        ...lanService?.networkInfo, version: '2.0'
    }));
    // LAN Network Permission Check (macOS Local Network Privacy)
    electron_1.ipcMain.handle('lan:check-network-permission', async () => {
        if (!lanService)
            return { granted: false, details: 'LAN service not started' };
        const result = await lanService.checkNetworkPermission();
        return result;
    });
    electron_1.ipcMain.handle('lan:get-permission-status', () => ({
        granted: lanService?.networkPermissionGranted ?? null,
        details: lanService?.networkPermissionDetails ?? null
    }));
    // Screenshot & Temp File IPC
    electron_1.ipcMain.handle('lan:screenshot', async () => {
        try {
            const sources = await electron_1.desktopCapturer.getSources({
                types: ['screen'],
                thumbnailSize: {
                    width: electron_1.screen.getPrimaryDisplay().workAreaSize.width,
                    height: electron_1.screen.getPrimaryDisplay().workAreaSize.height
                }
            });
            if (sources.length > 0) {
                const img = sources[0].thumbnail;
                const tempPath = path.join(path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp'), `screenshot_${Date.now()}.png`);
                const buffer = img.toPNG();
                fs.writeFileSync(tempPath, buffer);
                return { success: true, path: tempPath };
            }
            return { success: false, error: 'No screen sources found' };
        }
        catch (e) {
            return { success: false, error: e.message };
        }
    });
    electron_1.ipcMain.handle('lan:save-temp-file', async (_event, base64Data, fileName) => {
        const tempPath = path.join(path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp'), `paste_${Date.now()}_${fileName}`);
        const buffer = Buffer.from(base64Data, 'base64');
        fs.writeFileSync(tempPath, buffer);
        return tempPath;
    });
    electron_1.ipcMain.handle('lan:refresh-discovery', () => {
        if (lanService) {
            lanService.refreshDiscovery();
            return true;
        }
        return false;
    });
    electron_1.ipcMain.handle('lan:get-receive-path', () => lanService ? lanService.getReceivePath() : '');
    electron_1.ipcMain.handle('lan:set-receive-path', (_event, dirPath) => {
        if (lanService)
            lanService.setReceivePath(dirPath);
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:load-local-file-as-base64', (_event, filePath) => {
        try {
            if (!fs.existsSync(filePath))
                return { success: false, error: 'File not found' };
            const buffer = fs.readFileSync(filePath);
            const ext = path.extname(filePath).slice(1).toLowerCase();
            const mimeMap = { png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', gif: 'image/gif', webp: 'image/webp', bmp: 'image/bmp', svg: 'image/svg+xml' };
            const mime = mimeMap[ext] || 'application/octet-stream';
            return { success: true, data: `data:${mime};base64,${buffer.toString('base64')}` };
        }
        catch (e) {
            return { success: false, error: e.message };
        }
    });
    electron_1.ipcMain.handle('lan:open-file-folder', (_event, filePath) => {
        if (!filePath)
            return { success: false, error: 'No file path' };
        electron_1.shell.showItemInFolder(filePath);
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:open-file', async (_event, filePath) => {
        if (!filePath)
            return { success: false, error: 'No file path' };
        try {
            await electron_1.shell.openPath(filePath);
            return { success: true };
        }
        catch (e) {
            return { success: false, error: e.message };
        }
    });
    // ============ Projects ============
    electron_1.ipcMain.handle('projects:get-all', (_event, onlyActive = true) => db.getAllProjects(onlyActive));
    electron_1.ipcMain.handle('projects:add', (_event, project) => {
        const result = db.addProject(project);
        notifyDataChange('projects');
        return result;
    });
    electron_1.ipcMain.handle('projects:update', (_event, project) => {
        const result = db.updateProject(project);
        notifyDataChange('projects');
        return result;
    });
    electron_1.ipcMain.handle('projects:delete', (_event, id) => {
        const result = db.deleteProject(id);
        notifyDataChange('projects');
        return result;
    });
    electron_1.ipcMain.handle('projects:get-stats', (_event, projectId) => db.getProjectStats(projectId));
    electron_1.ipcMain.handle('projects:get-todos', (_event, projectId) => db.getTodosByProject(projectId));
    // ============ Export ============
    electron_1.ipcMain.handle('export:word', async (_event, options) => {
        const { createReportDocx } = await Promise.resolve().then(() => __importStar(require('./export-word')));
        return createReportDocx(options);
    });
    // ============ LAN Collaboration Broadcast ============
    electron_1.ipcMain.handle('lan:broadcast-task-update', (_event, task) => {
        if (lanService)
            lanService.broadcastTaskStatusChange(task);
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:broadcast-task-status-change', (_event, task) => {
        if (lanService)
            lanService.broadcastTaskStatusChange(task);
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:broadcast-task-comment', (_event, data) => {
        if (lanService)
            lanService.broadcastTaskComment(data);
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:broadcast-collaboration-started', (_event, data) => {
        if (lanService)
            lanService.broadcastCollaborationStarted(data);
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:broadcast-collaboration-ended', (_event, data) => {
        if (lanService)
            lanService.broadcastCollaborationEnded(data);
        return { success: true };
    });
    // ============ LAN File Transfer ============
    electron_1.ipcMain.handle('lan:send-file', (_event, peerId, filePath, fileName, resumeOffset, fileId) => {
        if (!lanService)
            return { success: false };
        return lanService.sendFile(peerId, filePath, fileName, resumeOffset || 0, fileId);
    });
    electron_1.ipcMain.handle('lan:get-message-history', (_event, limit) => {
        if (!lanService)
            return [];
        return lanService.getMessageHistory(limit);
    });
    electron_1.ipcMain.handle('lan:get-messages-between', (_event, userId1, userId2, limit, offset) => {
        if (!lanService)
            return [];
        return lanService.getMessagesBetween(userId1, userId2, limit, offset);
    });
    electron_1.ipcMain.handle('lan:mark-messages-read', (_event, myUserId, peerId) => {
        if (lanService)
            lanService.markMessagesRead(myUserId, peerId);
        return { success: true };
    });
    electron_1.ipcMain.handle('lan:get-unread-count', (_event, myUserId, peerId) => {
        if (!lanService)
            return 0;
        return lanService.getUnreadCount(myUserId, peerId);
    });
    electron_1.ipcMain.handle('lan:get-all-unread-counts', (_event, myUserId) => {
        if (!lanService)
            return {};
        return lanService.getAllUnreadCounts(myUserId);
    });
    electron_1.ipcMain.handle('lan:get-file-transfer-history', (_event, limit, offset) => {
        if (!lanService)
            return [];
        return lanService.getFileTransferHistory(limit, offset);
    });
    electron_1.ipcMain.handle('lan:show-open-dialog', async (_event, options) => {
        const { dialog } = await Promise.resolve().then(() => __importStar(require('electron')));
        const mw = (0, window_manager_1.getMainWindow)();
        if (!mw)
            return { canceled: true, filePaths: [] };
        const result = await dialog.showOpenDialog(mw, {
            title: options?.title || '选择文件',
            properties: options?.properties || ['openFile'],
        });
        return { canceled: result.canceled, filePaths: result.filePaths };
    });
} // end registerLanHandlers
//# sourceMappingURL=lan-handlers.js.map