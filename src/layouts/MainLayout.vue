<template>
  <div class="h-screen flex flex-col bg-base-200">
    <!-- 顶部标题栏 -->
    <div class="navbar bg-base-100 shadow-sm px-4 flex-none min-h-[2.5rem] h-10">
      <div class="flex-none">
        <label class="btn btn-square btn-ghost btn-sm lg:hidden" @click="sidebarOpen = !sidebarOpen">
          <SvgIcon name="menu" :size="16" />
        </label>
        <button class="btn btn-square btn-ghost btn-sm hidden lg:flex" @click="sidebarCollapsed = !sidebarCollapsed" :title="sidebarCollapsed ? '展开' : '折叠'">
          <SvgIcon name="chevronLeft" :size="16" :class="{ 'rotate-180': sidebarCollapsed }" />
        </button>
      </div>
      <div class="flex-1">
        <span class="btn btn-ghost text-base gap-2 normal-case min-h-0 h-8">
          <SvgIcon name="zap" class="text-primary" :size="16" />
          SuperTool
        </span>
      </div>
      <div class="flex-none gap-2">
        <!-- 搜索按钮 -->
        <button class="btn btn-ghost btn-sm gap-1" @click="openGlobalSearch" title="全局搜索 (Ctrl+K)">
          <SvgIcon name="search" :size="16" />
          <kbd class="kbd kbd-xs hidden sm:inline-flex">Ctrl+K</kbd>
        </button>
        <!-- 局域网 -->
        <button class="btn btn-ghost btn-sm gap-1" @click="toggleLan" :class="{ 'text-primary': showLan }" title="局域网协作">
          <IconNetwork size="16" stroke-width="1.5" />
        </button>
        <!-- 主题切换 -->
        <label class="swap swap-rotate btn btn-ghost btn-circle btn-sm">
          <input type="checkbox" :checked="isDark" @change="toggleTheme" />
          <SvgIcon name="sun" class="swap-off" :size="20" stroke-width="0" />
          <SvgIcon name="moon" class="swap-on" :size="20" stroke-width="0" />
        </label>
      </div>
    </div>

    <div class="flex flex-1 overflow-hidden">
      <!-- 左侧导航栏 -->
      <aside class="flex-none bg-base-100 border-r border-base-300 transition-all duration-200 overflow-y-auto overflow-x-hidden"
             :class="[
               sidebarOpen ? 'w-48' : 'w-0 lg:w-auto',
               sidebarCollapsed ? 'lg:w-16' : 'lg:w-48'
             ]"
             :style="{ minWidth: sidebarCollapsed ? '4rem' : '12rem' }">
        <nav class="menu menu-sm p-2 gap-1 w-full">
          <!-- 业务 -->
          <div class="menu-title px-3" v-show="!sidebarCollapsed">
            <span class="text-xs font-semibold text-base-content/50 uppercase tracking-wider">业务</span>
          </div>
          <li v-for="item in navGroups.business" :key="item.path" class="w-full">
            <router-link :to="item.path" class="gap-3 w-full" active-class="active" @click="onNavClick(item.viewId, item.path)">
              <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="1.5" />
              <span v-show="!sidebarCollapsed">{{ item.label }}</span>
            </router-link>
          </li>

          <!-- 运维 -->
          <div class="menu-title px-3 mt-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-semibold text-base-content/50 uppercase tracking-wider">运维</span>
          </div>
          <li v-for="item in navGroups.ops" :key="item.path" class="w-full">
            <router-link :to="item.path" class="gap-3 w-full" active-class="active" @click="onNavClick(item.viewId, item.path)">
              <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="1.5" />
              <span v-show="!sidebarCollapsed">{{ item.label }}</span>
            </router-link>
          </li>

          <!-- 开发 -->
          <div class="menu-title px-3 mt-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-semibold text-base-content/50 uppercase tracking-wider">开发</span>
          </div>
          <li v-for="item in navGroups.dev" :key="item.path" class="w-full">
            <router-link :to="item.path" class="gap-3 w-full" active-class="active" @click="onNavClick(item.viewId, item.path)">
              <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="1.5" />
              <span v-show="!sidebarCollapsed">{{ item.label }}</span>
            </router-link>
          </li>

          <!-- 安全 -->
          <div class="menu-title px-3 mt-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-semibold text-base-content/50 uppercase tracking-wider">安全</span>
          </div>
          <li v-for="item in navGroups.security" :key="item.path" class="w-full">
            <router-link :to="item.path" class="gap-3 w-full" active-class="active" @click="onNavClick(item.viewId, item.path)">
              <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="1.5" />
              <span v-show="!sidebarCollapsed">{{ item.label }}</span>
            </router-link>
          </li>
        </nav>
      </aside>

      <!-- 移动端遮罩 -->
      <div v-if="sidebarOpen" class="fixed inset-0 bg-black/30 z-30 lg:hidden" @click="sidebarOpen = false"></div>

      <!-- 主内容区 -->
      <main class="flex-1 overflow-y-auto p-4 lg:p-6">
        <router-view />
      </main>

      <!-- 右侧 LAN 面板 -->
      <aside v-if="showLan" class="flex-none w-96 bg-base-100 border-l border-base-300 overflow-y-auto">
        <div class="sticky top-0 bg-base-100 border-b border-base-300 p-3 flex items-center justify-between z-10">
          <h3 class="font-semibold text-base"><IconNetwork size="16" stroke-width="1.5" class="shrink-0 inline-block align-text-bottom" /> 局域网协作</h3>
          <button class="btn btn-ghost btn-sm btn-circle" @click="showLan = false">
            <SvgIcon name="x" size="14" />
          </button>
        </div>
        <div class="p-3">
          <LanUsers @open-chat="onOpenChat" />
          <ChatPanel v-if="chatPeer" :peer="chatPeer" @close="chatPeer = null" class="mt-3" />
        </div>
      </aside>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { getTauriAPI } from '@/utils/tauri-api'
import { useAppStore } from '@/stores/appStore'
import LanUsers from '@/views/lan/LanUsers.vue'
import ChatPanel from '@/components/ChatPanel.vue'

import {
  IconChecklist,
  IconCalendarWeek,
  IconFolder,
  IconCoin,
  IconServer,
  IconRocket,
  IconFileText,
  IconWorld,
  IconDatabase,
  IconTool,
  IconNotebook,
  IconGitBranch,
  IconLock,
  IconShieldLock,
  IconCloudDownload,
  IconTrash,
  IconNetwork,
} from '@tabler/icons-vue'

const iconMap: Record<string, any> = {
  'todo': IconChecklist,
  'weekly-report': IconCalendarWeek,
  'projects': IconFolder,
  'accounting': IconCoin,
  'servers': IconServer,
  'cicd': IconRocket,
  'log-aggregator': IconFileText,
  'nginx': IconWorld,
  'database': IconDatabase,
  'devtools': IconTool,
  'notes': IconNotebook,
  'git': IconGitBranch,
  'mfa': IconLock,
  'vpn': IconShieldLock,
  'data-backup': IconCloudDownload,
  'disk-cleaner': IconTrash,
}

const router = useRouter()
const appStore = useAppStore()

const sidebarOpen = ref(false)
const sidebarCollapsed = ref(false)
const showLan = ref(false)
const isDark = ref(false)
const chatPeer = ref<{ id: string; name: string; avatar?: string; version?: string } | null>(null)
const lanStarted = ref(false)

const navGroups = {
  business: [
    { path: '/', icon: '📝', label: '任务', viewId: 'todo' },
    { path: '/weekly', icon: '📊', label: '周报', viewId: 'weekly-report' },
    { path: '/projects', icon: '📁', label: '项目', viewId: 'projects' },
    { path: '/accounting', icon: '💰', label: '记账本', viewId: 'accounting' },
  ],
  ops: [
    { path: '/servers', icon: '🖥️', label: '服务器', viewId: 'servers' },
    { path: '/cicd', icon: '🚀', label: 'CI/CD', viewId: 'cicd' },
    { path: '/logs', icon: '📋', label: '日志聚合', viewId: 'log-aggregator' },
    { path: '/nginx', icon: '🌐', label: 'Nginx', viewId: 'nginx' },
  ],
  dev: [
    { path: '/database', icon: '🗄️', label: '数据库', viewId: 'database' },
    { path: '/devtools', icon: '🛠️', label: '开发工具', viewId: 'devtools' },
    { path: '/notes', icon: '📓', label: '笔记', viewId: 'notes' },
    { path: '/git', icon: '🔀', label: 'Git 仓库', viewId: 'git' },
  ],
  security: [
    { path: '/mfa', icon: '🔐', label: 'MFA', viewId: 'mfa' },
    { path: '/vpn', icon: '🌐', label: 'VPN', viewId: 'vpn' },
    { path: '/backup', icon: '💾', label: '备份', viewId: 'data-backup' },
    { path: '/disk-cleaner', icon: '🧹', label: '磁盘清理', viewId: 'disk-cleaner' },
  ],
}

function onNavClick(viewId: string, path: string) {
  appStore.recordNavClick(viewId)
  sidebarOpen.value = false
  router.push(path)
}

async function toggleLan() {
  if (!showLan.value && !lanStarted.value) {
    try {
      const api = getTauriAPI()
      const userInfo = await api.lanGetUserInfo().catch(() => ({ id: '', name: '' }))
      const userId = userInfo.id || `user_${crypto.randomUUID().slice(0, 8)}`
      const userName = userInfo.name || 'User'
      await api.startLan(userId, userName)
      lanStarted.value = true
    } catch (e) {
      console.error('[Layout] Failed to start LAN:', e)
    }
  }
  showLan.value = !showLan.value
  if (!showLan.value) chatPeer.value = null
}

function onOpenChat(peer: { id: string; name: string; avatar?: string; version?: string }) {
  chatPeer.value = peer
}

async function toggleTheme() {
  isDark.value = !isDark.value
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  try {
    await getTauriAPI().setSetting('theme', isDark.value ? 'dark' : 'light')
  } catch {}
}

function openGlobalSearch() {
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true }))
}

let unlistenFns: (() => void)[] = []

onMounted(async () => {
  const api = getTauriAPI()
  try {
    const theme = await api.getSetting('theme')
    isDark.value = theme === 'dark'
    document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  } catch {}

  const unlistenNav = await api.onMenuNav((view: string) => {
    const routeMap: Record<string, string> = {
      'todo': '/', 'weekly-report': '/weekly', 'projects': '/projects',
      'accounting': '/accounting', 'servers': '/servers', 'cicd': '/cicd',
      'log-aggregator': '/logs', 'nginx': '/nginx', 'database': '/database', 'devtools': '/devtools',
      'notes': '/notes', 'git': '/git', 'mfa': '/mfa', 'vpn': '/vpn',
      'data-backup': '/backup', 'disk-cleaner': '/disk-cleaner', 'report': '/report', 'settings': '/settings',
    }
    const path = routeMap[view]
    if (path) router.push(path)
  }).catch(() => () => {})
  unlistenFns.push(unlistenNav as () => void)
})

onUnmounted(async () => {
  for (const unlisten of unlistenFns) {
    try { unlisten() } catch {}
  }
})
</script>
