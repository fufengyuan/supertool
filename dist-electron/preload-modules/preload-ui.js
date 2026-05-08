"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** UI: Menu Events, Update, Quick Switch, Weekly Report, Backup/Import/Export, Data Change Events */
const electron_1 = require("electron");
exports.default = {
    // Export/Backup API
    exportData: (options) => electron_1.ipcRenderer.invoke('backup:export-data', options),
    importJson: (options) => electron_1.ipcRenderer.invoke('backup:import-json', options),
    exportCsv: (options) => electron_1.ipcRenderer.invoke('backup:export-csv', options),
    exportWordReport: (reportData) => electron_1.ipcRenderer.invoke('export:word', reportData),
    // Weekly Report API
    saveWeeklyReport: (report) => electron_1.ipcRenderer.invoke('weekly-report:save', report),
    getWeeklyReports: (limit) => electron_1.ipcRenderer.invoke('weekly-report:get-all', limit),
    getWeeklyReport: (id) => electron_1.ipcRenderer.invoke('weekly-report:get', id),
    // Auto Backup API
    setAutoBackup: (settings) => electron_1.ipcRenderer.invoke('backup:auto-backup', settings),
    onAutoBackupCompleted: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('backup:auto-backup-completed', wrapper);
        return () => electron_1.ipcRenderer.removeListener('backup:auto-backup-completed', wrapper);
    },
    // Update API
    getAppVersion: () => electron_1.ipcRenderer.invoke('update:get-version'),
    checkForUpdates: () => electron_1.ipcRenderer.invoke('update:check'),
    installUpdate: () => electron_1.ipcRenderer.invoke('update:install'),
    onUpdateChecking: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('update:checking', wrapper);
        return () => electron_1.ipcRenderer.removeListener('update:checking', wrapper);
    },
    onUpdateAvailable: (callback) => {
        const wrapper = (_event, info) => callback(info);
        electron_1.ipcRenderer.on('update:available', wrapper);
        return () => electron_1.ipcRenderer.removeListener('update:available', wrapper);
    },
    onUpdateNotAvailable: (callback) => {
        const wrapper = (_event, info) => callback(info);
        electron_1.ipcRenderer.on('update:not-available', wrapper);
        return () => electron_1.ipcRenderer.removeListener('update:not-available', wrapper);
    },
    onUpdateDownloadProgress: (callback) => {
        const wrapper = (_event, progress) => callback(progress);
        electron_1.ipcRenderer.on('update:download-progress', wrapper);
        return () => electron_1.ipcRenderer.removeListener('update:download-progress', wrapper);
    },
    onUpdateDownloaded: (callback) => {
        const wrapper = (_event, info) => callback(info);
        electron_1.ipcRenderer.on('update:downloaded', wrapper);
        return () => electron_1.ipcRenderer.removeListener('update:downloaded', wrapper);
    },
    onUpdateError: (callback) => {
        const wrapper = (_event, error) => callback(error);
        electron_1.ipcRenderer.on('update:error', wrapper);
        return () => electron_1.ipcRenderer.removeListener('update:error', wrapper);
    },
    // Menu Events
    onMenuNewTask: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:new-task', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:new-task', wrapper);
    },
    onMenuExportMarkdown: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:export-markdown', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:export-markdown', wrapper);
    },
    onMenuExportWord: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:export-word', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:export-word', wrapper);
    },
    onMenuExportJson: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:export-json', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:export-json', wrapper);
    },
    onMenuImportJson: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:import-json', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:import-json', wrapper);
    },
    onMenuClearCompleted: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:clear-completed', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:clear-completed', wrapper);
    },
    onMenuSearchTasks: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:search-tasks', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:search-tasks', wrapper);
    },
    onMenuSelectAll: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:select-all', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:select-all', wrapper);
    },
    onMenuDeleteSelected: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:delete-selected', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:delete-selected', wrapper);
    },
    onMenuToggleTheme: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:toggle-theme', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:toggle-theme', wrapper);
    },
    onMenuToggleLocale: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:toggle-locale', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:toggle-locale', wrapper);
    },
    onMenuSwitchView: (callback) => {
        const wrapper = (_event, view) => callback(view);
        electron_1.ipcRenderer.on('menu:switch-view', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:switch-view', wrapper);
    },
    onMenuToggleLanPanel: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:toggle-lan-panel', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:toggle-lan-panel', wrapper);
    },
    onMenuToggleComplete: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:toggle-complete', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:toggle-complete', wrapper);
    },
    onMenuSetPriority: (callback) => {
        const wrapper = (_event, priority) => callback(priority);
        electron_1.ipcRenderer.on('menu:set-priority', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:set-priority', wrapper);
    },
    onMenuSetTag: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:set-tag', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:set-tag', wrapper);
    },
    onMenuAbout: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:about', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:about', wrapper);
    },
    onMenuShortcutsHelp: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:shortcuts-help', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:shortcuts-help', wrapper);
    },
    onMenuCheckUpdate: (callback) => {
        const wrapper = () => callback();
        electron_1.ipcRenderer.on('menu:check-update', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:check-update', wrapper);
    },
    onMenuNav: (callback) => {
        const wrapper = (_event, view) => callback(view);
        electron_1.ipcRenderer.on('menu:nav', wrapper);
        return () => electron_1.ipcRenderer.removeListener('menu:nav', wrapper);
    },
    onQuickSwitch: (callback) => {
        electron_1.ipcRenderer.on('quick-switch:open', callback);
        return () => electron_1.ipcRenderer.removeListener('quick-switch:open', callback);
    },
    triggerMenuNav: (view) => electron_1.ipcRenderer.send('menu:nav', view),
    // Data Change Events
    onDataChanged: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('data:changed', wrapper);
        return () => electron_1.ipcRenderer.removeListener('data:changed', wrapper);
    },
};
//# sourceMappingURL=preload-ui.js.map