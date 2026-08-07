import { ref } from 'vue';
import type { Ref } from 'vue';
import { useErrorHandler } from './useErrorHandler';
import type { Todo } from '../types';

/**
 * useTodoBatch - 抽取批量操作（批量删除、批量完成）和选择逻辑
 */
export function useTodoBatch(
  todoStore: { todos: Ref<Todo[]>; deleteTodos: (ids: string[]) => Promise<void> },
  todosApi: { updateTodo: (todo: Todo) => Promise<Todo> }
) {
  const { handleError } = useErrorHandler();
  const selectedTodos: Ref<string[]> = ref<any[]>([]);

  const toggleSelected = (id: string): void => {
    const index = selectedTodos.value.indexOf(id);
    if (index > -1) {
      selectedTodos.value.splice(index, 1);
    } else {
      selectedTodos.value.push(id);
    }
  };

  const batchComplete = async (): Promise<void> => {
    const selectedIds = [...selectedTodos.value];
    const snapshots: Array<{ id: string; completed: boolean; completedAt: string | null; updatedAt: string }> = [];
    const updates: Promise<Todo>[] = [];
    selectedIds.forEach(id => {
      const todo = todoStore.todos.value.find(t => t.id === id);
      if (todo && !todo.completed) {
        snapshots.push({ id, completed: todo.completed, completedAt: todo.completedAt, updatedAt: todo.updatedAt });
        todo.completed = true;
        todo.completedAt = new Date().toISOString();
        todo.updatedAt = new Date().toISOString();
        updates.push(todosApi.updateTodo(todo));
      }
    });
    try {
      await Promise.all(updates);
    } catch (error) {
      handleError(error, { context: 'batchComplete' });
      snapshots.forEach(s => {
        const todo = todoStore.todos.value.find(t => t.id === s.id);
        if (todo) {
          todo.completed = s.completed;
          todo.completedAt = s.completedAt;
          todo.updatedAt = s.updatedAt;
        }
      });
    }
    selectedTodos.value = [];
  };

  const batchDelete = async (): Promise<void> => {
    const ids = [...selectedTodos.value];
    if (ids.length === 0) {return;}
    const snapshot = todoStore.todos.value.filter(t => ids.includes(t.id));
    try {
      await todoStore.deleteTodos(ids);
    } catch (error) {
      handleError(error, { context: 'batchDelete' });
      snapshot.forEach(s => {
        if (!todoStore.todos.value.find(t => t.id === s.id)) {
          todoStore.todos.value.push(s);
        }
      });
    }
    selectedTodos.value = [];
  };

  const selectAll = (): void => {
    selectedTodos.value = todoStore.todos.value.map(todo => todo.id);
  };

  return {
    selectedTodos,
    toggleSelected,
    batchComplete,
    batchDelete,
    selectAll,
  };
}
