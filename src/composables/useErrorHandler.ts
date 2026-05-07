/**
 * useErrorHandler.ts — 全局错误处理 composable
 *
 * - handleError(error, context) — 统一错误处理
 * - showErrorToast(error) — 显示错误 Toast
 * - logError(error, context) — 记录错误日志
 * - 错误分类: network / database / validation / unknown
 */
import { useToast } from './useToast';
import { error as logErrorEntry } from '../services/logger';
import type { ErrorCategory, ErrorHandlerOptions } from '../types';

/**
 * 根据错误消息或错误对象分类错误类型
 */
function classifyError(err: unknown): ErrorCategory {
  const msg = (typeof err === 'string' ? err : (err as Error)?.message ?? '').toLowerCase();

  if (
    msg.includes('network') ||
    msg.includes('fetch') ||
    msg.includes('connection') ||
    msg.includes('timeout') ||
    msg.includes('enotfound') ||
    msg.includes('econnrefused') ||
    msg.includes('err_internet_disconnected')
  ) {
    return 'network';
  }

  if (
    msg.includes('database') ||
    msg.includes('sqlite') ||
    msg.includes('sql') ||
    msg.includes('disk') ||
    msg.includes('read-only') ||
    msg.includes('ENOSPC')
  ) {
    return 'database';
  }

  if (
    msg.includes('validation') ||
    msg.includes('invalid') ||
    msg.includes('required') ||
    msg.includes('must be') ||
    msg.includes('should be') ||
    msg.includes('格式') ||
    msg.includes('不能为空') ||
    msg.includes('无效')
  ) {
    return 'validation';
  }

  return 'unknown';
}

/**
 * 获取错误的用户友好消息
 */
function getUserMessage(err: unknown, category: ErrorCategory): string {
  switch (category) {
    case 'network':
      return '网络连接失败，请检查网络设置后重试';
    case 'database':
      return '数据库操作失败，请检查磁盘空间或重启应用';
    case 'validation':
      return typeof err === 'string' ? err : (err as Error)?.message || '输入数据格式不正确';
    default:
      return typeof err === 'string' ? err : (err as Error)?.message || '发生未知错误';
  }
}

/**
 * 获取错误提示 Toast 消息
 */
function getToastMessage(err: unknown, category: ErrorCategory, context: string): string {
  const userMsg = getUserMessage(err, category);
  return context ? `[${context}] ${userMsg}` : userMsg;
}

/**
 * 统一错误处理 composable
 */
export function useErrorHandler() {
  const toast = useToast();

  /**
   * 记录错误日志
   */
  function logError(error: unknown, context: string = ''): ErrorCategory {
    const category = classifyError(error);
    const msg = typeof error === 'string' ? error : (error as Error)?.message ?? String(error);
    logErrorEntry(`[${category}] ${msg}`, context);
    return category;
  }

  /**
   * 显示错误 Toast
   */
  function showErrorToast(error: unknown, context: string = ''): void {
    const category = classifyError(error);
    const message = getToastMessage(error, category, context);
    toast.error(message, 5000);
  }

  /**
   * 统一错误处理 — 同时记录日志和显示 Toast
   */
  function handleError(
    error: unknown,
    { context = '', showToast = true, rethrow = false }: ErrorHandlerOptions = {}
  ): { category: ErrorCategory; userMessage: string } {
    const category = logError(error, context);

    if (showToast) {
      showErrorToast(error, context);
    }

    if (rethrow) {
      throw error;
    }

    return {
      category,
      userMessage: getUserMessage(error, category),
    };
  }

  return {
    handleError,
    showErrorToast,
    logError,
    classifyError,
  };
}

export { classifyError, getUserMessage };
