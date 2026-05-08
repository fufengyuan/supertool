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
exports.getMainWindow = getMainWindow;
exports.setMainWindow = setMainWindow;
exports.createWindow = createWindow;
exports.setupWindowLifecycle = setupWindowLifecycle;
const electron_1 = require("electron");
const path = __importStar(require("path"));
let mainWindow;
let _windowOpts;
function getMainWindow() {
    return mainWindow;
}
function setMainWindow(window) {
    mainWindow = window;
}
function createWindow(opts) {
    _windowOpts = opts;
    mainWindow = new electron_1.BrowserWindow({
        width: 1280,
        height: 800,
        minWidth: 900,
        minHeight: 550,
        show: false,
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            sandbox: false,
            preload: path.join(opts.appDirname, 'preload.js')
        },
        title: 'SuperTool',
        icon: opts.getAppIcon()
    });
    // 默认最大化
    mainWindow.maximize();
    mainWindow.show();
    const menu = electron_1.Menu.buildFromTemplate(opts.createMenuTemplate());
    electron_1.Menu.setApplicationMenu(menu);
    if (opts.isDev) {
        mainWindow.loadURL('http://localhost:5173');
        mainWindow.webContents.openDevTools({ mode: 'bottom' });
    }
    else {
        mainWindow.loadFile(path.join(opts.appDirname, '../dist/index.html'));
    }
    mainWindow.on('close', (event) => {
        if (!electron_1.app.isQuitting) {
            event.preventDefault();
            mainWindow.hide();
        }
        return false;
    });
    mainWindow.on('closed', () => {
        mainWindow = undefined;
    });
    // 全局右键菜单 — 支持复制/粘贴/全选
    mainWindow.webContents.on('context-menu', (_event, params) => {
        const template = [];
        if (params.selectionText) {
            template.push({ role: 'copy' });
            template.push({ type: 'separator' });
        }
        if (params.isEditable) {
            template.push({ role: 'cut' });
            template.push({ role: 'copy' });
            template.push({ role: 'paste' });
            template.push({ type: 'separator' });
            template.push({ role: 'selectAll' });
        }
        else if (!params.selectionText) {
            template.push({ role: 'paste' });
            template.push({ type: 'separator' });
        }
        template.push({ role: 'reload' });
        template.push({ role: 'toggleDevTools' });
        electron_1.Menu.buildFromTemplate(template).popup({ window: mainWindow });
    });
}
function setupWindowLifecycle() {
    electron_1.app.on('activate', () => {
        if (mainWindow) {
            if (mainWindow.isMinimized())
                mainWindow.restore();
            if (!mainWindow.isVisible())
                mainWindow.show();
            mainWindow.focus();
        }
        else if (_windowOpts) {
            createWindow(_windowOpts);
        }
    });
    electron_1.app.on('window-all-closed', () => {
        if (process.platform !== 'darwin')
            electron_1.app.quit();
    });
}
//# sourceMappingURL=window-manager.js.map