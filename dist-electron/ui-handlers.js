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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerUiHandlers = registerUiHandlers;
const logger_1 = require("./logger");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const fs_1 = require("fs");
const archiver_1 = __importDefault(require("archiver"));
const app_bootstrap_1 = require("./app-bootstrap");
const window_manager_1 = require("./window-manager");
const menu_manager_1 = require("./menu-manager");
const shell_env_manager_1 = require("./shell-env-manager");
const encryption_manager_1 = require("./encryption-manager");
/** 拦截 db_connections 的密码字段解密 */
function decryptConnectionPasswords(conns) {
    return conns.map((c) => ({
        ...c,
        password: c.password ? (0, encryption_manager_1.decryptPassword)(c.password) : c.password,
    }));
}
/** 拦截 db_connections 的密码字段加密 */
function encryptConnectionPasswords(conns) {
    return conns.map((c) => ({
        ...c,
        password: c.password ? (0, encryption_manager_1.encryptPassword)(c.password) : c.password,
    }));
}
function registerUiHandlers(db, notifyDataChange, dismissNotification, updateShortcuts, testNotification, scanProjectModules, totpUtils) {
    // ============ Todos ============
    electron_1.ipcMain.handle('todos:get-all', () => db.getAllTodos());
    electron_1.ipcMain.handle('todos:add', (_event, todo) => {
        const result = db.addTodo(todo);
        notifyDataChange('todos');
        return result;
    });
    electron_1.ipcMain.handle('todos:update', (_event, todo) => {
        if (todo.completed || todo.dueDate)
            dismissNotification(todo.id);
        const result = db.updateTodo(todo);
        notifyDataChange('todos');
        return result;
    });
    electron_1.ipcMain.handle('todos:delete', (_event, id) => {
        dismissNotification(id);
        const result = db.deleteTodo(id);
        notifyDataChange('todos');
        return result;
    });
    electron_1.ipcMain.handle('todos:delete-many', (_event, ids) => {
        const result = db.deleteTodos(ids);
        notifyDataChange('todos');
        return result;
    });
    electron_1.ipcMain.handle('todos:update-order', (_event, todos) => db.updateTodoOrder(todos));
    electron_1.ipcMain.handle('todos:create-repeat-instance', (_event, todo) => db.createRepeatInstance(todo));
    // ============ Tags ============
    electron_1.ipcMain.handle('tags:get-all', () => db.getAllTags());
    electron_1.ipcMain.handle('tags:add', (_event, name) => db.addTag(name));
    electron_1.ipcMain.handle('tags:delete', (_event, name) => db.deleteTag(name));
    // ============ Settings ============
    electron_1.ipcMain.handle('settings:get', (_event, key) => {
        const value = db.getSetting(key);
        // db_connections 的密码需要解密后返回给前端
        if (key === 'db_connections' && value) {
            try {
                const conns = JSON.parse(value);
                return JSON.stringify(conns.map((c) => ({
                    ...c,
                    password: c.password ? (0, encryption_manager_1.decryptPassword)(c.password) : c.password,
                })));
            }
            catch { /* fallback to raw value */ }
        }
        return value;
    });
    electron_1.ipcMain.handle('settings:set', (_event, key, value) => {
        // db_connections 的密码需要加密后存储
        if (key === 'db_connections' && value) {
            try {
                const conns = JSON.parse(value);
                const encrypted = JSON.stringify(conns.map((c) => ({
                    ...c,
                    password: c.password && c.password.includes(':') ? c.password : (c.password ? (0, encryption_manager_1.encryptPassword)(c.password) : c.password),
                })));
                return db.setSetting(key, encrypted);
            }
            catch { /* fallback to raw set */ }
        }
        return db.setSetting(key, value);
    });
    // ============ Shortcuts ============
    electron_1.ipcMain.handle('shortcuts:update', async (_event, shortcuts) => {
        await updateShortcuts(shortcuts, db);
        return { success: true };
    });
    // ============ Notification ============
    electron_1.ipcMain.handle('notification:get-settings', () => ({ reminderTime: parseInt(db.getSetting('reminder_time') || '15') }));
    electron_1.ipcMain.handle('notification:set-settings', (_event, settings) => {
        db.setSetting('reminder_time', settings.reminderTime.toString());
        return settings;
    });
    electron_1.ipcMain.handle('notification:test', () => testNotification());
    electron_1.ipcMain.handle('notifications:dismiss', (_event, todoId) => {
        dismissNotification(todoId);
        return { success: true };
    });
    // ============ Subtasks ============
    electron_1.ipcMain.handle('subtasks:get-for-todo', (_event, todoId) => db.getSubtasksForTodo(todoId));
    electron_1.ipcMain.handle('subtasks:add', (_event, subtask) => db.addSubtask(subtask));
    electron_1.ipcMain.handle('subtasks:update', (_event, subtask) => db.updateSubtask(subtask));
    electron_1.ipcMain.handle('subtasks:delete', (_event, subtaskId) => db.deleteSubtask(subtaskId));
    electron_1.ipcMain.handle('subtasks:update-todo-completion', (_event, todoId) => db.updateTodoCompletionBasedOnSubtasks(todoId));
    // ============ Backup/Import ============
    electron_1.ipcMain.handle('backup:export-data', async (_event, _options = {}) => {
        try {
            const allData = db.exportAllData();
            const result = await electron_1.dialog.showSaveDialog((0, window_manager_1.getMainWindow)(), {
                title: '导出完整备份',
                defaultPath: `supertool-backup-${new Date().toISOString().slice(0, 10)}.stbackup`,
                filters: [{ name: 'SuperTool Backup', extensions: ['stbackup'] }, { name: 'All Files', extensions: ['*'] }]
            });
            if (result.canceled || !result.filePath)
                return { success: false, message: '用户取消了导出' };
            const filePath = result.filePath;
            const finalPath = filePath.endsWith('.stbackup') ? filePath : filePath + '.stbackup';
            const manifest = {
                version: allData.version, exportedAt: allData.exportedAt,
                tableCounts: {
                    todos: allData.todos.length, subtasks: allData.subtasks.length, tags: allData.tags.length,
                    settings: Object.keys(allData.settings).length, projects: allData.projects.length,
                    notes: allData.notes.length, noteGroups: allData.noteGroups.length, mfaSecrets: allData.mfaSecrets.length,
                    servers: allData.servers.length, serverGroups: allData.serverGroups.length,
                    cicdConfigs: allData.cicdConfigs.length, deployModules: allData.deployModules?.length || 0,
                    deployLogs: allData.deployLogs?.length || 0, deployHistory: allData.deployHistory?.length || 0,
                    deployStepLogs: allData.deployStepLogs?.length || 0, weeklyReports: allData.weeklyReports.length,
                    users: allData.users?.length || 0, messages: allData.messages?.length || 0,
                    chatMessages: allData.chatMessages?.length || 0, fileTransfers: allData.fileTransfers?.length || 0,
                    accountingCategories: allData.accountingCategories?.length || 0, accountingRecords: allData.accountingRecords?.length || 0,
                }
            };
            const output = (0, fs_1.createWriteStream)(finalPath);
            const archive = (0, archiver_1.default)('zip', { zlib: { level: 6 } });
            await new Promise((resolve, reject) => {
                output.on('close', () => resolve());
                output.on('error', (err) => reject(new Error(`备份文件写入失败: ${err.message}`)));
                archive.on('error', (err) => reject(new Error(`ZIP 归档失败: ${err.message}`)));
                archive.on('warning', (warn) => console.warn('Archiver warning:', warn));
                archive.pipe(output);
                archive.append(JSON.stringify(allData, null, 2), { name: 'all-data.json' });
                archive.append(JSON.stringify(manifest, null, 2), { name: 'manifest.json' });
                const receiptDir = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'accounting-receipts');
                if (fs.existsSync(receiptDir)) {
                    const files = fs.readdirSync(receiptDir);
                    for (const file of files) {
                        const filePath = path.join(receiptDir, file);
                        if (fs.statSync(filePath).isFile()) {
                            archive.append(fs.readFileSync(filePath), { name: `receipts/${file}` });
                        }
                    }
                    manifest.receiptCount = files.length;
                }
                else {
                    manifest.receiptCount = 0;
                }
                manifest.accountingBudgets = allData.accountingBudgets?.length || 0;
                manifest.accountingTemplates = allData.accountingTemplates?.length || 0;
                archive.finalize();
            });
            if (!fs.existsSync(finalPath))
                throw new Error('备份文件未生成');
            const stat = fs.statSync(finalPath);
            if (stat.size === 0)
                throw new Error('备份文件为空，导出失败');
            (0, logger_1.info)(`备份已生成: ${finalPath} (${(stat.size / 1024 / 1024).toFixed(2)}MB)`);
            const { default: unzipper } = await Promise.resolve(`${'unzipper'}`).then(s => __importStar(require(s)));
            const verifyZip = await unzipper.Open.file(finalPath);
            if (!verifyZip.files.some((f) => f.path === 'all-data.json'))
                throw new Error('ZIP 文件验证失败：缺少 all-data.json');
            (0, logger_1.info)('ZIP 文件验证通过');
            const totalItems = Object.values(manifest.tableCounts).reduce((a, b) => a + b, 0);
            return { success: true, path: finalPath, tableCount: Object.keys(manifest.tableCounts).length, totalItems };
        }
        catch (error) {
            const errMsg = error instanceof Error ? error.message : String(error);
            console.error('Export error:', error);
            return { success: false, message: errMsg };
        }
    });
    electron_1.ipcMain.handle('backup:import-json', async (_event, options = {}) => {
        try {
            const result = await electron_1.dialog.showOpenDialog((0, window_manager_1.getMainWindow)(), {
                title: '导入数据',
                filters: [{ name: 'SuperTool Backup', extensions: ['stbackup'] }, { name: 'All Files', extensions: ['*'] }],
                properties: ['openFile']
            });
            if (result.canceled || !result.filePaths?.length)
                return { success: false, message: '用户取消了导入' };
            const filePath = result.filePaths[0];
            let importData;
            if (filePath.endsWith('.stbackup')) {
                if (!fs.existsSync(filePath))
                    return { success: false, message: '备份文件不存在' };
                const stat = fs.statSync(filePath);
                if (stat.size === 0)
                    return { success: false, message: '备份文件为空（0 字节）' };
                const { default: unzipper } = await Promise.resolve(`${'unzipper'}`).then(s => __importStar(require(s)));
                let allDataJson = null;
                try {
                    const cd = await unzipper.Open.file(filePath);
                    const entry = cd.files.find((f) => f.path === 'all-data.json');
                    if (entry)
                        allDataJson = (await entry.buffer()).toString('utf8');
                }
                catch (zipError) {
                    const zipMsg = zipError instanceof Error ? zipError.message : String(zipError);
                    console.warn(`[Backup] unzipper failed: ${zipMsg}, falling back to system unzip`);
                    const tmpDir = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp');
                    const extractDir = path.join(tmpDir, `stbackup-${Date.now()}`);
                    fs.mkdirSync(extractDir, { recursive: true });
                    try {
                        await new Promise((resolve, reject) => {
                            const { execFile } = require('child_process');
                            execFile('unzip', ['-o', filePath, 'all-data.json', '-d', extractDir], (err) => {
                                if (fs.existsSync(path.join(extractDir, 'all-data.json')))
                                    resolve();
                                else
                                    reject(err || new Error('提取失败'));
                            });
                        });
                        const extractedPath = path.join(extractDir, 'all-data.json');
                        if (fs.existsSync(extractedPath))
                            allDataJson = fs.readFileSync(extractedPath, 'utf8');
                    }
                    finally {
                        try {
                            fs.rmSync(extractDir, { recursive: true, force: true });
                        }
                        catch { }
                    }
                }
                if (!allDataJson)
                    return { success: false, message: '备份文件格式错误：缺少 all-data.json' };
                importData = JSON.parse(allDataJson);
            }
            else {
                return { success: false, message: '不支持的文件格式' };
            }
            const mode = options.importMode === 'replace' ? 'replace' : 'merge';
            const importResult = db.importAllData(importData, mode);
            if (filePath.endsWith('.stbackup')) {
                try {
                    const { default: unzipper } = await Promise.resolve(`${'unzipper'}`).then(s => __importStar(require(s)));
                    const cd = await unzipper.Open.file(filePath);
                    const receiptFiles = cd.files.filter((f) => f.path.startsWith('receipts/'));
                    if (receiptFiles.length > 0) {
                        const receiptDir = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'accounting-receipts');
                        if (!fs.existsSync(receiptDir))
                            fs.mkdirSync(receiptDir, { recursive: true });
                        for (const file of receiptFiles) {
                            if (file.path.endsWith('/'))
                                continue;
                            fs.writeFileSync(path.join(receiptDir, path.basename(file.path)), await file.buffer());
                        }
                    }
                }
                catch (extractErr) {
                    console.warn('[Backup] Failed to extract receipt files:', extractErr);
                }
            }
            return { success: true, importedCount: importResult.imported, skippedCount: importResult.skipped };
        }
        catch (error) {
            const errMsg = error instanceof Error ? error.message : String(error);
            console.error('Import error:', error);
            return { success: false, message: errMsg };
        }
    });
    // ============ Weekly Reports ============
    electron_1.ipcMain.handle('weekly-report:save', (_event, report) => {
        try {
            return db.saveWeeklyReport(report);
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('weekly-report:get-all', (_event, limit = 20) => {
        try {
            return db.getWeeklyReports(limit);
        }
        catch {
            return [];
        }
    });
    electron_1.ipcMain.handle('weekly-report:get', (_event, id) => {
        try {
            return db.getWeeklyReport(id);
        }
        catch {
            return null;
        }
    });
    // ============ MFA / TOTP ============
    electron_1.ipcMain.handle('mfa:get-secrets', () => db.getAllMfaSecrets());
    electron_1.ipcMain.handle('mfa:add-secret', (_event, data) => {
        if (!data.name?.trim())
            return { success: false, error: '名称不能为空' };
        if (!data.secret?.trim())
            return { success: false, error: '密钥不能为空' };
        // validateBase32 is passed from main.ts
        return { success: false, error: 'validation not available in this module' };
    });
    electron_1.ipcMain.handle('mfa:update-secret', (_event, id, updates) => {
        try {
            const result = db.updateMfaSecret(id, updates);
            if (!result)
                return { success: false, error: '条目不存在' };
            return { success: true, data: result };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('mfa:delete-secret', (_event, id) => {
        try {
            db.deleteMfaSecret(id);
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ Note Groups ============
    electron_1.ipcMain.handle('note-groups:get-all', () => db.getAllNoteGroups());
    electron_1.ipcMain.handle('note-groups:add', (_event, data) => db.addNoteGroup(data));
    electron_1.ipcMain.handle('note-groups:update', (_event, id, updates) => db.updateNoteGroup(id, updates));
    electron_1.ipcMain.handle('note-groups:delete', (_event, id) => db.deleteNoteGroup(id));
    // ============ Notes ============
    electron_1.ipcMain.handle('notes:get-all', (_event, query, groupId) => db.getAllNotes(query, groupId));
    electron_1.ipcMain.handle('notes:get-by-id', (_event, id) => db.getNoteById(id));
    electron_1.ipcMain.handle('notes:add', (_event, data) => db.addNote(data));
    electron_1.ipcMain.handle('notes:update', (_event, id, updates) => db.updateNote(id, updates));
    electron_1.ipcMain.handle('notes:delete', (_event, id) => db.deleteNote(id));
    // ============ Log Aggregator Presets ============
    electron_1.ipcMain.handle('logPresets:get-all', () => {
        const presets = db.getLogPresets();
        return presets.map((p) => {
            let serverIds = [];
            let keywords = [];
            try {
                serverIds = JSON.parse(p.serverIds || '[]');
            }
            catch { }
            try {
                keywords = JSON.parse(p.keywords || '[]');
            }
            catch { }
            return { ...p, serverIds, keywords };
        });
    });
    electron_1.ipcMain.handle('logPresets:add', (_event, preset) => db.addLogPreset(preset.name, preset.serverIds, preset.logPath, preset.logType, preset.keywords, preset.maxLines, preset.presetGroup));
    electron_1.ipcMain.handle('logPresets:update', (_event, id, updates) => db.updateLogPreset(id, updates));
    electron_1.ipcMain.handle('logPresets:delete', (_event, id) => db.deleteLogPreset(id));
    electron_1.ipcMain.handle('shell-env:get', () => ({ loaded: (0, shell_env_manager_1.isShellEnvLoaded)(), vars: (0, shell_env_manager_1.getLoadedShellEnv)() }));
    electron_1.ipcMain.handle('get-app-path', () => (0, app_bootstrap_1.getSuperToolDataDir)());
    electron_1.ipcMain.handle('get-menu-icon', (_event, name) => {
        const img = (0, menu_manager_1.getMenuIcon)(name);
        if (!img)
            return null;
        return img.toDataURL();
    });
    // ============ Module Scanner ============
    electron_1.ipcMain.handle('modules:scan', async (_event, projectPath) => {
        try {
            const modules = scanProjectModules(projectPath);
            return { success: true, modules };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ MFA TOTP Utilities ============
    electron_1.ipcMain.handle('mfa:generate-code', (_event, secret, digits, period, algorithm) => {
        try {
            if (!totpUtils.validateBase32(secret))
                return { code: '', remaining: 0 };
            const raw = totpUtils.generateTOTP(secret, digits, period, algorithm);
            const remaining = totpUtils.getRemainingTime(period);
            return { code: totpUtils.formatCode(raw), remaining };
        }
        catch {
            return { code: '', remaining: 0 };
        }
    });
    electron_1.ipcMain.handle('mfa:parse-uri', (_event, uri) => {
        const parsed = totpUtils.parseOtpAuthUri(uri);
        if (!parsed)
            return { success: false, error: '无效的 otpauth:// URI 格式' };
        if (!totpUtils.validateBase32(parsed.secret))
            return { success: false, error: 'URI 中的密钥格式无效' };
        return { success: true, ...parsed };
    });
    // ============ Backup Operations ============
    electron_1.ipcMain.handle('backup:auto-backup', async (_event, settings) => {
        const { scheduleAutoBackup } = await Promise.resolve().then(() => __importStar(require('./auto-backup-manager')));
        db.setSetting('auto_backup_enabled', settings.enabled ? '1' : '0');
        if (settings.path)
            db.setSetting('auto_backup_path', settings.path);
        if (settings.interval)
            db.setSetting('auto_backup_interval', settings.interval.toString());
        if (settings.enabled) {
            await scheduleAutoBackup();
            return { success: true };
        }
        else {
            const { stopAutoBackup } = await Promise.resolve().then(() => __importStar(require('./auto-backup-manager')));
            stopAutoBackup();
            return { success: true };
        }
    });
    electron_1.ipcMain.handle('backup:export-csv', async (_event, options) => {
        try {
            const data = db.exportAllData();
            // Simple CSV conversion for todos (expand as needed)
            if (!data.todos)
                return { success: false, error: 'No data to export' };
            const csvHeader = Object.keys(data.todos[0] || {}).join(',');
            const csvRows = data.todos.map((todo) => Object.values(todo).map(v => typeof v === 'string' && v.includes(',') ? `"${v}"` : v).join(','));
            const csv = [csvHeader, ...csvRows].join('\n');
            return { success: true, csv };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
} // end registerUiHandlers
//# sourceMappingURL=ui-handlers.js.map