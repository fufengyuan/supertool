// @ts-nocheck
import { getTauriAPI } from '../utils/tauri-api'
import { ref } from 'vue';
import type { Ref } from 'vue';
import { useErrorHandler } from './useErrorHandler';
import type { Todo, Comment } from '../types';

interface TodosApi {
  getUserInfo: () => Promise<{ name: string }>;
  broadcastTaskComment: (todoId: string, comment: Comment) => Promise<void>;
  broadcastCollaborationStarted: (todoId: string, editorName: string) => Promise<void>;
  broadcastCollaborationEnded: (todoId: string, editorName: string) => Promise<void>;
}

interface TodoStore {
  todos: Ref<Todo[]>;
}

/**
 * useTodoCollaboration - 抽取协作、LAN 事件监听逻辑
 * 管理协作编辑状态、评论、局域网广播
 */
export function useTodoCollaboration(todosApi: TodosApi) {
  const { handleError } = useErrorHandler();
  const collaboratingUsers: Ref<Record<string, string>> = ref<any>({});
  const taskComments: Ref<Record<string, Comment[]>> = ref<any>({});
  const commentInputs: Ref<Record<string, string>> = ref<any>({});

  const addComment = async (todoId: string): Promise<void> => {
    console.log("[useTodoCollaboration.ts] addComment() called")
    const content = commentInputs.value[todoId];
    if (!content || !content.trim()) {return;}

    try {
      const userInfo = await todosApi.getUserInfo();

      const newComment: Comment = {
        id: crypto.randomUUID(),
        author: userInfo.name,
        content: content.trim(),
        timestamp: new Date().toISOString(),
      };

      if (!taskComments.value[todoId]) {
        taskComments.value[todoId] = [];
      }
      taskComments.value[todoId].push(newComment);

      commentInputs.value[todoId] = '';

      await todosApi.broadcastTaskComment(todoId, newComment);
    } catch (error) {
      handleError(error, { context: 'addComment' });
    }
  };

  const startCollaborationEdit = async (todo: Todo): Promise<void> => {
    console.log("[useTodoCollaboration.ts] startCollaborationEdit() called")
    try {
      const userInfo = await todosApi.getUserInfo();

      collaboratingUsers.value[todo.id] = userInfo.name;
      await todosApi.onCollaborationStarted(todo.id, userInfo.name);
    } catch (error) {
      handleError(error, { context: 'startCollaborationEdit' });
    }
  };

  const endCollaborationEdit = async (todoId: string): Promise<void> => {
    console.log("[useTodoCollaboration.ts] endCollaborationEdit() called")
    try {
      const userInfo = await todosApi.getUserInfo();

      delete collaboratingUsers.value[todoId];

      await todosApi.onCollaborationEnded(todoId, userInfo.name);
    } catch (error) {
      handleError(error, { context: 'endCollaborationEdit' });
    }
  };

  const handleMarkdownDoubleClick = async (todo: Todo, onEdit: (todo: Todo) => void): Promise<void> => {
    console.log("[useTodoCollaboration.ts] handleMarkdownDoubleClick() called")
    const currentUser = await todosApi.getUserInfo();
    if (collaboratingUsers.value[todo.id] && collaboratingUsers.value[todo.id] !== currentUser.name) {
      if (confirm(`${collaboratingUsers.value[todo.id]} 正在编辑此任务，是否要覆盖其编辑？`)) {
        await startCollaborationEdit(todo);
        onEdit(todo);
      }
    } else {
      await startCollaborationEdit(todo);
      onEdit(todo);
    }
  };

  const setupLanListeners = (todoStore: TodoStore): void => {
    console.log("[useTodoCollaboration.ts] setupLanListeners() called")
    getTauriAPI().onTaskUpdated((data) => {
      const todoIndex = todoStore.todos.value.findIndex(t => t.id === data.todo.id);
      if (todoIndex !== -1) {
        todoStore.todos.value[todoIndex] = { ...data.todo };
      }
    });

    getTauriAPI().onTaskStatusChanged((data) => {
      const todoIndex = todoStore.todos.value.findIndex(t => t.id === data.todo.id);
      if (todoIndex !== -1) {
        todoStore.todos.value[todoIndex] = { ...data.todo };
      }
    });

    getTauriAPI().onTaskCommentAdded((data) => {
      if (!taskComments.value[data.todoId]) {
        taskComments.value[data.todoId] = [];
      }
      taskComments.value[data.todoId].push(data.comment as Comment);
    });

    getTauriAPI().onCollaborationStarted((data) => {
      collaboratingUsers.value[data.todoId] = data.editorName;
    });

    getTauriAPI().onCollaborationEnded((data) => {
      delete collaboratingUsers.value[data.todoId];
    });
  };

  return {
    collaboratingUsers,
    taskComments,
    commentInputs,
    addComment,
    startCollaborationEdit,
    endCollaborationEdit,
    handleMarkdownDoubleClick,
    setupLanListeners,
  };
}
