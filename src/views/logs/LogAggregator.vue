<template>
  <div class="h-full flex flex-col p-4 bg-base-200 text-base-content">
    <div class="flex gap-4 flex-1 min-h-0">
      <!-- 左侧：预设列表（按分组展示） -->
      <div class="w-[260px] flex flex-col gap-4 overflow-y-auto">
        <div class="bg-base-100 rounded-box p-3">
          <h3 class="text-sm text-base-content/70 mb-3 font-medium">查询预设</h3>

          <!-- 分组 -->
          <div
            v-for="groupEntry in groupedPresets"
            :key="groupEntry.presetGroup"
            class="mb-1"
          >
            <div class="flex items-center gap-1.5 px-2 py-1 cursor-pointer select-none rounded transition-colors duration-200 hover:bg-primary/10" @click="togglePresetGroup(groupEntry.presetGroup)">
              <span class="text-[10px] text-base-content/60 min-w-[10px] inline-flex items-center">
                <SvgIcon v-if="collapsedPresetGroups.has(groupEntry.presetGroup)" name="chevronRight" size="10" />
                <SvgIcon v-else name="chevronDown" size="10" />
              </span>
              <span class="font-semibold text-xs text-base-content/60 flex-1">{{ groupEntry.presetGroup }}</span>
              <span class="text-[11px] text-base-content/60 opacity-60">{{ groupEntry.presets.length }}</span>
            </div>
            <div v-show="!collapsedPresetGroups.has(groupEntry.presetGroup)" class="pl-1 flex flex-col gap-0.5">
              <div
                v-for="preset in groupEntry.presets"
                :key="preset.id"
                class="flex justify-between items-center px-2.5 py-2 bg-base-200 rounded-lg cursor-pointer transition-all duration-200 relative"
                :class="{ 'bg-primary text-primary-content': selectedPreset?.id === preset.id }"
                @click="selectAndQuery(preset)"
              >
                <div class="flex flex-col mr-7">
                  <span class="font-medium">{{ preset.name }}</span>
                  <span class="text-xs opacity-70">{{ preset.serverIds.length }} 节点 · {{ preset.logType }}</span>
                </div>
                <button @click.stop="editPreset(preset)" class="btn btn-ghost btn-xs opacity-50 hover:opacity-100" title="编辑"><SvgIcon name="pencil" size="14" /></button>
                <button @click.stop="deletePreset(preset.id)" class="btn btn-ghost btn-xs opacity-50 hover:opacity-100 hover:text-error" title="删除"><SvgIcon name="x" size="14" /></button>
                <span v-if="isStreaming && selectedPreset?.id === preset.id" class="absolute right-2 top-1/2 -translate-y-1/2 w-2 h-2 bg-green-400 rounded-full animate-pulse"></span>
              </div>
            </div>
          </div>

          <div v-if="presets.length === 0" class="text-center text-base-content/60 text-xs p-3">
            <template v-if="allServers.length === 0">
              <div class="text-center">
                <p><SvgIcon name="monitor" size="14" class="inline" /> 尚未配置服务器</p>
                <p class="text-xs opacity-70 mt-1">日志聚合需要先添加 SSH 服务器：</p>
                <button @click="goToServers" class="btn btn-primary btn-sm mt-2">前往配置服务器 <SvgIcon name="arrowRight" size="14" class="inline" /></button>
              </div>
            </template>
            <template v-else>
              暂无预设，点击上方按钮添加
            </template>
          </div>
        </div>
      </div>

      <!-- 右侧：日志输出 -->
      <div class="flex-1 flex flex-col min-h-0 bg-base-100 rounded-box overflow-hidden relative">
        <!-- 查询模式切换栏 -->
        <div class="flex items-center gap-3 px-3 py-2 border-b border-base-content/10 bg-base-100">
          <div class="flex gap-0.5 bg-base-200 rounded-lg p-0.5">
            <button
              :class="['btn btn-sm rounded text-xs transition-all', queryMode === 'stream' ? 'btn-primary' : 'btn-ghost text-base-content/60']"
              @click="switchQueryMode('stream')"
            ><SvgIcon name="send" size="14" /> 流式查询</button>
            <button
              :class="['btn btn-sm rounded text-xs transition-all', queryMode === 'search' ? 'btn-primary' : 'btn-ghost text-base-content/60']"
              @click="switchQueryMode('search')"
            ><SvgIcon name="search" size="14" /> 日志搜索</button>
          </div>
          <button @click="openNewPresetForm" class="btn btn-primary btn-sm">+ 新增预设</button>

          <!-- 搜索模式：关键字输入 -->
          <div v-if="queryMode === 'search'" class="flex items-center gap-2 flex-1">
            <input
              v-model="searchKeyword"
              :placeholder="searchPlaceholder"
              class="input input-bordered flex-1 h-8 min-h-0 text-xs"
              @keyup.enter="doSearch"
            />
            <div class="flex items-center gap-1.5 text-xs text-base-content/60 whitespace-nowrap">
              <label>
                上下文行数
                <div class="inline-flex items-center gap-1">
                  <button
                    :class="['btn btn-xs rounded', searchContextLines === 0 ? 'btn-primary' : 'btn-ghost border border-base-content/10']"
                    @click="searchContextLines = 0"
                    title="精准搜索（仅匹配行）"
                  >0行</button>
                  <button
                    :class="['btn btn-xs rounded', searchContextLines === 10 ? 'btn-primary' : 'btn-ghost border border-base-content/10']"
                    @click="searchContextLines = 10"
                    title="模糊搜索（匹配行上下各10行）"
                  >±10行</button>
                  <input
                    v-model.number="searchContextLines"
                    type="number"
                    min="0"
                    max="500"
                    class="input input-bordered w-[60px] h-7 min-h-0 text-xs text-center px-1"
                    placeholder="自定义"
                  />
                </div>
              </label>
              <span class="text-[11px]">{{ searchContextLines === 0 ? '精确匹配' : `匹配行上下各 ${searchContextLines} 行` }}</span>
            </div>
            <button
              @click="doSearch"
              :disabled="!selectedPreset || !searchKeyword.trim() || isSearching"
              class="btn btn-primary btn-sm whitespace-nowrap"
            >
              <template v-if="isSearching"><SvgIcon name="refresh" size="14" :class="{ 'animate-spin': isSearching }" /> 搜索中...</template><template v-else><SvgIcon name="search" size="14" /> 搜索</template>
            </button>
          </div>
        </div>

        <div class="flex justify-between items-center px-3 py-2 border-b border-base-content/10 text-xs flex-wrap gap-2">
          <div class="text-base-content/60 flex gap-1 flex-wrap">
            <template v-if="queryMode === 'stream' && selectedPreset?.keywords?.length">
              <span>{{ displayLines.length }} 行</span>
              <span class="text-base-content/30">/</span>
              <span class="text-base-content/40">{{ logLines.length }} 行(全部)</span>
            </template>
            <template v-else-if="queryMode === 'search' && hasSearched">
              <span>{{ matchIndices.length }} 个匹配</span>
              <span class="text-base-content/30">/</span>
              <span class="text-base-content/40">{{ displayLines.length }} 行</span>
              <!-- 搜索导航 -->
              <span class="ml-2 flex items-center gap-1">
                <button @click="prevMatch" :disabled="matchIndices.length === 0 || currentMatchIndex <= 0"
                  class="btn btn-ghost btn-xs px-1" title="上一个匹配 (Shift+N)">
                  <SvgIcon name="chevronUp" size="12" />
                </button>
                <span class="text-xs font-mono min-w-[40px] text-center">
                  {{ matchIndices.length > 0 ? currentMatchIndex + 1 : 0 }}/{{ matchIndices.length }}
                </span>
                <button @click="nextMatch" :disabled="matchIndices.length === 0 || currentMatchIndex >= matchIndices.length - 1"
                  class="btn btn-ghost btn-xs px-1" title="下一个匹配 (N)">
                  <SvgIcon name="chevronDown" size="12" />
                </button>
              </span>
            </template>
            <template v-else>
              <span>{{ displayLines.length }} 行</span>
            </template>
            <span v-if="activeServers.size > 0">· {{ activeServers.size }} 个节点在线</span>
            <span v-if="selectedPreset" class="text-primary font-medium">· 当前：{{ selectedPreset.name }}</span>
          </div>
          <div class="flex items-center gap-2">
            <button
              @click="stopQuery"
              v-if="isStreaming"
              class="btn btn-error btn-sm animate-pulse"
              title="终止当前日志查询"
            >
              <SvgIcon name="stopSquare" size="14" /> 终止
            </button>
            <button
              @click="resumeQuery"
              v-if="!isStreaming && selectedPreset && queryMode === 'stream' && logLines.length > 0"
              class="btn btn-primary btn-sm"
              title="继续查询同预设"
            >
              <SvgIcon name="refresh" size="14" /> 继续
            </button>
            <button @click="clearLogs" class="btn btn-ghost btn-sm border border-base-content/10">清除</button>
            <button @click="exportLogs" class="btn btn-ghost btn-sm border border-base-content/10"><SvgIcon name="download" size="14" /> 导出</button>
            <button @click="downloadRemoteLogs" v-if="selectedPreset && selectedPreset.logType === 'file'" class="btn btn-ghost btn-sm border border-base-content/10" title="下载远程日志文件到本地"><SvgIcon name="download" size="14" /> 下载日志</button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto p-2 font-mono text-xs leading-relaxed allow-select" ref="logContainer" @scroll="onScroll">
          <div v-if="displayLines.length === 0 && !isStreaming && !hasSearched" class="flex items-center justify-center h-full text-base-content/60">
            <p v-if="queryMode === 'stream'">选择左侧预设开始查询日志</p>
            <p v-else>输入关键字后点击搜索</p>
          </div>

          <div v-if="queryMode === 'search' && searchKeyword.trim() && !isSearching && displayLines.length === 0 && hasSearched" class="flex items-center justify-center h-full text-base-content/60">
            <p>未找到匹配结果</p>
          </div>

          <!-- 虚拟滚动容器：用 paddingTop/Bottom 撑起全量滚动空间，只渲染视口附近的行 -->
          <div v-if="displayLines.length > 0" :style="{ paddingTop: (visibleStart * VIRTUAL_LINE_HEIGHT) + 'px', paddingBottom: ((totalItems - visibleEnd) * VIRTUAL_LINE_HEIGHT) + 'px' }">
            <!-- 滚动到顶部时自动加载更多历史日志 -->
            <div v-if="loadingMore" class="text-center py-2 text-base-content/40 text-[10px]">
              <SvgIcon name="refresh" size="12" class="inline-block animate-spin" /> 加载更多历史日志...
            </div>
            <div
              v-for="line in visibleLines"
              :key="line.id"
              class="flex gap-2 py-0.5 hover:bg-white/5"
              :class="{
                'bg-warning/10 border-l-4 border-warning': line.isMatch,
                'bg-primary/20 border-l-4 border-primary': line.id === currentMatchId
              }"
            >
              <span v-if="queryMode === 'search'" class="text-base-content/60 min-w-[50px] text-[11px] opacity-60 text-right">{{ line.lineNum || '' }}</span>
              <span class="min-w-[80px] font-medium" :style="{ color: getServerColor(line.serverId) }">[{{ line.serverName }}]</span>
              <span
                class="log-line-text flex-1 whitespace-pre-wrap break-all"
                :class="{ 'text-error': line.level === 'error', 'text-warning': line.level === 'warn', 'text-base-content/40': line.level === 'debug' }"
                v-html="highlightSearchResult(line.content)"
              ></span>
            </div>
          </div>
        </div>

        <!-- 回到底部浮动按钮 -->
        <button
          v-if="queryMode === 'stream' && showScrollBottom"
          @click="scrollToBottom"
          class="btn btn-primary btn-sm rounded-full absolute bottom-4 right-4 z-10 shadow-lg hover:scale-105 hover:shadow-xl transition-all"
          title="回到底部"
        >
          <SvgIcon name="arrowDown" size="14" /> 回到底部
        </button>
      </div>
    </div>

    <!-- 预设表单弹窗 -->
    <div v-if="showPresetForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000]">
      <div class="bg-base-100 p-5 rounded-2xl w-[680px] max-h-[80vh] overflow-y-auto relative">
        <button @click="showPresetForm = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭">
          <SvgIcon name="x" size="16" />
        </button>
        <h3 class="mt-0 mb-4 text-lg font-semibold">{{ editingPreset ? '编辑预设' : '新增预设' }}</h3>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">预设名称</label>
          <input v-model="presetForm.name" placeholder="例如：API服务日志" class="input input-bordered w-full" />
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">分组</label>
          <input
            v-model="presetForm.presetGroup"
            list="group-suggestions"
            placeholder="例如：生产 / 测试"
            class="input input-bordered w-full"
          />
          <datalist id="group-suggestions">
            <option value="生产" />
            <option value="测试" />
            <option value="开发" />
            <option value="预发" />
          </datalist>
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">服务器</label>
          <GroupedServerSelector
            :servers="allServers"
            :groups="allGroups"
            v-model="presetForm.serverIds"
            mode="multi"
          />
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">日志类型</label>
          <select v-model="presetForm.logType" class="select select-bordered w-full">
            <option value="file">文件 (tail)</option>
            <option value="journalctl">Journalctl</option>
            <option value="docker">Docker</option>
            <option value="custom">自定义</option>
          </select>
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">日志路径 / 容器名</label>
          <textarea v-model="presetForm.logPath" placeholder="/var/log/app/api.log&#10;/var/log/app/error.log&#10;(每行一个路径)" class="textarea textarea-bordered w-full font-mono resize-y leading-relaxed min-h-[60px]" rows="3" />
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">关键字（逗号分隔）</label>
          <input v-model="presetForm.keywordsInput" placeholder="ERROR, Exception" class="input input-bordered w-full" />
        </div>
        <div class="mb-3">
          <label class="block text-xs mb-1 text-base-content/60">初始行数</label>
          <input v-model.number="presetForm.maxLines" type="number" min="50" max="50000" class="input input-bordered w-20" />
        </div>
        <div class="flex justify-end gap-2 mt-5">
          <button @click="showPresetForm = false" class="btn btn-ghost">取消</button>
          <button @click="savePreset" class="btn btn-primary">保存</button>
        </div>
      </div>
    </div>
  </div>

    <!-- 确认删除对话框 -->
    <dialog ref="deleteConfirmDialog" class="modal">
      <div class="modal-box max-w-sm relative">
        <button @click="cancelDeletePreset" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭">
          <SvgIcon name="x" size="16" />
        </button>
        <h3 class="text-lg font-bold flex items-center gap-2">
          <SvgIcon name="alertTriangle" size="18" class="text-warning" />
          <span>确认删除</span>
        </h3>
        <p class="py-3 text-sm">{{ deleteConfirmMessage }}</p>
        <div class="modal-action">
          <button class="btn btn-error btn-sm" @click="executeDeletePreset"><SvgIcon name="trash" size="14" /> 确认删除</button>
          <button class="btn btn-sm" @click="cancelDeletePreset">取消</button>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button @click="cancelDeletePreset">close</button></form>
    </dialog>
</template>

<script setup lang="ts">// @ts-nocheck
defineOptions({ name: 'LogAggregator' })
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, onMounted, onUnmounted, onActivated, onDeactivated, nextTick, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast'
import type { Server } from '../../types'
import GroupedServerSelector from '../server/GroupedServerSelector.vue'

const toast = useToast()

// 状态
const presets = ref<any[]>([])
const allServers = ref<Server[]>([])
const allGroups = ref<Array<{ id: string; name: string; color: string; parentId: string | null }>>([])
const selectedPreset = ref<any | null>(null)
const logLines = ref<Array<{ id: string; serverId: string; serverName: string; timestamp: number; content: string; level: string; isMatch?: boolean; matched?: boolean; lineNum?: string }>>([])
const isStreaming = ref(false)
const followMode = ref(true)
const userScrolledUp = ref(false)
const activeServers = ref(new Set<string>())
const streamId = ref('')
const logContainer = ref<HTMLElement | null>(null)
let scrollingFromRAF = false
let pendingScroll = false

// 查询模式
const queryMode = ref<'stream' | 'search'>('stream')

// 搜索模式状态
const searchKeyword = ref('')
const searchContextLines = ref(10)
const isSearching = ref(false)
const hasSearched = ref(false)
// 搜索导航
const matchIndices = ref<number[]>([])
const currentMatchIndex = ref(-1)
const currentMatchId = ref<string | null>(null)
// 加载更多历史日志
const loadingMore = ref(false)

// 下载状态
const isDownloadingLog = ref(false)

// 滚动状态
const showScrollBottom = ref(false)
// 虚拟滚动：只渲染可视区内的行
const VIRTUAL_LINE_HEIGHT = 24    // px，匹配 contain-intrinsic-size: 1.5rem
const OVERSCAN = 100               // 视口上下额外渲染的行数
const scrollTop = ref(0)
const containerHeight = ref(0)

const totalItems = computed(() => displayLines.value.length)

const visibleStart = computed(() => {
  return Math.max(0, Math.floor(scrollTop.value / VIRTUAL_LINE_HEIGHT) - OVERSCAN)
})

const visibleEnd = computed(() => {
  return Math.min(totalItems.value, Math.ceil((scrollTop.value + containerHeight.value) / VIRTUAL_LINE_HEIGHT) + OVERSCAN)
})

// 模板只渲染这部分行，其余行用 paddingTop/paddingBottom 撑开滚动空间
const visibleLines = computed(() => {
  return displayLines.value.slice(visibleStart.value, visibleEnd.value)
})

// 预设分组折叠状态
const collapsedPresetGroups = ref(new Set<string>())

// 预设表单
const showPresetForm = ref(false)
const editingPreset = ref<string | null>(null)

// 确认删除
const deleteConfirmDialog = ref<HTMLDialogElement | null>(null)
const deleteConfirmMessage = ref('')
let pendingDeletePresetId: string | null = null

function deletePreset(id: string) {
  const preset = presets.value.find(p => p.id === id)
  if (!preset) {return}
  pendingDeletePresetId = id
  deleteConfirmMessage.value = `确定删除预设"${preset.name}"？`
  deleteConfirmDialog.value?.showModal()
}

function executeDeletePreset() {
  if (!pendingDeletePresetId) {return}
  const id = pendingDeletePresetId
  pendingDeletePresetId = null
  deleteConfirmDialog.value?.close()
  doDeletePreset(id)
}

function cancelDeletePreset() {
  pendingDeletePresetId = null
  deleteConfirmDialog.value?.close()
}

const presetForm = ref({
  name: '',
  presetGroup: '未分组',
  serverIds: [] as string[],
  logType: 'file' as 'file' | 'journalctl' | 'docker' | 'custom',
  logPath: '',
  keywordsInput: '',
  maxLines: 2000
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
    if (!groups.has(g)) {groups.set(g, [])}
    groups.get(g)!.push(preset)
  }
  // 排序：生产 → 测试 → 预发 → 开发 → 其他 → 未分组
  const groupOrder = ['生产', '测试', '预发', '开发']
  const sorted = [...groups.entries()].sort(([a], [b]) => {
    const ai = groupOrder.indexOf(a)
    const bi = groupOrder.indexOf(b)
    if (ai !== -1 && bi !== -1) {return ai - bi}
    if (ai !== -1) {return -1}
    if (bi !== -1) {return 1}
    if (a === '未分组') {return 1}
    if (b === '未分组') {return -1}
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
  if (upper.includes('ERROR') || upper.includes('FATAL') || upper.includes('CRITICAL') || upper.includes('EXCEPTION')) {return 'error'}
  if (upper.includes('WARN') || upper.includes('WARNING')) {return 'warn'}
  if (upper.includes('DEBUG')) {return 'debug'}
  return 'info'
}

// 从日志行中解析时间戳，返回毫秒时间戳或 null
// 支持常见格式：2026-05-29 10:30:15, 2026-05-29T10:30:15, May 29 10:30:15 等
function parseLogTimestamp(content: string): number | null {
  // 只检查行首 40 个字符（时间戳通常在行首）
  const head = content.slice(0, 40)

  // 格式1: 2026-05-29 10:30:15 或 2026-05-29T10:30:15（可选毫秒）
  let m = head.match(/(\d{4})[-/](\d{2})[-/](\d{2})[T ](\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?/)
  if (m) {
    const d = new Date(`${m[1]}-${m[2]}-${m[3]}T${m[4]}:${m[5]}:${m[6]}${m[7] ? '.' + m[7].padEnd(3, '0') : ''}`)
    if (!isNaN(d.getTime())) {return d.getTime()}
  }

  // 格式2: May 29 10:30:15（syslog 风格）
  m = head.match(/([A-Z][a-z]{2})\s+(\d{1,2})\s+(\d{2}):(\d{2}):(\d{2})/)
  if (m) {
    const d = new Date(`${m[1]} ${m[2]} ${new Date().getFullYear()} ${m[3]}:${m[4]}:${m[5]}`)
    if (!isNaN(d.getTime())) {return d.getTime()}
  }

  // 格式3: 仅时间 10:30:15（无日期，用今天补全）
  m = head.match(/(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?/)
  if (m) {
    const now = new Date()
    const d = new Date(now.getFullYear(), now.getMonth(), now.getDate(), parseInt(m[1]), parseInt(m[2]), parseInt(m[3]), m[4] ? parseInt(m[4].padEnd(3, '0')) : 0)
    if (!isNaN(d.getTime())) {return d.getTime()}
  }

  return null
}

// 获取预设关键字
function getKeywordsFromPreset(): string {
  if (selectedPreset.value?.keywords?.length) {
    return selectedPreset.value.keywords.join(', ')
  }
  return ''
}

// 搜索结果高亮：缓存正则，仅在关键词变化时重建
const _highlightRegex = computed<RegExp | null>(() => {
  const kw = queryMode.value === 'search' ? searchKeyword.value : getKeywordsFromPreset()
  if (!kw?.trim()) {return null}
  const escapedKw = kw.trim().replace(/[.*+?^${}()|[\]\\]/g, String.fromCharCode(92) + '&')
  return new RegExp(`(${escapedKw})`, 'gi')
})

function highlightSearchResult(content: string): string {
  if (typeof content !== 'string') {return ''}
  let result = content.replace(/</g, '&lt;').replace(/>/g, '&gt;')
  const regex = _highlightRegex.value
  if (!regex) {return result}
  result = result.replace(regex, '<mark>$1</mark>')
  return result
}

// 搜索模式输入框占位提示
const searchPlaceholder = computed(() => {
  const preset = selectedPreset.value
  const kw = preset?.keywords?.length ? preset.keywords.join(', ') : ''
  if (kw) {return `搜索日志... 预设关键字：${kw}`}
  return '搜索关键字'
})

// 显示的行（过滤）
// 流式模式：使用 flush 时预计算的 matched 标记，避免每次重扫
// 优化：当无关键字时直接返回原数组（零拷贝），避免 computed 无谓重建
const displayLines = computed(() => {
  if (queryMode.value === 'search') {
    return logLines.value
  }
  // 流式模式：无关键字直接显示全部（零过滤开销）
  if (!selectedPreset.value?.keywords?.length) {
    return logLines.value
  }
  return logLines.value.filter(line => line.matched !== false)
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
  presetForm.value = { name: '', presetGroup: '未分组', serverIds: [], logType: 'file', logPath: '', keywordsInput: '', maxLines: 2000 }
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

// 当预设切换时，重新计算存量行的 matched 标记
function recalculateMatched() {
  const keywords = queryMode.value === 'stream' && selectedPreset.value?.keywords?.length
    ? selectedPreset.value.keywords.map((k: string) => k.toLowerCase())
    : []
  for (const line of logLines.value) {
    line.matched = keywords.length === 0 || keywords.some(kw => line.content.toLowerCase().includes(kw))
  }
}

// 选择预设并查询
async function selectAndQuery(preset: any) {
  // 搜索模式下只选中预设
  if (queryMode.value === 'search') {
    selectedPreset.value = preset
    return
  }

  // 如果已经是当前预设且正在流，不做任何事（只有停止按钮能中断）
  if (selectedPreset.value?.id === preset.id && isStreaming.value) {
    scrollToBottom()
    return
  }

  selectedPreset.value = preset
  recalculateMatched()
  if (isStreaming.value) {
    await stopQuery()
    // 小幅延迟，确保后端旧流完全清理后再启动新流
    await new Promise(r => setTimeout(r, 100))
  }
  await startQueryFromPreset(preset)
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
    if (id) {await getTauriAPI().logsStopStream(id)}
  } catch (e) {
    console.error('Failed to stop stream:', e)
  }
  followMode.value = false
  userScrolledUp.value = false
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
  userScrolledUp.value = false
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
  toast.info('正在搜索...')

  try {
    const result = await getTauriAPI().logSearch({
      preset_id: selectedPreset.value.id,
      keyword: searchKeyword.value.trim(),
      lines: searchContextLines.value
    })

    if (result?.matches) {
      for (const match of (result.matches || [])) {
        for (const m of match.lines) {
          const parsedTime = parseLogTimestamp(m.content)
          logLines.value.push({
            id: `${match.serverId}-${m.lineNum}-${Date.now()}`,
            serverId: match.serverId,
            serverName: match.serverName,
            timestamp: parsedTime ?? Date.now(),
            content: m.content,
            level: detectLevel(m.content),
            isMatch: m.isMatch,
            lineNum: String(m.lineNum),
            sortKey: parsedTime ?? Date.now()
          } as any)
        }
      }
	      // 清除之前的搜索导航状态
	      matchIndices.value = []
	      currentMatchIndex.value = -1
	      currentMatchId.value = null
	      const totalMatches = result.matches?.reduce((s: number, m: any) => s + (m.matchCount || 0), 0) || 0
	      toast.success(`搜索完成：${totalMatches} 个匹配，${logLines.value.length} 行结果`)
	      // 搜索完成后更新匹配索引
	      nextTick(() => updateMatchIndices())
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
  if (logFlushTimer) {return}
  logFlushTimer = setTimeout(() => {
    logFlushTimer = null
    if (logBuffer.length === 0) {return}
    const batch = logBuffer.splice(0, logBuffer.length)
    const len = logLines.value.length
    // 预分配数组空间，避免频繁扩容
    const newLines: Array<{ id: string; serverId: string; serverName: string; timestamp: number; content: string; level: string; matched?: boolean; sortKey: number }> = new Array(batch.length)
    const now = Date.now()
    // 预计算当前预设关键字（流式模式下只需计算一次）
    const presetKeywords = queryMode.value === 'stream' && selectedPreset.value?.keywords?.length
      ? selectedPreset.value.keywords.map((k: string) => k.toLowerCase())
      : []
    let validCount = 0
    for (let i = 0; i < batch.length; i++) {
      const data = batch[i]
      if (!data?.line || typeof data.line !== 'string' || !data?.serverId) {continue}
      const content = data.line
      const parsedTime = parseLogTimestamp(content)
      newLines[validCount++] = {
        id: `${data.serverId}-${now}-${Math.random()}`,
        serverId: data.serverId,
        serverName: data.serverName,
        timestamp: now,
        content,
        level: detectLevel(content),
        matched: presetKeywords.length === 0 || presetKeywords.some(kw => content.toLowerCase().includes(kw)),
        sortKey: parsedTime ?? (now + validCount * 0.001)
      }
      activeServers.value.add(data.serverId)
    }
	    newLines.length = validCount

	    // 批量追加（不排序，按服务器返回的原始顺序显示）
	    logLines.value.push(...newLines)

    // 智能裁剪：仅在超出上限时裁剪，避免每次 flush 都排序
    const MAX_LINES = 3000
    if (logLines.value.length > MAX_LINES) {
      const overflow = logLines.value.length - MAX_LINES
      // 原地删除头部多余元素，保持数组引用不变
      logLines.value.splice(0, overflow)
      // 同步调整虚拟滚动偏移量，避免裁剪后 visibleStart 索引错位（paddingTop 跳跃）
      if (!followMode.value && logContainer.value && scrollTop.value > 0) {
        const adjustedScroll = Math.max(0, scrollTop.value - overflow * VIRTUAL_LINE_HEIGHT)
        logContainer.value.scrollTop = adjustedScroll
        scrollTop.value = adjustedScroll
      }
    }
    if (followMode.value) {
      nextTick(() => {
        requestAnimationFrame(() => {
          scrollToBottomSilent()
        })
      })
    }
  }, 30) // 30ms flush，约 33fps，更流畅
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
  // 使用微任务延迟重置标志，避免 onScroll 误判
  Promise.resolve().then(() => { scrollingFromRAF = false })
}

function scrollToBottom() {
  if (logContainer.value) {
    userScrolledUp.value = false
    followMode.value = true
    scrollToBottomSilent()
    showScrollBottom.value = false
  }
}

// ── 搜索导航 ──
function updateMatchIndices() {
  matchIndices.value = []
  for (let i = 0; i < displayLines.value.length; i++) {
    if (displayLines.value[i].isMatch) {
      matchIndices.value.push(i)
    }
  }
}

function scrollToLineIndex(idx: number) {
  // scroll the target line into view
  const targetTop = idx * VIRTUAL_LINE_HEIGHT
  const halfVisible = containerHeight.value / 2
  if (logContainer.value) {
    scrollingFromRAF = true
    logContainer.value.scrollTop = Math.max(0, targetTop - halfVisible)
    Promise.resolve().then(() => { scrollingFromRAF = false })
  }
}

function nextMatch() {
  if (matchIndices.value.length === 0) return
  const next = (currentMatchIndex.value + 1) % matchIndices.value.length
  currentMatchIndex.value = next
  const idx = matchIndices.value[next]
  currentMatchId.value = displayLines.value[idx]?.id ?? null
  scrollToLineIndex(idx)
}

function prevMatch() {
  if (matchIndices.value.length === 0) return
  const prev = (currentMatchIndex.value - 1 + matchIndices.value.length) % matchIndices.value.length
  currentMatchIndex.value = prev
  const idx = matchIndices.value[prev]
  currentMatchId.value = displayLines.value[idx]?.id ?? null
  scrollToLineIndex(idx)
}

// ── 滚动到顶部自动加载更多历史日志 ──
// TODO: Requires backend logs_load_more Tauri command. For now, gracefully degrades.
async function loadMoreHistory() {
  if (queryMode.value !== 'stream' || !selectedPreset.value || loadingMore.value) return
  loadingMore.value = true
  try {
    // Placeholder - backend API not yet implemented
    console.log('[LogAggregator] loadMoreHistory triggered, count=', logLines.value.length)
    // TODO: await getTauriAPI().logsLoadMore(streamId.value, logLines.value.length)
  } catch (e) {
    console.warn('[LogAggregator] loadMoreHistory failed:', e)
  } finally {
    loadingMore.value = false
  }
}

// 滚动事件
function onScroll() {
  if (!logContainer.value) {return}
  // 程序化滚动期间，忽略 onScroll
  if (scrollingFromRAF) {return}

  const el = logContainer.value
  scrollTop.value = el.scrollTop
  // 仅在大小变化时更新，避免每帧赋值
  if (containerHeight.value !== el.clientHeight) {
    containerHeight.value = el.clientHeight
  }

  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 50

  // 用户向上滚动 → 停止自动追踪
  userScrolledUp.value = !atBottom && isStreaming.value
  // 只有用户手动滚动时才改变 followMode（程序化滚动被 scrollingFromRAF 拦截）
  if (!atBottom && isStreaming.value) {
    followMode.value = false
  } else if (atBottom) {
    followMode.value = true
  }

  showScrollBottom.value = userScrolledUp.value && isStreaming.value

  // 滚动到顶部时自动加载更多历史日志
  if (el.scrollTop < 50 && isStreaming.value && !loadingMore.value) {
    loadMoreHistory()
  }
}
// 继续查询（终止后重新启动同一预设，不清除日志）
async function resumeQuery() {
  if (!selectedPreset.value) {return}
  // 清理缓冲但不清除已有日志
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  pendingScroll = false

  streamId.value = `stream_${Date.now()}`
  activeServers.value = new Set<string>()
  followMode.value = true
  userScrolledUp.value = false
  showScrollBottom.value = false

  const command = buildCommand(selectedPreset.value)

  try {
    const result = await getTauriAPI().logsStartStream({
      streamId: streamId.value,
      serverIds: JSON.parse(JSON.stringify(selectedPreset.value.serverIds)),
      command
    })
    if (result?.success) {
      isStreaming.value = true
    } else {
      streamId.value = ''
      toast.error(result?.error || '启动日志流失败')
    }
  } catch (e: any) {
    console.error('Failed to resume log stream:', e)
    streamId.value = ''
    toast.error('启动日志流失败: ' + e.message)
  }
}

// 切换查询模式
async function switchQueryMode(mode: 'stream' | 'search') {
  queryMode.value = mode

  // 清理缓冲和定时器，避免旧流残留
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  pendingScroll = false
  logLines.value = []
  hasSearched.value = false

  if (mode === 'stream') {
    console.log("[switchQueryMode] called")
    // 切换到流式模式：先停止旧查询，如果已有选中的预设则自动启动流式查询
    if (isStreaming.value) {
      await stopQuery()
    }
    followMode.value = true
    recalculateMatched()
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
  const lines = logLines.value
  if (lines.length === 0) {
    toast.warning('没有可导出的日志')
    return
  }
  const text = lines.map(l => {
    const ts = l.timestamp ? new Date(l.timestamp).toISOString().slice(11, 19) : ''
    const shown = l.matched !== false
    return `[${ts}][${l.serverName}]${shown ? '' : ' [已过滤]'} ${l.content}`
  }).join('\n')

  const blob = new Blob([text], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `logs_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.txt`
  a.click()
  URL.revokeObjectURL(url)
  toast.success(`已导出 ${lines.length} 行日志`)
}

// 下载远程日志文件
async function downloadRemoteLogs() {
  if (!selectedPreset.value) {
    toast.warning('请先选择预设')
    return
  }
  if (!selectedPreset.value.logPath) {
    toast.warning('预设未配置日志路径')
    return
  }
  if (!selectedPreset.value.serverIds?.length) {
    toast.warning('预设未配置服务器')
    return
  }

  // 获取第一个日志路径
  const paths = selectedPreset.value.logPath.split('\n').filter(p => p.trim())
  if (paths.length === 0) {
    toast.warning('日志路径为空')
    return
  }
  const logPath = paths[0].trim()

  // 获取第一个服务器
  const serverId = selectedPreset.value.serverIds[0]
  const server = allServers.value.find(s => s.id === serverId)
  if (!server) {
    toast.warning('服务器不存在')
    return
  }

  try {
    isDownloadingLog.value = true
    toast.info(`正在从 ${server.name} 下载 ${logPath}...`)

    // 获取下载目录
    const downloadsDir = await getTauriAPI().getDownloadsDir()
    const fileName = logPath.split('/').pop() || 'log.txt'
    const timestamp = new Date().toISOString().slice(0, 10)
    const localPath = downloadsDir.endsWith('/') || downloadsDir.endsWith('\\')
      ? downloadsDir + `${server.name}_${timestamp}_${fileName}`
      : downloadsDir + '/' + `${server.name}_${timestamp}_${fileName}`

    // 下载文件
    await getTauriAPI().downloadFile(serverId, logPath, localPath)

    toast.success(`下载成功: ${localPath}`)

    // 打开文件所在目录
    await getTauriAPI().lanOpenFileFolder(localPath)
  } catch (error: any) {
    handleError(error, { context: 'downloadRemoteLogs' })
  } finally {
    isDownloadingLog.value = false
  }
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
    presetForm.value = { name: '', presetGroup: '未分组', serverIds: [], logType: 'file', logPath: '', keywordsInput: '', maxLines: 2000 }
    await loadPresets()
    toast.success('预设已保存')
  } catch (e: any) {
    console.error('Failed to save preset:', e)
    toast.error('保存预设失败: ' + (e.message || '未知错误'))
  }
}

async function doDeletePreset(id: string) {
  try {
    console.log("[deletePreset] called")
    await getTauriAPI().logPresetsDelete(id)
    if (selectedPreset.value?.id === id) {
      if (isStreaming.value) {await stopQuery()}
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
  const { useAppStore } = await import("../../stores/appStore");
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
const onLineHandler = (data: any) => { if (data?.streamId === streamId.value) {addLogLine(data)} }
const onEndHandler = (data: any) => {
  if (!data?.serverId) {return}
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
let _cleanupKeyDown: (() => void) | undefined

onMounted(async () => {
    console.log("[components/LogAggregator.vue] mounted")
  await Promise.all([loadPresets(), loadServers()])

  // 初始化虚拟滚动容器高度
  if (logContainer.value) {
    containerHeight.value = logContainer.value.clientHeight
  }

  /* Event listeners for log streaming from Tauri backend */
  cleanupLogsLine = await getTauriAPI().onLogsLine(onLineHandler);
  cleanupLogsServerEnd = await getTauriAPI().onLogsServerEnd(onEndHandler);
  cleanupLogsError = await getTauriAPI().onLogsError(onErrorHandler);
  cleanupStreamStopped = await getTauriAPI().onLogsStreamStopped(onStreamStoppedHandler);

  _cleanupDataChanged = await getTauriAPI().onDataChanged?.(({ type }: { type: string }) => {
    if (type === 'servers') {loadServers()}
  })

  // N/N 快捷键搜索导航
  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'n' || e.key === 'N') {
      if (queryMode.value !== 'search' || matchIndices.value.length === 0) return
      e.preventDefault()
      if (e.shiftKey) {
        prevMatch()
      } else {
        nextMatch()
      }
    }
  }
  window.addEventListener('keydown', onKeyDown)
  _cleanupKeyDown = () => window.removeEventListener('keydown', onKeyDown)
})

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

// 用于记录切出前是否有活跃查询，等用户切回后自动恢复
let wasQueryBeforeDeactivate = false

// 切到其他 tab 时暂停后台日志流，避免持续占用主线程（导致部署时页面卡死）
onDeactivated(() => {
  wasQueryBeforeDeactivate = isStreaming.value
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  logBuffer.length = 0
  if (isStreaming.value && streamId.value) {
    getTauriAPI().logsStopStream(streamId.value).catch(() => {})
    isStreaming.value = false
    streamId.value = ''
  }
})

// 切回时恢复日志流
onActivated(async () => {
  if (selectedPreset.value && wasQueryBeforeDeactivate) {
    await resumeQuery()
  }
})

</script>

<!-- 用于 v-html 渲染的 <mark> 标签样式（必须全局/非 scoped） -->
<style>
.log-line-text mark {
  background: #fbbf24;
  color: #000;
  padding: 0 2px;
  border-radius: 2px;
}
</style>
