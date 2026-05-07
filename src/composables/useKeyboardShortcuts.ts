import { onMounted, onUnmounted } from 'vue';
import type { ViewMode } from '../types';

interface KeyboardShortcutsOptions {
  focusNewTask?: () => void;
  focusSearch?: () => void;
  toggleSidebar?: () => void;
  toggleTheme?: () => void;
  setViewMode?: (mode: ViewMode) => void;
}

/**
 * useKeyboardShortcuts - 全局键盘快捷键管理
 *
 * 快捷键列表:
 *   Ctrl/Cmd+N  — 聚焦新建任务输入
 *   Ctrl/Cmd+F  — 聚焦搜索框
 *   Ctrl/Cmd+B  — 切换侧边栏折叠
 *   Ctrl/Cmd+D  — 切换暗黑模式
 *   Ctrl/Cmd+1~9 — 快速切换导航视图
 *   Escape      — 关闭模态框/取消编辑
 */
export function useKeyboardShortcuts(options: KeyboardShortcutsOptions = {}): void {
  const { focusNewTask, focusSearch, toggleSidebar, toggleTheme, setViewMode } = options;

  // 导航视图映射: Ctrl+1 -> 'todo', Ctrl+2 -> 'weekly-report', ...
  const viewModeMap: ViewMode[] = [
    'todo', // 1
    'weekly-report', // 2
    'projects', // 3
    'servers', // 4
    'data-backup', // 5
    'notifications', // 6
  ];

  /**
   * 判断是否应该忽略快捷键（在输入框内时）
   * 但 Ctrl+A / Ctrl+C / Ctrl+V / Ctrl+X 等常用编辑快捷键例外
   */
  function shouldIgnoreInInput(_key: string): boolean {
    const target = (event as KeyboardEvent)?.target;
    const tag = (target as HTMLElement)?.tagName;
    if (!tag) return false;
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (target as HTMLElement)?.isContentEditable;
    // 在输入框内仍然允许 Escape 和 Ctrl+F
    if (!isInput) return false;
    // 在输入框内时，只允许 Escape
    return true;
  }

  function handleKeyDown(e: KeyboardEvent): void {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    const mod = isMac ? e.metaKey : e.ctrlKey;

    if (!mod && e.key !== 'Escape') return;

    const key = e.key;

    // === Escape: 关闭模态框/取消编辑 ===
    if (key === 'Escape') {
      // 查找并点击最近的关闭按钮
      const modalOverlay = document.querySelector('.form-modal-overlay');
      if (modalOverlay) {
        e.preventDefault();
        const closeBtn = modalOverlay.querySelector(
          '.form-modal-close, .close-btn, [class*="close"]'
        ) as HTMLElement;
        if (closeBtn) closeBtn.click();
        return;
      }
      // 也尝试关闭聊天面板、右侧面板等
      const chatClose = document.querySelector('.chat-panel .close-btn') as HTMLElement;
      if (chatClose) {
        e.preventDefault();
        chatClose.click();
        return;
      }
      return;
    }

    if (!mod) return;

    // === 在输入框内时忽略大部分快捷键 ===
    if (shouldIgnoreInInput(key)) return;

    switch (key) {
      // Ctrl/Cmd+N: 聚焦新建任务输入
      case 'n':
      case 'N':
        e.preventDefault();
        focusNewTask?.();
        break;

      // Ctrl/Cmd+F: 聚焦搜索框
      case 'f':
      case 'F':
        e.preventDefault();
        focusSearch?.();
        break;

      // Ctrl/Cmd+B: 切换侧边栏折叠
      case 'b':
      case 'B':
        e.preventDefault();
        toggleSidebar?.();
        break;

      // Ctrl/Cmd+D: 切换暗黑模式
      case 'd':
      case 'D':
        e.preventDefault();
        toggleTheme?.();
        break;

      // Ctrl/Cmd+1~9: 快速切换导航视图
      case '1':
      case '2':
      case '3':
      case '4':
      case '5':
      case '6':
      case '7':
      case '8':
      case '9': {
        const index = parseInt(key) - 1;
        if (index < viewModeMap.length) {
          e.preventDefault();
          setViewMode?.(viewModeMap[index]);
        }
        break;
      }
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown);
  });
}
