import { ref, computed } from 'vue';
import type { Ref, ComputedRef } from 'vue';
import type { ViewMode } from '../types';

interface LanPeer {
  id: string;
  name: string;
}

interface AppStore {
  sidebarCollapsed: boolean;
  viewMode: ViewMode;
  setViewMode: (mode: ViewMode) => void;
}

interface ProjectStore {
  clearCurrentProject: () => void;
}

/**
 * useAppNavigation - 视图切换、路由逻辑、侧边栏状态
 * 管理 App.vue 中的导航和弹窗状态
 */
export function useAppNavigation(appStore: AppStore, projectStore: ProjectStore) {
  const chatPeer: Ref<LanPeer | null> = ref(null);
  const assignPeer: Ref<LanPeer | null> = ref(null);

  const sidebarCollapsed: ComputedRef<boolean> = computed({
    get: () => appStore.sidebarCollapsed,
    set: (v: boolean) => { appStore.sidebarCollapsed = v; },
  });

  const navigateTo = (mode: ViewMode): void => {
    if (mode === 'projects') projectStore.clearCurrentProject();
    appStore.setViewMode(mode);
  };

  let _lastToggleTime = 0
  const TOGGLE_COOLDOWN = 500 // ms 内对同一视图的重复点击视为误触

  const toggleView = (mode: ViewMode): void => {
    const now = Date.now()
    // 当前已在目标视图 → 切回任务首页（但需要冷却期防误触）
    if (appStore.viewMode === mode) {
      if (now - _lastToggleTime < TOGGLE_COOLDOWN) return // 冷却期内忽略
      _lastToggleTime = now
      appStore.setViewMode('todo')
      return
    }
    _lastToggleTime = now
    appStore.setViewMode(mode)
  };

  const onOpenChat = (peer: LanPeer): void => { chatPeer.value = peer; };
  const onOpenAssign = (peer: LanPeer): void => { assignPeer.value = peer; };
  const closeChat = (): void => { chatPeer.value = null; };
  const closeAssign = (): void => { assignPeer.value = null; };

  const onSelectPeer = (_peer: LanPeer): void => { /* handled elsewhere */ };
  const onTaskAssigned = (_task: unknown): void => { /* handled elsewhere */ };

  return {
    chatPeer,
    assignPeer,
    sidebarCollapsed,
    navigateTo,
    toggleView,
    onOpenChat,
    onOpenAssign,
    closeChat,
    closeAssign,
    onSelectPeer,
    onTaskAssigned,
  };
}
