"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.startServerHeartbeat = startServerHeartbeat;
exports.stopServerHeartbeat = stopServerHeartbeat;
exports.getServerService = getServerService;
exports.setServerService = setServerService;
exports.registerServerHandlers = registerServerHandlers;
const electron_1 = require("electron");
const window_manager_1 = require("./window-manager");
const encryption_manager_1 = require("./encryption-manager");
let serverService = undefined;
let heartbeatTimer;
function startServerHeartbeat() {
    if (heartbeatTimer)
        clearInterval(heartbeatTimer);
    heartbeatTimer = setInterval(async () => {
        if (!serverService)
            return;
        for (const serverId of serverService.getActiveConnections()) {
            try {
                await serverService.execCommand(serverId, 'echo heartbeat');
            }
            catch {
                serverService.disconnect(serverId);
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('server:heartbeat-failed', { serverId });
            }
        }
    }, 60 * 1000);
}
function stopServerHeartbeat() {
    if (heartbeatTimer) {
        clearInterval(heartbeatTimer);
        heartbeatTimer = undefined;
    }
}
function getServerService() { return serverService; }
function setServerService(s) { serverService = s; }
function registerServerHandlers(db, requireService, notifyDataChange) {
    // ============ Server Management ============
    electron_1.ipcMain.handle('servers:get-all', () => db.getAllServers().map((s) => ({ ...s, password: (0, encryption_manager_1.decryptPassword)(s.password) })));
    electron_1.ipcMain.handle('servers:get-by-id', (_event, serverId) => {
        const server = db.getServerById(serverId);
        if (server)
            server.password = (0, encryption_manager_1.decryptPassword)(server.password);
        return server;
    });
    electron_1.ipcMain.handle('servers:add', (_event, server) => {
        if (server.password)
            server = { ...server, password: (0, encryption_manager_1.encryptPassword)(server.password) };
        const result = db.addServer(server);
        notifyDataChange('servers');
        return result;
    });
    electron_1.ipcMain.handle('servers:update', (_event, server) => {
        if (server.password) {
            server = { ...server, password: (0, encryption_manager_1.encryptPassword)(server.password) };
        }
        else {
            const existing = db.getServerById(server.id);
            if (existing && existing.password) {
                server = { ...server, password: existing.password };
            }
        }
        const result = db.updateServer(server);
        notifyDataChange('servers');
        return result;
    });
    electron_1.ipcMain.handle('servers:delete', (_event, serverId) => {
        const result = db.deleteServer(serverId);
        notifyDataChange('servers');
        return result;
    });
    electron_1.ipcMain.handle('servers:groups:get-all', () => db.getAllServerGroups());
    electron_1.ipcMain.handle('servers:groups:add', (_event, group) => {
        const result = db.addServerGroup(group);
        notifyDataChange('servers');
        return result;
    });
    electron_1.ipcMain.handle('servers:groups:update', (_event, groupId, updates) => {
        const result = db.updateServerGroup(groupId, updates);
        notifyDataChange('servers');
        return result;
    });
    electron_1.ipcMain.handle('servers:groups:delete', (_event, groupId) => {
        const result = db.deleteServerGroup(groupId);
        notifyDataChange('servers');
        return result;
    });
    electron_1.ipcMain.handle('servers:test-connection', async (_event, server) => {
        const ServerService = requireService('server-service');
        const svc = new ServerService();
        try {
            const plainPassword = server.password ? (0, encryption_manager_1.decryptPassword)(server.password) : undefined;
            await svc.testConnection({ ...server, password: plainPassword });
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('servers:connect', async (_event, serverId) => {
        const ServerService = requireService('server-service');
        const wasNew = !serverService;
        if (!serverService)
            serverService = new ServerService();
        if (wasNew) {
            serverService.on('connected', (data) => {
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('server:connected', data);
            });
            serverService.on('disconnected', (data) => {
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('server:disconnected', data);
            });
            serverService.on('connection-error', (data) => {
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('server:connection-error', data);
            });
            serverService.on('terminal-data', (data) => {
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('terminal:data', data);
            });
            serverService.on('terminal-close', (data) => {
                if ((0, window_manager_1.getMainWindow)())
                    (0, window_manager_1.getMainWindow)().webContents.send('terminal:close', data);
            });
        }
        if (serverService.isConnected(serverId)) {
            return { success: true, alreadyConnected: true };
        }
        const server = db.getServerById(serverId);
        if (!server)
            return { success: false, error: 'Server not found' };
        try {
            await serverService.connect({ ...server, password: (0, encryption_manager_1.decryptPassword)(server.password) });
            startServerHeartbeat();
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('servers:is-connected', (_event, serverId) => {
        return serverService ? serverService.isConnected(serverId) : false;
    });
    electron_1.ipcMain.handle('servers:disconnect', (_event, serverId) => {
        if (serverService) {
            serverService.disconnect(serverId);
            if (serverService.getActiveConnections().length === 0)
                stopServerHeartbeat();
        }
        return { success: true };
    });
    electron_1.ipcMain.handle('servers:exec', async (_event, serverId, command) => {
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        const server = db.getServerById(serverId);
        if (server && server.requiresApproval)
            return { success: false, requiresApproval: true, serverName: server.name, error: '该服务器已开启执行审核' };
        const allowedCommands = ['ls', 'cat', 'pwd', 'whoami', 'hostname', 'uname', 'df', 'free', 'ps', 'netstat', 'tail', 'grep', 'find', 'top', 'date', 'uptime', 'id', 'env', 'echo', 'wc', 'sort', 'uniq', 'head', 'stat', 'file', 'du', 'ping'];
        const blockedCommands = ['rm', 'wget', 'curl', 'chmod', 'chown', 'sudo', 'dd', 'mkfs', 'reboot', 'shutdown', 'kill', 'iptables', 'mount', 'umount', 'mktemp', 'crontab', 'passwd', 'useradd', 'userdel', 'groupadd', 'visudo', 'chroot', 'systemctl', 'service', 'nohup', 'at', 'batch'];
        const cmd = command.trim().split(/\s+/)[0].toLowerCase().replace(/^[.'"/\\]/g, '');
        if (blockedCommands.includes(cmd)) {
            return { success: false, error: `Command blocked for security: ${cmd}` };
        }
        if (allowedCommands.length > 0 && !allowedCommands.includes(cmd)) {
            return { success: false, error: `Command not in allowed list: ${cmd}. Allowed: ${allowedCommands.join(', ')}` };
        }
        try {
            return await serverService.execCommand(serverId, command);
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ Terminal Operations ============
    electron_1.ipcMain.handle('servers:terminal:create', async (_event, serverId, terminalId, rows, cols) => {
        if (!serverService)
            return { success: false, error: '服务未连接' };
        try {
            return await serverService.createTerminal(serverId, terminalId, rows, cols);
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('servers:terminal:write', (_event, terminalId, data) => {
        if (!serverService)
            return { success: false };
        return serverService.writeToTerminal(terminalId, data);
    });
    electron_1.ipcMain.handle('servers:terminal:close', (_event, terminalId) => {
        if (!serverService)
            return { success: false };
        return serverService.closeTerminal(terminalId);
    });
    electron_1.ipcMain.handle('servers:terminal:resize', (_event, terminalId, rows, cols) => {
        if (!serverService)
            return { success: false };
        return serverService.resizeTerminal(terminalId, rows, cols);
    });
} // end registerServerHandlers
//# sourceMappingURL=server-handlers.js.map