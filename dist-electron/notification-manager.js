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
exports.playNotificationSound = playNotificationSound;
exports.dismissNotification = dismissNotification;
exports.isNotified = isNotified;
exports.markNotified = markNotified;
exports.scheduleNotifiedIdsCleanup = scheduleNotifiedIdsCleanup;
exports.checkTaskNotifications = checkTaskNotifications;
exports.startNotificationCheck = startNotificationCheck;
exports.stopNotificationCheck = stopNotificationCheck;
exports.testNotification = testNotification;
const logger_1 = require("./logger");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const window_manager_1 = require("./window-manager");
// ============ 通知音效 ============
function playNotificationSound() {
    try {
        const { exec } = require('child_process');
        if (process.platform === 'darwin') {
            exec('afplay /System/Library/Sounds/Glass.aiff 2>/dev/null');
        }
        else if (process.platform === 'linux') {
            exec('paplay /usr/share/sounds/freedesktop/stereo/bell.oga 2>/dev/null || canberra-gtk-play -i message 2>/dev/null || true');
        }
        else if (process.platform === 'win32') {
            exec('powershell -c [console]::beep(800,300) 2>$null');
        }
    }
    catch { /* 静默失败，不影响通知显示 */ }
}
// ============ 通知去重 ============
const notifiedTodoIds = new Set();
function dismissNotification(todoId) {
    if (todoId)
        notifiedTodoIds.delete(todoId);
    else
        notifiedTodoIds.clear();
}
function isNotified(todoId) {
    return notifiedTodoIds.has(todoId);
}
function markNotified(todoId) {
    notifiedTodoIds.add(todoId);
}
// 每天午夜自动清空通知记录
function scheduleNotifiedIdsCleanup() {
    const now = new Date();
    const tomorrow = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
    const msUntilMidnight = tomorrow.getTime() - now.getTime();
    setTimeout(() => {
        notifiedTodoIds.clear();
        (0, logger_1.info)('[Notification] Cleared notifiedTodoIds at midnight');
        scheduleNotifiedIdsCleanup();
    }, msUntilMidnight);
}
// ============ 通知检查 ============
let notificationTimer;
function checkTaskNotifications(db) {
    if (!(0, window_manager_1.getMainWindow)() || !electron_1.Notification.isSupported())
        return;
    try {
        const todos = db.getAllTodos();
        const now = new Date();
        const reminderTime = parseInt(db.getSetting('reminder_time') || '15');
        const upcomingTodos = todos.filter((todo) => {
            if (todo.completed || !todo.dueDate)
                return false;
            const dueDate = new Date(todo.dueDate);
            const timeDiff = dueDate.getTime() - now.getTime();
            return timeDiff > 0 && timeDiff <= reminderTime * 60 * 1000;
        });
        upcomingTodos.forEach((todo) => {
            if (notifiedTodoIds.has(todo.id))
                return;
            const dueDate = new Date(todo.dueDate);
            const timeDiff = dueDate.getTime() - now.getTime();
            const hoursLeft = Math.floor(timeDiff / (1000 * 60 * 60));
            const minutesLeft = Math.floor((timeDiff % (1000 * 60 * 60)) / (1000 * 60));
            let timeString = '';
            if (hoursLeft > 0) {
                timeString = `${hoursLeft}小时${minutesLeft}分钟后到期`;
            }
            else {
                timeString = `${minutesLeft}分钟后到期`;
            }
            playNotificationSound();
            const notification = new electron_1.Notification({
                title: `任务提醒: ${todo.text}`,
                body: `任务将在${timeString}到期\n优先级: ${todo.priority}\n标签: ${todo.tag || '未分类'}`,
                silent: false
            });
            notification.on('click', () => {
                if ((0, window_manager_1.getMainWindow)()) {
                    if ((0, window_manager_1.getMainWindow)().isMinimized())
                        (0, window_manager_1.getMainWindow)().restore();
                    if (!(0, window_manager_1.getMainWindow)().isVisible())
                        (0, window_manager_1.getMainWindow)().show();
                    (0, window_manager_1.getMainWindow)().focus();
                    (0, window_manager_1.getMainWindow)().webContents.send('notification:clicked', { todoId: todo.id });
                }
            });
            notification.show();
            notifiedTodoIds.add(todo.id);
        });
    }
    catch (error) {
        console.error('Error checking task notifications:', error);
    }
}
function startNotificationCheck(db) {
    checkTaskNotifications(db);
    clearInterval(notificationTimer);
    notificationTimer = setInterval(() => checkTaskNotifications(db), 5 * 60 * 1000);
}
function stopNotificationCheck() {
    if (notificationTimer) {
        clearInterval(notificationTimer);
        notificationTimer = undefined;
    }
}
// ============ 通知测试 ============
function testNotification() {
    try {
        if (electron_1.Notification.isSupported()) {
            playNotificationSound();
            const iconPath = electron_1.app.isPackaged
                ? path.join(process.resourcesPath, 'icons', '512x512.png')
                : path.join(electron_1.app.getAppPath(), 'build', 'icons', '512x512.png');
            const n = new electron_1.Notification({
                title: '测试通知',
                body: '这是一条测试通知，说明通知功能正常工作。',
                icon: fs.existsSync(iconPath) ? iconPath : undefined,
            });
            n.show();
            (0, logger_1.info)('[Notification] Test notification shown.');
            return true;
        }
        console.warn('[Notification] Not supported on this OS');
    }
    catch (e) {
        console.error('[Notification] Test failed:', e);
    }
    return false;
}
//# sourceMappingURL=notification-manager.js.map