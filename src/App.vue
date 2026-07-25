<template>
  <div class="app-root">
    <FloatingTodoPanel v-if="isFloatingTodo" />
    <template v-else>
      <router-view />
      <!-- 全局组件 -->
      <ToastContainer />
      <GlobalSearch ref="globalSearchRef" />
      <AboutDialog v-model="showAboutDialog" />
      <QuickSwitch ref="quickSwitchRef" @select="onQuickSwitchSelect" />
      <ToolCommandPalette ref="toolPaletteRef" />
    </template>
    <!-- 启动过渡页：等 webview 初始化（菜单监听器注册等）完成后淡出 -->
    <Transition name="splash-fade">
      <div v-if="showSplash" class="app-splash">
        <div class="app-splash__icon">⚡</div>
        <div class="app-splash__title">SuperTool</div>
        <div class="app-splash__loader">
          <span></span><span></span><span></span>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { getTauriAPI } from '@/utils/tauri-api'
import ToastContainer from '@/components/ui/ToastContainer.vue'
import GlobalSearch from '@/components/GlobalSearch.vue'
import AboutDialog from '@/components/AboutDialog.vue'
import QuickSwitch from '@/components/QuickSwitch.vue'
import ToolCommandPalette from '@/components/ToolCommandPalette.vue'
import FloatingTodoPanel from '@/components/FloatingTodoPanel.vue'
import { useAppStore } from '@/stores/appStore'
import { useLanStore } from '@/stores/lanStore'

const isFloatingTodo = ref(false)
const isDark = ref(false)
const showAboutDialog = ref(false)
// 启动过渡页：floating-todo 窗口不显示
const showSplash = ref(true)
const quickSwitchRef = ref<InstanceType<typeof QuickSwitch> | null>(null)
const globalSearchRef = ref<InstanceType<typeof GlobalSearch> | null>(null)
const toolPaletteRef = ref<InstanceType<typeof ToolCommandPalette> | null>(null)
const router = useRouter()
const appStore = useAppStore()

async function onQuickSwitchSelect(viewId: string) {
  // Navigate via router
  const routeMap: Record<string, string> = {
    'todo': '/',
    'weekly-report': '/weekly',
    'projects': '/projects',
    'accounting': '/accounting',
    'servers': '/servers',
    'cicd': '/cicd',
    'log-aggregator': '/logs',
    'nginx': '/nginx',
    'database': '/database',
    'devtools': '/devtools',
    'notes': '/notes',
    'git': '/git',
    'mfa': '/mfa',
    'vpn': '/vpn',
    'data-backup': '/backup',
    'report': '/report',
    'settings': '/settings',
  }
  const path = routeMap[viewId]
  if (path) {
    router.push(path)
  }
}

let unlistenFns: (() => void)[] = []

// 阻止双击选中文本（桌面应用风格），但保留滑动选中功能
function onDoubleClick(e: MouseEvent) {
  // 排除输入框、编辑器、代码块等需要双击选中的元素
  const target = e.target as HTMLElement
  const isEditable = target.tagName === 'INPUT' || 
                     target.tagName === 'TEXTAREA' || 
                     target.isContentEditable ||
                     target.closest('[contenteditable]') ||
                     target.closest('.allow-select') ||
                     target.closest('code') ||
                     target.closest('pre') ||
                     target.closest('.monaco-editor')
  if (!isEditable) {
    // 清除双击产生的选中
    const sel = window.getSelection()
    if (sel) {sel.removeAllRanges()}
  }
}

onMounted(async () => {
  // Detect if this is the floating-todo window
  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    const label = getCurrentWebviewWindow().label
    isFloatingTodo.value = label === 'floating-todo'
    if (isFloatingTodo.value) {
      showSplash.value = false // floating window: skip splash
      return // floating window: skip global setup
    }
  } catch {} // not in Tauri, or webview not available

  // 添加全局双击事件监听
  document.addEventListener('dblclick', onDoubleClick)

  // Cmd+K / Ctrl+K 唤起开发工具搜索弹窗
  function onCmdK(e: KeyboardEvent) {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
    const mod = isMac ? e.metaKey : e.ctrlKey
    if (mod && e.key === 'k') {
      e.preventDefault()
      toolPaletteRef.value?.open()
    }
  }
  document.addEventListener('keydown', onCmdK)
  unlistenFns.push(() => document.removeEventListener('keydown', onCmdK))

  const api = getTauriAPI()

  // Initialize LAN message store (persistent listener across navigation)
  try {
    useLanStore().init()
  } catch { /* lan store may fail if not available */ }

  // Check theme
  try {
    const theme = await api.getSetting('theme')
    isDark.value = theme === 'dark'
    document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  } catch {}

  // Menu shortcuts — 等这些监听器注册完成，菜单点击才会响应
  const unlistenAbout = await api.onMenuAbout(() => {
    showAboutDialog.value = true
  }).catch(() => () => {})
  unlistenFns.push(unlistenAbout as () => void)

  const unlistenToggleTheme = await api.onMenuToggleTheme(async () => {
    isDark.value = !isDark.value
    document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  }).catch(() => () => {})
  unlistenFns.push(unlistenToggleTheme as () => void)

  const unlistenSearch = await api.onMenuSearch(() => {
    globalSearchRef.value?.open()
  }).catch(() => () => {})
  unlistenFns.push(unlistenSearch as () => void)

  // 监听器已注册完成，淡出 splash
  // 保留最小显示时间避免闪烁
  setTimeout(() => { showSplash.value = false }, 300)

  // Initialize frequent menu with stored click data
  setTimeout(() => appStore.updateNativeFrequentMenu(), 500)
})

onUnmounted(async () => {
  document.removeEventListener('dblclick', onDoubleClick)
  for (const unlisten of unlistenFns) {
    try { unlisten() } catch {}
  }
})
</script>

<style scoped>
.app-splash {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: #1a1a2e;
  color: #fff;
  user-select: none;
  pointer-events: all;
}
:global([data-theme='light']) .app-splash {
  background: #f5f5f7;
  color: #1a1a2e;
}
.app-splash__icon {
  font-size: 64px;
  line-height: 1;
  margin-bottom: 16px;
  animation: app-splash-pulse 1.4s ease-in-out infinite;
}
.app-splash__title {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 1px;
  margin-bottom: 18px;
}
.app-splash__loader {
  display: flex;
  gap: 6px;
}
.app-splash__loader span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.4;
  animation: app-splash-bounce 1.2s ease-in-out infinite;
}
.app-splash__loader span:nth-child(2) { animation-delay: 0.15s; }
.app-splash__loader span:nth-child(3) { animation-delay: 0.3s; }

@keyframes app-splash-pulse {
  0%, 100% { transform: scale(1); opacity: 0.85; }
  50% { transform: scale(1.12); opacity: 1; }
}
@keyframes app-splash-bounce {
  0%, 100% { transform: translateY(0); opacity: 0.4; }
  50% { transform: translateY(-6px); opacity: 1; }
}

.splash-fade-leave-active {
  transition: opacity 0.4s ease, visibility 0.4s ease;
}
.splash-fade-leave-to {
  opacity: 0;
  visibility: hidden;
}
</style>
