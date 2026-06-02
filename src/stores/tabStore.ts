/**
 * tabStore - 标签页状态管理
 *
 * 标签页系统让用户可以像浏览器一样同时打开多个页面，
 * 页面间切换时保持各自状态（会话选择、滚动位置等）。
 *
 * 设计原则：
 * - 每个侧边栏导航项对应一个标签页（basePath）
 * - 子路由（如 /agent/chat）归入父标签页，不创建新标签页
 * - KeepAlive include 控制哪些组件保持活跃
 * - 关闭标签页时释放关联组件的缓存
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

// ============ 类型定义 ============

export interface Tab {
  /** 标签页唯一标识 = basePath */
  id: string
  /** 基础路径（侧边栏导航路径） */
  basePath: string
  /** 显示名称 */
  label: string
  /** 图标标识（viewId，映射到 iconMap） */
  viewId: string
  /** 该标签页内最近一次访问的完整路径 */
  currentPath: string
}

// ============ 已知导航路径及其关联组件 ============

interface RouteInfo {
  label: string
  viewId: string
  componentNames: string[]
}

export const KNOWN_ROUTES: Record<string, RouteInfo> = {
  '/':        { label: '综合看板', viewId: 'dashboard',      componentNames: ['Dashboard'] },
  '/todo':      { label: '任务',    viewId: 'todo',             componentNames: ['TodoList'] },
  '/weekly':    { label: '周报',    viewId: 'weekly-report',    componentNames: ['WeeklyReport'] },
  '/projects':  { label: '项目',    viewId: 'projects',         componentNames: ['ProjectList', 'ProjectDetail'] },
  '/accounting':{ label: '记账本',  viewId: 'accounting',       componentNames: ['AccountingBook'] },
  '/servers':   { label: '服务器',  viewId: 'servers',          componentNames: ['ServerManager'] },
  '/cicd':      { label: 'CI/CD',   viewId: 'cicd',             componentNames: ['CiCdConfig'] },
  '/logs':      { label: '日志聚合',viewId: 'log-aggregator',   componentNames: ['LogAggregator'] },
  '/nginx':     { label: 'Nginx',   viewId: 'nginx',            componentNames: ['NginxManager'] },
  '/database':  { label: '数据库',  viewId: 'database',         componentNames: ['DBManager'] },
  '/agent':     { label: 'Agent',   viewId: 'agent',            componentNames: ['HermesChat'] },
  '/agent/profiles': { label: 'Profiles', viewId: 'agent-profiles', componentNames: ['AgentProfiles'] },
  '/agent/tools': { label: '工具', viewId: 'tools', componentNames: ['ToolsManager'] },
  '/agent/cron': { label: '定时任务', viewId: 'cron', componentNames: ['CronManager'] },
  '/agent/providers': { label: '模型提供商', viewId: 'providers', componentNames: ['ProviderManager'] },
  '/agent/models': { label: '模型管理', viewId: 'models', componentNames: ['ModelsPage'] },
  '/agent/skills': { label: '技能', viewId: 'skills', componentNames: ['SkillsBrowser'] },
  '/agent/memory': { label: '记忆', viewId: 'memory', componentNames: ['MemoryManager'] },
  '/agent/sessions': { label: 'Sessions', viewId: 'agent-sessions', componentNames: ['SessionsPage'] },
  '/agent/settings': { label: 'Agent 设置', viewId: 'agent-settings', componentNames: ['SettingsPage'] },
  '/kanban':    { label: '看板',    viewId: 'kanban',           componentNames: ['KanbanBoard'] },
  '/alert':     { label: '告警',    viewId: 'alert',            componentNames: ['AlertView'] },
  '/devtools':  { label: '开发工具',viewId: 'devtools',         componentNames: ['DevTools'] },
  '/notes':     { label: '笔记',    viewId: 'notes',            componentNames: ['NoteManager'] },
  '/git':       { label: 'Git 仓库',viewId: 'git',              componentNames: ['GitRepoList'] },
  '/mfa':       { label: 'MFA',     viewId: 'mfa',              componentNames: ['MfaManager'] },
  '/vpn':       { label: 'VPN',     viewId: 'vpn',              componentNames: ['VPNManager'] },
  '/backup':    { label: '备份',    viewId: 'data-backup',      componentNames: ['DataBackup'] },
  '/disk-cleaner': { label: '磁盘清理', viewId: 'disk-cleaner', componentNames: ['DiskCleaner'] },
  '/report':    { label: '报告',    viewId: 'report',           componentNames: ['TodoReport'] },
  '/image':     { label: '图像处理', viewId: 'image-processor', componentNames: ['ImageProcessor'] },
  '/settings':  { label: '设置',     viewId: 'settings',          componentNames: ['SettingsView'] },
}

// 已知导航路径的有序列表（用于匹配子路由的父标签页）
const KNOWN_BASE_PATHS = Object.keys(KNOWN_ROUTES)

/**
 * 根据任意路径推断其所属的基础路径（标签页 ID）
 * 例如：/agent/chat → /agent,  /project/5 → /projects
 */
function resolveBasePath(path: string): string | null {
  if (path === '/') {return '/'}

  const segments = path.split('/').filter(Boolean)
  for (let len = segments.length; len > 0; len--) {
    const candidate = '/' + segments.slice(0, len).join('/')
    if (KNOWN_BASE_PATHS.includes(candidate)) {
      return candidate
    }
  }
  return null
}

// ============ 导航动作映射（用于 TabBar 点击等）============

/**
 * 从侧边栏 viewId 获取对应路径
 */
export const VIEW_ID_TO_PATH: Record<string, string> = {
  dashboard: '/', todo: '/todo', 'weekly-report': '/weekly',
  projects: '/projects', accounting: '/accounting', servers: '/servers',
  cicd: '/cicd', 'log-aggregator': '/logs', nginx: '/nginx',
  database: '/database', agent: '/agent', alert: '/alert',
  devtools: '/devtools', notes: '/notes', git: '/git',
  mfa: '/mfa', vpn: '/vpn', 'data-backup': '/backup',
  'disk-cleaner': '/disk-cleaner', report: '/report', settings: '/settings',
  'image-processor': '/image',
  tools: '/agent/tools',
  cron: '/agent/cron',
  providers: '/agent/providers',
  models: '/agent/models',
  skills: '/agent/skills',
  memory: '/agent/memory',
  'agent-sessions': '/agent/sessions',
  'agent-settings': '/agent/settings',
}

// ============ Store ============

export const useTabStore = defineStore('tabs', () => {
  const tabs = ref<Tab[]>([])
  const activeTabId = ref<string>('')

  /** KeepAlive include 列表 — 所有打开标签页的关联组件名 */
  const includeList = computed<string[]>(() => {
    const names = new Set<string>()
    for (const tab of tabs.value) {
      const routeInfo = KNOWN_ROUTES[tab.id]
      if (routeInfo) {
        for (const name of routeInfo.componentNames) {
          names.add(name)
        }
      }
    }
    return Array.from(names)
  })

  // ============ 操作 ============

  /**
   * 通过 basePath 打开或激活一个标签页
   * 返回该标签页，若新建则返回 undefined
   */
  function openOrActivate(basePath: string, label?: string, viewId?: string): Tab | undefined {
    const existing = tabs.value.find(t => t.id === basePath)
    if (existing) {
      activeTabId.value = existing.id
      return existing
    }

    const routeInfo = KNOWN_ROUTES[basePath]
    const tab: Tab = {
      id: basePath,
      basePath,
      label: label || routeInfo?.label || basePath,
      viewId: viewId || routeInfo?.viewId || '',
      currentPath: basePath,
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
    return tab
  }

  /**
   * 关闭指定标签页
   * 自动切换到下一个/上一个标签页，关闭最后一个时打开看板
   */
  function closeTab(id: string) {
    const idx = tabs.value.findIndex(t => t.id === id)
    if (idx === -1) {return}

    tabs.value.splice(idx, 1)

    // 如果关闭的是当前标签页，切换到相邻标签页
    if (activeTabId.value === id) {
      if (tabs.value.length > 0) {
        const newIdx = Math.min(idx, tabs.value.length - 1)
        activeTabId.value = tabs.value[newIdx].id
      } else {
        activeTabId.value = ''
      }
    }
  }

  /**
   * 激活指定标签页（由标签栏点击触发）
   */
  function activate(id: string) {
    activeTabId.value = id
  }

  /**
   * 同步路由变化到标签页状态
   * 当用户通过内部链接/其他方式导航时调用
   */
  function syncRoute(path: string) {
    const basePath = resolveBasePath(path)
    if (!basePath) {return}

    // 确保标签页存在
    const tab = openOrActivate(basePath)

    // 只有路径变了才更新 currentPath（避免触发 <component :is :key> 不必要重建）
    if (tab && tab.currentPath !== path) {
      tab.currentPath = path
    }
  }

  /**
   * 获取活跃标签页对象
   */
  const activeTab = computed<Tab | undefined>(() => {
    return tabs.value.find(t => t.id === activeTabId.value)
  })

  return {
    tabs,
    activeTabId,
    includeList,
    activeTab,
    openOrActivate,
    closeTab,
    activate,
    syncRoute,
  }
})
