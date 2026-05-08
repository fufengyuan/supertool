"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.initAutoUpdater = initAutoUpdater;
exports.setupAutoUpdater = setupAutoUpdater;
exports.registerUpdateHandlers = registerUpdateHandlers;
const logger_1 = require("./logger");
const electron_1 = require("electron");
const window_manager_1 = require("./window-manager");
let autoUpdater;
function initAutoUpdater() {
    const { autoUpdater: updater } = require('electron-updater');
    autoUpdater = updater;
    autoUpdater.autoDownload = true;
    autoUpdater.autoInstallOnAppQuit = true;
    autoUpdater.logger = console;
}
function setupAutoUpdater(isDev) {
    if (isDev) {
        (0, logger_1.info)('[AutoUpdate] Skipping in development mode');
        return;
    }
    autoUpdater.on('checking-for-update', () => { if ((0, window_manager_1.getMainWindow)())
        (0, window_manager_1.getMainWindow)().webContents.send('update:checking'); });
    autoUpdater.on('update-available', (info) => { if ((0, window_manager_1.getMainWindow)())
        (0, window_manager_1.getMainWindow)().webContents.send('update:available', info); });
    autoUpdater.on('update-not-available', (info) => { if ((0, window_manager_1.getMainWindow)())
        (0, window_manager_1.getMainWindow)().webContents.send('update:not-available', info); });
    autoUpdater.on('download-progress', (progressObj) => {
        if ((0, window_manager_1.getMainWindow)())
            (0, window_manager_1.getMainWindow)().webContents.send('update:download-progress', {
                percent: progressObj.percent, transferred: progressObj.transferred,
                total: progressObj.total, bytesPerSecond: progressObj.bytesPerSecond
            });
    });
    autoUpdater.on('update-downloaded', (info) => { if ((0, window_manager_1.getMainWindow)())
        (0, window_manager_1.getMainWindow)().webContents.send('update:downloaded', info); });
    autoUpdater.on('error', (err) => { if ((0, window_manager_1.getMainWindow)())
        (0, window_manager_1.getMainWindow)().webContents.send('update:error', err.message); });
    setTimeout(() => { autoUpdater.checkForUpdates(); }, 3000);
}
function registerUpdateHandlers() {
    electron_1.ipcMain.handle('update:get-version', () => electron_1.app.getVersion());
    electron_1.ipcMain.handle('update:check', async () => {
        try {
            const result = await autoUpdater.checkForUpdates();
            return { success: true, info: result?.updateInfo };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('update:install', () => { autoUpdater.quitAndInstall(); return { success: true }; });
}
//# sourceMappingURL=auto-updater-manager.js.map