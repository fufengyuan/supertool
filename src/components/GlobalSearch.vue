<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-150 ease"
      leave-active-class="transition-opacity duration-150 ease"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div v-if="isOpen" class="fixed inset-0 z-[9999] flex items-start justify-center pt-[15vh] bg-black/40" @mousedown.self="close">
        <div class="w-[560px] max-w-[90vw] bg-base-100 border border-base-content/10 rounded-2xl shadow-[0_20px_60px_rgba(0,0,0,0.3)] overflow-hidden flex flex-col">
          <!-- 搜索输入区 -->
          <div class="relative flex items-center px-5 py-4 border-b border-base-content/10">
            <SvgIcon name="search" size="20" class="absolute left-6 text-base-content/60 pointer-events-none" />
            <input
              ref="inputRef"
              v-model="query"
              type="text"
              class="flex-1 border-none outline-none text-base bg-transparent text-base-content pl-8 placeholder:text-base-content/60"
              placeholder="搜索任务、项目、服务器、笔记、CI/CD…"
              @keydown.down.prevent="navigateResults(1)"
              @keydown.up.prevent="navigateResults(-1)"
              @keydown.enter.prevent="selectResult"
              @keydown.esc.prevent="close"
              @keydown.space.stop
            />
            <kbd class="text-[11px] px-2 py-0.5 rounded bg-base-200 text-base-content/60 border border-base-content/10">ESC</kbd>
          </div>

          <!-- 搜索结果 -->
          <div v-if="query.trim()" class="max-h-[360px] overflow-y-auto py-2">
            <template v-if="results.length > 0">
              <!-- 任务结果 -->
              <template v-if="todoResults.length > 0">
                <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">任务</div>
                <div
                  v-for="(item, idx) in todoResults"
                  :key="'todo-' + item.id"
                  class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
                  :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === getGlobalIndex('todo', idx) }"
                  @click="selectItem(item, 'todo')"
                  @mouseenter="activeIndex = getGlobalIndex('todo', idx)"
                >
                  <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0 bg-[rgba(34,197,94,0.1)] text-[#22c55e]">✓</span>
                  <div class="flex-1 min-w-0">
                    <span class="block text-sm font-medium text-base-content truncate" v-html="highlightMatch(item.text, query)"></span>
                    <span class="block text-xs text-base-content/60 mt-0.5">{{ item.tag || '未分类' }} · {{ formatDate(item.createdAt) }}</span>
                  </div>
                  <span v-if="item.priority" class="text-[11px] px-2 py-0.5 rounded font-medium shrink-0"
                    :class="{
                      'bg-[rgba(239,68,68,0.1)] text-[#ef4444]': item.priority === 'high',
                      'bg-[rgba(245,158,11,0.1)] text-[#f59e0b]': item.priority === 'medium',
                      'bg-[rgba(34,197,94,0.1)] text-[#22c55e]': item.priority === 'low'
                    }"
                  >{{ priorityLabel(item.priority) }}</span>
                </div>
              </template>

              <!-- 项目结果 -->
              <template v-if="projectResults.length > 0">
                <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">项目</div>
                <div
                  v-for="(item, idx) in projectResults"
                  :key="'project-' + item.id"
                  class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
                  :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === getGlobalIndex('project', idx) }"
                  @click="selectItem(item, 'project')"
                  @mouseenter="activeIndex = getGlobalIndex('project', idx)"
                >
                  <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0 bg-[rgba(99,102,241,0.1)]" :style="{ color: item.color || '#6c63ff' }"><SvgIcon name="folder" size="14" class="inline-block align-text-bottom" /></span>
                  <div class="flex-1 min-w-0">
                    <span class="block text-sm font-medium text-base-content truncate" v-html="highlightMatch(item.name, query)"></span>
                    <span class="block text-xs text-base-content/60 mt-0.5">{{ item.description || '暂无描述' }}</span>
                  </div>
                </div>
              </template>

              <!-- 服务器结果 -->
              <template v-if="serverResults.length > 0">
                <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">服务器</div>
                <div
                  v-for="(item, idx) in serverResults"
                  :key="'server-' + item.id"
                  class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
                  :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === getGlobalIndex('server', idx) }"
                  @click="selectItem(item, 'server')"
                  @mouseenter="activeIndex = getGlobalIndex('server', idx)"
                >
                  <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0 bg-[rgba(251,146,60,0.1)]"><SvgIcon name="server" size="14" class="inline-block align-text-bottom" /></span>
                  <div class="flex-1 min-w-0">
                    <span class="block text-sm font-medium text-base-content truncate" v-html="highlightMatch(item.name, query)"></span>
                    <span class="block text-xs text-base-content/60 mt-0.5">{{ item.host }}:{{ item.port }}</span>
                  </div>
                </div>
              </template>

              <!-- 笔记结果 -->
              <template v-if="noteResults.length > 0">
                <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">笔记</div>
                <div
                  v-for="(item, idx) in noteResults"
                  :key="'note-' + item.id"
                  class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
                  :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === getGlobalIndex('note', idx) }"
                  @click="selectItem(item, 'note')"
                  @mouseenter="activeIndex = getGlobalIndex('note', idx)"
                >
                  <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0 bg-[rgba(168,85,247,0.1)]"><SvgIcon name="book" size="14" class="inline-block align-text-bottom" /></span>
                  <div class="flex-1 min-w-0">
                    <span class="block text-sm font-medium text-base-content truncate" v-html="highlightMatch(item.title, query)"></span>
                    <span class="block text-xs text-base-content/60 mt-0.5">{{ item.group || '未分组' }}</span>
                  </div>
                </div>
              </template>

              <!-- CI/CD 结果 -->
              <template v-if="cicdResults.length > 0">
                <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">CI/CD</div>
                <div
                  v-for="(item, idx) in cicdResults"
                  :key="'cicd-' + item.id"
                  class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
                  :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === getGlobalIndex('cicd', idx) }"
                  @click="selectItem(item, 'cicd')"
                  @mouseenter="activeIndex = getGlobalIndex('cicd', idx)"
                >
                  <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0 bg-[rgba(245,158,11,0.1)]"><SvgIcon name="rocket" size="14" class="inline-block align-text-bottom" /></span>
                  <div class="flex-1 min-w-0">
                    <span class="block text-sm font-medium text-base-content truncate" v-html="highlightMatch(item.name, query)"></span>
                    <span class="block text-xs text-base-content/60 mt-0.5">{{ item.groupName || '未分组' }} · {{ item.deployBranch }}</span>
                  </div>
                </div>
              </template>

              <!-- MFA 结果 -->
              <template v-if="mfaResults.length > 0">
                <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">MFA</div>
                <div
                  v-for="(item, idx) in mfaResults"
                  :key="'mfa-' + item.id"
                  class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
                  :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === getGlobalIndex('mfa', idx) }"
                  @click="selectItem(item, 'mfa')"
                  @mouseenter="activeIndex = getGlobalIndex('mfa', idx)"
                >
                  <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0 bg-[rgba(239,68,68,0.1)]"><SvgIcon name="lock" size="14" class="inline-block align-text-bottom" /></span>
                  <div class="flex-1 min-w-0">
                    <span class="block text-sm font-medium text-base-content truncate" v-html="highlightMatch(item.name, query)"></span>
                    <span class="block text-xs text-base-content/60 mt-0.5">{{ item.account || item.issuer || '' }}</span>
                  </div>
                </div>
              </template>

              <!-- Git 结果 -->
              <template v-if="gitResults.length > 0">
                <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">Git 仓库</div>
                <div
                  v-for="(item, idx) in gitResults"
                  :key="'git-' + item.id"
                  class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
                  :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === getGlobalIndex('git', idx) }"
                  @click="selectItem(item, 'git')"
                  @mouseenter="activeIndex = getGlobalIndex('git', idx)"
                >
                  <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0 bg-[rgba(59,130,246,0.1)]"><SvgIcon name="gitBranch" size="14" class="inline-block align-text-bottom" /></span>
                  <div class="flex-1 min-w-0">
                    <span class="block text-sm font-medium text-base-content truncate" v-html="highlightMatch(item.name, query)"></span>
                    <span class="block text-xs text-base-content/60 mt-0.5">{{ item.path || item.url || '' }}</span>
                  </div>
                </div>
              </template>

              <!-- VPN 结果 -->
              <template v-if="vpnResults.length > 0">
                <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">VPN</div>
                <div
                  v-for="(item, idx) in vpnResults"
                  :key="'vpn-' + item.id"
                  class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
                  :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === getGlobalIndex('vpn', idx) }"
                  @click="selectItem(item, 'vpn')"
                  @mouseenter="activeIndex = getGlobalIndex('vpn', idx)"
                >
                  <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0 bg-[rgba(20,184,166,0.1)]"><SvgIcon name="globe" size="14" class="inline-block align-text-bottom" /></span>
                  <div class="flex-1 min-w-0">
                    <span class="block text-sm font-medium text-base-content truncate" v-html="highlightMatch(item.name, query)"></span>
                  </div>
                </div>
              </template>
            </template>

            <!-- 无结果 -->
            <div v-else class="flex flex-col items-center py-10 px-5 text-base-content/60">
              <SvgIcon name="search" size="32" stroke-width="1.5" class="opacity-30 mb-3" />
              <p class="m-0 text-sm">未找到匹配的结果</p>
              <p class="text-xs opacity-70 mt-1">尝试其他关键词</p>
            </div>
          </div>

          <!-- 默认快捷操作 (无搜索词时) -->
          <div v-else class="max-h-[360px] overflow-y-auto py-2">
            <div class="px-5 py-1.5 text-[11px] font-semibold uppercase text-base-content/60 tracking-wider">常用功能</div>
            <div
              v-for="(item, idx) in frequentNavs"
              :key="item.viewId"
              class="flex items-center gap-3 px-5 py-2.5 cursor-pointer transition-[background] duration-100 ease hover:bg-[var(--bg-hover,#f0f0f0)]"
              :class="{ 'bg-[var(--bg-hover,#f0f0f0)]': activeIndex === idx }"
              @click="quickNavigateFreq(item)"
              @mouseenter="activeIndex = idx"
            >
              <span class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0"><SvgIcon :name="item.icon" size="14" /></span>
              <div class="flex-1 min-w-0">
                <span class="block text-sm font-medium text-base-content truncate">{{ item.label }}</span>
                <span class="block text-xs text-base-content/60 mt-0.5">点击 {{ item.count }} 次</span>
              </div>
              <kbd class="text-[11px] px-2 py-0.5 rounded bg-base-200 text-base-content/60 border border-base-content/10 shrink-0">{{ idx + 1 }}</kbd>
            </div>
          </div>

          <!-- 底部提示 -->
          <div class="flex gap-4 px-5 py-2.5 border-t border-base-content/10 bg-base-200">
            <span class="flex items-center gap-1.5 text-xs text-base-content/60"><kbd class="text-[10px] px-1.5 py-[1px] rounded bg-base-100 border border-base-content/10">↑↓</kbd> 导航</span>
            <span class="flex items-center gap-1.5 text-xs text-base-content/60"><kbd class="text-[10px] px-1.5 py-[1px] rounded bg-base-100 border border-base-content/10">↵</kbd> 打开</span>
            <span class="flex items-center gap-1.5 text-xs text-base-content/60"><kbd class="text-[10px] px-1.5 py-[1px] rounded bg-base-100 border border-base-content/10">esc</kbd> 关闭</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">// @ts-nocheck
import { getTauriAPI } from '../utils/tauri-api'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useAppStore } from '../stores/appStore'
import { useTodoStore } from '../stores/todoStore'
import { useRouter } from 'vue-router'

interface SearchResult {
  id: string
  type: 'todo' | 'project' | 'server' | 'note' | 'cicd' | 'mfa' | 'git' | 'vpn' | 'accounting'
  [key: string]: any
}

const appStore = useAppStore()
const todoStore = useTodoStore()
const router = useRouter()

const freqNavInfo: Record<string, { icon: string; label: string; route: string }> = {
  'todo': { icon: 'file', label: '任务', route: '/' },
  'weekly-report': { icon: 'barChart', label: '周报', route: '/weekly' },
  'projects': { icon: 'folder', label: '项目', route: '/projects' },
  'accounting': { icon: 'coin', label: '记账本', route: '/accounting' },
  'servers': { icon: 'server', label: '服务器', route: '/servers' },
  'cicd': { icon: 'rocket', label: 'CI/CD', route: '/cicd' },
  'log-aggregator': { icon: 'file', label: '日志聚合', route: '/logs' },
  'database': { icon: 'archive', label: '数据库', route: '/database' },
  'devtools': { icon: 'settings', label: '开发工具', route: '/devtools' },
  'notes': { icon: 'book', label: '笔记', route: '/notes' },
  'git': { icon: 'gitBranch', label: 'Git 仓库', route: '/git' },
  'mfa': { icon: 'lock', label: 'MFA', route: '/mfa' },
  'vpn': { icon: 'globe', label: 'VPN', route: '/vpn' },
  'data-backup': { icon: 'save', label: '备份', route: '/backup' },
}

const frequentNavs = computed(() => {
  return appStore.getFrequentNavs(6)
    .map(item => ({ ...item, ...freqNavInfo[item.viewId] }))
    .filter(item => item.route) // 只显示已知路由
})

const isOpen = ref(false)
const query = ref('')
const activeIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

// 数据源
const todos = computed(() => todoStore.todos)
const projects = ref<any[]>([])
const servers = ref<any[]>([])
const notes = ref<any[]>([])
const cicdConfigs = ref<any[]>([])
const mfaSecrets = ref<any[]>([])
const gitRepos = ref<any[]>([])
const vpnConfigs = ref<any[]>([])
const accountingCategories = ref<any[]>([])

// 加载所有搜索数据
const loadData = async () => {
  const api = getTauriAPI()
  // 项目
  try { projects.value = await api.getProjects(true) || [] } catch {}
  // 服务器
  try { servers.value = await api.getAllServers() || [] } catch {}
  // 笔记
  try { notes.value = await api.getAllNotes() || [] } catch {}
  // CI/CD
  try { cicdConfigs.value = await api.getCicdConfigs() || [] } catch {}
  // MFA
  try { mfaSecrets.value = await api.getAllMfaSecrets() || [] } catch {}
  // Git
  try { const res = await api.getGitRepos(); gitRepos.value = res?.data || [] } catch {}
  // VPN (OpenVPN)
  try { vpnConfigs.value = await api.openvpnGetAll() || [] } catch {}
}

// 模糊匹配
const fuzzyMatch = (text: string, pattern: string): boolean => {
  if (!text || !pattern) {return false}
  const lowerText = text.toLowerCase()
  const lowerPattern = pattern.toLowerCase()
  return lowerText.includes(lowerPattern)
}

// 搜索结果
const todoResults = computed(() => {
  if (!query.value.trim()) {return []}
  return todos.value.filter(t =>
    fuzzyMatch(t.text, query.value) ||
    fuzzyMatch(t.tag, query.value) ||
    fuzzyMatch(t.description, query.value)
  ).slice(0, 8)
})

const projectResults = computed(() => {
  if (!query.value.trim()) {return []}
  return projects.value.filter(p =>
    fuzzyMatch(p.name, query.value) ||
    fuzzyMatch(p.description, query.value)
  ).slice(0, 5)
})

const serverResults = computed(() => {
  if (!query.value.trim()) {return []}
  return servers.value.filter(s =>
    fuzzyMatch(s.name, query.value) ||
    fuzzyMatch(s.host, query.value)
  ).slice(0, 5)
})

const noteResults = computed(() => {
  if (!query.value.trim()) {return []}
  return notes.value.filter(n =>
    fuzzyMatch(n.title, query.value) ||
    fuzzyMatch(n.content, query.value) ||
    fuzzyMatch(n.group, query.value)
  ).slice(0, 5)
})

const cicdResults = computed(() => {
  if (!query.value.trim()) {return []}
  return cicdConfigs.value.filter(c =>
    fuzzyMatch(c.name, query.value) ||
    fuzzyMatch(c.groupName, query.value) ||
    fuzzyMatch(c.deployBranch, query.value)
  ).slice(0, 5)
})

const mfaResults = computed(() => {
  if (!query.value.trim()) {return []}
  return mfaSecrets.value.filter(m =>
    fuzzyMatch(m.name, query.value) ||
    fuzzyMatch(m.account, query.value) ||
    fuzzyMatch(m.issuer, query.value)
  ).slice(0, 5)
})

const gitResults = computed(() => {
  if (!query.value.trim()) {return []}
  return gitRepos.value.filter(g =>
    fuzzyMatch(g.name, query.value) ||
    fuzzyMatch(g.url, query.value) ||
    fuzzyMatch(g.path, query.value)
  ).slice(0, 5)
})

const vpnResults = computed(() => {
  if (!query.value.trim()) {return []}
  return vpnConfigs.value.filter(v =>
    fuzzyMatch(v.name, query.value)
  ).slice(0, 5)
})

const results = computed(() => [
  ...todoResults.value,
  ...projectResults.value,
  ...serverResults.value,
  ...noteResults.value,
  ...cicdResults.value,
  ...mfaResults.value,
  ...gitResults.value,
  ...vpnResults.value
])

// 全局索引计算（按顺序：todo → project → server → note → cicd → mfa → git → vpn）
const resultTypes = ['todo', 'project', 'server', 'note', 'cicd', 'mfa', 'git', 'vpn'] as const
const resultCountMap = computed(() => ({
  todo: todoResults.value.length,
  project: projectResults.value.length,
  server: serverResults.value.length,
  note: noteResults.value.length,
  cicd: cicdResults.value.length,
  mfa: mfaResults.value.length,
  git: gitResults.value.length,
  vpn: vpnResults.value.length,
}))

const getGlobalIndex = (type: string, localIdx: number): number => {
  let offset = 0
  for (const t of resultTypes) {
    if (t === type) {break}
    offset += resultCountMap.value[t as keyof typeof resultCountMap.value] || 0
  }
  return offset + localIdx
}

// 键盘导航
const navigateResults = (direction: number) => {
  const totalResults = results.value.length
  const totalQuickActions = frequentNavs.value.length
  const total = query.value.trim() ? totalResults : totalQuickActions
  if (total === 0) {return}
  activeIndex.value = (activeIndex.value + direction + total) % total
}

// 选择结果
const selectResult = () => {
  if (!query.value.trim()) {
    quickNavigateByIndex(activeIndex.value)
    return
  }
  let offset = 0
  for (const t of resultTypes) {
    const count = resultCountMap.value[t as keyof typeof resultCountMap.value] || 0
    if (activeIndex.value < offset + count) {
      const localIdx = activeIndex.value - offset
      const items: Record<string, any[]> = {
        todo: todoResults.value, project: projectResults.value, server: serverResults.value,
        note: noteResults.value, cicd: cicdResults.value, mfa: mfaResults.value,
        git: gitResults.value, vpn: vpnResults.value,
      }
      selectItem(items[t][localIdx], t)
      return
    }
    offset += count
  }
}

const selectItem = (item: any, type: string) => {
  close()
  const navMap: Record<string, string> = {
    todo: 'todo', project: 'projects', server: 'servers', note: 'notes',
    cicd: 'cicd', mfa: 'mfa', git: 'git', vpn: 'vpn',
  }
  appStore.recordNavClick(navMap[type] || type)
  
  if (type === 'todo') {
    router.push('/')
    setTimeout(() => {
      window.dispatchEvent(new CustomEvent('navigate-to-todo', { detail: { todoId: item.id } }))
    }, 200)
  } else if (type === 'project') {
    router.push('/projects')
  } else if (type === 'server') {
    router.push('/servers')
  } else if (type === 'note') {
    router.push('/notes')
  } else if (type === 'cicd') {
    router.push('/cicd')
  } else if (type === 'mfa') {
    router.push('/mfa')
  } else if (type === 'git') {
    router.push('/git')
  } else if (type === 'vpn') {
    router.push('/vpn')
  }
}

const quickNavigateFreq = (item: any) => {
  close()
  appStore.recordNavClick(item.viewId)
  router.push(item.route)
}

const quickNavigateByIndex = (index: number) => {
  const navs = frequentNavs.value
  if (navs[index]) {quickNavigateFreq(navs[index])}
}

// 打开/关闭
const open = async () => {
  isOpen.value = true
  query.value = ''
  activeIndex.value = 0
  await loadData()
  await nextTick()
  inputRef.value?.focus()
}

const close = () => {
  isOpen.value = false
  query.value = ''
  activeIndex.value = 0
}

// 打开全局搜索快捷键
const handleKeydown = (e: KeyboardEvent) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    open()
  }
  if (e.key === 'Escape' && isOpen.value) {
    close()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

// 暴露 open 方法给父组件
defineExpose({ open })

// 计算优先级的显示标签
const priorityLabel = (priority: string): string => {
  const labels: Record<string, string> = {
    high: '高',
    medium: '中',
    low: '低',
  }
  return labels[priority] || priority
}

// 格式化日期
const formatDate = (dateStr: string): string => {
  if (!dateStr) {return ''}
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  if (days === 0) {return '今天'}
  if (days === 1) {return '昨天'}
  if (days < 7) {return `${days}天前`}
  return date.toLocaleDateString('zh-CN')
}

// 高亮匹配文本
const highlightMatch = (text: string, query: string): string => {
  if (!text || !query) {return text || ''}
  const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi')
  return text.replace(regex, '<mark style="background: rgba(108,99,255,0.2); color: inherit; padding: 0 2px; border-radius: 2px;">$1</mark>')
}
</script>
