import { ref } from 'vue';
import type { Ref } from 'vue';
import type { ToastItem, ToastType } from '../types';

/**
 * useToast - 全局 Toast 通知管理 composable
 * 最多同时显示 5 个 toast，支持自动超时关闭
 */

const MAX_TOASTS = 5;
const DEFAULT_DURATION = 3000;

const toasts: Ref<ToastItem[]> = ref<any[]>([]);
let nextId = 1;

export const toast = { success: (_m: string) => {}, error: (_m: string) => {}, warning: (_m: string) => {}, info: (_m: string) => {} }

export function useToast() {
  /**
   * 添加一个 toast 通知
   */
  const addToast = ({ message, type = 'info' as ToastType, duration = DEFAULT_DURATION }: {
    message: string;
    type?: ToastType;
    duration?: number;
  }): number => {
    // 超过最大数量时移除最早的
    if (toasts.value.length >= MAX_TOASTS) {
      toasts.value.shift();
    }

    const id = nextId++;
    const toast: ToastItem = { id, message, type, duration, progress: 100 };
    toasts.value.push(toast);

    // 自动关闭
    if (duration > 0) {
      const startTime = Date.now();
      const timer = setInterval(() => {
        const elapsed = Date.now() - startTime;
        const remaining = 1 - elapsed / duration;
        const t = toasts.value.find((t) => t.id === id);
        if (t) {
          t.progress = Math.max(0, remaining * 100);
        }
        if (elapsed >= duration) {
          clearInterval(timer);
          removeToast(id);
        }
      }, 50);
      toast._timer = timer;
    }

    return id;
  };

  /**
   * 手动移除一个 toast
   */
  const removeToast = (id: number): void => {
    const index = toasts.value.findIndex((t) => t.id === id);
    if (index !== -1) {
      const toast = toasts.value[index];
      if (toast._timer) clearInterval(toast._timer);
      toasts.value.splice(index, 1);
    }
  };

  /**
   * 快捷方法
   */
  const success = (message: string, duration?: number): number => addToast({ message, type: 'success', duration });
  const error = (message: string, duration?: number): number => addToast({ message, type: 'error', duration });
  const warning = (message: string, duration?: number): number => addToast({ message, type: 'warning', duration });
  const info = (message: string, duration?: number): number => addToast({ message, type: 'info', duration });

  /**
   * 清空所有 toast (主要用于测试)
   */
  const clearToasts = (): void => {
    toasts.value.forEach((t) => {
      if (t._timer) clearInterval(t._timer);
    });
    toasts.value = [];
    nextId = 1;
  };

  return {
    toasts,
    addToast,
    removeToast,
    success,
    error,
    warning,
    info,
    clearToasts,
  };
}
