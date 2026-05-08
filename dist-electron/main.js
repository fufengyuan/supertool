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
exports.getLanServiceInstance = getLanServiceInstance;
const logger_1 = require("./logger");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const db = require("./database");
const modules_scanner_1 = require("./modules-scanner");
// ============ Ctrl+C 退出修复 ============
// 开发模式下 concurrently 发送 SIGINT 但 Electron GUI 进程默认忽略
// 必须显式监听并强制退出，否则终端只能靠 kill 或关终端窗口
let isExiting = false;
process.on('SIGINT', () => {
    if (isExiting)
        return;
    isExiting = true;
    (0, logger_1.info)('\n[Main] SIGINT received, exiting...');
    // 移除其他可能的信号监听，防止 Electron 内部拦截
    process.removeAllListeners('SIGINT');
    process.exit(0);
});
process.on('SIGTERM', () => {
    if (isExiting)
        return;
    isExiting = true;
    process.exit(0);
});
function getServicePath() {
    // 生产模式: 必须用 ASAR 内部的 services（resourcesPath 下的旧版已废弃）
    if (electron_1.app.isPackaged) {
        return path.join(electron_1.app.getAppPath(), 'dist-electron', 'services');
    }
    // 开发模式: services 在 dist-electron/services/
    return path.join(__dirname, 'services');
}
function requireService(name) {
    const p = path.join(getServicePath(), name);
    const mod = require(p);
    // Handle both `export = Foo` (CommonJS) and `export default Foo` (ESM interop)
    if (mod && mod.default)
        return mod.default;
    return mod;
}
const LanService = requireService('lan-service');
const dbManager = requireService('db-manager');
const { generateTOTP, getRemainingTime, validateBase32, parseOtpAuthUri, formatCode } = requireService('totp');
const { openVPNManager } = require('./services/openvpn-manager');
const uds_api_1 = require("./uds-api");
// Auto updater (only in production)
// Lazy init to avoid 'app not ready' crash in electron-updater
let lanService = null;
function getLanServiceInstance() { return lanService; }
// 判断是否为开发环境
// 优先使用 app.isPackaged（Electron 官方推荐），打包后为 true，开发时为 false
// 同时支持环境变量覆盖
const isDev = !process.env.FORCE_PROD && (process.env.NODE_ENV === 'development' ||
    process.env.ELECTRON_IS_DEV === '1' ||
    !electron_1.app.isPackaged);
const window_manager_1 = require("./window-manager");
const tray_manager_1 = require("./tray-manager");
const menu_manager_1 = require("./menu-manager");
const notification_manager_1 = require("./notification-manager");
const shell_env_manager_1 = require("./shell-env-manager");
const app_bootstrap_1 = require("./app-bootstrap");
const encryption_manager_1 = require("./encryption-manager");
const app_icon_manager_1 = require("./app-icon-manager");
const cli_installer_1 = require("./cli-installer");
const auto_backup_manager_1 = require("./auto-backup-manager");
const auto_updater_manager_1 = require("./auto-updater-manager");
const git_sync_manager_1 = require("./git-sync-manager");
const redis_stream_manager_1 = require("./redis-stream-manager");
const http_request_manager_1 = require("./http-request-manager");
const cicd_handlers_1 = require("./cicd-handlers");
const sftp_handlers_1 = require("./sftp-handlers");
const git_handlers_1 = require("./git-handlers");
const db_handlers_1 = require("./db-handlers");
const lan_handlers_1 = require("./lan-handlers");
const server_handlers_1 = require("./server-handlers");
const accounting_handlers_1 = require("./accounting-handlers");
const ui_handlers_1 = require("./ui-handlers");
const log_handlers_1 = require("./log-handlers");
const vpn_handlers_1 = require("./vpn-handlers");
// 初始化局域网服务
function initLanService() {
    lanService = new LanService((0, app_bootstrap_1.getSuperToolDataDir)());
    lanService.start();
    lanService.setCallbacks({
        onPeerDiscovered: (peer) => {
            if ((0, window_manager_1.getMainWindow)()) {
                // 剥离 WebSocket 等不可序列化属性
                const { ws, ...serializablePeer } = peer;
                (0, window_manager_1.getMainWindow)().webContents.send('lan:peer-discovered', serializablePeer);
            }
        },
        onPeerLost: (peer) => {
            if ((0, window_manager_1.getMainWindow)()) {
                const { ws, ...serializablePeer } = peer;
                (0, window_manager_1.getMainWindow)().webContents.send('lan:peer-lost', serializablePeer);
            }
        },
        onMessage: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:message', data);
            // Note: message already saved by lan-service.saveMessage(), don't duplicate
            if (electron_1.Notification.isSupported()) {
                (0, notification_manager_1.playNotificationSound)();
                new electron_1.Notification({ title: `来自 ${data.fromName} 的消息`, body: data.content }).show();
            }
        },
        onTaskAssigned: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:task-assigned', data);
            if (electron_1.Notification.isSupported()) {
                (0, notification_manager_1.playNotificationSound)();
                new electron_1.Notification({ title: '新任务分配', body: `${data.fromName} 分配了一个任务: ${data.task?.text ?? '新任务'}` }).show();
            }
            // 接收方保存到本地 DB
            try {
                const taskData = data.task;
                if (!taskData)
                    return;
                const now = new Date().toISOString();
                const myId = lanService?.userId || 'unknown';
                const todoId = taskData.id || `lan-recv-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
                db.addTodo({
                    id: todoId,
                    text: taskData.text,
                    completed: false,
                    priority: taskData.priority || 'medium',
                    dueDate: taskData.dueDate || null,
                    description: taskData.note || '',
                    markdownDescription: '',
                    tag: '',
                    createdAt: now,
                    updatedAt: now,
                    assignedTo: myId,
                    assignedBy: data.from || 'unknown',
                    assignedAt: now,
                    owner: data.from || 'unknown',
                    orderNum: 0,
                    repeatType: '',
                    repeatInterval: 0,
                    repeatCount: 0,
                    repeatEndDate: undefined,
                    parentTodoId: undefined,
                    projectId: undefined,
                });
            }
            catch (err) {
                console.error('[LAN] 接收任务保存失败:', err);
            }
        },
        onProjectSync: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:project-sync', data);
            try {
                const project = data.project;
                const existingProjects = db.getAllProjects(false);
                const exists = existingProjects.some(p => p.id === project.id);
                if (exists)
                    db.updateProject(project);
                else
                    db.addProject(project);
            }
            catch (error) {
                console.error('保存同步项目失败:', error);
            }
        },
        onFileTransferStarted: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:file-transfer-started', data);
        },
        onFileTransferProgress: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:file-transfer-progress', data);
        },
        onFileTransferCompleted: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:file-transfer-completed', data);
            // 只有接收方才弹"接收完成"通知（fromUserId !== 本机 userId）
            if (lanService && data.fromUserId !== lanService.userId && electron_1.Notification.isSupported()) {
                (0, notification_manager_1.playNotificationSound)();
                new electron_1.Notification({ title: '📁 文件接收完成', body: `从 ${data.fromUserName} 接收: ${data.fileName}\\n保存路径: ${data.filePath || '未知'}` }).show();
            }
        },
        onFileReceived: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:file-received', data);
        },
        onFileTransferError: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:file-transfer-error', data);
            if (electron_1.Notification.isSupported()) {
                (0, notification_manager_1.playNotificationSound)();
                new electron_1.Notification({ title: '文件传输错误', body: `文件传输失败: ${data.fileName}` }).show();
            }
        },
        onTaskUpdated: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:task-updated', data);
        },
        onTaskStatusChanged: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:task-status-changed', data);
            // 同步更新本地 DB 状态
            try {
                const todoData = data.todo;
                if (todoData && todoData.id) {
                    const existing = db.getTodoById(todoData.id);
                    if (existing) {
                        db.updateTodo({
                            ...existing,
                            completed: todoData.completed,
                            completedAt: todoData.completedAt || null,
                            updatedAt: new Date().toISOString()
                        });
                    }
                }
            }
            catch (err) {
                console.error('[LAN] 同步任务状态失败:', err);
            }
        },
        onTaskCommentAdded: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:task-comment-added', data);
        },
        onCollaborationStarted: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:collaboration-started', data);
        },
        onCollaborationEnded: (data) => {
            if ((0, window_manager_1.getMainWindow)())
                (0, window_manager_1.getMainWindow)().webContents.send('lan:collaboration-ended', data);
        }
    });
}
// macOS 26 GPU 进程兼容性修复
if (process.platform === 'darwin') {
    electron_1.app.commandLine.appendSwitch('disable-gpu');
    electron_1.app.commandLine.appendSwitch('disable-gpu-compositing');
    electron_1.app.commandLine.appendSwitch('disable-gpu-sandbox');
    electron_1.app.commandLine.appendSwitch('no-sandbox');
    electron_1.app.commandLine.appendSwitch('disable-features', 'GpuProcess,VizDisplayCompositor');
}
// ============ 自动安装 stool CLI 二进制到 /usr/local/bin ============
// 初始化数据库
electron_1.app.whenReady().then(async () => {
    // 加载用户 Shell 环境变量（~/.zshrc / ~/.bashrc 等），在数据库初始化前执行
    await (0, shell_env_manager_1.loadUserShellEnv)();
    // 开发模式下 electron 二进制默认叫 "Electron"，必须覆盖
    electron_1.app.setName('SuperTool');
    // Ensure unified data directory structure (~/.supertool/)
    (0, app_bootstrap_1.ensureSuperToolDirs)();
    // 自动安装 stool CLI 二进制
    (0, cli_installer_1.installStoolCliBinary)();
    // 自动分发 stool-cli 技能到 Hermes 技能目录
    (0, cli_installer_1.installHermesSkills)();
    // macOS Dock 图标和名称
    if (process.platform === 'darwin') {
        const dockIcon = (0, app_icon_manager_1.getAppIcon)();
        if (!dockIcon.isEmpty()) {
            electron_1.app.dock.setIcon(dockIcon);
            (0, logger_1.info)('[AppIcon] Set dock icon successfully');
        }
        else {
            console.warn('[AppIcon] Cannot set dock icon: image is empty');
        }
        // 确保 Dock 菜单显示正确名称
        electron_1.app.dock.setBadge('');
    }
    db.initDatabase();
    (0, window_manager_1.createWindow)({ isDev, getAppIcon: app_icon_manager_1.getAppIcon, createMenuTemplate: menu_manager_1.createMenuTemplate, appDirname: __dirname });
    (0, window_manager_1.setupWindowLifecycle)();
    (0, tray_manager_1.createTray)({ isDev, getAppIcon: app_icon_manager_1.getAppIcon, createMenuTemplate: menu_manager_1.createMenuTemplate, appDirname: __dirname, createWindow: window_manager_1.createWindow, setupWindowLifecycle: window_manager_1.setupWindowLifecycle });
    initLanService();
    (0, lan_handlers_1.registerLanHandlers)(lanService, db, notifyDataChange);
    (0, notification_manager_1.startNotificationCheck)(db);
    (0, notification_manager_1.scheduleNotifiedIdsCleanup)();
    (0, git_sync_manager_1.registerGitSyncHandlers)();
    (0, auto_backup_manager_1.scheduleAutoBackup)();
    (0, git_sync_manager_1.startGitSyncTimer)();
    (0, redis_stream_manager_1.registerRedisStreamHandlers)();
    (0, http_request_manager_1.registerHttpApiHandlers)();
    // Start UDS API for CLI (stool)
    try {
        (0, uds_api_1.startUdsApi)();
    }
    catch (e) {
        console.error('[API] Failed to start:', e.message, e.stack);
    }
    // Load custom shortcuts from database
    ;
    (async () => {
        try {
            await (0, menu_manager_1.loadAndRegisterShortcuts)(db);
        }
        catch (e) {
            console.error('[Shortcuts] Failed to load:', e.message);
        }
    })();
    (async () => {
        const enabled = await db.getSetting('git_sync_enabled');
        if (enabled === 'true') {
            try {
                const simpleGit = (await Promise.resolve().then(() => __importStar(require('simple-git')))).default;
                const syncPath = path.join((0, app_bootstrap_1.getSuperToolDataDir)(), 'sync-data');
                if (fs.existsSync(path.join(syncPath, '.git'))) {
                    const git = simpleGit(syncPath);
                    await git.fetch();
                    const behind = (await git.status()).behind;
                    if (behind > 0) {
                        await git.pull();
                        const pulled = JSON.parse(fs.readFileSync(path.join(syncPath, 'data.json'), 'utf8'));
                        db.importAllData(pulled, 'merge');
                    }
                }
            }
            catch (e) {
                console.error('[Git Sync] Startup pull failed:', e.message);
            }
        }
    })();
    (0, auto_updater_manager_1.initAutoUpdater)();
    (0, auto_updater_manager_1.setupAutoUpdater)(isDev);
    (0, auto_updater_manager_1.registerUpdateHandlers)();
    // 加载用户 Shell 环境变量（已在启动时加载）
});
electron_1.app.on('before-quit', () => {
    electron_1.app.isQuitting = true;
    if (lanService)
        lanService.stop();
    (0, uds_api_1.stopUdsApi)();
});
// ============ Data Change Notification Helper ============
function notifyDataChange(type, data) {
    const win = (0, window_manager_1.getMainWindow)();
    if (win)
        win.webContents.send('data:changed', { type, data });
}
// ============ UI Handlers ============
(0, ui_handlers_1.registerUiHandlers)(db, notifyDataChange, notification_manager_1.dismissNotification, menu_manager_1.updateShortcuts, notification_manager_1.testNotification, modules_scanner_1.scanProjectModules, { validateBase32, generateTOTP, getRemainingTime, formatCode, parseOtpAuthUri });
// ============ LAN Handlers ============
// NOTE: Must be called INSIDE app.whenReady() after initLanService(), so lanService is not null.
// Moved to line ~274.
// ============ CI/CD Handlers ============
(0, cicd_handlers_1.registerCicdHandlers)(db, notifyDataChange);
// ============ Server Handlers ============
(0, server_handlers_1.registerServerHandlers)(db, requireService, notifyDataChange);
// ============ SFTP Handlers ============
(0, sftp_handlers_1.registerSftpHandlers)(server_handlers_1.getServerService, db, requireService);
// ============ Database Handlers ============
(0, db_handlers_1.registerDbHandlers)(dbManager, db, encryption_manager_1.decryptPassword);
// ============ Log Handlers ============
(0, log_handlers_1.registerLogHandlers)(db, requireService);
// ============ Git Handlers ============
(0, git_handlers_1.registerGitHandlers)(db);
// ============ Accounting Handlers ============
(0, accounting_handlers_1.registerAccountingHandlers)(db);
// ============ VPN Handlers ============
(0, vpn_handlers_1.registerVpnHandlers)(db, openVPNManager);
// 关闭数据库
electron_1.app.on('will-quit', () => {
    (0, notification_manager_1.stopNotificationCheck)();
    (0, auto_backup_manager_1.stopAutoBackup)();
    (0, server_handlers_1.stopServerHeartbeat)();
    db.closeDatabase();
    if (lanService)
        lanService.stop();
    // Clean up active log streams to close SSH connections
    (0, log_handlers_1.getActiveLogStreams)().forEach((stops) => stops.forEach(s => { try {
        s.stop();
    }
    catch { } }));
    (0, log_handlers_1.getActiveLogStreams)().clear();
});
//# sourceMappingURL=main.js.map