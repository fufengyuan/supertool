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
    </template>
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
import FloatingTodoPanel from '@/components/FloatingTodoPanel.vue'
import { useAppStore } from '@/stores/appStore'

const isFloatingTodo = ref(false)
const isDark = ref(false)
const showAboutDialog = ref(false)
const quickSwitchRef = ref<InstanceType<typeof QuickSwitch> | null>(null)
const globalSearchRef = ref<InstanceType<typeof GlobalSearch> | null>(null)
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
    if (isFloatingTodo.value) return // floating window: skip global setup
  } catch {} // not in Tauri, or webview not available

  // 添加全局双击事件监听
  document.addEventListener('dblclick', onDoubleClick)
  const api = getTauriAPI()
  
  // Check theme
  try {
    const theme = await api.getSetting('theme')
    isDark.value = theme === 'dark'
    document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  } catch {}

  // Menu shortcuts
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
