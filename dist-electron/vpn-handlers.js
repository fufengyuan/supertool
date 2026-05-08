"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerVpnHandlers = registerVpnHandlers;
const electron_1 = require("electron");
function registerVpnHandlers(db, openVPNManager) {
    // ============ OpenVPN ============
    electron_1.ipcMain.handle('openvpn:get-all', () => db.getOpenVPNConfigs());
    electron_1.ipcMain.handle('openvpn:add', (_event, data) => db.addOpenVPNConfig(data.name, data.filePath, data.content));
    electron_1.ipcMain.handle('openvpn:delete', (_event, id) => db.deleteOpenVPNConfig(id));
    electron_1.ipcMain.handle('openvpn:connect', async (_event, configId, configName, content, sudoPassword) => openVPNManager.connect(configId, configName, content, sudoPassword));
    electron_1.ipcMain.handle('openvpn:retry-with-password', async (_event, password) => openVPNManager.retryWithPassword(password));
    electron_1.ipcMain.handle('openvpn:disconnect', async () => openVPNManager.disconnect());
    electron_1.ipcMain.handle('openvpn:get-status', () => openVPNManager.getStatus());
    electron_1.ipcMain.handle('openvpn:get-logs', () => openVPNManager.getStatus().log);
    electron_1.ipcMain.handle('openvpn:check-available', async () => openVPNManager.checkAvailable());
    electron_1.ipcMain.handle('openvpn:validate-config', (_event, content) => openVPNManager.validateConfig(content));
    electron_1.ipcMain.handle('openvpn:get-traffic-stats', () => openVPNManager.getTrafficStats());
} // end registerVpnHandlers
//# sourceMappingURL=vpn-handlers.js.map