// @ts-nocheck
import { getTauriAPI } from '../utils/tauri-api'
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useErrorHandler } from '../composables/useErrorHandler';
import type { ViewMode } from '../types';
import { getSetting, setSetting } from '../services/database';

// 需要持久化的状态键
const STATE_KEYS = {
  viewMode: 'app_viewMode',
  filter: 'app_filter',
  tagFilter: 'app_tagFilter',
  priorityFilter: 'app_priorityFilter',
  sortBy: 'app_sortBy',
  sidebarCollapsed: 'app_sidebarCollapsed',
} as const;

/**
 * appStore - 应用状态管理
 * 管理暗黑模式、视图模式、侧边栏状态、语言等 UI 状态
 */
export const useAppStore = defineStore('app', () => {
  const { handleError } = useErrorHandler();

  // ============ 状态 ============
  const isDark = ref(false);
  const viewMode = ref<ViewMode>('todo');
  const showLan = ref(false);
  const sidebarCollapsed = ref(false);
  // i18n locale access — use safe getter to avoid tree-shaking issues
function getLocale(): string {
  try {
    const saved = localStorage.getItem('locale');
    if (saved && ['zh-CN', 'en'].includes(saved)) return saved;
  } catch {}
  const lang = typeof navigator !== 'undefined' ? navigator.language : 'zh-CN';
  return lang.startsWith('zh') ? 'zh-CN' : 'en';
}
const locale = ref(getLocale());

  // ============ 操作 ============

  const initTheme = async (): Promise<void> => {
    try {
      if (getTauriAPI()) {
        const savedTheme = await getTauriAPI().getSetting('theme');
        if (savedTheme === 'dark') {
          setDark(true);
          return;
        } else if (savedTheme === 'light') {
          setDark(false);
          return;
        }
      }
      // 降级到 localStorage
      const localTheme = localStorage.getItem('theme');
      if (localTheme === 'dark') {
        setDark(true);
      }
    } catch (err) {
      handleError(err, { context: 'initTheme', showToast: false });
    }
  };

  /**
   * 初始化时从 Tauri API/localStorage 恢复上次的应用状态
   * 包括：viewMode, filter, tagFilter, priorityFilter, sortBy, sidebarCollapsed, locale
   */
  const restoreState = async (): Promise<void> => {
    try {
      // 恢复 viewMode
      const savedViewMode = await getSetting(STATE_KEYS.viewMode);
      if (savedViewMode) {
        viewMode.value = savedViewMode as ViewMode;
      }

      // 恢复 sidebarCollapsed
      const savedSidebar = await getSetting(STATE_KEYS.sidebarCollapsed);
      if (savedSidebar !== null) {
        sidebarCollapsed.value = savedSidebar === 'true';
      }

      // 恢复 locale
      const savedLocale = await getSetting('locale');
      if (savedLocale) {
        setLocale(savedLocale);
      }

      // 恢复 todoStore 的过滤/排序状态（通过 getTauriAPI() 或 localStorage）
      const filterVal = await getSetting(STATE_KEYS.filter);
      const tagFilterVal = await getSetting(STATE_KEYS.tagFilter);
      const priorityFilterVal = await getSetting(STATE_KEYS.priorityFilter);
      const sortByVal = await getSetting(STATE_KEYS.sortBy);

      // 通过自定义事件传递给 TodoList（因为 todoStore 还没加载完时可能无法直接访问）
      // 或者使用 localStorage 作为桥接
      if (filterVal) localStorage.setItem('restore_filter', filterVal);
      if (tagFilterVal) localStorage.setItem('restore_tagFilter', tagFilterVal);
      if (priorityFilterVal) localStorage.setItem('restore_priorityFilter', priorityFilterVal);
      if (sortByVal) localStorage.setItem('restore_sortBy', sortByVal);
    } catch (err) {
      handleError(err, { context: 'restoreState', showToast: false });
    }
  };

  /**
   * 退出时保存当前应用状态
   */
  const saveState = async (): Promise<void> => {
    try {
      const promises = [
        setSetting(STATE_KEYS.viewMode, viewMode.value),
        setSetting(STATE_KEYS.sidebarCollapsed, String(sidebarCollapsed.value)),
        setSetting('locale', locale.value),
      ];

      // 保存 todoStore 的过滤/排序状态
      try {
        const todoStore = window.__todoStore;
        if (todoStore) {
          promises.push(setSetting(STATE_KEYS.filter, todoStore.filter || 'all'));
          promises.push(setSetting(STATE_KEYS.tagFilter, todoStore.tagFilter || 'all'));
          promises.push(setSetting(STATE_KEYS.priorityFilter, todoStore.priorityFilter || 'all'));
          promises.push(setSetting(STATE_KEYS.sortBy, todoStore.sortBy || ''));
        }
      } catch {}

      await Promise.allSettled(promises);
    } catch (err) {
      handleError(err, { context: 'saveState', showToast: false });
    }
  };

  const toggleTheme = async (): Promise<void> => {
    const newDark = !isDark.value;
    setDark(newDark);
    try {
      if (getTauriAPI()) {
        await getTauriAPI().setSetting('theme', newDark ? 'dark' : 'light');
      }
    } catch (err) {
      handleError(err, { context: 'toggleTheme', showToast: false });
    }
  };

  const setDark = (dark: boolean): void => {
    isDark.value = dark;
    if (dark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  };

  const setViewMode = async (mode: ViewMode): Promise<void> => {
    viewMode.value = mode;
    // 实时保存 viewMode
    try {
      await setSetting(STATE_KEYS.viewMode, mode);
    } catch (err) {
      handleError(err, { context: 'setViewMode', showToast: false });
    }
  };

  const toggleLan = (): void => {
    showLan.value = !showLan.value;
  };

  const setLan = (show: boolean): void => {
    showLan.value = show;
  };

  const toggleSidebar = async (): Promise<void> => {
    sidebarCollapsed.value = !sidebarCollapsed.value;
    try {
      await setSetting(STATE_KEYS.sidebarCollapsed, String(sidebarCollapsed.value));
    } catch (err) {
      handleError(err, { context: 'toggleSidebar', showToast: false });
    }
  };

  const setLocale = (loc: string): void => {
    locale.value = loc;
    localStorage.setItem('locale', loc);
    locale.value = loc;
    // Notify Vue components to re-render with new locale
    window.dispatchEvent(new CustomEvent('locale-changed', { detail: loc }));
    try {
      setSetting('locale', loc);
    } catch {}
  };

  const toggleLocale = (): void => {
    setLocale(locale.value === 'zh-CN' ? 'en' : 'zh-CN');
  };

  return {
    // 状态
    isDark,
    viewMode,
    showLan,
    sidebarCollapsed,
    locale,
    // 操作
    initTheme,
    restoreState,
    saveState,
    toggleTheme,
    setDark,
    setViewMode,
    toggleLan,
    setLan,
    toggleSidebar,
    setLocale,
    toggleLocale,
  };
});
