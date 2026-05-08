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
exports.performAutoBackup = performAutoBackup;
exports.scheduleAutoBackup = scheduleAutoBackup;
exports.stopAutoBackup = stopAutoBackup;
exports.setBackupTimer = setBackupTimer;
exports.getBackupTimer = getBackupTimer;
const logger_1 = require("./logger");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const db = require("./database");
const app_bootstrap_1 = require("./app-bootstrap");
const window_manager_1 = require("./window-manager");
// ============ Auto Backup ============
let autoBackupTimer;
async function performAutoBackup() {
    try {
        const data = db.exportAllData();
        const backupPath = db.getSetting('auto_backup_path') || (0, app_bootstrap_1.getSuperToolDataDir)();
        if (!fs.existsSync(backupPath))
            fs.mkdirSync(backupPath, { recursive: true });
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const filePath = path.join(backupPath, `supertool-backup-${timestamp}.stbackup`);
        // Create manifest（与手动备份一致）
        const manifest = {
            version: data.version,
            exportedAt: data.exportedAt,
            tableCounts: {
                todos: data.todos.length,
                subtasks: data.subtasks.length,
                tags: data.tags.length,
                settings: Object.keys(data.settings).length,
                projects: data.projects.length,
                notes: data.notes.length,
                noteGroups: data.noteGroups.length,
                mfaSecrets: data.mfaSecrets.length,
                servers: data.servers.length,
                serverGroups: data.serverGroups.length,
                cicdConfigs: data.cicdConfigs.length,
                deployModules: data.deployModules?.length || 0,
                deployLogs: data.deployLogs?.length || 0,
                deployHistory: data.deployHistory?.length || 0,
                deployStepLogs: data.deployStepLogs?.length || 0,
                weeklyReports: data.weeklyReports.length,
                users: data.users?.length || 0,
                messages: data.messages?.length || 0,
                chatMessages: data.chatMessages?.length || 0,
                fileTransfers: data.fileTransfers?.length || 0,
                accountingCategories: data.accountingCategories?.length || 0,
                accountingRecords: data.accountingRecords?.length || 0,
            }
        };
        // 使用 archiver 创建 .stbackup ZIP（与手动备份完全一致的格式）
        const archiver = (await Promise.resolve().then(() => __importStar(require('archiver')))).default;
        const { createWriteStream } = await Promise.resolve().then(() => __importStar(require('fs')));
        const output = createWriteStream(filePath);
        const archive = archiver('zip', { zlib: { level: 6 } });
        await new Promise((resolve, reject) => {
            output.on('close', () => resolve());
            output.on('error', (err) => reject(new Error(`备份文件写入失败: ${err.message}`)));
            archive.on('error', (err) => reject(new Error(`ZIP 归档失败: ${err.message}`)));
            archive.pipe(output);
            archive.append(JSON.stringify(data, null, 2), { name: 'all-data.json' });
            archive.append(JSON.stringify(manifest, null, 2), { name: 'manifest.json' });
            // Include accounting receipt files
            const receiptDir = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'accounting-receipts');
            if (fs.existsSync(receiptDir)) {
                const files = fs.readdirSync(receiptDir);
                for (const file of files) {
                    const fileP = path.join(receiptDir, file);
                    if (fs.statSync(fileP).isFile()) {
                        const content = fs.readFileSync(fileP);
                        archive.append(content, { name: `receipts/${file}` });
                    }
                }
            }
            archive.finalize();
        });
        // 验证
        if (!fs.existsSync(filePath))
            throw new Error('备份文件未生成');
        const stat = fs.statSync(filePath);
        if (stat.size === 0)
            throw new Error('备份文件为空');
        (0, logger_1.info)(`[Auto Backup] Backup created: ${filePath} (${(stat.size / 1024).toFixed(1)}KB)`);
        if ((0, window_manager_1.getMainWindow)())
            (0, window_manager_1.getMainWindow)().webContents.send('backup:auto-backup-completed', { success: true, path: filePath });
    }
    catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        console.error('[Auto Backup] Failed:', errMsg);
        if ((0, window_manager_1.getMainWindow)())
            (0, window_manager_1.getMainWindow)().webContents.send('backup:auto-backup-completed', { success: false, error: errMsg });
    }
}
function scheduleAutoBackup() {
    if (autoBackupTimer) {
        clearTimeout(autoBackupTimer);
        autoBackupTimer = undefined;
    }
    const enabled = db.getSetting('auto_backup_enabled');
    if (enabled !== 'true')
        return;
    const frequency = db.getSetting('auto_backup_frequency') || 'daily';
    const backupTime = db.getSetting('auto_backup_time') || '02:00';
    const [hours, minutes] = backupTime.split(':').map(Number);
    function msUntilNextBackup() {
        const now = new Date();
        const target = new Date(now);
        target.setHours(hours, minutes, 0, 0);
        if (target <= now) {
            target.setDate(target.getDate() + (frequency === 'weekly' ? 7 : 1));
        }
        return target.getTime() - now.getTime();
    }
    function scheduleNext() {
        const ms = msUntilNextBackup();
        (0, logger_1.info)(`[Auto Backup] Next backup in ${Math.round(ms / 60000)} minutes`);
        autoBackupTimer = setTimeout(async () => { await performAutoBackup(); scheduleNext(); }, ms);
    }
    scheduleNext();
}
function stopAutoBackup() { if (autoBackupTimer) {
    clearTimeout(autoBackupTimer);
    autoBackupTimer = undefined;
} }
function setBackupTimer(t) { autoBackupTimer = t; }
function getBackupTimer() { return autoBackupTimer; }
//# sourceMappingURL=auto-backup-manager.js.map