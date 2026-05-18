<template>
  <div class="app-root">
    <router-view />
    <!-- 全局组件 -->
    <ToastContainer />
    <GlobalSearch ref="globalSearchRef" />
    <AboutDialog v-model="showAboutDialog" />
    <QuickSwitch ref="quickSwitchRef" @select="onQuickSwitchSelect" />
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
import { useAppStore } from '@/stores/appStore'
import { useTabStore, KNOWN_ROUTES } from '@/stores/tabStore'

const isDark = ref(false)
const showAboutDialog = ref(false)
const quickSwitchRef = ref<InstanceType<typeof QuickSwitch> | null>(null)
const globalSearchRef = ref<InstanceType<typeof GlobalSearch> | null>(null)
const router = useRouter()
const appStore = useAppStore()
const tabStore = useTabStore()

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
    const routeInfo = KNOWN_ROUTES[path]
    if (routeInfo) {
      tabStore.openOrActivate(path, routeInfo.label, routeInfo.viewId)
    }
    router.push(path)
  }
}

let unlistenFns: (() => void)[] = []

onMounted(async () => {
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
  for (const unlisten of unlistenFns) {
    try { unlisten() } catch {}
  }
})
</script>
