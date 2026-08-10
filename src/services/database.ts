
// 数据库服务 - 封装 Tauri API 调用
import type { Todo, Tag } from '../types';
import { getTauriAPI } from '../utils/tauri-api'

// 检查是否在 Tauri 环境中
const isTauri = (): boolean => {
  return typeof window !== 'undefined' && !!getTauriAPI();
};

// ============ Todos 操作 ============

export const getTodos = async (): Promise<Todo[]> => {
  if (isTauri()) {
    return await getTauriAPI().getTodos();
  }
  // 降级到 localStorage
  const saved = localStorage.getItem('todos');
  return saved ? JSON.parse(saved) : [];
};

export const addTodo = async (todo: Partial<Todo>): Promise<Todo> => {
  if (isTauri()) {
    return await getTauriAPI().addTodo(todo);
  }
  // 降级到 localStorage
  const todos = await getTodos();
  todos.push(todo as Todo);
  localStorage.setItem('todos', JSON.stringify(todos));
  return todo as Todo;
};

export const updateTodo = async (todo: Todo): Promise<Todo> => {
  if (isTauri()) {
    return await getTauriAPI().updateTodo(todo);
  }
  // 降级到 localStorage
  const todos = await getTodos();
  const index = todos.findIndex((t) => t.id === todo.id);
  if (index !== -1) {
    todos[index] = todo;
    localStorage.setItem('todos', JSON.stringify(todos));
  }
  return todo;
};

export const deleteTodo = async (id: string): Promise<string> => {
  if (isTauri()) {
    await getTauriAPI().deleteTodo(id);
    return id;
  }
  // 降级到 localStorage
  const todos = await getTodos();
  const filtered = todos.filter((t) => t.id !== id);
  localStorage.setItem('todos', JSON.stringify(filtered));
  return id;
};

export const deleteTodos = async (ids: string[]): Promise<string[]> => {
  if (isTauri()) {
    // 后端 delete_todo 单条删除，批量循环调用
    for (const id of ids) {
      await getTauriAPI().deleteTodo(id);
    }
    return ids;
  }
  // 降级到 localStorage
  const todos = await getTodos();
  const filtered = todos.filter((t) => !ids.includes(t.id));
  localStorage.setItem('todos', JSON.stringify(filtered));
  return ids;
};

// ============ Tags 操作 ============

export const getTags = async (): Promise<string[]> => {
  if (isTauri()) {
    return await getTauriAPI().getTags();
  }
  // 降级到 localStorage
  const saved = localStorage.getItem('tags');
  return saved ? JSON.parse(saved) : ['工作', '生活', '学习', '其他'];
};

export const addTag = async (name: string): Promise<Tag> => {
  if (isTauri()) {
    return await getTauriAPI().addTag(name);
  }
  // 降级到 localStorage
  const tags = await getTags();
  if (!tags.includes(name)) {
    tags.push(name);
    localStorage.setItem('tags', JSON.stringify(tags));
  }
  return { name };
};

export const deleteTag = async (name: string): Promise<string> => {
  if (isTauri()) {
    await getTauriAPI().deleteTag(name);
    return name;
  }
  // 降级到 localStorage
  const tags = await getTags();
  const filtered = tags.filter((t) => t !== name);
  localStorage.setItem('tags', JSON.stringify(filtered));
  return name;
};

// ============ Settings 操作 ============

export const getSetting = async (key: string): Promise<string | null> => {
  if (isTauri()) {
    return await getTauriAPI().getSetting(key);
  }
  // 降级到 localStorage
  return localStorage.getItem(key);
};

export const setSetting = async (key: string, value: string): Promise<{ key: string; value: string } | undefined> => {
  if (isTauri()) {
    return await getTauriAPI().setSetting(key, value);
  }
  // 降级到 localStorage
  localStorage.setItem(key, value);
  return { key, value };
};

// ============ 数据迁移 (从 localStorage 到 SQLite) ============

export const migrateFromLocalStorage = async (): Promise<boolean> => {
  if (!isTauri()) {
    return false;
  }

  // 检查是否已迁移
  const migrated = await getSetting('migrated');
  if (migrated === 'true') {
    return false;
  }

  // 迁移 todos
  const todosData = localStorage.getItem('todos');
  if (todosData) {
    const todos: Todo[] = JSON.parse(todosData);
    for (const todo of todos) {
      await addTodo(todo);
    }
  }

  // 迁移 tags
  const tagsData = localStorage.getItem('tags');
  if (tagsData) {
    const tags: string[] = JSON.parse(tagsData);
    for (const tag of tags) {
      if (tag && tag !== '自定义') {
        await addTag(tag);
      }
    }
  }

  // 迁移主题设置
  const theme = localStorage.getItem('theme');
  if (theme) {
    await setSetting('theme', theme);
  }

  // 标记已迁移
  await setSetting('migrated', 'true');

  return true;
};
