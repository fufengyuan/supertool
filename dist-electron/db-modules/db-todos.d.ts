import { type Todo } from './db-core';
export declare function getAllTodos(): Todo[];
export declare function addTodo(todo: Todo): Todo;
export declare function getTodoById(id: string): Todo | null;
export declare function updateTodo(todo: Todo): Todo;
export declare function deleteTodo(id: string): string;
export declare function deleteTodos(ids: string[]): string[];
export declare function getAllTags(): string[];
export declare function addTag(name: string): string;
export declare function deleteTag(name: string): string;
export declare function getSetting(key: string): string | null;
export declare function setSetting(key: string, value: string): {
    key: string;
    value: string;
};
