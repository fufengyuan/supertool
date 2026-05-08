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
exports.registerGitSyncHandlers = registerGitSyncHandlers;
exports.startGitSyncTimer = startGitSyncTimer;
exports.stopGitSyncTimer = stopGitSyncTimer;
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const electron_1 = require("electron");
const db = require("./database");
const app_bootstrap_1 = require("./app-bootstrap");
const window_manager_1 = require("./window-manager");
// ============ Git Sync ============
let gitSyncInterval = null;
const syncGit = async () => {
    try {
        const simpleGit = (await Promise.resolve().then(() => __importStar(require('simple-git')))).default;
        const syncPath = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'sync-data');
        if (!fs.existsSync(syncPath))
            return;
        const git = simpleGit(syncPath);
        const enabled = await db.getSetting('git_sync_enabled');
        if (enabled !== 'true')
            return;
        // Export current data — strip sensitive fields for Git safety
        const allData = db.exportAllData();
        // Remove passwords from server data before writing to Git
        const safeServers = (allData.servers || []).map((s) => ({ ...s, password: '' }));
        allData.servers = safeServers;
        fs.writeFileSync(path.join(syncPath, 'data.json'), JSON.stringify(allData, null, 2));
        await git.add('data.json');
        const status = await git.status();
        if (status.staged.length === 0 && status.conflicted.length === 0)
            return;
        await git.commit(`Auto-sync: ${new Date().toISOString()}`);
        await git.fetch();
        const behind = (await git.status()).behind;
        if (behind > 0) {
            await git.pull();
            const pulled = JSON.parse(fs.readFileSync(path.join(syncPath, 'data.json'), 'utf8'));
            db.importAllData(pulled, 'merge');
        }
        await git.push();
        await db.setSetting('git_sync_last_sync', new Date().toISOString());
        await db.setSetting('git_sync_status', 'ok');
        await db.setSetting('git_sync_error', '');
        (0, window_manager_1.getMainWindow)()?.webContents.send('git-sync:status-updated', { status: 'ok' });
    }
    catch (err) {
        console.error('[Git Sync] Error:', err.message);
        await db.setSetting('git_sync_status', 'error');
        await db.setSetting('git_sync_error', err.message);
        (0, window_manager_1.getMainWindow)()?.webContents.send('git-sync:status-updated', { status: 'error', error: err.message });
    }
};
function registerGitSyncHandlers() {
    electron_1.ipcMain.handle('git-sync:configure', async (_event, config) => {
        try {
            if (config.enabled !== undefined)
                await db.setSetting('git_sync_enabled', config.enabled);
            if (config.remote_url !== undefined)
                await db.setSetting('git_sync_remote_url', config.remote_url);
            if (config.branch !== undefined)
                await db.setSetting('git_sync_branch', config.branch || 'main');
            if (config.interval !== undefined)
                await db.setSetting('git_sync_interval', config.interval || '5');
            if (config.ssh_key_path !== undefined)
                await db.setSetting('git_sync_ssh_key', config.ssh_key_path || '');
            return { success: true };
        }
        catch (err) {
            return { success: false, message: err.message };
        }
    });
    electron_1.ipcMain.handle('git-sync:init', async () => {
        try {
            const simpleGit = (await Promise.resolve().then(() => __importStar(require('simple-git')))).default;
            const syncPath = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'sync-data');
            if (!fs.existsSync(syncPath))
                fs.mkdirSync(syncPath, { recursive: true });
            const git = simpleGit(syncPath);
            // Check if already a git repo
            const isRepo = fs.existsSync(path.join(syncPath, '.git'));
            if (!isRepo) {
                await git.init();
            }
            const remoteUrl = await db.getSetting('git_sync_remote_url');
            if (remoteUrl) {
                try {
                    await git.addRemote('origin', remoteUrl);
                }
                catch { /* remote exists */ }
            }
            // Initial commit
            const allData = db.exportAllData();
            // Strip passwords for Git safety
            allData.servers = (allData.servers || []).map((s) => ({ ...s, password: '' }));
            fs.writeFileSync(path.join(syncPath, 'data.json'), JSON.stringify(allData, null, 2));
            await git.add('data.json');
            try {
                await git.commit('Initial sync');
            }
            catch { /* no changes */ }
            if (remoteUrl) {
                try {
                    await git.push('origin', await db.getSetting('git_sync_branch') || 'main', ['--force']);
                }
                catch (err) {
                    return { success: false, message: 'Push failed: ' + err.message };
                }
            }
            await db.setSetting('git_sync_enabled', 'true');
            return { success: true };
        }
        catch (err) {
            return { success: false, message: err.message };
        }
    });
    electron_1.ipcMain.handle('git-sync:pull', async () => {
        try {
            const simpleGit = (await Promise.resolve().then(() => __importStar(require('simple-git')))).default;
            const syncPath = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'sync-data');
            const git = simpleGit(syncPath);
            const branch = await db.getSetting('git_sync_branch') || 'main';
            await git.fetch();
            await git.pull('origin', branch);
            const pulled = JSON.parse(fs.readFileSync(path.join(syncPath, 'data.json'), 'utf8'));
            const result = db.importAllData(pulled, 'merge');
            await db.setSetting('git_sync_last_sync', new Date().toISOString());
            await db.setSetting('git_sync_status', 'ok');
            return { success: true, importedCount: result.imported, skippedCount: result.skipped };
        }
        catch (err) {
            await db.setSetting('git_sync_error', err.message);
            return { success: false, message: err.message };
        }
    });
    electron_1.ipcMain.handle('git-sync:push', async () => {
        try {
            const simpleGit = (await Promise.resolve().then(() => __importStar(require('simple-git')))).default;
            const syncPath = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'sync-data');
            const git = simpleGit(syncPath);
            const branch = await db.getSetting('git_sync_branch') || 'main';
            const allData = db.exportAllData();
            fs.writeFileSync(path.join(syncPath, 'data.json'), JSON.stringify(allData, null, 2));
            await git.add('data.json');
            try {
                await git.commit(`Manual sync: ${new Date().toISOString()}`);
            }
            catch { /* no changes */ }
            await git.push('origin', branch);
            await db.setSetting('git_sync_last_sync', new Date().toISOString());
            await db.setSetting('git_sync_status', 'ok');
            return { success: true };
        }
        catch (err) {
            await db.setSetting('git_sync_error', err.message);
            return { success: false, message: err.message };
        }
    });
    electron_1.ipcMain.handle('git-sync:status', async () => {
        try {
            const enabled = await db.getSetting('git_sync_enabled');
            const remoteUrl = await db.getSetting('git_sync_remote_url');
            const branch = await db.getSetting('git_sync_branch') || 'main';
            const interval = await db.getSetting('git_sync_interval') || '5';
            const lastSync = await db.getSetting('git_sync_last_sync');
            const status = await db.getSetting('git_sync_status');
            const error = await db.getSetting('git_sync_error');
            const sshKey = await db.getSetting('git_sync_ssh_key') || '';
            return { enabled: enabled === 'true', remoteUrl, branch, interval: parseInt(interval), lastSync, status, error, sshKey };
        }
        catch (err) {
            return { enabled: false, remoteUrl: null, branch: 'main', interval: 5, lastSync: null, status: 'error', error: err.message, sshKey: '' };
        }
    });
}
// Start periodic sync on app ready
async function startGitSyncTimer() {
    if (gitSyncInterval)
        clearInterval(gitSyncInterval);
    const enabled = await db.getSetting('git_sync_enabled');
    if (enabled !== 'true')
        return;
    const interval = parseInt(await db.getSetting('git_sync_interval') || '5');
    gitSyncInterval = setInterval(syncGit, interval * 60 * 1000);
}
function stopGitSyncTimer() {
    if (gitSyncInterval) {
        clearInterval(gitSyncInterval);
        gitSyncInterval = null;
    }
}
//# sourceMappingURL=git-sync-manager.js.map