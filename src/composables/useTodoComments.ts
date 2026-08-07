// @ts-nocheck
import { getTauriAPI } from '../utils/tauri-api'
import { useErrorHandler } from './useErrorHandler';
import type { Comment } from '../types';

/**
 * useTodoComments - 评论相关操作 composable
 * 管理评论的广播和格式化
 */
export function useTodoComments() {
  const { handleError } = useErrorHandler();

  // 评论广播（通过 LAN 同步评论到其他客户端）
  const broadcastComment = async (todoId: string, comment: Comment): Promise<void> => {
    try {
      const plainComment = JSON.parse(JSON.stringify(comment))
      await getTauriAPI().broadcastTaskComment(todoId, plainComment);
    } catch (err) {
      handleError(err, { context: 'broadcastComment', showToast: false });
    }
  };

  // 获取当前用户信息用于评论
  const getCurrentUser = async (): Promise<{ name: string }> => {
    try {
      return await getTauriAPI().getUserInfo();
    } catch (err) {
      handleError(err, { context: 'getCurrentUser', showToast: false });
      return { name: '未知用户' };
    }
  };

  // 格式化评论时间
  const formatCommentTime = (timestamp: string): string => {
    if (!timestamp) {return '';}
    const date = new Date(timestamp);
    return date.toLocaleString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return {
    broadcastComment,
    getCurrentUser,
    formatCommentTime,
  };
}
