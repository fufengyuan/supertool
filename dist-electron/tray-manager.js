"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createTray = createTray;
const electron_1 = require("electron");
const window_manager_1 = require("./window-manager");
let tray;
function createTrayIcon() {
    const size = 16;
    const canvas = Buffer.alloc(size * size * 4);
    for (let y = 0; y < size; y++) {
        for (let x = 0; x < size; x++) {
            const idx = (y * size + x) * 4;
            const cx = size / 2;
            const cy = size / 2;
            const dist = Math.sqrt((x - cx) ** 2 + (y - cy) ** 2);
            if (dist <= size / 2 - 1) {
                canvas[idx] = 118;
                canvas[idx + 1] = 75;
                canvas[idx + 2] = 162;
                canvas[idx + 3] = 255;
            }
            else {
                canvas[idx + 3] = 0;
            }
        }
    }
    return electron_1.nativeImage.createFromBuffer(canvas, { width: size, height: size });
}
function createTray(opts) {
    const icon = createTrayIcon();
    tray = new electron_1.Tray(icon);
    const contextMenu = electron_1.Menu.buildFromTemplate([
        {
            label: '显示窗口',
            click: () => {
                if ((0, window_manager_1.getMainWindow)()) {
                    (0, window_manager_1.getMainWindow)().show();
                    (0, window_manager_1.getMainWindow)().focus();
                }
                else {
                    opts.createWindow(opts);
                    opts.setupWindowLifecycle();
                }
            }
        },
        { type: 'separator' },
        { label: '退出', click: () => { electron_1.app.isQuitting = true; electron_1.app.quit(); } }
    ]);
    tray.setToolTip('SuperTool');
    tray.setContextMenu(contextMenu);
    tray.on('click', () => {
        if ((0, window_manager_1.getMainWindow)()) {
            if ((0, window_manager_1.getMainWindow)().isVisible())
                (0, window_manager_1.getMainWindow)().focus();
            else {
                (0, window_manager_1.getMainWindow)().show();
                (0, window_manager_1.getMainWindow)().focus();
            }
        }
        else {
            opts.createWindow(opts);
            opts.setupWindowLifecycle();
        }
    });
}
//# sourceMappingURL=tray-manager.js.map