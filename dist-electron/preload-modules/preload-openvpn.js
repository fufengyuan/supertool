"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** OpenVPN API */
const electron_1 = require("electron");
exports.default = {
    openvpnGetAll: () => electron_1.ipcRenderer.invoke('openvpn:get-all'),
    openvpnAdd: (data) => electron_1.ipcRenderer.invoke('openvpn:add', data),
    openvpnDelete: (id) => electron_1.ipcRenderer.invoke('openvpn:delete', id),
    openvpnConnect: (configId, configName, content, sudoPassword) => electron_1.ipcRenderer.invoke('openvpn:connect', configId, configName, content, sudoPassword),
    openvpnRetryWithPassword: (password) => electron_1.ipcRenderer.invoke('openvpn:retry-with-password', password),
    openvpnDisconnect: () => electron_1.ipcRenderer.invoke('openvpn:disconnect'),
    openvpnGetStatus: () => electron_1.ipcRenderer.invoke('openvpn:get-status'),
    openvpnGetLogs: () => electron_1.ipcRenderer.invoke('openvpn:get-logs'),
    openvpnCheckAvailable: () => electron_1.ipcRenderer.invoke('openvpn:check-available'),
    openvpnValidateConfig: (content) => electron_1.ipcRenderer.invoke('openvpn:validate-config', content),
    openvpnGetTrafficStats: () => electron_1.ipcRenderer.invoke('openvpn:get-traffic-stats'),
    importOvpnFile: () => electron_1.ipcRenderer.invoke('dialog:openOvpnFile'),
    readFileContent: (filePath) => electron_1.ipcRenderer.invoke('file:read-content', filePath),
};
//# sourceMappingURL=preload-openvpn.js.map