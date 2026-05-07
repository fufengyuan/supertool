<template>
  <div class="app-container" :class="{ dark: appStore.isDark }">
    <!-- 左侧导航栏 -->
    <aside class="sidebar" :class="{ collapsed: sidebarCollapsed }">
      <div class="sidebar-header">
        <div class="sidebar-logo">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/></svg>
        </div>
        <span class="sidebar-title">SuperTool</span>
      </div>

      <!-- 搜索过滤 -->
      <div class="sidebar-search" v-show="!sidebarCollapsed">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input v-model="sidebarFilter" placeholder="搜索功能..." class="sidebar-search-input" @keydown.escape="sidebarFilter = ''" />
        <button v-if="sidebarFilter" class="search-clear" @click="sidebarFilter = ''">×</button>
      </div>

      <nav class="sidebar-nav">
        <!-- 业务工作区 -->
        <div class="nav-group" v-show="navGroupVisible('business')">
          <div class="nav-group-label">业务</div>
          <button class="nav-item" v-show="matchesFilter('任务 todo')" :class="{ active: activeView === 'todo' }" @click="activeView = 'todo'" title="任务">
            <span class="nav-icon">📝</span>
            <span class="nav-label">任务</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('周报 weekly')" :class="{ active: activeView === 'weekly-report' }" @click="activeView = 'weekly-report'" title="周报">
            <span class="nav-icon">📊</span>
            <span class="nav-label">周报</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('项目 project')" :class="{ active: activeView === 'projects' }" @click="activeView = 'projects'" title="项目">
            <span class="nav-icon">📁</span>
            <span class="nav-label">项目</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('记账 accounting')" :class="{ active: activeView === 'accounting' }" @click="activeView = 'accounting'" title="记账本">
            <span class="nav-icon">💰</span>
            <span class="nav-label">记账本</span>
          </button>
        </div>

        <!-- 运维管理 -->
        <div class="nav-group" v-show="navGroupVisible('ops')">
          <div class="nav-group-label">运维</div>
          <button class="nav-item" v-show="matchesFilter('服务器 server ssh')" :class="{ active: activeView === 'servers' }" @click="activeView = 'servers'" title="服务器">
            <span class="nav-icon">🖥️</span>
            <span class="nav-label">服务器</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('ci cd 部署 deploy')" :class="{ active: activeView === 'cicd' }" @click="activeView = 'cicd'" title="CI/CD">
            <span class="nav-icon">🚀</span>
            <span class="nav-label">CI/CD</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('日志 log')" :class="{ active: activeView === 'log-aggregator' }" @click="activeView = 'log-aggregator'" title="日志聚合">
            <span class="nav-icon">📋</span>
            <span class="nav-label">日志聚合</span>
          </button>
        </div>

        <!-- 开发工具 -->
        <div class="nav-group" v-show="navGroupVisible('devtools')">
          <div class="nav-group-label">开发</div>
          <button class="nav-item" v-show="matchesFilter('数据库 database sql')" :class="{ active: activeView === 'database' }" @click="activeView = 'database'" title="数据库">
            <span class="nav-icon">🗄️</span>
            <span class="nav-label">数据库</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('开发工具 devtools')" :class="{ active: activeView === 'devtools' }" @click="activeView = 'devtools'" title="开发工具">
            <span class="nav-icon">🛠️</span>
            <span class="nav-label">开发工具</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('笔记 note')" :class="{ active: activeView === 'notes' }" @click="activeView = 'notes'" title="笔记">
            <span class="nav-icon">📓</span>
            <span class="nav-label">笔记</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('git 仓库')" :class="{ active: activeView === 'git' }" @click="activeView = 'git'" title="Git 仓库">
            <span class="nav-icon">🔀</span>
            <span class="nav-label">Git 仓库</span>
          </button>
        </div>

        <!-- 安全与网络 -->
        <div class="nav-group" v-show="navGroupVisible('security')">
          <div class="nav-group-label">安全</div>
          <button class="nav-item" v-show="matchesFilter('mfa 验证码 otp')" :class="{ active: activeView === 'mfa' }" @click="activeView = 'mfa'" title="MFA">
            <span class="nav-icon">🔐</span>
            <span class="nav-label">MFA</span>
          </button>
          <button class="nav-item" v-show="matchesFilter('vpn')" :class="{ active: activeView === 'vpn' }" @click="activeView = 'vpn'" title="VPN">
            <span class="nav-icon">🌐</span>
            <span class="nav-label">VPN</span>
          </button>
        </div>
      </nav>

      <div class="sidebar-footer">
        <div class="nav-group-label">设置</div>
        <button class="nav-item" v-show="matchesFilter('备份 backup')" :class="{ active: activeView === 'data-backup' }" @click="activeView = 'data-backup'" title="备份">
          <span class="nav-icon">💾</span>
          <span class="nav-label">备份</span>
        </button>
        <button class="nav-item" v-show="matchesFilter('局域网 lan')" :class="{ active: showLan }" @click="toggleLan" title="局域网协作">
          <span class="nav-icon">🔗</span>
          <span class="nav-label">局域网</span>
        </button>
        <button class="nav-item sidebar-collapse-btn" @click="sidebarCollapsed = !sidebarCollapsed" :title="sidebarCollapsed ? '展开' : '收起'">
          <svg v-if="sidebarCollapsed" class="nav-icon" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="13 17 18 12 13 7"/><polyline points="6 17 11 12 6 7"/></svg>
          <svg v-else class="nav-icon" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="11 17 6 12 11 7"/><polyline points="18 17 13 12 18 7"/></svg>
          <span class="nav-label">{{ sidebarCollapsed ? '展开' : '收起' }}</span>
        </button>
      </div>
    </aside>

    <!-- 主内容区 -->
    <div class="content-wrapper">
      <main class="main-content">
        <div v-show="activeView === 'todo'"><TodoList /></div>
        <div v-show="activeView === 'weekly-report'"><TodoReport /></div>
        <div v-show="activeView === 'projects'">
          <ProjectList v-if="!selectedProjectId" @select-project="onSelectProject" />
          <ProjectDetail v-if="selectedProjectId" :project-id="selectedProjectId" @go-back="selectedProjectId = null" />
        </div>
        <div v-show="activeView === 'accounting'"><AccountingBook /></div>
        <div v-show="activeView === 'servers'"><ServerManager /></div>
        <div v-show="activeView === 'cicd'" class="cicd-wrapper">
          <div class="cicd-tabs">
            <button :class="{ active: cicdTab === 'deploy' }" @click="cicdTab = 'deploy'">部署</button>
            <button :class="{ active: cicdTab === 'config' }" @click="cicdTab = 'config'">配置</button>
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
    <aside class="right-panel" v-if="showLan">
      <div class="panel-header">
        <h3>局域网协作</h3>
        <button class="panel-close" @click="showLan = false; chatPeer = null">×</button>
      </div>
      <div class="lan-panel-content">
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
const TodoList = defineAsyncComponent(() => import('./components/TodoList.vue'))
const TodoReport = defineAsyncComponent(() => import('./components/TodoReport.vue'))
const ProjectList = defineAsyncComponent(() => import('./components/ProjectList.vue'))
const ProjectDetail = defineAsyncComponent(() => import('./components/ProjectDetail.vue'))
const ServerManager = defineAsyncComponent(() => import('./components/server/ServerManager.vue'))
const DBManager = defineAsyncComponent(() => import('./components/db/DBManager.vue'))
const GitRepoList = defineAsyncComponent(() => import('./components/GitRepoList.vue'))
const LogAggregator = defineAsyncComponent(() => import('./components/LogAggregator.vue'))
const NoteManager = defineAsyncComponent(() => import('./components/NoteManager.vue'))
const MfaManager = defineAsyncComponent(() => import('./components/MfaManager.vue'))
const VPNManager = defineAsyncComponent(() => import('./components/VPNManager.vue'))
const DataBackup = defineAsyncComponent(() => import('./components/DataBackup.vue'))
const AccountingBook = defineAsyncComponent(() => import('./components/AccountingBook.vue'))
const DevTools = defineAsyncComponent(() => import('./components/devtools/DevTools.vue'))
const CiCdConfig = defineAsyncComponent(() => import('./components/cicd/CiCdConfig.vue'))
const DeployPanel = defineAsyncComponent(() => import('./components/cicd/DeployPanel.vue'))
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

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif; background: var(--main-bg); color: var(--main-text); -webkit-font-smoothing: antialiased; }

.app-container { display: flex; height: 100vh; overflow: hidden; background: var(--main-bg); color: var(--main-text); transition: background 0.3s ease, color 0.3s ease; }

.sidebar {
  width: 220px; min-width: 220px; background: var(--sidebar-bg); display: flex; flex-direction: column;
  user-select: none; transition: width 0.25s ease, min-width 0.25s ease;
  border-right: 1px solid var(--border-color);
}
.sidebar.collapsed { width: 48px; min-width: 48px; }

.sidebar-header { display: flex; align-items: center; gap: 10px; padding: 16px 18px; border-bottom: 1px solid var(--border-color); overflow: hidden; }
.sidebar.collapsed .sidebar-header { padding: 16px 0; justify-content: center; gap: 0; }
.sidebar-logo { display: flex; align-items: center; color: var(--primary-color); }
.sidebar-title { font-size: 16px; font-weight: 700; color: var(--sidebar-text); letter-spacing: -0.3px; white-space: nowrap; }
.sidebar.collapsed .sidebar-title { display: none; }

.sidebar-search { padding: 8px 10px 4px; position: relative; }
.sidebar-search svg { position: absolute; left: 19px; top: 50%; transform: translateY(-50%); color: var(--sidebar-text-dim); pointer-events: none; }
.sidebar-search-input {
  width: 100%; padding: 7px 26px 7px 28px; border: 1px solid var(--border-color); border-radius: 6px;
  background: var(--input-bg); color: var(--main-text); font-size: 12px; outline: none;
}
.sidebar-search-input::placeholder { color: var(--sidebar-text-dim); }
.sidebar-search-input:focus { border-color: var(--primary-color); }
.search-clear { position: absolute; right: 14px; top: 50%; transform: translateY(-50%); background: none; border: none; color: var(--sidebar-text-dim); font-size: 16px; cursor: pointer; }

.sidebar-nav { flex: 1; padding: 8px 10px 0; display: flex; flex-direction: column; gap: 4px; overflow-y: auto; }
.nav-group { display: flex; flex-direction: column; gap: 1px; }
.nav-group-label { font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.8px; color: var(--sidebar-group-label); padding: 6px 12px 2px; }
.sidebar.collapsed .nav-group-label { padding: 6px 4px 2px; font-size: 0; }

.nav-item {
  display: flex; align-items: center; gap: 10px; padding: 10px 12px; border: none; border-radius: 8px;
  background: transparent; color: var(--sidebar-text-dim); font-size: 14px; font-weight: 500; cursor: pointer;
  transition: all 0.15s ease; text-align: left; width: 100%; overflow: hidden;
}
.sidebar.collapsed .nav-item { padding: 10px 0; justify-content: center; gap: 0; }
.nav-item:hover { background: var(--sidebar-hover); color: var(--sidebar-text); }
.nav-item.active { background: var(--primary-color); color: #fff; }
.nav-icon { width: 18px; height: 18px; flex-shrink: 0; display: flex; align-items: center; justify-content: center; font-size: 16px; }
.nav-label { white-space: nowrap; }
.sidebar.collapsed .nav-label { display: none; }

.sidebar-footer { padding: 8px 10px; border-top: 1px solid var(--border-color); display: flex; flex-direction: column; gap: 2px; }

.content-wrapper { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.main-content { flex: 1; min-height: 0; height: 100vh; overflow-y: auto; }

.right-panel {
  width: 400px; min-width: 400px; background: var(--sidebar-bg); border-left: 1px solid var(--border-color);
  display: flex; flex-direction: column;
  transition: width 0.3s ease, min-width 0.3s ease;
}
.right-panel:has(.chat-panel) {
  width: 1100px; min-width: 1100px;
}
.lan-panel-content {
  flex: 1; min-height: 0;
  display: flex;
  gap: 0;
  overflow: hidden;
}
.lan-panel-content .lan-panel {
  flex: 0 0 380px;
  overflow-y: auto;
  border-right: 1px solid var(--border-color, rgba(255,255,255,0.06));
}
.panel-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 18px; border-bottom: 1px solid var(--border-color); }
.panel-header h3 { font-size: 15px; font-weight: 600; color: var(--main-text); }
.panel-close { width: 28px; height: 28px; border: none; border-radius: 6px; background: transparent; color: var(--sidebar-text-dim); font-size: 18px; cursor: pointer; display: flex; align-items: center; justify-content: center; }
.panel-close:hover { background: var(--sidebar-hover); }

/* CI/CD Tabs */
.cicd-wrapper { width: 100%; height: 100%; display: flex; flex-direction: column; }
.cicd-tabs { display: flex; gap: 4px; padding: 12px 24px 0; }
.cicd-tabs button {
  padding: 8px 16px; border: none; border-radius: 8px 8px 0 0; background: var(--sidebar-bg);
  color: var(--main-text-secondary); font-size: 13px; font-weight: 500; cursor: pointer;
}
.cicd-tabs button.active { background: var(--card-bg); color: var(--main-text); }

/* Scrollbar */
.main-content::-webkit-scrollbar, .sidebar-nav::-webkit-scrollbar { width: 6px; }
.main-content::-webkit-scrollbar-track, .sidebar-nav::-webkit-scrollbar-track { background: transparent; }
.main-content::-webkit-scrollbar-thumb, .sidebar-nav::-webkit-scrollbar-thumb { background: var(--border-color); border-radius: 3px; }
</style>
