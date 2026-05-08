import type { Todo, Subtask } from './preload-core';
declare const _default: {
    getAppPath: () => Promise<any>;
    getMenuIcon: (name: string) => Promise<any>;
    getShellEnv: () => Promise<any>;
    getTodos: () => Promise<any>;
    addTodo: (todo: Todo) => Promise<any>;
    updateTodo: (todo: Todo) => Promise<any>;
    deleteTodo: (id: string) => Promise<any>;
    deleteTodos: (ids: string[]) => Promise<any>;
    getTags: () => Promise<any>;
    addTag: (name: string) => Promise<any>;
    deleteTag: (name: string) => Promise<any>;
    getSetting: (key: string) => Promise<any>;
    setSetting: (key: string, value: string) => Promise<any>;
    updateShortcuts: (shortcuts: Record<string, string>) => Promise<any>;
    getNotificationSettings: () => Promise<any>;
    setNotificationSettings: (settings: {
        reminderTime: number;
    }) => Promise<any>;
    testNotification: () => Promise<any>;
    dismissNotifications: (todoId: string) => Promise<any>;
    onNotificationClicked: (callback: (data: {
        todoId: string;
    }) => void) => () => Electron.IpcRenderer;
    getSubtasksForTodo: (todoId: string) => Promise<any>;
    addSubtask: (subtask: Subtask) => Promise<any>;
    updateSubtask: (subtask: Subtask) => Promise<any>;
    deleteSubtask: (subtaskId: string) => Promise<any>;
    updateTodoCompletionBasedOnSubtasks: (todoId: string) => Promise<any>;
    updateTodoOrder: (todos: Todo[]) => Promise<any>;
    createRepeatInstance: (todo: Todo) => Promise<any>;
};
export default _default;
