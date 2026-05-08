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
exports.trackMenuUsage = trackMenuUsage;
exports.refreshApplicationMenu = refreshApplicationMenu;
exports.getMenuIcon = getMenuIcon;
exports.loadAndRegisterShortcuts = loadAndRegisterShortcuts;
exports.updateShortcuts = updateShortcuts;
exports.buildFavoritesMenu = buildFavoritesMenu;
exports.createMenuTemplate = createMenuTemplate;
const logger_1 = require("./logger");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const window_manager_1 = require("./window-manager");
const MENU_USAGE_PATH = path.join(os.homedir(), '.supertool', 'menu-usage.json');
function loadMenuUsage() {
    try {
        if (fs.existsSync(MENU_USAGE_PATH)) {
            return JSON.parse(fs.readFileSync(MENU_USAGE_PATH, 'utf-8'));
        }
    }
    catch { }
    return {};
}
function saveMenuUsage(usage) {
    try {
        fs.writeFileSync(MENU_USAGE_PATH, JSON.stringify(usage, null, 2));
    }
    catch { }
}
function trackMenuUsage(actionId) {
    const usage = loadMenuUsage();
    const now = Date.now();
    if (!usage[actionId])
        usage[actionId] = { count: 0, lastUsed: 0 };
    usage[actionId].count++;
    usage[actionId].lastUsed = now;
    saveMenuUsage(usage);
    refreshApplicationMenu();
}
function refreshApplicationMenu() {
    if (!(0, window_manager_1.getMainWindow)())
        return;
    const menu = electron_1.Menu.buildFromTemplate(createMenuTemplate());
    electron_1.Menu.setApplicationMenu(menu);
}
function getTopMenuItems(limit = 6) {
    const usage = loadMenuUsage();
    const now = Date.now();
    const oneDay = 86400000;
    const scored = Object.entries(usage)
        .filter(([id]) => id.startsWith('nav:'))
        .map(([id, entry]) => {
        const daysAgo = (now - entry.lastUsed) / oneDay;
        const decay = daysAgo <= 7 ? 1.0 : daysAgo <= 30 ? 0.5 : 0.2;
        return { id, score: entry.count * decay };
    })
        .filter(x => x.score > 0)
        .sort((a, b) => b.score - a.score);
    return scored.slice(0, limit);
}
// 菜单项定义映射
const MENU_ITEM_MAP = {
    'new-task': { label: '新建任务', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:new-task'), accelerator: 'CmdOrCtrl+N' },
    'search-tasks': { label: '搜索任务', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:search-tasks'), accelerator: 'CmdOrCtrl+F' },
    'nav:todo': { label: '任务列表', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'todo'), accelerator: 'CmdOrCtrl+1' },
    'nav:weekly-report': { label: '周报', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'weekly-report'), accelerator: 'CmdOrCtrl+2' },
    'nav:projects': { label: '项目', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'projects'), accelerator: 'CmdOrCtrl+3' },
    'nav:servers': { label: '服务器管理', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'servers'), accelerator: 'CmdOrCtrl+4' },
    'nav:cicd': { label: 'CI/CD 部署', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'cicd'), accelerator: 'CmdOrCtrl+5' },
    'nav:database': { label: '数据库管理', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'database'), accelerator: 'CmdOrCtrl+6' },
    'nav:notes': { label: '笔记', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'notes'), accelerator: 'CmdOrCtrl+7' },
    'nav:devtools': { label: '开发工具', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'devtools'), accelerator: 'CmdOrCtrl+8' },
    'nav:accounting': { label: '记账本', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'accounting'), accelerator: 'CmdOrCtrl+9' },
    'nav:mfa': { label: 'MFA 验证码', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'mfa') },
    'nav:openvpn': { label: 'OpenVPN', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'openvpn') },
    'nav:log-aggregator': { label: '日志聚合', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'log-aggregator') },
    'nav:git': { label: 'Git 仓库管理', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'git') },
    'nav:data-backup': { label: '数据备份', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'data-backup') },
    'nav:notifications': { label: '通知设置', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'notifications') },
    'toggle-theme': { label: '切换主题', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:toggle-theme') },
    'toggle-lan': { label: '局域网协作', action: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:toggle-lan-panel') },
};
// ============ Menu Icons ============
const MENU_ICONS_DIR = path.join(__dirname, 'icons', 'menu-png');
const menuIconCache = new Map();
function getMenuIcon(name) {
    if (menuIconCache.has(name))
        return menuIconCache.get(name);
    const isDev = process.env.NODE_ENV === 'development' || !fs.existsSync(path.join(__dirname, '../dist'));
    const baseDir = isDev
        ? path.join(__dirname, '..', 'electron', 'icons', 'menu-png')
        : path.join(process.resourcesPath, 'electron', 'icons', 'menu-png');
    const pngPath = path.join(baseDir, `${name}@2x.png`);
    if (!fs.existsSync(pngPath))
        return undefined;
    const img = electron_1.nativeImage.createFromPath(pngPath);
    img.setTemplateImage(true);
    menuIconCache.set(name, img);
    return img;
}
// ============ 自定义快捷键管理 ============
const registeredAccelerators = new Map();
const DEFAULT_SHORTCUTS = {
    'shortcut_new_task': 'CmdOrCtrl+N',
    'shortcut_search': 'CmdOrCtrl+F',
    'shortcut_global_search': 'CmdOrCtrl+K',
    'shortcut_quick_switch': 'CmdOrCtrl+Shift+Tab',
    'shortcut_settings': 'CmdOrCtrl+,',
    'shortcut_toggle_theme': 'CmdOrCtrl+Shift+D',
    'shortcut_toggle_locale': 'CmdOrCtrl+Shift+L',
    'shortcut_nav:todo': 'CmdOrCtrl+1',
    'shortcut_nav:weekly-report': 'CmdOrCtrl+2',
    'shortcut_nav:projects': 'CmdOrCtrl+3',
    'shortcut_nav:servers': 'CmdOrCtrl+4',
    'shortcut_nav:cicd': 'CmdOrCtrl+5',
    'shortcut_nav:database': 'CmdOrCtrl+6',
    'shortcut_nav:notes': 'CmdOrCtrl+7',
    'shortcut_nav:devtools': 'CmdOrCtrl+8',
};
const SHORTCUT_ACTIONS = {
    'shortcut_new_task': () => (0, window_manager_1.getMainWindow)()?.webContents.send('menu:new-task'),
    'shortcut_search': () => (0, window_manager_1.getMainWindow)()?.webContents.send('menu:search-tasks'),
    'shortcut_quick_switch': () => (0, window_manager_1.getMainWindow)()?.webContents.send('quick-switch:open'),
    'shortcut_toggle_theme': () => (0, window_manager_1.getMainWindow)()?.webContents.send('menu:toggle-theme'),
    'shortcut_toggle_locale': () => (0, window_manager_1.getMainWindow)()?.webContents.send('menu:toggle-locale'),
};
function createNavAction(viewMode) {
    return () => (0, window_manager_1.getMainWindow)()?.webContents.send('menu:nav', viewMode);
}
for (const key of Object.keys(DEFAULT_SHORTCUTS)) {
    if (key.startsWith('shortcut_nav:') && !SHORTCUT_ACTIONS[key]) {
        const viewMode = key.replace('shortcut_', '');
        SHORTCUT_ACTIONS[key] = createNavAction(viewMode);
    }
}
function registerAccelerator(key, accelerator) {
    if (!accelerator || !SHORTCUT_ACTIONS[key])
        return false;
    registeredAccelerators.set(key, accelerator);
    const menuKey = key.replace('shortcut_', '');
    if (MENU_ITEM_MAP[menuKey]) {
        MENU_ITEM_MAP[menuKey].accelerator = accelerator;
    }
    return true;
}
async function loadAndRegisterShortcuts(db) {
    for (const [key, defaultAccel] of Object.entries(DEFAULT_SHORTCUTS)) {
        try {
            const saved = db.getSetting(key);
            const accelerator = saved || defaultAccel;
            registerAccelerator(key, accelerator);
        }
        catch {
            registerAccelerator(key, defaultAccel);
        }
    }
    (0, logger_1.info)(`[Shortcuts] Registered ${registeredAccelerators.size} accelerators`);
}
async function updateShortcuts(shortcuts, db) {
    for (const [key, accelerator] of Object.entries(shortcuts)) {
        const dbKey = `shortcut_${key}`;
        registerAccelerator(dbKey, accelerator);
        db.setSetting(dbKey, accelerator);
    }
    if ((0, window_manager_1.getMainWindow)()) {
        const menu = electron_1.Menu.buildFromTemplate(createMenuTemplate());
        electron_1.Menu.setApplicationMenu(menu);
    }
    (0, logger_1.info)('[Shortcuts] Updated and rebuilt menu');
}
function buildFavoritesMenu() {
    const topItems = getTopMenuItems(6);
    if (topItems.length === 0) {
        return {
            label: '常用',
            submenu: [
                { label: '任务列表', icon: getMenuIcon('todo'), accelerator: 'CmdOrCtrl+1', click: () => { trackMenuUsage('nav:todo'); (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'todo'); } },
                { label: '周报', icon: getMenuIcon('weekly-report'), accelerator: 'CmdOrCtrl+2', click: () => { trackMenuUsage('nav:weekly-report'); (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'weekly-report'); } },
                { label: '项目', icon: getMenuIcon('projects'), accelerator: 'CmdOrCtrl+3', click: () => { trackMenuUsage('nav:projects'); (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'projects'); } },
                { type: 'separator' },
                { label: '服务器管理', icon: getMenuIcon('servers'), accelerator: 'CmdOrCtrl+4', click: () => { trackMenuUsage('nav:servers'); (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'servers'); } },
                { label: 'CI/CD 部署', icon: getMenuIcon('cicd'), accelerator: 'CmdOrCtrl+5', click: () => { trackMenuUsage('nav:cicd'); (0, window_manager_1.getMainWindow)().webContents.send('menu:nav', 'cicd'); } },
            ]
        };
    }
    const items = topItems.map(({ id }) => {
        const item = MENU_ITEM_MAP[id];
        const iconKey = id.replace('nav:', '');
        return {
            label: item?.label || id,
            icon: getMenuIcon(iconKey),
            accelerator: item?.accelerator,
            click: () => { trackMenuUsage(id); item?.action(); }
        };
    });
    return { label: '常用', submenu: items };
}
function trackedItem(actionId, label, sendChannel, sendArg, accelerator) {
    const iconKey = actionId.replace('nav:', '');
    return {
        label,
        icon: getMenuIcon(iconKey),
        accelerator,
        click: () => { trackMenuUsage(actionId); (0, window_manager_1.getMainWindow)().webContents.send(sendChannel, sendArg); }
    };
}
function createMenuTemplate() {
    return [
        buildFavoritesMenu(),
        {
            label: '业务',
            submenu: [
                trackedItem('nav:todo', '任务列表', 'menu:nav', 'todo', 'CmdOrCtrl+1'),
                { label: '新建任务', icon: getMenuIcon('new-task'), accelerator: 'CmdOrCtrl+N', click: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:new-task') },
                { label: '搜索任务', icon: getMenuIcon('search'), accelerator: 'CmdOrCtrl+F', click: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:search-tasks') },
                { type: 'separator' },
                trackedItem('nav:weekly-report', '周报', 'menu:nav', 'weekly-report', 'CmdOrCtrl+2'),
                trackedItem('nav:projects', '项目', 'menu:nav', 'projects', 'CmdOrCtrl+3'),
                trackedItem('nav:accounting', '记账本', 'menu:nav', 'accounting', 'CmdOrCtrl+9'),
            ]
        },
        {
            label: '运维',
            submenu: [
                trackedItem('nav:servers', '服务器管理', 'menu:nav', 'servers', 'CmdOrCtrl+4'),
                trackedItem('nav:cicd', 'CI/CD 部署', 'menu:nav', 'cicd', 'CmdOrCtrl+5'),
                trackedItem('nav:log-aggregator', '日志聚合', 'menu:nav', 'log-aggregator'),
            ]
        },
        {
            label: '开发',
            submenu: [
                trackedItem('nav:database', '数据库管理', 'menu:nav', 'database', 'CmdOrCtrl+6'),
                trackedItem('nav:devtools', '开发工具', 'menu:nav', 'devtools', 'CmdOrCtrl+8'),
                trackedItem('nav:notes', '笔记', 'menu:nav', 'notes', 'CmdOrCtrl+7'),
            ]
        },
        {
            label: '安全',
            submenu: [
                trackedItem('nav:mfa', 'MFA 验证码', 'menu:nav', 'mfa'),
                trackedItem('nav:openvpn', 'OpenVPN', 'menu:nav', 'openvpn'),
            ]
        },
        {
            label: '系统',
            submenu: [
                { label: '关于', icon: getMenuIcon('about'), click: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:about') },
                { type: 'separator' },
                trackedItem('nav:notifications', '通知设置', 'menu:nav', 'notifications'),
                trackedItem('nav:data-backup', '数据备份', 'menu:nav', 'data-backup'),
                { type: 'separator' },
                { label: '切换语言', icon: getMenuIcon('language'), click: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:toggle-locale') },
                { label: '切换主题', icon: getMenuIcon('theme'), click: () => (0, window_manager_1.getMainWindow)().webContents.send('menu:toggle-theme') },
                { type: 'separator' },
                { label: '快速切换', icon: getMenuIcon('quick-switch'), accelerator: 'CmdOrCtrl+Shift+Tab', click: () => (0, window_manager_1.getMainWindow)().webContents.send('quick-switch:open') },
                { type: 'separator' },
                { role: 'cut' },
                { role: 'copy' },
                { role: 'paste' },
                { role: 'selectAll' },
                { type: 'separator' },
                { role: 'quit' }
            ]
        }
    ];
}
//# sourceMappingURL=menu-manager.js.map