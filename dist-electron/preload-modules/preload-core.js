"use strict";
/**
 * 核心类型定义 + 参数净化 + contextBridge 合并逻辑
 * 供所有 preload 模块共享使用
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.sanitizeArgs = sanitizeArgs;
exports.exposeAPI = exposeAPI;
exports.setupDragDrop = setupDragDrop;
/** 全局 IPC 参数净化 —— 渲染进程侧 patch，根治 "An object could not be cloned" 错误 */
function sanitizeArgs(fn) {
    return function (...args) {
        const clean = args.map(a => {
            if (a !== null && typeof a === 'object') {
                try {
                    return JSON.parse(JSON.stringify(a));
                }
                catch { }
            }
            return a;
        });
        return fn.apply(this, clean);
    };
}
/** 合并所有模块并暴露到 contextBridge */
function exposeAPI(modules) {
    const { contextBridge } = require('electron');
    const merged = {};
    for (const mod of modules) {
        for (const [key, value] of Object.entries(mod)) {
            merged[key] = typeof value === 'function' ? sanitizeArgs(value) : value;
        }
    }
    contextBridge.exposeInMainWorld('electronAPI', merged);
}
/** 设置原生拖拽事件监听 */
function setupDragDrop() {
    document.addEventListener('drop', (e) => {
        const dragEvent = e;
        const dt = dragEvent.dataTransfer;
        if (!dt || !dt.files || dt.files.length === 0)
            return;
        const files = dt.files;
        const paths = [];
        for (let i = 0; i < files.length; i++) {
            if (files[i].path)
                paths.push(files[i].path);
        }
        if (paths.length > 0) {
            window.dispatchEvent(new CustomEvent('electron-file-drop', { detail: paths }));
        }
    }, { capture: true });
}
//# sourceMappingURL=preload-core.js.map