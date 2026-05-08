<template>
  <div class="log-aggregator">
    <div class="log-header">
      <h2>🔍 日志聚合中心</h2>
      <div class="header-actions">
        <button @click="openNewPresetForm" class="btn-add-preset">+ 新增预设</button>
      </div>
    </div>

    <div class="log-layout">
      <!-- 左侧：预设列表（按分组展示） -->
      <div class="log-sidebar">
        <div class="preset-section">
          <h3>查询预设</h3>

          <!-- 分组 -->
          <div
            v-for="groupEntry in groupedPresets"
            :key="groupEntry.presetGroup"
            class="preset-group"
          >
            <div class="preset-group-header" @click="togglePresetGroup(groupEntry.presetGroup)">
              <span class="group-toggle">{{ collapsedPresetGroups.has(groupEntry.presetGroup) ? '▶' : '▼' }}</span>
              <span class="group-label">{{ groupEntry.presetGroup }}</span>
              <span class="group-count">{{ groupEntry.presets.length }}</span>
            </div>
            <div v-show="!collapsedPresetGroups.has(groupEntry.presetGroup)" class="preset-group-body">
              <div
                v-for="preset in groupEntry.presets"
                :key="preset.id"
                class="preset-item"
                :class="{ active: selectedPreset?.id === preset.id, streaming: isStreaming && selectedPreset?.id === preset.id }"
                @click="selectAndQuery(preset)"
              >
                <div class="preset-info">
                  <span class="preset-name">{{ preset.name }}</span>
                  <span class="preset-meta">{{ preset.serverIds.length }} 节点 · {{ preset.logType }}</span>
                </div>
                <button @click.stop="editPreset(preset)" class="btn-edit-preset" title="编辑">✏️</button>
                <button @click.stop="deletePreset(preset.id)" class="btn-delete-preset" title="删除">×</button>
              </div>
            </div>
          </div>

          <div v-if="presets.length === 0" class="empty-presets">
            <template v-if="allServers.length === 0">
              <div class="empty-presets-guide">
                <p>🔌 尚未配置服务器</p>
                <p class="guide-text">日志聚合需要先添加 SSH 服务器：</p>
                <button @click="goToServers" class="btn-guide">前往配置服务器 →</button>
              </div>
            </template>
            <template v-else>
              暂无预设，点击上方按钮添加
            </template>
          </div>
        </div>
      </div>

      <!-- 右侧：日志输出 -->
      <div class="log-main">
        <!-- 查询模式切换栏 -->
        <div class="mode-bar">
          <div class="mode-tabs">
            <button
              :class="['mode-tab', { active: queryMode === 'stream' }]"
              @click="switchQueryMode('stream')"
            >📡 流式查询</button>
            <button
              :class="['mode-tab', { active: queryMode === 'search' }]"
              @click="switchQueryMode('search')"
            >🔍 日志搜索</button>
          </div>

          <!-- 搜索模式：关键字输入 -->
          <div v-if="queryMode === 'search'" class="search-bar">
            <input
              v-model="searchKeyword"
              placeholder="搜索关键字"
              class="search-input"
              @keyup.enter="doSearch"
            />
            <div class="search-options">
              <label>
                上下文行数
                <div class="context-controls">
                  <button
                    :class="['context-btn', { active: searchContextLines === 0 }]"
                    @click="searchContextLines = 0"
                    title="精准搜索（仅匹配行）"
                  >0行</button>
                  <button
                    :class="['context-btn', { active: searchContextLines === 10 }]"
                    @click="searchContextLines = 10"
                    title="模糊搜索（匹配行上下各10行）"
                  >±10行</button>
                  <input
                    v-model.number="searchContextLines"
                    type="number"
                    min="0"
                    max="500"
                    class="context-input"
                    placeholder="自定义"
                  />
                </div>
              </label>
              <span class="context-hint">{{ searchContextLines === 0 ? '精确匹配' : `匹配行上下各 ${searchContextLines} 行` }}</span>
            </div>
            <button
              @click="doSearch"
              :disabled="!selectedPreset || !searchKeyword.trim() || isSearching"
              class="btn-search"
            >
              {{ isSearching ? '🔄 搜索中...' : '🔍 搜索' }}
            </button>
          </div>
        </div>

        <div class="log-toolbar">
          <div class="log-stats">
            <span>{{ displayLines.length }} 行</span>
            <span v-if="activeServers.size > 0">· {{ activeServers.size }} 个节点在线</span>
            <span v-if="selectedPreset" class="current-preset">· 当前：{{ selectedPreset.name }}</span>
          </div>
          <div class="toolbar-actions">
            <button
              @click="stopQuery"
              v-if="isStreaming"
              class="btn-stop"
              title="终止当前日志查询"
            >
              ⏹ 终止
            </button>
            <button
              v-if="queryMode === 'stream'"
              @click="toggleFollowMode"
              class="btn-follow"
              :class="{ active: followMode }"
              :title="followMode ? '关闭跟踪（自由滚动）' : '开启跟踪（始终显示最新日志）'"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2v4m0 12v4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M2 12h4m12 0h4M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83"/>
              </svg>
              {{ followMode ? '📌 跟踪中' : '📍 跟踪' }}
            </button>
            <button @click="clearLogs" class="btn-clear">清除</button>
            <button @click="exportLogs" class="btn-export">📋 导出</button>
          </div>
        </div>

        <div class="log-content" ref="logContainer" @scroll="onScroll">
          <div v-if="displayLines.length === 0 && !isStreaming && !hasSearched" class="log-empty">
            <p v-if="queryMode === 'stream'">选择左侧预设开始查询日志</p>
            <p v-else>输入关键字后点击搜索</p>
          </div>

          <div v-if="queryMode === 'search' && searchKeyword.trim() && !isSearching && displayLines.length === 0 && hasSearched" class="search-empty">
            <p>未找到匹配结果</p>
          </div>

          <div
            v-for="line in displayLines"
            :key="line.id"
            class="log-line"
            :class="[`level-${line.level}`, { 'is-match': line.isMatch }]"
          >
            <span v-if="queryMode === 'search'" class="log-line-num">{{ line.lineNum || '' }}</span>
            <span class="log-server" :style="{ color: getServerColor(line.serverId) }">[{{ line.serverName }}]</span>
            <span class="log-content-text" v-html="highlightSearchResult(line.content)"></span>
          </div>
        </div>

        <!-- 回到底部浮动按钮 -->
        <button
          v-if="queryMode === 'stream' && showScrollBottom"
          @click="scrollToBottom"
          class="btn-scroll-bottom"
          title="点击后进入跟踪模式"
        >
          ↓ 回到底部并跟踪
        </button>
      </div>
    </div>

    <!-- 预设表单弹窗 -->
    <div v-if="showPresetForm" class="modal-overlay" @click.self="showPresetForm = false">
      <div class="modal-content">
        <h3>{{ editingPreset ? '编辑预设' : '新增预设' }}</h3>
        <div class="form-group">
          <label>预设名称</label>
          <input v-model="presetForm.name" placeholder="例如：API服务日志" class="form-input" />
        </div>
        <div class="form-group">
          <label>分组</label>
          <input
            v-model="presetForm.presetGroup"
            list="group-suggestions"
            placeholder="例如：生产 / 测试"
            class="form-input"
          />
          <datalist id="group-suggestions">
            <option value="生产" />
            <option value="测试" />
            <option value="开发" />
            <option value="预发" />
          </datalist>
        </div>
        <div class="form-group">
          <label>服务器</label>
          <GroupedServerSelector
            :servers="allServers"
            :groups="allGroups"
            v-model="presetForm.serverIds"
            mode="multi"
          />
        </div>
        <div class="form-group">
          <label>日志类型</label>
          <select v-model="presetForm.logType" class="form-select">
            <option value="file">文件 (tail)</option>
            <option value="journalctl">Journalctl</option>
            <option value="docker">Docker</option>
            <option value="custom">自定义</option>
          </select>
        </div>
        <div class="form-group">
          <label>日志路径 / 容器名</label>
          <textarea v-model="presetForm.logPath" placeholder="/var/log/app/api.log&#10;/var/log/app/error.log&#10;(每行一个路径)" class="form-input form-textarea" rows="3" />
        </div>
        <div class="form-group">
          <label>关键字（逗号分隔）</label>
          <input v-model="presetForm.keywordsInput" placeholder="ERROR, Exception" class="form-input" />
        </div>
        <div class="form-group">
          <label>初始行数</label>
          <input v-model.number="presetForm.maxLines" type="number" min="50" max="50000" class="form-input small" />
        </div>
        <div class="modal-actions">
          <button @click="showPresetForm = false" class="btn-cancel">取消</button>
          <button @click="savePreset" class="btn-save">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import { useToast } from '../composables/useToast'
import type { Server } from '../types'
import GroupedServerSelector from '@/components/server/GroupedServerSelector.vue'

const toast = useToast()

// 状态
const presets = ref<any[]>([])
const allServers = ref<Server[]>([])
const allGroups = ref<Array<{ id: string; name: string; color: string; parentId: string | null }>>([])
const selectedPreset = ref<any | null>(null)
const logLines = ref<Array<{ id: string; serverId: string; serverName: string; timestamp: number; content: string; level: string; isMatch?: boolean; lineNum?: string }>>([])
const isStreaming = ref(false)
const followMode = ref(true)
const activeServers = ref(new Set<string>())
const streamId = ref('')
const logContainer = ref<HTMLElement | null>(null)
let scrollingFromRAF = false
let lastScrollTop = 0
let pendingScroll = false

// 查询模式
const queryMode = ref<'stream' | 'search'>('stream')

// 搜索模式状态
const searchKeyword = ref('')
const searchContextLines = ref(10)
const isSearching = ref(false)
const hasSearched = ref(false)

// 滚动状态
const showScrollBottom = ref(false)

// 预设分组折叠状态
const collapsedPresetGroups = ref(new Set<string>())

// 预设表单
const showPresetForm = ref(false)
const editingPreset = ref<string | null>(null)
const presetForm = ref({
  name: '',
  presetGroup: '未分组',
  serverIds: [] as string[],
  logType: 'file' as 'file' | 'journalctl' | 'docker' | 'custom',
  logPath: '',
  keywordsInput: '',
  maxLines: 500
})

// 颜色映射
const serverColors = new Map<string, string>()
const colorPalette = ['#4ade80', '#60a5fa', '#f472b6', '#fbbf24', '#a78bfa', '#34d399', '#f87171', '#38bdf8']
let colorIndex = 0

function getServerColor(serverId: string): string {
  if (!serverColors.has(serverId)) {
    serverColors.set(serverId, colorPalette[colorIndex % colorPalette.length])
    colorIndex++
  }
  return serverColors.get(serverId)!
}

// 预设按分组
const groupedPresets = computed(() => {
  const groups = new Map<string, any[]>()
  for (const preset of presets.value) {
    const g = preset.presetGroup || '未分组'
    if (!groups.has(g)) groups.set(g, [])
    groups.get(g)!.push(preset)
  }
  // 排序：生产 → 测试 → 预发 → 开发 → 其他 → 未分组
  const groupOrder = ['生产', '测试', '预发', '开发']
  const sorted = [...groups.entries()].sort(([a], [b]) => {
    const ai = groupOrder.indexOf(a)
    const bi = groupOrder.indexOf(b)
    if (ai !== -1 && bi !== -1) return ai - bi
    if (ai !== -1) return -1
    if (bi !== -1) return 1
    if (a === '未分组') return 1
    if (b === '未分组') return -1
    return a.localeCompare(b, 'zh')
  })
  return sorted.map(([presetGroup, items]) => ({ presetGroup, presets: items }))
})

// 构建命令
function buildCommand(preset: any): string {
  const paths = preset.logPath.split('\n').map((p: string) => p.trim()).filter((p: string) => p)
  const quotePath = (p: string) => {
    if (p.startsWith('~')) {
      // ~ 展开必须在引号外：$HOME'/rest'（bash 中 $HOME 展开 + 单引号保护其余部分）
      const rest = p.slice(1).replace(/'/g, "'\\''")
      return rest ? `$HOME'${rest}'` : '$HOME'
    }
    return `'${p.replace(/'/g, "'\\''")}'`
  }

  switch (preset.logType) {
    case 'file':
      // tail supports multiple files natively
      return `tail -n ${preset.maxLines} -f ${paths.map(quotePath).join(' ')}`
    case 'journalctl':
      // journalctl supports -u multiple times
      return `journalctl ${paths.map((u: string) => `-u ${quotePath(u)}`).join(' ')} -n ${preset.maxLines} -f --no-pager`
    case 'docker':
      // docker logs doesn't support multiple containers, chain them
      return paths.map((c: string) => `(echo "=== ${quotePath(c)} ===" && docker logs --tail ${preset.maxLines} -f ${quotePath(c)} 2>&1)`).join(' & ')
    case 'custom':
      return preset.logPath
    default:
      return `tail -n ${preset.maxLines} -f ${paths.map(quotePath).join(' ')}`
  }
}

// 检测日志级别
function detectLevel(content: string): string {
  const upper = content.toUpperCase()
  if (upper.includes('ERROR') || upper.includes('FATAL') || upper.includes('CRITICAL') || upper.includes('EXCEPTION')) return 'error'
  if (upper.includes('WARN') || upper.includes('WARNING')) return 'warn'
  if (upper.includes('DEBUG')) return 'debug'
  return 'info'
}

// 获取预设关键字
function getKeywordsFromPreset(): string {
  if (selectedPreset.value?.keywords?.length) {
    return selectedPreset.value.keywords.join(', ')
  }
  return ''
}

// 搜索结果高亮
function highlightSearchResult(content: string): string {
  if (typeof content !== 'string') return ''
  let result = content.replace(/</g, '&lt;').replace(/>/g, '&gt;')

  const kw = queryMode.value === 'search' ? searchKeyword.value : getKeywordsFromPreset()
  if (!kw?.trim()) return result

  const escapedKw = kw.trim().replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${escapedKw})`, 'gi')
  result = result.replace(regex, '<mark>$1</mark>')
  return result
}

// 显示的行（过滤）
const displayLines = computed(() => {
  if (queryMode.value === 'search') {
    return logLines.value
  }
  // 流式模式：按预设关键字过滤
  const keywords = getKeywordsFromPreset().split(',').map(k => k.trim()).filter(k => k)
  if (keywords.length === 0) return logLines.value
  return logLines.value.filter(line =>
    keywords.some(kw => line.content.toLowerCase().includes(kw.toLowerCase()))
  )
})

// 预设分组折叠
function togglePresetGroup(group: string) {
  if (collapsedPresetGroups.value.has(group)) {
    collapsedPresetGroups.value.delete(group)
  } else {
    collapsedPresetGroups.value.add(group)
  }
}

// 预设管理
function openNewPresetForm() {
  editingPreset.value = null
  presetForm.value = { name: '', presetGroup: '未分组', serverIds: [], logType: 'file', logPath: '', keywordsInput: '', maxLines: 500 }
  showPresetForm.value = true
}

function editPreset(preset: any) {
  editingPreset.value = preset.id
  presetForm.value = {
    name: preset.name,
    presetGroup: preset.presetGroup || '未分组',
    serverIds: [...preset.serverIds],
    logType: preset.logType,
    logPath: preset.logPath,
    keywordsInput: preset.keywords.join(', '),
    maxLines: preset.maxLines
  }
  showPresetForm.value = true
}

// 选择预设并查询
async function selectAndQuery(preset: any) {
  // 搜索模式下只选中预设
  if (queryMode.value === 'search') {
    selectedPreset.value = preset
    return
  }

  // 点击正在流的预设则停止
  if (selectedPreset.value?.id === preset.id && isStreaming.value) {
    await stopQuery()
    return
  }

  selectedPreset.value = preset
  if (isStreaming.value) {
    await stopQuery()
  }
  startQueryFromPreset(preset)
}

// 停止查询
async function stopQuery() {
  // 清空缓冲区和定时器
  if (logFlushTimer) {
    clearTimeout(logFlushTimer)
    logFlushTimer = null
  }
  logBuffer.length = 0
  pendingScroll = false

  // 先保存当前 streamId，避免被后续调用覆盖
  const id = streamId.value
  streamId.value = ''
  isStreaming.value = false

  try {
    if (id) await getTauriAPI().logsStopStream(id)
  } catch (e) {
    console.error('Failed to stop stream:', e)
  }
  followMode.value = false
}

// 从预设开始查询
async function startQueryFromPreset(preset: any) {
  // 清理旧缓冲区
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  pendingScroll = false

  streamId.value = `stream_${Date.now()}`
  logLines.value = []
  activeServers.value = new Set<string>()
  followMode.value = true
  lastScrollTop = 0
  showScrollBottom.value = false

  const command = buildCommand(preset)

  try {
    const result = await getTauriAPI().logsStartStream({
      streamId: streamId.value,
      serverIds: JSON.parse(JSON.stringify(preset.serverIds)),
      command
    })
    if (result?.success) {
      isStreaming.value = true
    } else {
      streamId.value = ''
      toast.error(result?.error || '启动日志流失败')
    }
  } catch (e: any) {
    console.error('Failed to start log stream:', e)
    streamId.value = ''
    toast.error('启动日志流失败: ' + e.message)
  }
}

// 执行搜索
async function doSearch() {
  if (!selectedPreset.value) {
    toast.warning('请先选择一个预设')
    return
  }
  if (!searchKeyword.value.trim()) {
    toast.warning('请输入搜索关键字')
    return
  }

  isSearching.value = true
  hasSearched.value = true
  logLines.value = []

  try {
    const result = await getTauriAPI().logSearch({
      preset_id: selectedPreset.value.id,
      keyword: searchKeyword.value.trim(),
      lines: searchContextLines.value
    })

    if (result?.matches) {
      for (const match of (result.matches || [])) {
        for (const m of match.lines) {
          logLines.value.push({
            id: `${match.serverId}-${m.lineNum}-${Date.now()}`,
            serverId: match.serverId,
            serverName: match.serverName,
            timestamp: Date.now(),
            content: m.content,
            level: detectLevel(m.content),
            isMatch: m.isMatch,
            lineNum: String(m.lineNum)
          })
        }
      }
      const totalMatches = result.matches?.reduce((s: number, m: any) => s + (m.matchCount || 0), 0) || 0
      toast.success(`搜索完成：${totalMatches} 个匹配，${logLines.value.length} 行结果`)
    } else {
      toast.error(result?.error || '搜索失败')
    }
  } catch (e: any) {
    console.error('Search failed:', e)
    toast.error('搜索失败: ' + e.message)
  } finally {
    isSearching.value = false
  }
}

// 日志行缓冲 — 批量添加减少 Vue 重渲染
const logBuffer: Array<{ serverId: string; serverName: string; line: string }> = []
let logFlushTimer: ReturnType<typeof setTimeout> | null = null

function scheduleFlush() {
  if (logFlushTimer) return
  logFlushTimer = setTimeout(() => {
    logFlushTimer = null
    if (logBuffer.length === 0) return
    const batch = logBuffer.splice(0, logBuffer.length)
    const newLines: Array<{ id: string; serverId: string; serverName: string; timestamp: number; content: string; level: string }> = []
    const now = Date.now()
    for (const data of batch) {
      if (!data?.line || typeof data.line !== 'string' || !data?.serverId) continue
      newLines.push({
        id: `${data.serverId}-${now}-${Math.random()}`,
        serverId: data.serverId,
        serverName: data.serverName,
        timestamp: now,
        content: data.line,
        level: detectLevel(data.line)
      })
      activeServers.value.add(data.serverId)
    }
    logLines.value.push(...newLines)
    if (logLines.value.length > 5000) {
      logLines.value = logLines.value.slice(-5000)
    }
    if (followMode.value) {
      nextTick(() => {
        requestAnimationFrame(() => {
          scrollToBottomSilent()
        })
      })
    }
  }, 50) // 50ms 批量 flush，约 20fps，兼顾流畅度和更新及时性
}

// 添加日志行 — 推入缓冲区，由批量 flush 处理
function addLogLine(data: { serverId: string; serverName: string; line: string }) {
  logBuffer.push(data)
  scheduleFlush()
}

// 静默滚动到底部（不触发 followMode 判断）
function scrollToBottomSilent() {
  if (!logContainer.value) {
    pendingScroll = false
    return
  }
  scrollingFromRAF = true
  logContainer.value.scrollTop = logContainer.value.scrollHeight
  lastScrollTop = logContainer.value.scrollTop
  // 使用微任务延迟重置标志，避免 onScroll 误判
  Promise.resolve().then(() => { scrollingFromRAF = false })
}

// 滚动事件
function onScroll() {
  if (!logContainer.value) return
  // 程序化滚动期间，忽略 onScroll
  if (scrollingFromRAF) return

  const { scrollTop, scrollHeight, clientHeight } = logContainer.value
  const atBottom = scrollHeight - scrollTop - clientHeight < 50
  const scrolledUp = scrollTop < lastScrollTop
  const scrollUpDistance = lastScrollTop - scrollTop

  // 只有用户主动向上滚动超过 150px 才退出跟踪（放宽阈值）
  if (followMode.value && scrolledUp && scrollUpDistance > 150 && isStreaming.value) {
    followMode.value = false
  }

  showScrollBottom.value = !followMode.value && !atBottom && isStreaming.value
  lastScrollTop = scrollTop
}

function scrollToBottom() {
  if (logContainer.value) {
    followMode.value = true
    scrollToBottomSilent()
    showScrollBottom.value = false
  }
}

function toggleFollowMode() {
  followMode.value = !followMode.value
  if (followMode.value) {
    scrollToBottomSilent()
    showScrollBottom.value = false
  }
}
// 切换查询模式
async function switchQueryMode(mode: 'stream' | 'search') {
  queryMode.value = mode
  logLines.value = []
  hasSearched.value = false

  if (mode === 'stream') {
    console.log("[switchQueryMode] called")
    // 切换到流式模式：先停止旧查询，如果已有选中的预设则自动启动流式查询
    if (isStreaming.value) {
      await stopQuery()
    }
    followMode.value = true
    if (selectedPreset.value) {
      await startQueryFromPreset(selectedPreset.value)
    }
  } else {
    // 切换到搜索模式：停止正在的流式查询
    if (isStreaming.value) {
      await stopQuery()
    }
    followMode.value = false
  }
  showScrollBottom.value = false
}

// 清除日志
function clearLogs() {
  logLines.value = []
  hasSearched.value = false
}

// 导出日志
function exportLogs() {
  const text = displayLines.value.map(l =>
    `[${l.serverName}] ${l.content}`
  ).join('\n')

  const blob = new Blob([text], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `logs_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.txt`
  a.click()
  URL.revokeObjectURL(url)
}

// 预设管理
async function loadPresets() {
  try {
    console.log("[loadPresets] called")
    presets.value = await getTauriAPI().logPresetsGetAll()
  } catch (e) {
    console.error('Failed to load presets:', e)
  }
}

async function savePreset() {
  if (!presetForm.value.name.trim()) {
    toast.warning('请输入预设名称')
    return
  }
  try {
    const data = {
      name: presetForm.value.name,
      presetGroup: presetForm.value.presetGroup || '未分组',
      serverIds: JSON.parse(JSON.stringify(presetForm.value.serverIds)),
      logPath: presetForm.value.logPath,
      logType: presetForm.value.logType,
      keywords: presetForm.value.keywordsInput.split(',').map((k: string) => k.trim()).filter((k: string) => k),
      maxLines: presetForm.value.maxLines
    }

    if (editingPreset.value) {
      await getTauriAPI().logPresetsUpdate(editingPreset.value, data)
    } else {
      await getTauriAPI().logPresetsAdd(data)
    }

    showPresetForm.value = false
    editingPreset.value = null
    presetForm.value = { name: '', presetGroup: '未分组', serverIds: [], logType: 'file', logPath: '', keywordsInput: '', maxLines: 500 }
    await loadPresets()
    toast.success('预设已保存')
  } catch (e: any) {
    console.error('Failed to save preset:', e)
    toast.error('保存预设失败: ' + (e.message || '未知错误'))
  }
}

async function deletePreset(id: string) {
  if (!confirm('确定删除此预设？')) return
  try {
    console.log("[deletePreset] called")
    await getTauriAPI().logPresetsDelete(id)
    if (selectedPreset.value?.id === id) {
      if (isStreaming.value) await stopQuery()
      selectedPreset.value = null
    }
    await loadPresets()
    toast.success('预设已删除')
  } catch (e: any) {
    console.error('Failed to delete preset:', e)
    toast.error('删除预设失败: ' + (e.message || '未知错误'))
  }
}

async function goToServers() {
  const { useAppStore } = await import('../stores/appStore')
  const appStore = useAppStore()
  appStore.setViewMode('servers')
}

async function loadServers() {
  try {
    console.log("[loadServers] called")
    allServers.value = (await getTauriAPI().getAllServers()) || []
    allGroups.value = (await getTauriAPI().getServerGroups?.()) || []
  } catch (e) {
    console.error('Failed to load servers:', e)
    allServers.value = []
    allGroups.value = []
  }
}

// 事件监听
const onLineHandler = (data: any) => { if (data?.streamId === streamId.value) addLogLine(data) }
const onEndHandler = (data: any) => {
  if (!data?.serverId) return
  activeServers.value.delete(data.serverId)
  if (activeServers.value.size === 0) {
    isStreaming.value = false
    streamId.value = ''
  }
}
const onErrorHandler = (data: any) => {
  console.error(`[Log Error] ${data?.serverId}:`, data?.error)
}
const onStreamStoppedHandler = (data: any) => {
  if (data?.streamId === streamId.value) {
    streamId.value = ''
    isStreaming.value = false
  }
}

let cleanupLogsLine: (() => void) | null = null
let cleanupLogsServerEnd: (() => void) | null = null
let cleanupLogsError: (() => void) | null = null
let cleanupStreamStopped: (() => void) | null = null
let _cleanupDataChanged: (() => void) | undefined

onMounted(async () => {
    console.log("[components/LogAggregator.vue] mounted")
  await Promise.all([loadPresets(), loadServers()])

  /* Event listeners for log streaming from Tauri backend */
  cleanupLogsLine = await getTauriAPI().onLogsLine(onLineHandler);
  cleanupLogsServerEnd = await getTauriAPI().onLogsServerEnd(onEndHandler);
  cleanupLogsError = await getTauriAPI().onLogsError(onErrorHandler);
  cleanupStreamStopped = await getTauriAPI().onLogsStreamStopped(onStreamStoppedHandler);

  /* TODO(tauri-events): _cleanupDataChanged = getTauriAPI().onDataChanged?.(({ type }) => {
    if (type === 'servers') loadServers()
  })
  */})

onUnmounted(async () => {
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  if (isStreaming.value && streamId.value) {
    try { await getTauriAPI().logsStopStream(streamId.value) } catch {}
  }
  cleanupLogsLine?.()
  cleanupLogsServerEnd?.()
  cleanupLogsError?.()
  cleanupStreamStopped?.()
  _cleanupDataChanged?.()
  serverColors.clear()
  colorIndex = 0
})
</script>

<style scoped>
.log-aggregator {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
  background: var(--color-base-200);
  color: var(--text-primary);
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.log-header h2 {
  margin: 0;
  font-size: 20px;
}

.btn-add-preset {
  background: var(--color-primary);
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}

.log-layout {
  display: flex;
  gap: 16px;
  flex: 1;
  min-height: 0;
}

.log-sidebar {
  width: 260px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
}

.preset-section {
  background: var(--color-base-100);
  border-radius: 8px;
  padding: 12px;
}

.preset-section h3 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--text-secondary);
}

/* 预设分组 */
.preset-group {
  margin-bottom: 4px;
}

.preset-group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  cursor: pointer;
  user-select: none;
  border-radius: 4px;
  transition: background 0.2s;
}

.preset-group-header:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.group-toggle {
  font-size: 10px;
  color: var(--text-secondary);
  min-width: 10px;
}

.group-label {
  font-weight: 600;
  font-size: 12px;
  color: var(--text-secondary);
  flex: 1;
}

.group-count {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.6;
}

.preset-group-body {
  padding-left: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.preset-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.preset-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  background: var(--color-base-200);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.preset-item:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.preset-item.active {
  background: var(--color-primary);
  color: white;
}

.preset-item.streaming::after {
  content: '';
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 8px;
  height: 8px;
  background: #4ade80;
  border-radius: 50%;
  animation: pulse-dot 1.5s infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; transform: translateY(-50%) scale(1); }
  50% { opacity: 0.5; transform: translateY(-50%) scale(1.3); }
}

.preset-info {
  display: flex;
  flex-direction: column;
  margin-right: 28px;
}

.preset-name {
  font-weight: 500;
}

.preset-meta {
  font-size: 12px;
  opacity: 0.7;
}

.btn-edit-preset {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  font-size: 14px;
  opacity: 0.5;
  padding: 2px;
}

.btn-edit-preset:hover {
  opacity: 1;
}

.btn-delete-preset {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  font-size: 18px;
  opacity: 0.5;
  padding: 2px;
}

.btn-delete-preset:hover {
  opacity: 1;
  color: #f87171;
}

.empty-presets {
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
  padding: 12px;
}

.form-group {
  margin-bottom: 12px;
}

.form-group label {
  display: block;
  font-size: 13px;
  margin-bottom: 4px;
  color: var(--text-secondary);
}

.form-input, .form-select {
  width: 100%;
  padding: 6px 10px;
  border-radius: 4px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-200);
  color: var(--text-primary);
  font-size: 13px;
}
.form-textarea {
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  resize: vertical;
  line-height: 1.5;
}

.form-input.small {
  width: 80px;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--color-base-100);
  padding: 20px;
  border-radius: 12px;
  width: 680px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-content h3 {
  margin: 0 0 16px 0;
}

.modal-content .form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.btn-cancel {
  background: none;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-primary);
}

.btn-save {
  background: var(--color-primary);
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
}

.log-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--color-base-100);
  border-radius: 8px;
  overflow: hidden;
}

/* 模式切换栏 */
.mode-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-100);
}

.mode-tabs {
  display: flex;
  gap: 2px;
  background: var(--color-base-200);
  border-radius: 6px;
  padding: 3px;
}

.mode-tab {
  padding: 6px 14px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.mode-tab.active {
  background: var(--color-primary);
  color: white;
  font-weight: 500;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.search-input {
  flex: 1;
  padding: 6px 10px;
  border-radius: 4px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-200);
  color: var(--text-primary);
  font-size: 13px;
}

.search-options {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.context-controls {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.context-btn {
  padding: 3px 8px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  background: var(--color-base-200);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.context-btn:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--text-primary);
  border-color: var(--color-primary);
}

.context-btn.active {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.context-input {
  width: 60px;
  padding: 3px 6px;
  border-radius: 4px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-200);
  color: var(--text-primary);
  font-size: 12px;
  text-align: center;
}

.context-hint {
  font-size: 11px;
}

.btn-search {
  padding: 6px 16px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
}

.btn-search:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.log-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  font-size: 13px;
  flex-wrap: wrap;
  gap: 8px;
}

.log-stats {
  color: var(--text-secondary);
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.current-preset {
  color: var(--color-primary);
  font-weight: 500;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 跟踪按钮 */
.btn-follow {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: none;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-follow:hover {
  background: var(--color-base-200);
  color: var(--text-primary);
}

.btn-follow.active {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
  animation: follow-pulse 2s infinite;
}

@keyframes follow-pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(var(--primary-rgb, 74, 222, 128), 0.4); }
  50% { box-shadow: 0 0 0 4px rgba(var(--primary-rgb, 74, 222, 128), 0); }
}

.btn-stop {
  background: #ef4444;
  color: white;
  border: none;
  padding: 4px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  animation: pulse-stop 1.5s infinite;
}

.btn-stop:hover {
  background: #dc2626;
}

@keyframes pulse-stop {
  0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
  50% { box-shadow: 0 0 0 4px rgba(239, 68, 68, 0); }
}

.btn-clear, .btn-export {
  background: none;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
}

.btn-clear:hover, .btn-export:hover {
  background: var(--color-base-200);
}

.log-content {
  flex: 1;
  overflow-y: auto;
  max-height: 100vh;
  padding: 8px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  line-height: 1.6;
}

.log-empty, .search-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
}

.log-line {
  display: flex;
  gap: 8px;
  padding: 2px 0;
}

.log-line:hover {
  background: rgba(255, 255, 255, 0.05);
}

.log-server {
  min-width: 80px;
  font-weight: 500;
}

.log-content-text {
  flex: 1;
  white-space: pre-wrap;
  word-break: break-all;
}

.level-error .log-content-text {
  color: #f87171;
}

.level-warn .log-content-text {
  color: #fbbf24;
}

.level-debug .log-content-text {
  color: #6b7280;
}

.log-content-text mark {
  background: #fbbf24;
  color: #000;
  padding: 0 2px;
  border-radius: 2px;
}

.log-line.is-match {
  background: rgba(251, 191, 36, 0.08);
  border-left: 3px solid #fbbf24;
}

.log-line-num {
  color: var(--text-secondary);
  min-width: 50px;
  font-size: 11px;
  opacity: 0.6;
  text-align: right;
}

/* 回到底部浮动按钮 */
.btn-scroll-bottom {
  position: absolute;
  bottom: 16px;
  right: 16px;
  z-index: 10;
  padding: 8px 14px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 20px;
  font-size: 12px;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  transition: all 0.2s;
  animation: pulse-glow 2s infinite;
}

.btn-scroll-bottom:hover {
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
}

@keyframes pulse-glow {
  0%, 100% { box-shadow: 0 2px 8px rgba(0,0,0,0.3); }
  50% { box-shadow: 0 2px 16px rgba(74, 222, 128, 0.4); }
}
</style>
