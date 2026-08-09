<template>
  <div class="h-screen flex flex-col bg-base-200">
    <!-- stool CLI 版本更新提示横幅（dmg 安装无 postinstall，靠 App 检测自动安装） -->
    <div v-if="cliUpdate.needUpdate" class="flex items-center gap-3 px-4 py-2 bg-warning/10 border-b border-warning/30 text-xs">
      <span class="text-base-content">🔧 stool CLI 有新版本（{{ cliUpdate.installed || '未安装' }} → {{ cliUpdate.bundled }}）</span>
      <button class="btn btn-primary btn-xs" :disabled="cliInstalling" @click="updateCli">
        {{ cliInstalling ? '安装中...' : '一键更新' }}
      </button>
      <button class="btn btn-ghost btn-xs ml-auto" @click="cliUpdate.needUpdate = false">忽略</button>
    </div>
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
            <div class="relative inline-flex items-center">
              <IconNetwork size="20" stroke-width="1.5" />
              <span v-if="lanStore.totalUnread > 0" class="absolute -top-1.5 -right-2 badge badge-error badge-xs min-w-4 h-4 text-[10px] px-1 shadow-[0_2px_8px_rgba(239,68,68,0.4)]">
                {{ lanStore.totalUnread > 99 ? '99+' : lanStore.totalUnread }}
              </span>
            </div>
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
        <!-- Tab 标签栏（至少有2个标签时显示） -->
        <TabBar v-if="tabStore.tabs.length > 0" />
        <main class="flex-1 overflow-y-auto p-4 lg:p-6">
          <router-view v-slot="{ Component }">
            <keep-alive :max="8" :include="tabStore.includeList">
              <component :is="Component" />
            </keep-alive>
          </router-view>
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
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getTauriAPI } from '@/utils/tauri-api'
import { useAppStore } from '@/stores/appStore'
import { useLanStore } from '@/stores/lanStore'
import { useTabStore } from '@/stores/tabStore'
import { useSettingsStore } from '@/utils/settings'
import { useTheme } from '@/utils/theme'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
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
  IconUsers,
  IconLayoutColumns,
  IconPhoto,
  IconBrain,
  IconCpu,
  IconSettings,
  IconKey,
  IconMessage2,
  IconTools,
  IconClock,
  IconBuildingStore,
} from '@tabler/icons-vue'

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
  'skills': IconBrain,
  'memory': IconCpu,
  'tools': IconTools,
  'cron': IconClock,
  'providers': IconBuildingStore,
  'models': IconBrain,
  'kanban': IconLayoutColumns,
  'image-processor': IconPhoto,
}

const router = useRouter()
const route = useRoute()
const appStore = useAppStore()

// stool CLI 版本检测与一键更新（dmg 安装场景的 CLI 分发入口）
const cliUpdate = ref<{ installed: string; bundled: string; needUpdate: boolean }>({ installed: '', bundled: '', needUpdate: false })
const cliInstalling = ref(false)

async function checkCliVersion() {
  try {
    const api = getTauriAPI()
    const res = await api.checkCliVersion()
    if (res?.success && res.data?.needUpdate) {
      cliUpdate.value = res.data
    }
    // 静默同步内置 skills 到用户技能目录（失败忽略，不影响启动）
    api.syncUserSkills().catch(() => {})
  } catch { /* 非 Tauri 环境（浏览器预览）忽略 */ }
}

async function updateCli() {
  cliInstalling.value = true
  try {
    const api = getTauriAPI()
    const res = await api.installCli()
    if (res?.success) {
      cliUpdate.value = { ...cliUpdate.value, needUpdate: false, installed: res.data?.installed || cliUpdate.value.bundled }
    } else {
      console.warn('CLI 更新失败:', res?.error)
    }
  } catch (e: any) {
    console.warn('CLI 更新失败:', e?.message || e)
  } finally {
    cliInstalling.value = false
  }
}
const tabStore = useTabStore()
const lanStore = useLanStore()

/** 当前路由是否为 Agent 相关 */

const sidebarCollapsed = ref(false)
const showLan = ref(false)
const settingsStore = useSettingsStore()
const { toggleTheme: toggleThemeSetting, applyTheme } = useTheme()
// 统一主题系统：settingsStore.theme 为 cupcake/sunset（深色主题即 sunset）
const isDark = computed(() => settingsStore.theme === 'sunset')
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
  dev: [
    { path: '/database', icon: '🗄️', label: '数据库', viewId: 'database' },
    { path: '/alert', icon: '🔔', label: '告警', viewId: 'alert' },
    { path: '/devtools', icon: '🛠️', label: '开发工具', viewId: 'devtools' },
    { path: '/notes', icon: '📓', label: '笔记', viewId: 'notes' },
    { path: '/git', icon: '🔀', label: 'Git 仓库', viewId: 'git' },
    { path: '/image', icon: '🖼️', label: '图像处理', viewId: 'image-processor' },
  ],
  security: [
    { path: '/mfa', icon: '🔐', label: 'MFA', viewId: 'mfa' },
    { path: '/vpn', icon: '🌐', label: 'VPN', viewId: 'vpn' },
    { path: '/backup', icon: '💾', label: '备份', viewId: 'data-backup' },
    { path: '/audit', icon: '📜', label: '审计', viewId: 'audit' },
    { path: '/disk-cleaner', icon: '🧹', label: '磁盘清理', viewId: 'disk-cleaner' },
  ],
}

function onNavClick(viewId: string, path: string) {
  appStore.recordNavClick(viewId)
  tabStore.openOrActivate(path)
}

// 监听路由变化 → 同步到 tabStore
watch(() => route.fullPath, (fullPath) => {
  const path = fullPath.split('?')[0].split('#')[0]
  tabStore.syncRoute(path)
}, { immediate: true })

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
  await toggleThemeSetting()
}

function openGlobalSearch() {
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true }))
}

let unlistenFns: (() => void)[] = []

onMounted(async () => {
  const api = getTauriAPI()
  // 启动时检测 CLI 版本差异（dmg 用户也能拿到新 CLI）
  checkCliVersion()
  // 统一主题初始化：从 settings.json 加载（cupcake/sunset），替代旧 getSetting('theme') dark/light
  try {
    await settingsStore.initializeSettings()
    applyTheme(settingsStore.theme)
  } catch {}

  // 全局键盘快捷键接线（Ctrl+1~8 导航 / Ctrl+N / Ctrl+F / Ctrl+B / Ctrl+D）
  useKeyboardShortcuts({
    focusNewTask: () => { router.push('/todo') },
    focusSearch: () => openGlobalSearch(),
    toggleSidebar: () => { sidebarCollapsed.value = !sidebarCollapsed.value },
    toggleTheme: () => { toggleThemeSetting() },
    setViewMode: (mode) => {
      const routeMap: Record<string, string> = {
        'todo': '/todo', 'weekly-report': '/weekly', 'projects': '/projects',
        'servers': '/servers', 'cicd': '/cicd', 'database': '/database',
        'notes': '/notes', 'devtools': '/devtools',
      }
      const target = routeMap[mode]
      if (target && route.path !== target) {router.push(target)}
    },
  })

  const unlistenNav = await api.onMenuNav((view: string) => {
    const routeMap: Record<string, string> = {
      'dashboard': '/', 'todo': '/todo', 'weekly-report': '/weekly', 'projects': '/projects',
      'accounting': '/accounting', 'servers': '/servers', 'cicd': '/cicd',
      'log-aggregator': '/logs', 'nginx': '/nginx', 'database': '/database', 'alert': '/alert', 'devtools': '/devtools',
      'notes': '/notes', 'git': '/git', 'mfa': '/mfa', 'vpn': '/vpn',
      'data-backup': '/backup', 'disk-cleaner': '/disk-cleaner', 'report': '/report', 'settings': '/settings',
      'image-processor': '/image',
      'sessions': '/terminal',
    }
    const path = routeMap[view]
    if (path) {router.push(path)}
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
