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
exports.registerSftpHandlers = registerSftpHandlers;
const async_exec_1 = require("./async-exec");
const logger_1 = require("./logger");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const app_bootstrap_1 = require("./app-bootstrap");
const window_manager_1 = require("./window-manager");
const encryption_manager_1 = require("./encryption-manager");
// Speed tracker utility for transfer progress
class SpeedTracker {
    constructor() {
        this.samples = [];
        this.windowMs = 2000;
    }
    add(bytes) {
        const now = Date.now();
        this.samples.push({ time: now, bytes });
        const cutoff = now - this.windowMs;
        this.samples = this.samples.filter(s => s.time > cutoff);
    }
    get speed() {
        if (this.samples.length < 2)
            return 0;
        const first = this.samples[0];
        const last = this.samples[this.samples.length - 1];
        const elapsed = (last.time - first.time) / 1000;
        if (elapsed === 0)
            return 0;
        return (last.bytes - first.bytes) / elapsed;
    }
    format() {
        const s = this.speed;
        if (s === 0)
            return '0 B/s';
        if (s < 1024)
            return `${s.toFixed(0)} B/s`;
        if (s < 1024 * 1024)
            return `${(s / 1024).toFixed(1)} KB/s`;
        return `${(s / (1024 * 1024)).toFixed(1)} MB/s`;
    }
    reset() { this.samples = []; }
}
// Upload session state
const uploadSessions = new Map();
const editorSessions = new Map();
function registerSftpHandlers(getServerService, db, requireService) {
    // ============ SFTP Basic Operations ============
    electron_1.ipcMain.handle('servers:sftp:list', async (_event, serverId, remotePath) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        try {
            const files = await serverService.listRemoteDir(serverId, remotePath);
            return { success: true, files };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('servers:sftp:download', async (_event, serverId, remotePath, localPath) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        try {
            const speedTracker = new SpeedTracker();
            return await serverService.downloadFile(serverId, remotePath, localPath, (transferred, total) => {
                speedTracker.add(transferred);
                if ((0, window_manager_1.getMainWindow)()) {
                    (0, window_manager_1.getMainWindow)().webContents.send('sftp:download-progress', {
                        serverId, percent: Math.round((transferred / total) * 100), transferred, total,
                        speed: speedTracker.speed, speedFormatted: speedTracker.format(), message: `${speedTracker.format()}`
                    });
                }
            });
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('servers:sftp:upload', async (_event, serverId, localPath, remotePath) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        try {
            const speedTracker = new SpeedTracker();
            return await serverService.uploadFile(serverId, localPath, remotePath, (transferred, total) => {
                speedTracker.add(transferred);
                if ((0, window_manager_1.getMainWindow)()) {
                    (0, window_manager_1.getMainWindow)().webContents.send('sftp:upload-progress', {
                        serverId, percent: Math.round((transferred / total) * 100), transferred, total,
                        speed: speedTracker.speed, speedFormatted: speedTracker.format(), message: `${speedTracker.format()}`
                    });
                }
            });
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('servers:sftp:upload-files', async (_event, serverId, remotePath, filePaths) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        const results = [];
        for (const localPath of filePaths) {
            try {
                const fileName = path.basename(localPath);
                const remoteFilePath = remotePath.endsWith('/') ? remotePath + fileName : remotePath + '/' + fileName;
                await serverService.uploadFile(serverId, localPath, remoteFilePath);
                results.push({ file: fileName, success: true });
            }
            catch (error) {
                results.push({ file: path.basename(localPath), success: false, error: error.message });
            }
        }
        return { success: true, results };
    });
    // 上传文件夹：打包为 tar.gz，上传，远程解压，删除压缩包
    electron_1.ipcMain.handle('servers:sftp:upload-folder', async (_event, serverId, localDirPath, remotePath) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        const { exec } = require('child_process');
        const { promisify } = require('util');
        const execAsync = promisify(exec);
        const folderName = path.basename(localDirPath);
        const archiveName = `${folderName}.tar.gz`;
        const tmpDir = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp');
        const tmpArchive = path.join(tmpDir, archiveName);
        try {
            await execAsync(`COPYFILE_DISABLE=1 tar --exclude='._*' -czf "${tmpArchive}" -C "${path.dirname(localDirPath)}" "${folderName}"`);
            const remoteArchive = remotePath.endsWith('/') ? remotePath + archiveName : remotePath + '/' + archiveName;
            const speedTracker1 = new SpeedTracker();
            await serverService.uploadFile(serverId, tmpArchive, remoteArchive, (transferred, total) => {
                speedTracker1.add(transferred);
                if ((0, window_manager_1.getMainWindow)()) {
                    (0, window_manager_1.getMainWindow)().webContents.send('sftp:upload-progress', {
                        serverId, percent: Math.round((transferred / total) * 100), transferred, total,
                        speed: speedTracker1.speed, speedFormatted: speedTracker1.format(), message: speedTracker1.format()
                    });
                }
            });
            const remoteDir = remotePath.endsWith('/') ? remotePath + folderName : remotePath + '/' + folderName;
            const uploadFolderCheck = await serverService.execCommand(serverId, `test -f "${remoteArchive}" && echo OK`);
            if (!uploadFolderCheck.output.includes('OK')) {
                await new Promise(r => setTimeout(r, 1500));
            }
            await serverService.execCommand(serverId, `tar -xzf "${remoteArchive}" -C "${path.dirname(remoteDir)}"`);
            await serverService.execCommand(serverId, `rm -f "${remoteArchive}"`);
            try {
                fs.unlinkSync(tmpArchive);
            }
            catch { }
            return { success: true, folderName };
        }
        catch (error) {
            try {
                fs.unlinkSync(tmpArchive);
            }
            catch { }
            return { success: false, error: error.message };
        }
    });
    // 获取用户下载目录
    electron_1.ipcMain.handle('sftp:get-downloads-dir', () => electron_1.app.getPath('downloads'));
    // 选择目录对话框
    electron_1.ipcMain.handle('sftp:show-open-dialog-dirs', async () => {
        const result = await electron_1.dialog.showOpenDialog((0, window_manager_1.getMainWindow)(), {
            title: '选择上传文件夹', properties: ['openDirectory']
        });
        return { canceled: result.canceled, filePaths: result.filePaths };
    });
    // Read directory contents
    electron_1.ipcMain.handle('fs:readdir', async (_event, dirPath) => {
        const fsMod = await Promise.resolve().then(() => __importStar(require('fs')));
        const pathMod = await Promise.resolve().then(() => __importStar(require('path')));
        try {
            const entries = fsMod.readdirSync(dirPath);
            return { success: true, entries: entries.map((e) => ({ name: e, path: pathMod.join(dirPath, e) })) };
        }
        catch (err) {
            return { success: false, error: err.message };
        }
    });
    // 服务器监控
    electron_1.ipcMain.handle('servers:monitor', async (_event, serverId, commands) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        const results = {};
        for (const cmd of commands) {
            try {
                const r = await serverService.execCommand(serverId, cmd);
                results[cmd] = r.success ? r.output : '';
            }
            catch {
                results[cmd] = '';
            }
        }
        return { success: true, results };
    });
    electron_1.ipcMain.handle('sftp:show-open-dialog', async () => {
        const result = await electron_1.dialog.showOpenDialog((0, window_manager_1.getMainWindow)(), {
            title: '选择上传文件', properties: ['openFile', 'multiSelections']
        });
        return { canceled: result.canceled, filePaths: result.filePaths };
    });
    // ============ OpenVPN File Dialog ============
    electron_1.ipcMain.handle('dialog:openOvpnFile', async () => {
        const result = await electron_1.dialog.showOpenDialog((0, window_manager_1.getMainWindow)(), {
            title: '导入 OpenVPN 配置文件 (.ovpn)',
            properties: ['openFile'],
            filters: [{ name: 'OpenVPN Config', extensions: ['ovpn', 'conf'] }],
        });
        return { canceled: result.canceled, filePaths: result.filePaths };
    });
    // ============ File Reading (for renderer) ============
    electron_1.ipcMain.handle('file:read-content', async (_event, filePath) => {
        return fs.readFileSync(filePath, 'utf-8');
    });
    // ============ SFTP 分阶段上传（大文件不卡死）============
    // Phase 1: 创建上传会话，返回 sessionId
    electron_1.ipcMain.handle('sftp:upload-session-start', async () => {
        const sessionId = `upload_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        const tmpDir = path.join(path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp'), sessionId);
        fs.mkdirSync(tmpDir, { recursive: true });
        uploadSessions.set(sessionId, { tmpDir, files: [] });
        return { success: true, sessionId };
    });
    // Phase 2: 向会话添加文件（可多次调用）
    electron_1.ipcMain.handle('sftp:upload-session-add', async (_event, sessionId, items) => {
        const session = uploadSessions.get(sessionId);
        if (!session)
            return { success: false, error: '会话不存在' };
        try {
            for (const item of items) {
                const targetPath = path.join(session.tmpDir, item.relativePath);
                const targetDir = path.dirname(targetPath);
                if (!fs.existsSync(targetDir))
                    fs.mkdirSync(targetDir, { recursive: true });
                fs.writeFileSync(targetPath, Buffer.from(item.data));
                session.files.push({ relativePath: item.relativePath, data: item.data });
            }
            return { success: true, fileCount: items.length };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // Phase 1.5: 检查冲突文件
    electron_1.ipcMain.handle('sftp:upload-session-check-conflicts', async (_event, sessionId, serverId, remotePath) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        const session = uploadSessions.get(sessionId);
        if (!session)
            return { success: false, error: '会话不存在' };
        try {
            const escapedRemote = remotePath.replace(/'/g, "'\\''");
            const findResult = await serverService.execCommand(serverId, `find '${escapedRemote}' -type f -printf '%P\\t%s\\n' 2>/dev/null || find '${escapedRemote}' -type f 2>/dev/null`);
            const remoteFiles = new Map();
            if (findResult.success && findResult.output.trim()) {
                for (const line of findResult.output.trim().split('\n')) {
                    const parts = line.split('\t');
                    if (parts.length === 2)
                        remoteFiles.set(parts[0], parseInt(parts[1]));
                    else if (parts.length === 1 && parts[0])
                        remoteFiles.set(parts[0], -1);
                }
            }
            const conflicts = [];
            for (const item of session.files) {
                if (remoteFiles.has(item.relativePath)) {
                    conflicts.push({
                        relativePath: item.relativePath,
                        remoteSize: remoteFiles.get(item.relativePath),
                        localSize: item.data.length
                    });
                }
            }
            return { success: true, conflicts, totalFiles: session.files.length };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // Phase 3: 提交上传（压缩 → 上传 → 解压 → 清理）
    electron_1.ipcMain.handle('sftp:upload-session-commit', async (_event, sessionId, serverId, remotePath, options = {}) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        const session = uploadSessions.get(sessionId);
        if (!session)
            return { success: false, error: '会话不存在' };
        const { exec } = require('child_process');
        const { promisify } = require('util');
        const execAsync = promisify(exec);
        const totalFiles = session.files.length;
        const archiveName = session.archiveName || `sftp_upload_${serverId}_${Buffer.from(remotePath).toString('base64').replace(/[^a-zA-Z0-9]/g, '')}.tar.gz`;
        const tmpArchive = path.join(path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp'), archiveName);
        session.archiveName = archiveName;
        const sendProgress = (percent, message) => {
            if ((0, window_manager_1.getMainWindow)()) {
                (0, window_manager_1.getMainWindow)().webContents.send('sftp:upload-progress', { serverId, percent, message });
            }
        };
        async function withRetry(fn, label, timeoutMs, maxRetries = 3) {
            for (let attempt = 1; attempt <= maxRetries; attempt++) {
                try {
                    const timeout = new Promise((_, reject) => {
                        setTimeout(() => reject(new Error(`${label} 超时 (${timeoutMs / 1000}s)`)), timeoutMs);
                    });
                    await Promise.race([fn(), timeout]);
                    return;
                }
                catch (error) {
                    const isLast = attempt === maxRetries;
                    const isTimeout = error.message.includes('超时');
                    (0, logger_1.info)(`[SFTP] ${label} 失败 (${attempt}/${maxRetries}): ${error.message}${isLast ? '' : `, ${isTimeout ? '即将重试' : '重试中...'}`}`);
                    if (isLast)
                        throw new Error(`${label} 失败: ${error.message}`);
                    await new Promise(r => setTimeout(r, 1000 * attempt));
                }
            }
        }
        try {
            const { overwrite = true } = options;
            sendProgress(5, '扫描远程目录...');
            let remoteFiles = new Map();
            let remoteArchiveExists = false;
            let remoteArchiveSize = 0;
            try {
                const escapedRemote = remotePath.replace(/'/g, "'\\''");
                try {
                    const statResult = await serverService.execCommand(serverId, `stat -c '%s' '${escapedRemote}${archiveName}' 2>/dev/null || stat -c '%s' '${escapedRemote}/${archiveName}' 2>/dev/null`);
                    if (statResult.success && statResult.output.trim()) {
                        remoteArchiveExists = true;
                        remoteArchiveSize = parseInt(statResult.output.trim());
                    }
                }
                catch { }
                const findResult = await serverService.execCommand(serverId, `find '${escapedRemote}' -type f -printf '%P\\t%s\\n' 2>/dev/null || find '${escapedRemote}' -type f 2>/dev/null`);
                if (findResult.success && findResult.output.trim()) {
                    for (const line of findResult.output.trim().split('\n')) {
                        const parts = line.split('\t');
                        if (parts.length === 2)
                            remoteFiles.set(parts[0], parseInt(parts[1]));
                        else if (parts.length === 1 && parts[0])
                            remoteFiles.set(parts[0], -1);
                    }
                }
            }
            catch { /* 获取失败，全部上传 */ }
            const filesToUpload = session.files.filter(item => {
                const remoteExists = remoteFiles.has(item.relativePath);
                if (!remoteExists)
                    return true;
                if (overwrite)
                    return true;
                return false;
            });
            const skippedCount = totalFiles - filesToUpload.length;
            if (skippedCount > 0)
                (0, logger_1.info)(`[SFTP] 跳过 ${skippedCount} 个文件`);
            if (filesToUpload.length === 0) {
                sendProgress(100, '所有文件已跳过');
                uploadSessions.delete(sessionId);
                try {
                    fs.rmSync(session.tmpDir, { recursive: true, force: true });
                }
                catch { }
                return { success: true, fileCount: 0, skippedCount };
            }
            sendProgress(25, '准备传输...');
            const uploadPaths = new Set(filesToUpload.map(f => f.relativePath));
            for (const item of session.files) {
                if (!uploadPaths.has(item.relativePath)) {
                    const targetPath = path.join(session.tmpDir, item.relativePath);
                    try {
                        fs.unlinkSync(targetPath);
                    }
                    catch { }
                }
            }
            try {
                await (0, async_exec_1.runCommand)(`find "${session.tmpDir}" -type d -empty -delete 2>/dev/null || true`);
            }
            catch { }
            sendProgress(35, '压缩中...');
            try {
                await (0, async_exec_1.runCommand)(`find "${session.tmpDir}" -name '._*' -delete 2>/dev/null || true`);
            }
            catch { }
            await execAsync(`COPYFILE_DISABLE=1 tar --exclude='._*' -czf "${tmpArchive}" -C "${session.tmpDir}" .`);
            const localArchiveSize = fs.statSync(tmpArchive).size;
            sendProgress(50, '上传中...');
            const remoteArchive = remotePath.endsWith('/') ? remotePath + archiveName : remotePath + '/' + archiveName;
            let needUpload = true;
            if (remoteArchiveExists && remoteArchiveSize === localArchiveSize) {
                (0, logger_1.info)(`[SFTP] 压缩包已存在且大小一致 (${remoteArchiveSize} bytes)，跳过上传`);
                needUpload = false;
            }
            if (needUpload) {
                const commitSpeedTracker = new SpeedTracker();
                await withRetry(async () => {
                    await serverService.uploadFile(serverId, tmpArchive, remoteArchive, (transferred, total) => {
                        commitSpeedTracker.add(transferred);
                        sendProgress(50 + Math.round((transferred / total) * 30), commitSpeedTracker.format());
                    });
                }, '上传压缩包', 300000, 3);
            }
            sendProgress(80, '解压中...');
            const escapedArchive = remoteArchive.replace(/'/g, "'\\''");
            const escapedRemote = remotePath.replace(/'/g, "'\\''");
            await withRetry(async () => {
                const checkRes = await serverService.execCommand(serverId, `test -f '${escapedArchive}' && echo OK`);
                if (!checkRes.output.includes('OK')) {
                    await new Promise(r => setTimeout(r, 1500));
                    const checkRes2 = await serverService.execCommand(serverId, `test -f '${escapedArchive}' && echo OK`);
                    if (!checkRes2.output.includes('OK'))
                        throw new Error(`压缩包不存在: ${remoteArchive}`);
                }
                const result = await serverService.execCommand(serverId, `tar -xzf '${escapedArchive}' -C '${escapedRemote}'`);
                if (!result.success)
                    throw new Error(result.errorOutput || '解压失败');
            }, '远程解压', 120000, 2);
            await serverService.execCommand(serverId, `rm -f '${escapedArchive}'`);
            uploadSessions.delete(sessionId);
            try {
                fs.rmSync(session.tmpDir, { recursive: true, force: true });
            }
            catch { }
            try {
                fs.unlinkSync(tmpArchive);
            }
            catch { }
            sendProgress(100, '完成');
            return { success: true, fileCount: filesToUpload.length, skippedCount, totalFiles };
        }
        catch (error) {
            sendProgress(0, `上传失败: ${error.message}`);
            return { success: false, error: error.message, canRetry: true, sessionId };
        }
    });
    // Phase 取消
    electron_1.ipcMain.handle('sftp:upload-session-cancel', async (_event, sessionId) => {
        const session = uploadSessions.get(sessionId);
        if (session) {
            try {
                fs.rmSync(session.tmpDir, { recursive: true, force: true });
            }
            catch { }
            try {
                fs.unlinkSync(path.join(path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp'), session.archiveName || ''));
            }
            catch { }
            uploadSessions.delete(sessionId);
        }
        return { success: true };
    });
    // 清理所有失效的会话
    function cleanupOrphanedSessions() {
        const maxAge = 30 * 60 * 1000;
        const now = Date.now();
        for (const [id, session] of uploadSessions.entries()) {
            if (now - parseInt(id.split('_')[1]) > maxAge) {
                try {
                    fs.rmSync(session.tmpDir, { recursive: true, force: true });
                }
                catch { }
                uploadSessions.delete(id);
            }
        }
    }
    setInterval(cleanupOrphanedSessions, 10 * 60 * 1000);
    // 拖拽上传
    electron_1.ipcMain.handle('servers:sftp:upload-dropped-items', async (_event, serverId, remotePath, items) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        if (!items || items.length === 0)
            return { success: false, error: '没有文件可上传' };
        const { exec } = require('child_process');
        const { promisify } = require('util');
        const execAsync = promisify(exec);
        const tmpBase = path.join(path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp'), `sftp_drop_${Date.now()}`);
        const archiveName = `sftp_upload_${Date.now()}.tar.gz`;
        const tmpArchive = path.join(path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp'), archiveName);
        const sendProgress = (percent, message) => {
            if ((0, window_manager_1.getMainWindow)()) {
                (0, window_manager_1.getMainWindow)().webContents.send('sftp:upload-progress', { serverId, percent, message });
            }
        };
        try {
            sendProgress(5, '准备文件...');
            for (const item of items) {
                const targetPath = path.join(tmpBase, item.relativePath);
                const targetDir = path.dirname(targetPath);
                if (!fs.existsSync(targetDir))
                    fs.mkdirSync(targetDir, { recursive: true });
                fs.writeFileSync(targetPath, Buffer.from(item.data));
            }
            sendProgress(15, '扫描远程目录...');
            let remoteFiles = new Map();
            try {
                const escapedRemote = remotePath.replace(/'/g, "'\\''");
                const findResult = await serverService.execCommand(serverId, `find '${escapedRemote}' -type f -printf '%P\\t%s\\n' 2>/dev/null || find '${escapedRemote}' -type f 2>/dev/null`);
                if (findResult.success && findResult.output.trim()) {
                    for (const line of findResult.output.trim().split('\n')) {
                        const parts = line.split('\t');
                        if (parts.length === 2)
                            remoteFiles.set(parts[0], parseInt(parts[1]));
                        else if (parts.length === 1 && parts[0])
                            remoteFiles.set(parts[0], -1);
                    }
                }
            }
            catch { }
            const filesToUpload = items.filter(item => {
                const remoteSize = remoteFiles.get(item.relativePath);
                if (remoteSize === undefined)
                    return true;
                if (remoteSize === -1)
                    return true;
                return remoteSize !== item.data.length;
            });
            const skippedCount = items.length - filesToUpload.length;
            if (skippedCount > 0)
                (0, logger_1.info)(`[SFTP] 跳过 ${skippedCount} 个已存在的文件`);
            if (filesToUpload.length === 0) {
                sendProgress(100, '所有文件已是最新');
                return { success: true, fileCount: 0, skippedCount };
            }
            sendProgress(25, '准备传输...');
            const uploadPaths = new Set(filesToUpload.map(f => f.relativePath));
            for (const item of items) {
                if (!uploadPaths.has(item.relativePath)) {
                    const targetPath = path.join(tmpBase, item.relativePath);
                    try {
                        fs.unlinkSync(targetPath);
                    }
                    catch { }
                }
            }
            try {
                await (0, async_exec_1.runCommand)(`find "${tmpBase}" -type d -empty -delete 2>/dev/null || true`);
            }
            catch { }
            sendProgress(35, '压缩中...');
            try {
                await (0, async_exec_1.runCommand)(`find "${tmpBase}" -name '._*' -delete 2>/dev/null || true`);
            }
            catch { }
            await execAsync(`COPYFILE_DISABLE=1 tar --exclude='._*' -czf "${tmpArchive}" -C "${tmpBase}" .`);
            sendProgress(50, '上传中...');
            const remoteArchive = remotePath.endsWith('/') ? remotePath + archiveName : remotePath + '/' + archiveName;
            const dropSpeedTracker = new SpeedTracker();
            await serverService.uploadFile(serverId, tmpArchive, remoteArchive, (transferred, total) => {
                dropSpeedTracker.add(transferred);
                sendProgress(50 + Math.round((transferred / total) * 30), dropSpeedTracker.format());
            });
            sendProgress(80, '解压中...');
            const escapedArchive = remoteArchive.replace(/'/g, "'\\''");
            const escapedRemote = remotePath.replace(/'/g, "'\\''");
            const dropCheck = await serverService.execCommand(serverId, `test -f '${escapedArchive}' && echo OK`);
            if (!dropCheck.output.includes('OK'))
                await new Promise(r => setTimeout(r, 1500));
            await serverService.execCommand(serverId, `tar -xzf '${escapedArchive}' -C '${escapedRemote}'`);
            await serverService.execCommand(serverId, `rm -f '${escapedArchive}'`);
            try {
                fs.rmSync(tmpBase, { recursive: true, force: true });
            }
            catch { }
            try {
                fs.unlinkSync(tmpArchive);
            }
            catch { }
            sendProgress(100, '完成');
            return { success: true, fileCount: filesToUpload.length, skippedCount, totalFiles: items.length };
        }
        catch (error) {
            try {
                fs.rmSync(tmpBase, { recursive: true, force: true });
            }
            catch { }
            try {
                fs.unlinkSync(tmpArchive);
            }
            catch { }
            sendProgress(0, '上传失败');
            return { success: false, error: error.message };
        }
    });
    // 从 buffer 上传文件
    electron_1.ipcMain.handle('servers:sftp:upload-from-buffer', async (_event, serverId, remotePath, buffer) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        try {
            const tmpDir = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp');
            const tmpFile = path.join(tmpDir, `sftp_upload_${Date.now()}_${path.basename(remotePath)}`);
            fs.writeFileSync(tmpFile, Buffer.from(buffer));
            const result = await serverService.uploadFile(serverId, tmpFile, remotePath);
            try {
                fs.unlinkSync(tmpFile);
            }
            catch { }
            return result;
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('servers:sftp:create-dir', async (_event, serverId, remotePath) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        try {
            return await serverService.createRemoteDir(serverId, remotePath);
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.handle('servers:sftp:delete', async (_event, serverId, remotePath) => {
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        try {
            try {
                return await serverService.deleteRemoteFile(serverId, remotePath);
            }
            catch {
                const escaped = remotePath.replace(/'/g, "'\\''");
                const result = await serverService.execCommand(serverId, `rm -rf '${escaped}'`);
                if (result.success)
                    return { success: true };
                return { success: false, error: result.errorOutput || '删除失败' };
            }
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    // ============ SFTP 远程文件编辑器 ============
    (0, logger_1.info)('[SFTP Editor] Module loaded, editorSessions Map created');
    electron_1.ipcMain.handle('sftp:open-editor', async (_event, serverId, remotePath) => {
        (0, logger_1.info)('[SFTP Editor] open-editor called, serverId:', serverId, 'remotePath:', remotePath);
        const serverService = getServerService();
        if (!serverService)
            return { success: false, error: '服务未初始化' };
        const fileName = path.basename(remotePath);
        const sessionId = `editor_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        const localPath = path.join(path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'tmp'), `sftp_edit_${sessionId}_${fileName}`);
        try {
            let currentService = serverService;
            if (!currentService || !currentService.isConnected(serverId)) {
                const ServerService = requireService('server-service');
                currentService = new ServerService();
                const server = db.getServerById(serverId);
                if (!server)
                    throw new Error('服务器不存在');
                await currentService.connect({ ...server, password: (0, encryption_manager_1.decryptPassword)(server.password) });
            }
            (0, logger_1.info)('[SFTP Editor] Downloading file to:', localPath);
            await (currentService || serverService).downloadFile(serverId, remotePath, localPath);
            (0, logger_1.info)('[SFTP Editor] Download complete');
            fs.copyFileSync(localPath, localPath + '.original');
            const preloadPath = path.join(__dirname, 'editor-preload.js');
            (0, logger_1.info)('[SFTP Editor] Preload path:', preloadPath, 'exists:', fs.existsSync(preloadPath));
            const editorWin = new electron_1.BrowserWindow({
                width: 1100, height: 700, minWidth: 600, minHeight: 400,
                title: `${fileName} — 远程编辑`, show: false,
                webPreferences: { nodeIntegration: false, contextIsolation: true, sandbox: false, preload: preloadPath, devTools: true },
            });
            editorWin.webContents.openDevTools({ mode: 'detach' });
            editorWin.webContents.on('did-finish-load', () => (0, logger_1.info)('[SFTP Editor] Page did-finish-load'));
            editorWin.webContents.on('did-fail-load', (_event, code, desc, url) => console.error('[SFTP Editor] Page did-fail-load:', code, desc, url));
            editorWin.webContents.on('render-process-gone', (_event, details) => console.error('[SFTP Editor] Renderer process gone:', details));
            editorWin.once('ready-to-show', () => {
                (0, logger_1.info)('[SFTP Editor] ready-to-show, showing window');
                editorWin.show();
            });
            setTimeout(() => {
                if (!editorWin.isDestroyed() && !editorWin.isVisible()) {
                    (0, logger_1.info)('[SFTP Editor] Timeout: forcing window show');
                    editorWin.show();
                }
            }, 5000);
            let editorHtmlPath;
            const devHtmlPath = path.join(__dirname, '..', 'electron', 'editor.html');
            const builtHtmlPath = path.join(__dirname, 'editor.html');
            if (fs.existsSync(devHtmlPath)) {
                editorHtmlPath = devHtmlPath;
                (0, logger_1.info)('[SFTP Editor] Using dev HTML path:', editorHtmlPath);
            }
            else {
                editorHtmlPath = builtHtmlPath;
                (0, logger_1.info)('[SFTP Editor] Using built HTML path:', editorHtmlPath, 'exists:', fs.existsSync(editorHtmlPath));
            }
            (0, logger_1.info)('[SFTP Editor] Loading editor HTML with sessionId:', sessionId);
            editorWin.loadFile(editorHtmlPath, { search: `?sessionId=${sessionId}` });
            editorSessions.set(sessionId, { serverId, remotePath, localPath, fileName, window: editorWin });
            (0, logger_1.info)('[SFTP Editor] Waiting for editor-ready signal for session:', sessionId);
            electron_1.ipcMain.once(`editor-ready:${sessionId}`, () => {
                (0, logger_1.info)('[SFTP Editor] editor-ready received for session:', sessionId);
                try {
                    const content = fs.readFileSync(localPath, 'utf8');
                    editorWin.webContents.send('file-content', { content, fileName, remotePath });
                }
                catch (err) {
                    editorWin.webContents.send('file-content', { content: '', fileName, remotePath, error: err.message });
                }
            });
            editorWin.on('closed', () => {
                (0, logger_1.info)('[SFTP Editor] closed event fired for session:', sessionId);
                const session = editorSessions.get(sessionId);
                if (session) {
                    try {
                        fs.unlinkSync(session.localPath);
                    }
                    catch { }
                    try {
                        fs.unlinkSync(session.localPath + '.original');
                    }
                    catch { }
                    editorSessions.delete(sessionId);
                    (0, logger_1.info)('[SFTP Editor] Orphan session cleaned up in closed event');
                }
            });
            return { success: true, sessionId };
        }
        catch (error) {
            try {
                fs.unlinkSync(localPath);
            }
            catch { }
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.on('editor-ready', (_event, sessionId) => {
        (0, logger_1.info)('[SFTP Editor] editor-ready IPC received, sessionId:', sessionId);
        electron_1.ipcMain.emit(`editor-ready:${sessionId}`);
    });
    electron_1.ipcMain.handle('save-file-content', async (_event, { sessionId, content }) => {
        (0, logger_1.info)('[SFTP Editor] save-file-content called, sessionId:', sessionId, 'content length:', content.length);
        (0, logger_1.info)('[SFTP Editor] Current sessions:', JSON.stringify([...editorSessions.entries()].map(([k, v]) => [k, v.fileName])));
        const session = editorSessions.get(sessionId);
        if (!session) {
            console.error('[SFTP Editor] Session not found! Expected:', sessionId, 'Available:', [...editorSessions.keys()]);
            return { success: false, error: '编辑会话不存在 (sessionId: ' + sessionId + ')' };
        }
        try {
            const serverService = getServerService();
            fs.writeFileSync(session.localPath, content, 'utf8');
            const speedTracker = new SpeedTracker();
            await serverService.uploadFile(session.serverId, session.localPath, session.remotePath, (transferred, total) => {
                speedTracker.add(transferred);
                if ((0, window_manager_1.getMainWindow)()) {
                    (0, window_manager_1.getMainWindow)().webContents.send('sftp:upload-progress', {
                        serverId: session.serverId,
                        percent: Math.round((transferred / total) * 100), transferred, total,
                        speedFormatted: speedTracker.format(),
                        message: `正在保存 ${session.fileName}... ${speedTracker.format()}`
                    });
                }
            });
            if ((0, window_manager_1.getMainWindow)()) {
                (0, window_manager_1.getMainWindow)().webContents.send('sftp:upload-done', { serverId: session.serverId });
            }
            return { success: true };
        }
        catch (error) {
            return { success: false, error: error.message };
        }
    });
    electron_1.ipcMain.on('close-window-after-save', (_event, sessionId) => {
        const session = editorSessions.get(sessionId);
        if (session) {
            editorSessions.delete(sessionId);
            try {
                fs.unlinkSync(session.localPath);
            }
            catch { }
            try {
                fs.unlinkSync(session.localPath + '.original');
            }
            catch { }
            session.window.destroy();
        }
    });
} // end registerSftpHandlers
//# sourceMappingURL=sftp-handlers.js.map