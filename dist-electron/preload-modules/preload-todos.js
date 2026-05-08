"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/** Todos, Tags, Settings, Subtasks, Order, Repeat, Notification API */
const electron_1 = require("electron");
exports.default = {
    // Core app
    getAppPath: () => electron_1.ipcRenderer.invoke('get-app-path'),
    getMenuIcon: (name) => electron_1.ipcRenderer.invoke('get-menu-icon', name),
    getShellEnv: () => electron_1.ipcRenderer.invoke('shell-env:get'),
    // Todos API
    getTodos: () => electron_1.ipcRenderer.invoke('todos:get-all'),
    addTodo: (todo) => electron_1.ipcRenderer.invoke('todos:add', todo),
    updateTodo: (todo) => electron_1.ipcRenderer.invoke('todos:update', todo),
    deleteTodo: (id) => electron_1.ipcRenderer.invoke('todos:delete', id),
    deleteTodos: (ids) => electron_1.ipcRenderer.invoke('todos:delete-many', ids),
    // Tags API
    getTags: () => electron_1.ipcRenderer.invoke('tags:get-all'),
    addTag: (name) => electron_1.ipcRenderer.invoke('tags:add', name),
    deleteTag: (name) => electron_1.ipcRenderer.invoke('tags:delete', name),
    // Settings API
    getSetting: (key) => electron_1.ipcRenderer.invoke('settings:get', key),
    setSetting: (key, value) => electron_1.ipcRenderer.invoke('settings:set', key, value),
    updateShortcuts: (shortcuts) => electron_1.ipcRenderer.invoke('shortcuts:update', shortcuts),
    // Notification API
    getNotificationSettings: () => electron_1.ipcRenderer.invoke('notification:get-settings'),
    setNotificationSettings: (settings) => electron_1.ipcRenderer.invoke('notification:set-settings', settings),
    testNotification: () => electron_1.ipcRenderer.invoke('notification:test'),
    dismissNotifications: (todoId) => electron_1.ipcRenderer.invoke('notifications:dismiss', todoId),
    onNotificationClicked: (callback) => {
        const wrapper = (_event, data) => callback(data);
        electron_1.ipcRenderer.on('notification:clicked', wrapper);
        return () => electron_1.ipcRenderer.removeListener('notification:clicked', wrapper);
    },
    // Subtasks API
    getSubtasksForTodo: (todoId) => electron_1.ipcRenderer.invoke('subtasks:get-for-todo', todoId),
    addSubtask: (subtask) => electron_1.ipcRenderer.invoke('subtasks:add', subtask),
    updateSubtask: (subtask) => electron_1.ipcRenderer.invoke('subtasks:update', subtask),
    deleteSubtask: (subtaskId) => electron_1.ipcRenderer.invoke('subtasks:delete', subtaskId),
    updateTodoCompletionBasedOnSubtasks: (todoId) => electron_1.ipcRenderer.invoke('subtasks:update-todo-completion', todoId),
    // Order API
    updateTodoOrder: (todos) => electron_1.ipcRenderer.invoke('todos:update-order', todos),
    // Repeat API
    createRepeatInstance: (todo) => electron_1.ipcRenderer.invoke('todos:create-repeat-instance', todo),
};
//# sourceMappingURL=preload-todos.js.map