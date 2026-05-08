<template>
  <div class="flex h-screen overflow-hidden bg-base-200 text-base-content transition-[background,color] duration-300" :data-theme="appStore.isDark ? 'dark' : 'light'">
    <!-- 左侧导航栏 -->
    <aside
      class="w-[220px] min-w-[220px] bg-base-100 flex flex-col select-none transition-[width,min-width] duration-250 border-r border-base-300"
      :class="{ '!w-[48px] !min-w-[48px]': sidebarCollapsed }"
    >
      <div class="flex items-center gap-2.5 px-[18px] py-4 border-b border-base-300 overflow-hidden" :class="sidebarCollapsed ? '!px-0 !justify-center !gap-0' : ''">
        <div class="flex items-center text-primary">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/></svg>
        </div>
        <span class="text-base font-bold text-base-content tracking-[-0.3px] whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">SuperTool</span>
      </div>

      <!-- 搜索过滤 -->
      <div class="px-2.5 pt-2 pb-1 relative" v-show="!sidebarCollapsed">
        <svg class="absolute left-[19px] top-1/2 -translate-y-1/2 text-base-content/50 pointer-events-none" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input v-model="sidebarFilter" placeholder="搜索功能..." class="w-full py-[7px] pl-[28px] pr-[26px] border border-base-300 rounded-md bg-base-200 text-base-content text-[12px] outline-none placeholder:text-base-content/50 focus:border-primary" @keydown.escape="sidebarFilter = ''" />
        <button v-if="sidebarFilter" class="absolute right-[14px] top-1/2 -translate-y-1/2 bg-transparent border-none text-base-content/50 text-base cursor-pointer" @click="sidebarFilter = ''">×</button>
      </div>

      <nav class="flex-1 px-2.5 pt-2 flex flex-col gap-1 overflow-y-auto">
        <!-- 业务工作区 -->
        <div class="flex flex-col gap-px" v-show="navGroupVisible('business')">
          <div class="text-[10px] font-semibold uppercase tracking-[0.8px] text-base-content/30 px-3 pt-1.5 pb-0.5" :class="sidebarCollapsed ? '!px-1 !text-[0px]' : ''">业务</div>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'todo' }]" @click="activeView = 'todo'" v-show="matchesFilter('任务 todo')" title="任务">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">📝</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">任务</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'weekly-report' }]" @click="activeView = 'weekly-report'" v-show="matchesFilter('周报 weekly')" title="周报">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">📊</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">周报</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'projects' }]" @click="activeView = 'projects'" v-show="matchesFilter('项目 project')" title="项目">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">📁</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">项目</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'accounting' }]" @click="activeView = 'accounting'" v-show="matchesFilter('记账 accounting')" title="记账本">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">💰</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">记账本</span>
          </button>
        </div>

        <!-- 运维管理 -->
        <div class="flex flex-col gap-px" v-show="navGroupVisible('ops')">
          <div class="text-[10px] font-semibold uppercase tracking-[0.8px] text-base-content/30 px-3 pt-1.5 pb-0.5" :class="sidebarCollapsed ? '!px-1 !text-[0px]' : ''">运维</div>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'servers' }]" @click="activeView = 'servers'" v-show="matchesFilter('服务器 server ssh')" title="服务器">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">🖥️</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">服务器</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'cicd' }]" @click="activeView = 'cicd'" v-show="matchesFilter('ci cd 部署 deploy')" title="CI/CD">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">🚀</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">CI/CD</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'log-aggregator' }]" @click="activeView = 'log-aggregator'" v-show="matchesFilter('日志 log')" title="日志聚合">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">📋</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">日志聚合</span>
          </button>
        </div>

        <!-- 开发工具 -->
        <div class="flex flex-col gap-px" v-show="navGroupVisible('devtools')">
          <div class="text-[10px] font-semibold uppercase tracking-[0.8px] text-base-content/30 px-3 pt-1.5 pb-0.5" :class="sidebarCollapsed ? '!px-1 !text-[0px]' : ''">开发</div>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'database' }]" @click="activeView = 'database'" v-show="matchesFilter('数据库 database sql')" title="数据库">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">🗄️</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">数据库</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'devtools' }]" @click="activeView = 'devtools'" v-show="matchesFilter('开发工具 devtools')" title="开发工具">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">🛠️</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">开发工具</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'notes' }]" @click="activeView = 'notes'" v-show="matchesFilter('笔记 note')" title="笔记">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">📓</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">笔记</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'git' }]" @click="activeView = 'git'" v-show="matchesFilter('git 仓库')" title="Git 仓库">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">🔀</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">Git 仓库</span>
          </button>
        </div>

        <!-- 安全与网络 -->
        <div class="flex flex-col gap-px" v-show="navGroupVisible('security')">
          <div class="text-[10px] font-semibold uppercase tracking-[0.8px] text-base-content/30 px-3 pt-1.5 pb-0.5" :class="sidebarCollapsed ? '!px-1 !text-[0px]' : ''">安全</div>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'mfa' }]" @click="activeView = 'mfa'" v-show="matchesFilter('mfa 验证码 otp')" title="MFA">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">🔐</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">MFA</span>
          </button>
          <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'vpn' }]" @click="activeView = 'vpn'" v-show="matchesFilter('vpn')" title="VPN">
            <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">🌐</span>
            <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">VPN</span>
          </button>
        </div>
      </nav>

      <div class="px-2.5 py-2 border-t border-base-300 flex flex-col gap-0.5">
        <div class="text-[10px] font-semibold uppercase tracking-[0.8px] text-base-content/30 px-3 pt-1.5 pb-0.5" :class="sidebarCollapsed ? '!px-1 !text-[0px]' : ''">设置</div>
        <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': activeView === 'data-backup' }]" @click="activeView = 'data-backup'" v-show="matchesFilter('备份 backup')" title="备份">
          <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">💾</span>
          <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">备份</span>
        </button>
        <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden" :class="[sidebarCollapsed ? '!px-0 !justify-center !gap-0' : '', { 'bg-primary text-white': showLan }]" @click="toggleLan" v-show="matchesFilter('局域网 lan')" title="局域网协作">
          <span class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center text-base">🔗</span>
          <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">局域网</span>
        </button>
        <button class="flex items-center gap-2.5 px-3 py-2.5 border-none rounded-lg bg-transparent text-base-content/50 text-sm font-medium cursor-pointer transition-all duration-150 text-left w-full overflow-hidden sidebar-collapse-btn" :class="sidebarCollapsed ? '!px-0 !justify-center !gap-0' : ''" @click="sidebarCollapsed = !sidebarCollapsed" :title="sidebarCollapsed ? '展开' : '收起'">
          <svg v-if="sidebarCollapsed" class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="13 17 18 12 13 7"/><polyline points="6 17 11 12 6 7"/></svg>
          <svg v-else class="w-[18px] h-[18px] flex-shrink-0 flex items-center justify-center" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="11 17 6 12 11 7"/><polyline points="18 17 13 12 18 7"/></svg>
          <span class="whitespace-nowrap" :class="sidebarCollapsed ? 'hidden' : ''">{{ sidebarCollapsed ? '展开' : '收起' }}</span>
        </button>
      </div>
    </aside>

    <!-- 主内容区 -->
    <div class="flex-1 min-w-0 flex flex-col">
      <main class="flex-1 min-h-0 h-screen overflow-y-auto">
        <div v-show="activeView === 'todo'"><TodoList /></div>
        <div v-show="activeView === 'weekly-report'"><TodoReport /></div>
        <div v-show="activeView === 'projects'">
          <ProjectList v-if="!selectedProjectId" @select-project="onSelectProject" />
          <ProjectDetail v-if="selectedProjectId" :project-id="selectedProjectId" @go-back="selectedProjectId = null" />
        </div>
        <div v-show="activeView === 'accounting'"><AccountingBook /></div>
        <div v-show="activeView === 'servers'"><ServerManager /></div>
        <div v-show="activeView === 'cicd'" class="w-full h-full flex flex-col">
          <div class="flex gap-1 px-6 pt-3">
            <button class="px-4 py-2 border-none rounded-t-lg text-[13px] font-medium cursor-pointer" :class="cicdTab === 'deploy' ? 'bg-base-100 text-base-content' : 'bg-base-100 text-base-content/60'" @click="cicdTab = 'deploy'">部署</button>
            <button class="px-4 py-2 border-none rounded-t-lg text-[13px] font-medium cursor-pointer" :class="cicdTab === 'config' ? 'bg-base-100 text-base-content' : 'bg-base-100 text-base-content/60'" @click="cicdTab = 'config'">配置</button>
          </div>
          <div v-show="cicdTab === 'deploy'"><DeployPanel /></div>
          <div v-show="cicdTab === 'config'"><CiCdConfig /></div>
        </div>
        <div v-show="activeView === 'log-aggregator'"><LogAggregator /></div>
        <div v-show="activeView === 'database'"><DBManager /></div>
        <div v-show="activeView === 'devtools'"><DevTools /></div>
        <div v-show="activeView === 'notes'"><NoteManager /></div>
        <div v-show="activeView === 'git'"><GitRepoList /></div>
        <div v-show="activeView === 'mfa'"><MfaManager /></div>
        <div v-show="activeView === 'vpn'"><VPNManager /></div>
        <div v-show="activeView === 'data-backup'"><DataBackup /></div>
      </main>
    </div>

    <!-- 右侧面板（局域网） -->
    <aside
      class="w-[400px] min-w-[400px] bg-base-100 border-l border-base-300 flex flex-col transition-[width,min-width] duration-300"
      :class="{ '!w-[1100px] !min-w-[1100px]': showLan && chatPeer }"
      v-if="showLan"
    >
      <div class="flex items-center justify-between px-[18px] py-4 border-b border-base-300">
        <h3 class="text-[15px] font-semibold text-base-content">局域网协作</h3>
        <button class="w-7 h-7 border-none rounded-md bg-transparent text-base-content/50 text-lg cursor-pointer flex items-center justify-center hover:bg-base-200" @click="showLan = false; chatPeer = null">×</button>
      </div>
      <div class="flex-1 min-h-0 flex gap-0 overflow-hidden">
        <LanUsers @open-chat="onOpenChat" />
        <ChatPanel
          v-if="chatPeer"
          :peer="chatPeer"
          @close="chatPeer = null"
          @refresh-unread="onRefreshUnread"
        />
      </div>
    </aside>

    <!-- Toast 通知 -->
    <ToastContainer />
    <!-- 全局搜索 -->
    <GlobalSearch />
    <!-- 关于对话框 -->
    <AboutDialog v-model="showAboutDialog" />
    <!-- 快速切换面板 (Cmd+Shift+K) -->
    <QuickSwitch ref="quickSwitchRef" @close="() => {}" @select="onQuickSwitchSelect" />
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { getTauriAPI } from '@/utils/tauri-api'
import { toast } from '@/composables/useToast'
import { useTodoStore } from '@/stores/todoStore'
import { useAppStore } from '@/stores/appStore'
import { defineAsyncComponent } from 'vue'
const TodoList = defineAsyncComponent(() => import('../views/todo/TodoList.vue'))
const TodoReport = defineAsyncComponent(() => import('../views/TodoReport.vue'))
const ProjectList = defineAsyncComponent(() => import('../views/ProjectList.vue'))
const ProjectDetail = defineAsyncComponent(() => import('../views/ProjectDetail.vue'))
const ServerManager = defineAsyncComponent(() => import('../views/server/ServerManager.vue'))
const DBManager = defineAsyncComponent(() => import('../views/db/DBManager.vue'))
const GitRepoList = defineAsyncComponent(() => import('../views/GitRepoList.vue'))
const LogAggregator = defineAsyncComponent(() => import('../views/LogAggregator.vue'))
const NoteManager = defineAsyncComponent(() => import('../views/NoteManager.vue'))
const MfaManager = defineAsyncComponent(() => import('../views/MfaManager.vue'))
const VPNManager = defineAsyncComponent(() => import('../views/VPNManager.vue'))
const DataBackup = defineAsyncComponent(() => import('../views/DataBackup.vue'))
const AccountingBook = defineAsyncComponent(() => import('../views/AccountingBook.vue'))
const DevTools = defineAsyncComponent(() => import('../views/devtools/DevTools.vue'))
const CiCdConfig = defineAsyncComponent(() => import('../views/cicd/CiCdConfig.vue'))
const DeployPanel = defineAsyncComponent(() => import('../views/cicd/DeployPanel.vue'))
const LanUsers = defineAsyncComponent(() => import('./components/lan/LanUsers.vue'))
const ChatPanel = defineAsyncComponent(() => import('./ChatPanel.vue'))
const ToastContainer = defineAsyncComponent(() => import('./components/ui/ToastContainer.vue'))
const GlobalSearch = defineAsyncComponent(() => import('./components/GlobalSearch.vue'))
const AboutDialog = defineAsyncComponent(() => import('./components/AboutDialog.vue'))
const QuickSwitch = defineAsyncComponent(() => import('./components/QuickSwitch.vue'))

type ViewId = 'todo' | 'weekly-report' | 'projects' | 'accounting' | 'servers' | 'cicd' | 'log-aggregator' | 'database' | 'devtools' | 'notes' | 'git' | 'mfa' | 'vpn' | 'data-backup'

const activeView = ref<ViewId>('todo')
const sidebarCollapsed = ref(false)
const sidebarFilter = ref('')
const showLan = ref(false)
const chatPeer = ref<{ id: string; name: string; avatar?: string; version?: string } | null>(null)
const lanStarted = ref(false)

async function toggleLan() {
  if (!showLan.value && !lanStarted.value) {
    // Start LAN service BEFORE showing the panel
    try {
      const api = getTauriAPI()
      const userInfo = await api.lanGetUserInfo().catch(() => ({ id: '', name: '' }))
      const userId = userInfo.id || `user_${crypto.randomUUID().slice(0, 8)}`
      const userName = userInfo.name || 'User'
      await api.startLan(userId, userName)
      lanStarted.value = true
      console.log('[App] LAN service started')
    } catch (e) {
      console.error('[App] Failed to start LAN:', e)
    }
  }
  showLan.value = !showLan.value
  if (!showLan.value) {
    chatPeer.value = null
  }
}
const cicdTab = ref<'deploy' | 'config'>('deploy')
const selectedProjectId = ref<string | null>(null)
const showAboutDialog = ref(false)
const quickSwitchRef = ref<InstanceType<typeof QuickSwitch> | null>(null)
const appStore = useAppStore()

function onSelectProject(project: { id: string }) { selectedProjectId.value = project.id }
function onQuickSwitchSelect(viewId: string) {
    console.log("[onSelectProject] called");
    activeView.value = viewId as ViewId }
function onOpenChat(peer: { id: string; name: string; avatar?: string; version?: string }) {
    chatPeer.value = peer
}
function onRefreshUnread() {
    // 通知 LanUsers 刷新未读计数
    window.dispatchEvent(new CustomEvent('lan:reload-unread'))
}

// Menu event listeners
let unlistenFns: (() => void)[] = []

onMounted(async () => {
    console.log("[onQuickSwitchSelect] called");
    console.log("[App.vue] mounted")
  const api = getTauriAPI()
  const todoStore = useTodoStore()
  const appStore = useAppStore()

  // Initialize theme
  await appStore.initTheme()

  const unlistenNav = await api.onMenuNav((view: string) => {
    console.log('[App] onMenuNav received, view:', view)
    activeView.value = view as ViewId
  })
  unlistenFns.push(unlistenNav as () => void)

  const unlistenNewTask = await api.onMenuNewTask(() => {
    console.log('[App] onMenuNewTask received')
    activeView.value = 'todo'
    setTimeout(() => {
      (document.querySelector('.todo-input-field') as HTMLElement | null)?.focus()
    }, 100)
  })
  unlistenFns.push(unlistenNewTask as () => void)

  const unlistenSearch = await api.onMenuSearchTasks(() => {
    console.log('[App] onMenuSearchTasks received')
    activeView.value = 'todo'
    setTimeout(() => {
      (document.querySelector('.search-input') as HTMLElement | null)?.focus()
    }, 100)
  })
  unlistenFns.push(unlistenSearch as () => void)

  const unlistenAbout = await api.onMenuAbout(() => {
    console.log('[App] onMenuAbout received')
    showAboutDialog.value = true
  })
  unlistenFns.push(unlistenAbout as () => void)

  const unlistenSelectAll = await api.onMenuSelectAll(() => {
    console.log('[App] onMenuSelectAll received')
  })
  unlistenFns.push(unlistenSelectAll as () => void)

  const unlistenDeleteSelected = await api.onMenuDeleteSelected(() => {
    console.log('[App] onMenuDeleteSelected received')
  })
  unlistenFns.push(unlistenDeleteSelected as () => void)

  const unlistenToggleComplete = await api.onMenuToggleComplete(() => {
    console.log('[App] onMenuToggleComplete received')
  })
  unlistenFns.push(unlistenToggleComplete as () => void)

  const unlistenClearCompleted = await api.onMenuClearCompleted(() => {
    console.log('[App] onMenuClearCompleted received')
    todoStore.clearCompleted()
  })
  unlistenFns.push(unlistenClearCompleted as () => void)

  const unlistenToggleTheme = await api.onMenuToggleTheme(() => {
    console.log('[App] onMenuToggleTheme received')
    appStore.toggleTheme()
  })
  unlistenFns.push(unlistenToggleTheme as () => void)

  const unlistenToggleLocale = await api.onMenuToggleLocale(() => {
    console.log('[App] onMenuToggleLocale received')
    appStore.toggleLocale()
  })
  unlistenFns.push(unlistenToggleLocale as () => void)

  console.log('[App] Menu event listeners registered')
})

onUnmounted(async () => {
  for (const unlisten of unlistenFns) {
    try { unlisten() } catch (e) { /* ignore */ }
  }
})

function matchesFilter(text: string): boolean {
  if (!sidebarFilter.value) return true
  return text.toLowerCase().includes(sidebarFilter.value.toLowerCase())
}

function navGroupVisible(group: string): boolean {
  if (!sidebarFilter.value) return true
  const keywords: Record<string, string> = {
    business: '任务 周报 项目 记账 todo weekly project accounting',
    ops: '服务器 日志 ci cd 部署 server ssh log aggregator deploy',
    devtools: '数据库 开发 笔记 database devtools note git repo',
    security: 'mfa 验证码 vpn otp',
  }
  return matchesFilter(keywords[group] || '')
}
</script>
