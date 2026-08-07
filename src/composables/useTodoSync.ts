// @ts-nocheck
import { getTauriAPI } from '../utils/tauri-api'
import { ref } from 'vue';
import type { Ref } from 'vue';
import { useErrorHandler } from './useErrorHandler';
import type { Todo, ExportOptions, ImportOptions } from '../types';

/**
 * useTodoSync - 数据同步/导入导出 composable
 * 封装 export/import 和协作广播相关调用
 */
export function useTodoSync() {
  const { handleError } = useErrorHandler();
  const syncing: Ref<boolean> = ref(false);

  // ============ 协作广播 ============

  const broadcastTaskUpdate = async (todo: Todo): Promise<void> => {
    try {
      const plainTodo = JSON.parse(JSON.stringify(todo))
      await getTauriAPI().broadcastTaskUpdate(plainTodo);
    } catch (err) {
      handleError(err, { context: 'broadcastTaskUpdate', showToast: false });
    }
  };

  const broadcastTaskComment = async (todoId: string, comment: unknown): Promise<void> => {
    try {
      const plainComment = JSON.parse(JSON.stringify(comment))
      await getTauriAPI().broadcastTaskComment(todoId, plainComment);
    } catch (err) {
      handleError(err, { context: 'broadcastTaskComment', showToast: false });
    }
  };

  const broadcastCollaborationStarted = async (todoId: string, editorName: string): Promise<void> => {
    try {
      await getTauriAPI().onCollaborationStarted(todoId, editorName);
    } catch (err) {
      handleError(err, { context: 'broadcastCollaborationStarted', showToast: false });
    }
  };

  const broadcastCollaborationEnded = async (todoId: string, editorName: string): Promise<void> => {
    try {
      await getTauriAPI().onCollaborationEnded(todoId, editorName);
    } catch (err) {
      handleError(err, { context: 'broadcastCollaborationEnded', showToast: false });
    }
  };

  // ============ 用户信息 ============

  const getUserInfo = async (): Promise<{ name: string }> => {
    try {
      return await getTauriAPI().getUserInfo();
    } catch (err) {
      handleError(err, { context: 'getUserInfo', showToast: false });
      return { name: '未知用户' };
    }
  };

  // ============ 导入/导出 ============

  const exportData = async (options: ExportOptions): Promise<unknown> => {
    syncing.value = true;
    try {
      return await getTauriAPI().exportData(options);
    } catch (err) {
      handleError(err, { context: 'exportData' });
      throw err;
    } finally {
      syncing.value = false;
    }
  };

  const importJson = async (options: ImportOptions): Promise<unknown> => {
    syncing.value = true;
    try {
      return await getTauriAPI().importJson(options);
    } catch (err) {
      handleError(err, { context: 'importJson' });
      throw err;
    } finally {
      syncing.value = false;
    }
  };

  const exportCsv = async (options: ExportOptions): Promise<unknown> => {
    syncing.value = true;
    try {
      return await getTauriAPI().exportCsv(options);
    } catch (err) {
      handleError(err, { context: 'exportCsv' });
      throw err;
    } finally {
      syncing.value = false;
    }
  };

  const exportWordReport = async (reportData: unknown): Promise<unknown> => {
    try {
      return await getTauriAPI().exportWordReport(reportData);
    } catch (err) {
      handleError(err, { context: 'exportWordReport' });
      throw err;
    }
  };

  return {
    syncing,
    // 协作广播
    broadcastTaskUpdate,
    broadcastTaskComment,
    broadcastCollaborationStarted,
    broadcastCollaborationEnded,
    // 用户信息
    getUserInfo,
    // 导入导出
    exportData,
    importJson,
    exportCsv,
    exportWordReport,
  };
}
