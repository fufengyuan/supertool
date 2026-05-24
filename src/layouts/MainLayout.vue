<template>
  <div class="h-screen flex flex-col bg-base-200">
    <!-- 无自定义标题栏 — 使用原生窗口边框 -->
    <div class="flex flex-1 overflow-hidden">
      <!-- 左侧导航栏 -->
      <aside class="sidebar-scrollbar flex-none bg-base-100 border-r border-base-300 transition-all duration-200 overflow-y-auto overflow-x-hidden flex flex-col"
             :class="[sidebarCollapsed ? 'w-16' : 'w-48']">
        <!-- 折叠按钮 (侧栏顶部) -->
        <div class="flex items-center justify-between p-2 border-b border-base-300">
          <span v-show="!sidebarCollapsed" class="text-sm font-bold text-base-content pl-1 truncate">SuperTool</span>
          <button class="flex items-center justify-center w-8 h-8 rounded-lg hover:bg-base-200 transition-colors" @click="sidebarCollapsed = !sidebarCollapsed" :title="sidebarCollapsed ? '展开侧栏' : '折叠侧栏'">
            <SvgIcon name="chevronLeft" :size="18" :class="{ 'rotate-180': sidebarCollapsed }" />
          </button>
        </div>

        <!-- 导航菜单 -->
        <nav class="flex-1 overflow-y-auto flex flex-col items-stretch p-2">
          <!-- 业务 -->
          <div class="py-1.5 mt-1 pl-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-bold text-base-content uppercase tracking-wider">业务</span>
          </div>
          <router-link v-for="item in navGroups.business" :key="item.path"
            :to="item.path"
            class="flex items-center gap-3 py-2 pl-5 pr-3 rounded-lg hover:bg-base-200 transition-colors cursor-pointer"
            :class="[$route.path === item.path ? 'bg-base-200 font-bold text-base-content' : 'text-base-content']"
            @click="onNavClick(item.viewId, item.path)">
            <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="2" />
            <span v-show="!sidebarCollapsed" class="text-sm font-medium">{{ item.label }}</span>
          </router-link>

          <!-- 运维 -->
          <div class="py-1.5 mt-2 pl-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-bold text-base-content uppercase tracking-wider">运维</span>
          </div>
          <router-link v-for="item in navGroups.ops" :key="item.path"
            :to="item.path"
            class="flex items-center gap-3 py-2 pl-4 pr-3 rounded-lg hover:bg-base-200 transition-colors cursor-pointer"
            :class="[$route.path === item.path ? 'bg-base-200 font-bold text-base-content' : 'text-base-content']"
            @click="onNavClick(item.viewId, item.path)">
            <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="2" />
            <span v-show="!sidebarCollapsed" class="text-sm font-medium">{{ item.label }}</span>
          </router-link>

          <!-- Agent -->
          <div class="py-1.5 mt-2 pl-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-bold text-base-content uppercase tracking-wider">Agent</span>
          </div>
          <router-link v-for="item in navGroups.agent" :key="item.path"
            :to="item.path"
            class="flex items-center gap-3 py-2 pl-4 pr-3 rounded-lg hover:bg-base-200 transition-colors cursor-pointer"
            :class="[$route.path === item.path ? 'bg-base-200 font-bold text-base-content' : 'text-base-content']"
            @click="onNavClick(item.viewId, item.path)">
            <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="2" />
            <span v-show="!sidebarCollapsed" class="text-sm font-medium">{{ item.label }}</span>
          </router-link>

          <!-- 开发 -->
          <div class="py-1.5 mt-2 pl-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-bold text-base-content uppercase tracking-wider">开发</span>
          </div>
          <router-link v-for="item in navGroups.dev" :key="item.path"
            :to="item.path"
            class="flex items-center gap-3 py-2 pl-4 pr-3 rounded-lg hover:bg-base-200 transition-colors cursor-pointer"
            :class="[$route.path === item.path ? 'bg-base-200 font-bold text-base-content' : 'text-base-content']"
            @click="onNavClick(item.viewId, item.path)">
            <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="2" />
            <span v-show="!sidebarCollapsed" class="text-sm font-medium">{{ item.label }}</span>
          </router-link>

          <!-- 安全 -->
          <div class="py-1.5 mt-2 pl-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-bold text-base-content uppercase tracking-wider">安全</span>
          </div>
          <router-link v-for="item in navGroups.security" :key="item.path"
            :to="item.path"
            class="flex items-center gap-3 py-2 pl-4 pr-3 rounded-lg hover:bg-base-200 transition-colors cursor-pointer"
            :class="[$route.path === item.path ? 'bg-base-200 font-bold text-base-content' : 'text-base-content']"
            @click="onNavClick(item.viewId, item.path)">
            <component :is="iconMap[item.viewId]" size="18" class="shrink-0" stroke-width="2" />
            <span v-show="!sidebarCollapsed" class="text-sm font-medium">{{ item.label }}</span>
          </router-link>
        </nav>

        <!-- 侧栏底部操作区 -->
        <div class="border-t border-base-300 p-2 flex flex-col gap-1">
          <!-- 搜索 -->
          <button class="flex items-center justify-center gap-2.5 py-2 w-full rounded-lg hover:bg-base-200 transition-colors" @click="openGlobalSearch" :title="'全局搜索 (Ctrl+K)'">
            <SvgIcon name="search" :size="20" />
            <span v-show="!sidebarCollapsed" class="text-sm">搜索</span>
          </button>
          <!-- 局域网 -->
          <button class="flex items-center justify-center gap-2.5 py-2 w-full rounded-lg hover:bg-base-200 transition-colors" @click="toggleLan" :class="{ 'text-primary': showLan }" title="局域网协作">
            <IconNetwork size="20" stroke-width="1.5" />
            <span v-show="!sidebarCollapsed" class="text-sm">局域网</span>
          </button>
          <!-- 主题切换 -->
          <button class="flex items-center justify-center gap-2.5 py-2 w-full rounded-lg hover:bg-base-200 transition-colors" @click="toggleTheme" title="切换主题">
            <SvgIcon :name="isDark ? 'sun' : 'moon'" :size="20" stroke-width="0" />
            <span v-show="!sidebarCollapsed" class="text-sm">{{ isDark ? '浅色' : '深色' }}</span>
          </button>
        </div>
      </aside>

      <!-- 主内容区（含标签栏） -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <TabBar class="bg-base-100 border-b border-base-300 shrink-0" />
        <main class="flex-1 overflow-y-auto p-4 lg:p-6">
          <!-- 每个标签页独立渲染 → 切换标签时全部保持挂载，v-show 显隐 -->
          <div
            v-for="tab in tabStore.tabs"
            :key="tab.id"
            v-show="tab.id === tabStore.activeTabId"
            class="h-full"
          >
            <component :is="resolveComponent(tab.currentPath)" :key="tab.currentPath" />
          </div>
          <div v-if="tabStore.tabs.length === 0" class="flex items-center justify-center h-full text-base-content/30 text-sm">
            请从左侧导航栏打开页面
          </div>
        </main>
      </div>

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
  <!-- 页面内查找组件 -->
  <PageFind />
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import PageFind from '@/components/PageFind.vue'
import TabBar from '@/components/TabBar.vue'
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getTauriAPI } from '@/utils/tauri-api'
import { useAppStore } from '@/stores/appStore'
import { useTabStore, VIEW_ID_TO_PATH, KNOWN_ROUTES } from '@/stores/tabStore'
import LanUsers from '@/views/lan/LanUsers.vue'
import ChatPanel from '@/views/lan/ChatPanel.vue'

import {
  IconLayoutDashboard,
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
  IconBell,
  IconRobot,
} from '@tabler/icons-vue'

import { defineAsyncComponent, type Component } from 'vue'

// 标签页 → 视图组件映射（异步懒加载）
const tabComponents: Record<string, Component> = {
  '/': defineAsyncComponent(() => import('@/views/dashboard/Dashboard.vue')),
  '/todo': defineAsyncComponent(() => import('@/views/todo/TodoList.vue')),
  '/weekly': defineAsyncComponent(() => import('@/views/weekly/WeeklyReport.vue')),
  '/projects': defineAsyncComponent(() => import('@/views/projects/ProjectList.vue')),
  '/accounting': defineAsyncComponent(() => import('@/views/accounting/AccountingBook.vue')),
  '/servers': defineAsyncComponent(() => import('@/views/server/ServerManager.vue')),
  '/cicd': defineAsyncComponent(() => import('@/views/cicd/CiCdConfig.vue')),
  '/logs': defineAsyncComponent(() => import('@/views/logs/LogAggregator.vue')),
  '/nginx': defineAsyncComponent(() => import('@/views/nginx/NginxManager.vue')),
  '/database': defineAsyncComponent(() => import('@/views/db/DBManager.vue')),
  '/agent': defineAsyncComponent(() => import('@/views/agent/AgentManager.vue')),
  '/agent/chat': defineAsyncComponent(() => import('@/views/agent/HermesChat.vue')),
  '/kanban': defineAsyncComponent(() => import('@/components/kanban/KanbanBoard.vue')),
  '/alert': defineAsyncComponent(() => import('@/views/alert/AlertView.vue')),
  '/devtools': defineAsyncComponent(() => import('@/views/devtools/DevTools.vue')),
  '/notes': defineAsyncComponent(() => import('@/views/notes/NoteManager.vue')),
  '/git': defineAsyncComponent(() => import('@/views/git/GitRepoList.vue')),
  '/mfa': defineAsyncComponent(() => import('@/views/mfa/MfaManager.vue')),
  '/vpn': defineAsyncComponent(() => import('@/views/vpn/VPNManager.vue')),
  '/backup': defineAsyncComponent(() => import('@/views/backup/DataBackup.vue')),
  '/disk-cleaner': defineAsyncComponent(() => import('@/components/DiskCleaner.vue')),
  '/report': defineAsyncComponent(() => import('@/views/reports/TodoReport.vue')),
  '/settings': defineAsyncComponent(() => import('@/views/settings/SettingsView.vue')),
}

/** 根据路径查找最匹配的视图组件（精确匹配 → 逐级回退 → 看板） */
function resolveComponent(path: string): Component {
  // 去掉 query params
  const cleanPath = path.split('?')[0].split('#')[0]
  const tryPath = (p: string) => tabComponents[p]
  // 精确路径
  if (tryPath(cleanPath)) {return tabComponents[cleanPath]}
  // 逐级回退（/agent/chat → /agent → /）
  const segs = cleanPath.split('/').filter(Boolean)
  for (let len = segs.length - 1; len >= 0; len--) {
    const p = '/' + segs.slice(0, len).join('/')
    if (tryPath(p)) {return tabComponents[p]}
  }
  return tabComponents['/']
}

const iconMap: Record<string, any> = {
  'dashboard': IconLayoutDashboard,
  'todo': IconChecklist,
  'weekly-report': IconCalendarWeek,
  'projects': IconFolder,
  'accounting': IconCoin,
  'servers': IconServer,
  'cicd': IconRocket,
  'log-aggregator': IconFileText,
  'nginx': IconWorld,
  'database': IconDatabase,
  'alert': IconBell,
  'devtools': IconTool,
  'notes': IconNotebook,
  'git': IconGitBranch,
  'mfa': IconLock,
  'vpn': IconShieldLock,
  'data-backup': IconCloudDownload,
  'disk-cleaner': IconTrash,
  'agent': IconRobot,
}

const router = useRouter()
const route = useRoute()
const appStore = useAppStore()
const tabStore = useTabStore()

const sidebarCollapsed = ref(false)
const showLan = ref(false)
const isDark = ref(false)
const chatPeer = ref<{ id: string; name: string; avatar?: string; version?: string } | null>(null)
const lanStarted = ref(false)

const navGroups = {
  business: [
    { path: '/', icon: '📊', label: '综合看板', viewId: 'dashboard' },
    { path: '/todo', icon: '📝', label: '任务', viewId: 'todo' },
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
  agent: [
    { path: '/agent', icon: '🤖', label: 'Agent', viewId: 'agent' },
    { path: '/kanban', icon: '📋', label: '看板', viewId: 'kanban' },
  ],
  dev: [
    { path: '/database', icon: '🗄️', label: '数据库', viewId: 'database' },
    { path: '/alert', icon: '🔔', label: '告警', viewId: 'alert' },
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
  const routeInfo = KNOWN_ROUTES[path]
  if (routeInfo) {
    tabStore.openOrActivate(path, routeInfo.label, routeInfo.viewId)
  } else {
    tabStore.syncRoute(path)
  }
  // router-link 已处理导航，无需再次 router.push
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
  if (!showLan.value) {chatPeer.value = null}
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

// 同步路由变化到标签页（内部导航自动创建/激活标签页）
watch(() => route.fullPath, (newPath) => {
  tabStore.syncRoute(newPath)
})

onMounted(async () => {
  const api = getTauriAPI()
  try {
    const theme = await api.getSetting('theme')
    isDark.value = theme === 'dark'
    document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  } catch {}

  // 为当前路由打开标签页
  tabStore.syncRoute(route.fullPath)

  const unlistenNav = await api.onMenuNav((view: string) => {
    const routeMap: Record<string, string> = {
      'dashboard': '/', 'todo': '/todo', 'weekly-report': '/weekly', 'projects': '/projects',
      'accounting': '/accounting', 'servers': '/servers', 'cicd': '/cicd',
      'log-aggregator': '/logs', 'nginx': '/nginx', 'database': '/database', 'agent': '/agent', 'alert': '/alert', 'devtools': '/devtools',
      'notes': '/notes', 'git': '/git', 'mfa': '/mfa', 'vpn': '/vpn',
      'data-backup': '/backup', 'disk-cleaner': '/disk-cleaner', 'report': '/report', 'settings': '/settings',
    }
    const path = routeMap[view]
    if (path) {
      const routeInfo = KNOWN_ROUTES[path]
      if (routeInfo) {
        tabStore.openOrActivate(path, routeInfo.label, routeInfo.viewId)
      }
      router.push(path)
    }
  }).catch(() => () => {})
  unlistenFns.push(unlistenNav as () => void)
})

onUnmounted(async () => {
  for (const unlisten of unlistenFns) {
    try { unlisten() } catch {}
  }
})
</script>

<style scoped>
.sidebar-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: transparent transparent;
}

.sidebar-scrollbar:hover {
  scrollbar-color: oklch(0.75 0 0) oklch(0.97 0 0);
}

.sidebar-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.sidebar-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.sidebar-scrollbar::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 3px;
  transition: background-color 0.2s;
}

.sidebar-scrollbar:hover::-webkit-scrollbar-thumb {
  background-color: oklch(0.75 0 0);
}
</style>
