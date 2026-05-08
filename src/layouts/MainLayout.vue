<template>
  <div class="h-screen flex flex-col bg-base-200">
    <!-- 顶部标题栏 -->
    <div class="navbar bg-base-100 shadow-sm px-4 flex-none min-h-[2.5rem] h-10">
      <div class="flex-none">
        <label class="btn btn-square btn-ghost btn-sm lg:hidden" @click="sidebarOpen = !sidebarOpen">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </label>
        <button class="btn btn-square btn-ghost btn-sm hidden lg:flex" @click="sidebarCollapsed = !sidebarCollapsed" :title="sidebarCollapsed ? '展开' : '折叠'">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" :class="{ 'rotate-180': sidebarCollapsed }" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
          </svg>
        </button>
      </div>
      <div class="flex-1">
        <span class="btn btn-ghost text-base gap-2 normal-case min-h-0 h-8">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          SuperTool
        </span>
      </div>
      <div class="flex-none gap-2">
        <!-- 搜索按钮 -->
        <button class="btn btn-ghost btn-sm gap-1" @click="openGlobalSearch" title="全局搜索 (Ctrl+K)">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <kbd class="kbd kbd-xs hidden sm:inline-flex">Ctrl+K</kbd>
        </button>
        <!-- 局域网 -->
        <button class="btn btn-ghost btn-sm gap-1" @click="toggleLan" :class="{ 'text-primary': showLan }" title="局域网协作">
          🌐
        </button>
        <!-- 主题切换 -->
        <label class="swap swap-rotate btn btn-ghost btn-circle btn-sm">
          <input type="checkbox" :checked="isDark" @change="toggleTheme" />
          <svg class="swap-off h-5 w-5 fill-current" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M5.64,17l-.71.71a1,1,0,0,0,0,1.41,1,1,0,0,0,1.41,0l.71-.71A1,1,0,0,0,5.64,17ZM5,12a1,1,0,0,0-1-1H3a1,1,0,0,0,0,2H4A1,1,0,0,0,5,12Zm7-7a1,1,0,0,0,1-1V3a1,1,0,0,0-2,0V4A1,1,0,0,0,12,5ZM5.64,7.05a1,1,0,0,0,.7.29,1,1,0,0,0,.71-.29,1,1,0,0,0,0-1.41l-.71-.71A1,1,0,0,0,4.93,6.34Zm12,.29a1,1,0,0,0,.7-.29l.71-.71a1,1,0,1,0-1.41-1.41L17,5.64a1,1,0,0,0,0,1.41A1,1,0,0,0,17.66,7.34ZM21,11H20a1,1,0,0,0,0,2h1a1,1,0,0,0,0-2Zm-9,8a1,1,0,0,0-1,1v1a1,1,0,0,0,2,0V20A1,1,0,0,0,12,19ZM18.36,17A1,1,0,0,0,17,17.05a1,1,0,0,0,0,1.41l.71.71a1,1,0,0,0,1.41,0,1,1,0,0,0,0-1.41ZM12,6.5A5.5,5.5,0,1,0,17.5,12,5.5,5.5,0,0,0,12,6.5Zm0,9A3.5,3.5,0,1,1,15.5,12,3.5,3.5,0,0,1,12,15.5Z"/></svg>
          <svg class="swap-on h-5 w-5 fill-current" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M21.64,13a1,1,0,0,0-1.05-.14,8.05,8.05,0,0,1-3.37.73A8.15,8.15,0,0,1,9.08,5.49a8.59,8.59,0,0,1,.25-2A1,1,0,0,0,8,2.36,10.14,10.14,0,1,0,22,14.05,1,1,0,0,0,21.64,13Zm-9.5,6.69A8.14,8.14,0,0,1,7.08,5.22v.27A10.15,10.15,0,0,0,17.22,15.63a9.79,9.79,0,0,0,2.1-.22A8.11,8.11,0,0,1,12.14,19.73Z"/></svg>
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
            <router-link :to="item.path" class="gap-3 w-full" active-class="active" @click="sidebarOpen = false">
              <span class="text-lg">{{ item.icon }}</span>
              <span v-show="!sidebarCollapsed">{{ item.label }}</span>
            </router-link>
          </li>

          <!-- 运维 -->
          <div class="menu-title px-3 mt-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-semibold text-base-content/50 uppercase tracking-wider">运维</span>
          </div>
          <li v-for="item in navGroups.ops" :key="item.path" class="w-full">
            <router-link :to="item.path" class="gap-3 w-full" active-class="active" @click="sidebarOpen = false">
              <span class="text-lg">{{ item.icon }}</span>
              <span v-show="!sidebarCollapsed">{{ item.label }}</span>
            </router-link>
          </li>

          <!-- 开发 -->
          <div class="menu-title px-3 mt-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-semibold text-base-content/50 uppercase tracking-wider">开发</span>
          </div>
          <li v-for="item in navGroups.dev" :key="item.path" class="w-full">
            <router-link :to="item.path" class="gap-3 w-full" active-class="active" @click="sidebarOpen = false">
              <span class="text-lg">{{ item.icon }}</span>
              <span v-show="!sidebarCollapsed">{{ item.label }}</span>
            </router-link>
          </li>

          <!-- 安全 -->
          <div class="menu-title px-3 mt-2" v-show="!sidebarCollapsed">
            <span class="text-xs font-semibold text-base-content/50 uppercase tracking-wider">安全</span>
          </div>
          <li v-for="item in navGroups.security" :key="item.path" class="w-full">
            <router-link :to="item.path" class="gap-3 w-full" active-class="active" @click="sidebarOpen = false">
              <span class="text-lg">{{ item.icon }}</span>
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
          <h3 class="font-semibold text-base">🌐 局域网协作</h3>
          <button class="btn btn-ghost btn-sm btn-circle" @click="showLan = false">✕</button>
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
import { ref, onMounted, onUnmounted } from 'vue'
import { getTauriAPI } from '@/utils/tauri-api'
import LanUsers from '@/views/lan/LanUsers.vue'
import ChatPanel from '@/components/ChatPanel.vue'

const sidebarOpen = ref(false)
const sidebarCollapsed = ref(false)
const showLan = ref(false)
const isDark = ref(false)
const chatPeer = ref<{ id: string; name: string; avatar?: string; version?: string } | null>(null)
const lanStarted = ref(false)

const navGroups = {
  business: [
    { path: '/', icon: '📝', label: '任务' },
    { path: '/weekly', icon: '📊', label: '周报' },
    { path: '/projects', icon: '📁', label: '项目' },
    { path: '/accounting', icon: '💰', label: '记账本' },
  ],
  ops: [
    { path: '/servers', icon: '🖥️', label: '服务器' },
    { path: '/cicd', icon: '🚀', label: 'CI/CD' },
    { path: '/logs', icon: '📋', label: '日志聚合' },
  ],
  dev: [
    { path: '/database', icon: '🗄️', label: '数据库' },
    { path: '/devtools', icon: '🛠️', label: '开发工具' },
    { path: '/notes', icon: '📓', label: '笔记' },
    { path: '/git', icon: '🔀', label: 'Git 仓库' },
  ],
  security: [
    { path: '/mfa', icon: '🔐', label: 'MFA' },
    { path: '/vpn', icon: '🌐', label: 'VPN' },
    { path: '/backup', icon: '💾', label: '备份' },
  ],
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
      'log-aggregator': '/logs', 'database': '/database', 'devtools': '/devtools',
      'notes': '/notes', 'git': '/git', 'mfa': '/mfa', 'vpn': '/vpn',
      'data-backup': '/backup', 'report': '/report', 'settings': '/settings',
    }
    const path = routeMap[view]
    if (path) window.location.hash = '#' + path
  }).catch(() => () => {})
  unlistenFns.push(unlistenNav as () => void)
})

onUnmounted(async () => {
  for (const unlisten of unlistenFns) {
    try { unlisten() } catch {}
  }
})
</script>
