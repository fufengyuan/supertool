import { getTauriAPI } from '../utils/tauri-api'
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useTodos } from '../composables/useTodos';
import { useErrorHandler } from '../composables/useErrorHandler';
import type { Todo, SortBy, FilterMode } from '../types';

const MAX_UNDO_SIZE = 50;

/**
 * todoStore - 任务状态管理
 * 管理 todos 数组、加载状态、过滤条件、搜索、撤销/重做等
 */
export const useTodoStore = defineStore('todos', () => {
  const todosApi = useTodos();
  const { filterTodos, createRepeatInstance } = todosApi;
  const { handleError } = useErrorHandler();

  // ============ 状态 ============
  const todos = ref<Todo[]>([]);
  const tags = ref<string[]>(['工作', '生活', '学习', '其他']);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 过滤和搜索状态
  const filter = ref<FilterMode>('all');
  const tagFilter = ref('all');
  const searchQuery = ref('');
  const priorityFilter = ref('all');
  const statusFilter = ref<FilterMode>('all');
  const sortBy = ref<SortBy>(null);

  // 撤销/重做栈
  const undoStack = ref<Todo[][]>([]);
  const redoStack = ref<Todo[][]>([]);

  // ============ 辅助函数 ============

  /** 保存当前状态快照到 undoStack */
  const pushUndoSnapshot = (): void => {
    undoStack.value.push(JSON.parse(JSON.stringify(todos.value)));
    if (undoStack.value.length > MAX_UNDO_SIZE) {
      undoStack.value.shift();
    }
    redoStack.value = [];
  };

  /** 同步本地 todos 到数据库（比较新旧状态差异） */
  const syncTodosToDB = async (oldTodos: Todo[], newTodos: Todo[]): Promise<void> => {
    const oldMap = new Map(oldTodos.map(t => [t.id, t]));
    const newMap = new Map(newTodos.map(t => [t.id, t]));

    // 找出被删除的项（在 old 中但不在 new 中）
    for (const [id] of oldMap) {
      if (!newMap.has(id)) {
        try {
          await todosApi.deleteTodo(id);
        } catch (err) {
          handleError(err, { context: 'undo/redo deleteTodo', showToast: false });
        }
      }
    }

    // 找出新增或修改的项
    for (const [id, newTodo] of newMap) {
      const oldTodo = oldMap.get(id);
      if (!oldTodo) {
        // 新增
        try {
          await todosApi.addTodo(newTodo);
        } catch (err) {
          handleError(err, { context: 'undo/redo addTodo', showToast: false });
        }
      } else if (JSON.stringify(oldTodo) !== JSON.stringify(newTodo)) {
        // 修改
        try {
          await todosApi.updateTodo(newTodo);
        } catch (err) {
          handleError(err, { context: 'undo/redo updateTodo', showToast: false });
        }
      }
    }
  };

  // ============ 计算属性 ============
  const totalCount = computed(() => todos.value.length);
  const activeCount = computed(() => todos.value.filter((t) => !t.completed).length);
  const completedCount = computed(() => todos.value.filter((t) => t.completed).length);

  const filteredTodos = computed(() => {
    return filterTodos(todos.value, {
      filter: filter.value,
      tagFilter: tagFilter.value,
      searchQuery: searchQuery.value,
      priorityFilter: priorityFilter.value,
      statusFilter: statusFilter.value,
      sortBy: sortBy.value,
    });
  });

  const tagCounts = computed(() => {
    const counts: Record<string, number> = {};
    todos.value.forEach((todo) => {
      const tag = todo.tag || '未分类';
      counts[tag] = (counts[tag] || 0) + 1;
    });
    return counts;
  });

  // ============ 操作 ============

  const loadTodos = async (): Promise<void> => {
    loading.value = true;
    error.value = null;
    try {
      todos.value = await todosApi.fetchTodos();
    } catch (err: unknown) {
      error.value = (err as Error).message;
      handleError(err, { context: 'loadTodos' });
    } finally {
      loading.value = false;
    }
  };

  const loadTags = async (): Promise<void> => {
    try {
      const loadedTags = await todosApi.fetchTags();
      if (loadedTags && loadedTags.length > 0) {
        tags.value = loadedTags;
      }
    } catch (err) {
      handleError(err, { context: 'loadTags', showToast: false });
    }
  };

  const addTodo = async (todoData: Partial<Todo>): Promise<Todo | undefined> => {
    try {
      const saved = await todosApi.addTodo(todoData);
      pushUndoSnapshot();
      // 后端返回不完整，合并传入的完整数据
      const fullTodo = { ...todoData, ...saved, id: saved.id } as Todo;
      todos.value.push(fullTodo);
      return fullTodo;
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'addTodo', rethrow: true });
    }
  };

  const updateTodo = async (todoData: Todo): Promise<Todo | undefined> => {
    try {
      // 找到旧版本用于快照
      const oldTodo = todos.value.find(t => t.id === todoData.id);
      const wasCompleted = oldTodo?.completed;
      pushUndoSnapshot();
      const updated = await todosApi.updateTodo(todoData);
      const index = todos.value.findIndex((t) => t.id === todoData.id);
      if (index !== -1) {
        // 后端可能只返回 {id}，使用传入的完整数据更新本地数组
        const fullUpdated = updated.id && !updated.text ? todoData : updated;
        todos.value[index] = fullUpdated;
      }
      // 重复任务触发修复：如果更新导致 completed 变为 true 且有 repeatType，创建下一个实例
      const finalTodo = todos.value.find(t => t.id === todoData.id);
      if (!wasCompleted && finalTodo?.completed && finalTodo.repeatType) {
        try {
          await createRepeatInstance(finalTodo);
        } catch (repeatErr) {
          handleError(repeatErr, { context: 'updateTodo createRepeatInstance', showToast: false });
        }
      }
      return todos.value[index];
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'updateTodo', rethrow: true });
    }
  };

  const deleteTodo = async (id: string): Promise<void> => {
    try {
      pushUndoSnapshot();
      await todosApi.deleteTodo(id);
      todos.value = todos.value.filter((t) => t.id !== id);
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'deleteTodo', rethrow: true });
    }
  };

  const deleteTodos = async (ids: string[]): Promise<void> => {
    try {
      pushUndoSnapshot();
      for (const id of ids) {
        await todosApi.deleteTodo(id);
      }
      todos.value = todos.value.filter((t) => !ids.includes(t.id));
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'deleteTodos', rethrow: true });
    }
  };

  const toggleTodo = async (id: string): Promise<void> => {
    const todo = todos.value.find((t) => t.id === id);
    if (!todo) {return;}

    pushUndoSnapshot();

    const prevCompleted = todo.completed;
    const prevCompletedAt = todo.completedAt;
    const now = new Date().toISOString();

    todo.completed = !prevCompleted;
    todo.completedAt = todo.completed ? now : null;
    todo.updatedAt = now;

    try {
      await todosApi.updateTodo(todo);

      // 局域网任务状态同步
      if (getTauriAPI().syncTaskStatus && (todo.assignedTo || todo.owner)) {
        getTauriAPI().syncTaskStatus(todo.id, !!todo.completed)
      }

      // 如果是重复任务且标记为完成，创建下一个实例
      if (todo.completed && todo.repeatType) {
        await todosApi.createRepeatInstance(todo);
      }
    } catch (err) {
      // 回滚
      todo.completed = prevCompleted;
      todo.completedAt = prevCompletedAt;
      error.value = (err as Error).message;
      handleError(err, { context: 'toggleTodo', rethrow: true });
    }
  };

  const updateTodoOrder = async (orderedTodos: Todo[]): Promise<void> => {
    try {
      for (const item of orderedTodos) {
        await todosApi.updateTodo(item);
      }
    } catch (err) {
      error.value = (err as Error).message;
      handleError(err, { context: 'updateTodoOrder', rethrow: true });
    }
  };

  const addTag = async (name: string): Promise<void> => {
    try {
      await todosApi.addTag(name);
      if (!tags.value.includes(name)) {
        tags.value.push(name);
      }
    } catch (err) {
      handleError(err, { context: 'addTag', rethrow: true });
    }
  };

  const deleteTag = async (name: string): Promise<void> => {
    try {
      await todosApi.deleteTag(name);
      tags.value = tags.value.filter((t) => t !== name);
    } catch (err) {
      handleError(err, { context: 'deleteTag', rethrow: true });
    }
  };

  // ============ 撤销/重做 ============

  const undo = async (): Promise<void> => {
    if (undoStack.value.length === 0) {return;}
    const currentState = JSON.parse(JSON.stringify(todos.value)) as Todo[];
    redoStack.value.push(currentState);
    const prevState = undoStack.value.pop()!;
    todos.value = prevState;
    try {
      await syncTodosToDB(currentState, prevState);
    } catch (err) {
      handleError(err, { context: 'undo', showToast: false });
    }
  };

  const redo = async (): Promise<void> => {
    if (redoStack.value.length === 0) {return;}
    const currentState = JSON.parse(JSON.stringify(todos.value)) as Todo[];
    undoStack.value.push(currentState);
    if (undoStack.value.length > MAX_UNDO_SIZE) {
      undoStack.value.shift();
    }
    const nextState = redoStack.value.pop()!;
    todos.value = nextState;
    try {
      await syncTodosToDB(currentState, nextState);
    } catch (err) {
      handleError(err, { context: 'redo', showToast: false });
    }
  };

  // ============ 设置器 ============

  const setSearchQuery = (query: string): void => {
    searchQuery.value = query;
  };

  const setFilter = (f: FilterMode): void => {
    filter.value = f;
  };

  const setTagFilter = (tag: string): void => {
    tagFilter.value = tag;
  };

  const setPriorityFilter = (priority: string): void => {
    priorityFilter.value = priority;
  };

  const setStatusFilter = (status: FilterMode): void => {
    statusFilter.value = status;
  };

  const setSortBy = (sort: SortBy): void => {
    sortBy.value = sort;
  };

  const resetFilters = (): void => {
    filter.value = 'all';
    tagFilter.value = 'all';
    searchQuery.value = '';
    priorityFilter.value = 'all';
    statusFilter.value = 'all';
    sortBy.value = null;
  };

  const clearCompleted = async (): Promise<void> => {
    const completedIds = todos.value.filter((t) => t.completed).map((t) => t.id);
    if (completedIds.length === 0) {return;}
    await deleteTodos(completedIds);
  };

  return {
    // 状态
    todos,
    tags,
    loading,
    error,
    // 过滤
    filter,
    tagFilter,
    searchQuery,
    priorityFilter,
    statusFilter,
    sortBy,
    // 撤销/重做
    undoStack,
    redoStack,
    // 计算属性
    totalCount,
    activeCount,
    completedCount,
    filteredTodos,
    tagCounts,
    // 操作
    loadTodos,
    loadTags,
    addTodo,
    updateTodo,
    deleteTodo,
    deleteTodos,
    toggleTodo,
    updateTodoOrder,
    addTag,
    deleteTag,
    undo,
    redo,
    setSearchQuery,
    setFilter,
    setTagFilter,
    setPriorityFilter,
    setStatusFilter,
    setSortBy,
    resetFilters,
    clearCompleted,
  };
});
