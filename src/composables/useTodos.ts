import { getTauriAPI } from '../utils/tauri-api'
import { ref } from 'vue';
import type { Ref } from 'vue';
import { useErrorHandler } from './useErrorHandler';
import { useTodoSync } from './useTodoSync';
import type {
  Todo,
  Subtask,
  TodoFilterOptions,
} from '../types';

/**
 * useTodos - 统一任务数据库操作 composable
 * 封装所有 Tauri API 的任务相关调用
 * 共享的同步/协作方法委托给 useTodoSync
 */
export function useTodos() {
  const loading: Ref<boolean> = ref(false);
  const error: Ref<string | null> = ref(null);
  const { handleError } = useErrorHandler();

  // 委托给 useTodoSync 获取共享方法
  const todoSync = useTodoSync();

  // ============ 基础 CRUD ============

  const fetchTodos = async (): Promise<Todo[]> => {
    loading.value = true;
    error.value = null;
    try {
      const todos = await getTauriAPI().getTodos();
      return todos || [];
    } catch (err: unknown) {
      error.value = (err as Error).message;
      handleError(err, { context: 'fetchTodos' });
      return [];
    } finally {
      loading.value = false;
    }
  };

  const addTodo = async (todoData: Partial<Todo>): Promise<Todo> => {
    error.value = null;
    try {
      const plainTodo = JSON.parse(JSON.stringify(todoData))
      const saved = await getTauriAPI().addTodo(plainTodo);
      // 广播新任务到局域网
      try {
        await todoSync.broadcastTaskUpdate(saved);
      } catch (broadcastErr) {
        handleError(broadcastErr, { context: 'broadcastTaskUpdate', showToast: false });
      }
      return saved;
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'addTodo', rethrow: true });
      throw err;
    }
  };

  const updateTodo = async (todoData: Todo): Promise<Todo> => {
    error.value = null;
    try {
      const plainTodo = JSON.parse(JSON.stringify(todoData))
      const updated = await getTauriAPI().updateTodo(plainTodo);
      return updated;
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'updateTodo', rethrow: true });
      throw err;
    }
  };

  const deleteTodo = async (id: string): Promise<string> => {
    error.value = null;
    try {
      await getTauriAPI().deleteTodo(id);
      return id;
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'deleteTodo', rethrow: true });
      throw err;
    }
  };

  const deleteTodos = async (ids: string[]): Promise<string[]> => {
    error.value = null;
    try {
      for (const id of ids) {
        await getTauriAPI().deleteTodo(id);
      }
      return ids;
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'deleteTodos', rethrow: true });
      throw err;
    }
  };

  // ============ 排序 ============

  const updateTodoOrder = async (todos: Todo[]): Promise<void> => {
    error.value = null;
    try {
      // ⚠️ 剥离 Vue Proxy
      const orderData = todos.map((todo, index) => ({ id: todo.id, orderNum: index }))
      for (const item of orderData) {
        await getTauriAPI().updateTodo(item)
      }
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'updateTodoOrder', rethrow: true });
      throw err;
    }
  };

  // ============ 重复任务 ============

  const createRepeatInstance = async (todo: Todo): Promise<Todo | undefined> => {
    error.value = null;
    try {
      const plainTodo = JSON.parse(JSON.stringify(todo))
      return await getTauriAPI().createRepeatInstance(plainTodo.id);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'createRepeatInstance', rethrow: true });
      throw err;
    }
  };

  // ============ 子任务 ============

  const getSubtasks = async (todoId: string): Promise<Subtask[]> => {
    error.value = null;
    try {
      return (await getTauriAPI().getSubtasksForTodo(todoId)) || [];
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'getSubtasks' });
      return [];
    }
  };

  const addSubtask = async (subtask: Subtask): Promise<Subtask> => {
    error.value = null;
    try {
      const plainSubtask = JSON.parse(JSON.stringify(subtask))
      return await getTauriAPI().addSubtask(plainSubtask);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'addSubtask', rethrow: true });
      throw err;
    }
  };

  const updateSubtask = async (subtask: Subtask): Promise<Subtask> => {
    error.value = null;
    try {
      const plainSubtask = JSON.parse(JSON.stringify(subtask))
      return await getTauriAPI().updateSubtask(plainSubtask);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'updateSubtask', rethrow: true });
      throw err;
    }
  };

  const deleteSubtask = async (subtaskId: string): Promise<void> => {
    error.value = null;
    try {
      return await getTauriAPI().deleteSubtask(subtaskId);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'deleteSubtask', rethrow: true });
      throw err;
    }
  };

  const updateTodoCompletionBasedOnSubtasks = async (todoId: string): Promise<void> => {
    error.value = null;
    try {
      return await getTauriAPI().updateTodoCompletionBasedOnSubtasks(todoId);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'updateTodoCompletionBasedOnSubtasks', rethrow: true });
      throw err;
    }
  };

  // ============ 标签 ============

  const fetchTags = async (): Promise<string[]> => {
    error.value = null;
    try {
      const tags = await getTauriAPI().getTags();
      return tags || [];
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'fetchTags' });
      return ['工作', '生活', '学习', '其他'];
    }
  };

  const addTag = async (name: string): Promise<string> => {
    error.value = null;
    try {
      const tag = await getTauriAPI().addTag(name);
      return tag.name;
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'addTag', rethrow: true });
      throw err;
    }
  };

  const deleteTag = async (name: string): Promise<void> => {
    error.value = null;
    try {
      return await getTauriAPI().deleteTag(name);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'deleteTag', rethrow: true });
      throw err;
    }
  };

  // ============ 设置 ============

  const getSetting = async (key: string): Promise<string | null> => {
    try {
      return await getTauriAPI().getSetting(key);
    } catch (err) {
      handleError(err, { context: 'getSetting' });
      return null;
    }
  };

  const setSetting = async (key: string, value: string): Promise<{ key: string; value: string } | undefined> => {
    try {
      return await getTauriAPI().setSetting(key, value);
    } catch (err) {
      handleError(err, { context: 'setSetting', rethrow: true });
      throw err;
    }
  };

  // ============ 搜索（带防抖） ============

  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  const debouncedSearchQuery: Ref<string> = ref('');

  const searchWithDebounce = (query: string, callback?: (q: string) => void, delay = 300): void => {
    clearTimeout(searchTimer!);
    searchTimer = setTimeout(() => {
      debouncedSearchQuery.value = query;
      if (callback) {callback(query);}
    }, delay);
  };

  const clearSearchTimer = (): void => {
    clearTimeout(searchTimer!);
  };

  // ============ 数据过滤工具函数 ============

  const filterTodos = (
    todos: Todo[],
    {
      filter = 'all',
      tagFilter = 'all',
      searchQuery = '',
      priorityFilter = 'all',
      statusFilter = 'all',
      sortBy = null,
    }: TodoFilterOptions = {}
  ): Todo[] => {
    let result = [...todos];

    // 状态过滤
    if (statusFilter === 'active') {
      result = result.filter((t) => !t.completed);
    } else if (statusFilter === 'completed') {
      result = result.filter((t) => t.completed);
    }

    // 任务状态过滤（all / active / completed）
    if (filter === 'active') {
      result = result.filter((t) => !t.completed);
    } else if (filter === 'completed') {
      result = result.filter((t) => t.completed);
    }

    // 标签过滤
    if (tagFilter !== 'all') {
      result = result.filter((t) => t.tag === tagFilter);
    }

    // 优先级过滤
    if (priorityFilter !== 'all') {
      result = result.filter((t) => t.priority === priorityFilter);
    }

    // 搜索过滤
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      result = result.filter(
        (t) =>
          t.text.toLowerCase().includes(query) ||
          (t.description && t.description.toLowerCase().includes(query)) ||
          (t.tag && t.tag.toLowerCase().includes(query))
      );
    }

    // 排序
    if (sortBy === 'priority') {
      const priorityOrder: Record<string, number> = { high: 0, medium: 1, low: 2 };
      result.sort((a, b) => {
        const aOrder = a.priority != null && priorityOrder[a.priority] !== undefined
          ? priorityOrder[a.priority]
          : 3;
        const bOrder = b.priority != null && priorityOrder[b.priority] !== undefined
          ? priorityOrder[b.priority]
          : 3;
        return aOrder - bOrder;
      });
    } else if (sortBy === 'dueDate') {
      result.sort((a, b) => {
        const aHasDate = !!a.dueDate;
        const bHasDate = !!b.dueDate;
        if (aHasDate && !bHasDate) {return -1;}
        if (!aHasDate && bHasDate) {return 1;}
        if (aHasDate && bHasDate) {
          return new Date(a.dueDate!).getTime() - new Date(b.dueDate!).getTime();
        }
        return 0;
      });
    } else if (sortBy === 'createdAt') {
      result.sort((a, b) => {
        return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
      });
    }

    return result;
  };

  return {
    // 状态
    loading,
    error,
    debouncedSearchQuery,
    // CRUD
    fetchTodos,
    addTodo,
    updateTodo,
    deleteTodo,
    deleteTodos,
    // 排序
    updateTodoOrder,
    // 重复任务
    createRepeatInstance,
    // 子任务
    getSubtasks,
    addSubtask,
    updateSubtask,
    deleteSubtask,
    updateTodoCompletionBasedOnSubtasks,
    // 标签
    fetchTags,
    addTag,
    deleteTag,
    // 设置
    getSetting,
    setSetting,
    // 搜索
    searchWithDebounce,
    clearSearchTimer,
    // 过滤
    filterTodos,
    // ====== 委托给 useTodoSync 的共享方法 ======
    // 协作广播
    getUserInfo: todoSync.getUserInfo,
    broadcastTaskUpdate: todoSync.broadcastTaskUpdate,
    broadcastTaskComment: todoSync.broadcastTaskComment,
    broadcastCollaborationStarted: todoSync.broadcastCollaborationStarted,
    broadcastCollaborationEnded: todoSync.broadcastCollaborationEnded,
    lanBroadcastCollaborationStarted: (data: string) => getTauriAPI().lanBroadcastCollaborationStarted(data),
    lanBroadcastCollaborationEnded: (data: string) => getTauriAPI().lanBroadcastCollaborationEnded(data),
    // 导入导出
    exportData: todoSync.exportData,
    importJson: todoSync.importJson,
    exportCsv: todoSync.exportCsv,
    exportWordReport: todoSync.exportWordReport,
    // 同步状态
    syncing: todoSync.syncing,
  };
}
