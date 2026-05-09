<template>
  <div class="flex flex-col flex-1 min-w-[300px] overflow-hidden">
    <div class="flex items-center justify-between px-2.5 py-1.5 border-b border-base-content/10 bg-base-100 shrink-0">
      <span class="font-semibold text-[13px]">日志</span>
      <div class="flex items-center gap-1.5">
        <!-- View mode toggle: Table / Graph / Console -->
        <div class="flex items-center gap-0.5 p-0.5 bg-base-200 rounded-md border border-base-content/20">
          <button
            class="flex items-center justify-center w-7 h-6 p-0 border-0 bg-transparent text-base-content/60 rounded cursor-pointer hover:text-base-content hover:bg-[var(--hover-bg)]"
            :class="{ '!text-primary !bg-primary/10': logViewMode === 'table' }"
            @click="$emit('update:logViewMode', 'table')"
            title="表格视图"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <line x1="3" y1="9" x2="21" y2="9" />
              <line x1="3" y1="15" x2="21" y2="15" />
              <line x1="9" y1="3" x2="9" y2="21" />
            </svg>
          </button>
          <button
            class="flex items-center justify-center w-7 h-6 p-0 border-0 bg-transparent text-base-content/60 rounded cursor-pointer hover:text-base-content hover:bg-[var(--hover-bg)]"
            :class="{ '!text-primary !bg-primary/10': logViewMode === 'graph' }"
            @click="$emit('switch-to-graph-view')"
            title="图形视图 (Git Graph)"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="6" y1="3" x2="6" y2="21" />
              <circle cx="6" cy="6" r="2" fill="currentColor" />
              <circle cx="6" cy="12" r="2" fill="currentColor" />
              <path d="M6 8 Q12 8 12 12" />
              <circle cx="12" cy="12" r="2" fill="currentColor" />
              <path d="M12 14 Q12 18 6 18" />
            </svg>
          </button>
          <button
            class="flex items-center justify-center w-7 h-6 p-0 border-0 bg-transparent text-base-content/60 rounded cursor-pointer hover:text-base-content hover:bg-[var(--hover-bg)]"
            :class="{ '!text-primary !bg-primary/10': logViewMode === 'console' }"
            @click="$emit('update:logViewMode', 'console')"
            title="Git Console"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
          </button>
        </div>
        <div class="relative">
          <svg class="absolute left-2 top-1/2 -translate-y-1/2 text-base-content/60 pointer-events-none" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            :value="logSearch"
            @input="$emit('update:logSearch', ($event.target as HTMLInputElement).value)"
            class="py-0.5 px-2 pl-[26px] border border-base-content/20 rounded bg-base-200 text-base-content text-xs outline-none w-[150px] transition-[border-color] duration-150 focus:border-primary focus:w-[200px]"
            placeholder="搜索提交..."
            spellcheck="false"
          />
        </div>
        <select :value="logBranchFilter" @change="$emit('update:logBranchFilter', ($event.target as HTMLSelectElement).value)" class="py-0.5 px-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-xs outline-none cursor-pointer">
          <option value="">所有分支</option>
          <option v-for="b in localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
        </select>
        <input :value="logDateFrom" @input="$emit('update:logDateFrom', ($event.target as HTMLInputElement).value)" type="date" class="py-0.5 px-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[11px] outline-none w-[130px]" title="From date" />
        <input :value="logDateTo" @input="$emit('update:logDateTo', ($event.target as HTMLInputElement).value)" type="date" class="py-0.5 px-1.5 border border-base-content/20 rounded bg-base-200 text-base-content text-[11px] outline-none w-[130px]" title="To date" />
        <button class="btn btn-ghost btn-xs" @click="$emit('update:showAuthorFilter', !showAuthorFilter)" :class="{ '!text-primary !bg-primary/10': (selectedAuthors as Set<string>).size > 0 }" title="Filter by author">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" /><circle cx="12" cy="7" r="4" />
          </svg>
          Authors
        </button>
        <div v-if="showAuthorFilter" class="absolute top-full right-0 z-[100] bg-base-100 border border-base-content/10 rounded-btn shadow-lg min-w-[180px] max-h-[200px] overflow-y-auto p-1 mt-1" @click.stop>
          <div v-for="a in logAuthors" :key="a" class="flex items-center gap-1.5 px-2.5 py-1 text-xs cursor-pointer hover:bg-[var(--hover-bg)]" @click.stop="$emit('toggle-author', a)">
            <input type="checkbox" :checked="(selectedAuthors as Set<string>).has(a)" class="accent-primary" />
            <span>{{ getAuthorName(a) }}</span>
          </div>
          <div v-if="logAuthors.length === 0" class="p-2.5 text-center text-xs text-base-content/60">No authors loaded</div>
        </div>
        <button class="btn btn-ghost btn-xs" @click="$emit('load-log')" :disabled="loading" title="刷新日志">
          <svg :class="{ 'animate-spin': loading }" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10" />
            <polyline points="1 20 1 14 7 14" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 提交历史表格 -->
    <div v-if="logViewMode === 'table'" class="flex-1 overflow-y-auto overflow-x-hidden">
      <table class="w-full border-collapse text-xs">
        <thead>
          <tr>
            <th class="w-[30px] text-center"><input type="checkbox" :checked="(selectedLogCommits as Set<string>).size > 0 && (selectedLogCommits as Set<string>).size === filteredLog.length" @click="$emit('toggle-select-all-log-commits')" class="accent-primary shrink-0 w-3.5 h-3.5 cursor-pointer" title="Select all" /></th>
            <th class="w-[70px]">Hash</th>
            <th class="w-[120px]">Author</th>
            <th class="w-[100px]">Date</th>
            <th class="min-w-[200px]">Message</th>
            <th class="w-[50px] text-center">Files</th>
            <th class="w-[120px]">Refs</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="commit in filteredLog"
            :key="commit.hash"
            class="cursor-pointer transition-colors duration-100 hover:bg-[var(--hover-bg)]"
            :class="{ 'bg-primary/10': (selectedCommit as any)?.hash === commit.hash || (selectedLogCommits as Set<string>).has(commit.hash) }"
            @click="$emit('select-commit', commit)"
            @contextmenu.prevent="$emit('log-context-menu', { event: $event, commit })"
          >
            <td class="w-[30px] text-center">
              <input type="checkbox" :checked="(selectedLogCommits as Set<string>).has(commit.hash)" @click.stop="$emit('toggle-log-commit-select', commit.hash)" class="accent-primary shrink-0 w-3.5 h-3.5 cursor-pointer" />
            </td>
            <td class="w-[70px]">
              <code class="font-mono text-[11px] text-primary bg-primary/10 px-1 py-[1px] rounded-sm">{{ commit.hash.substring(0, 7) }}</code>
            </td>
            <td class="w-[120px]">
              <span class="text-base-content">{{ getAuthorName(commit.author) }}</span>
            </td>
            <td class="w-[100px]">
              <span class="text-base-content/60 text-[11px]" :title="formatFullDate(commit.date)">{{ formatRelativeDate(commit.date) }}</span>
            </td>
            <td class="min-w-[200px]">
              <span class="truncate block max-w-[560px]">{{ commit.message }}</span>
            </td>
            <td class="w-[50px] text-center">
              <span v-if="commit.fileCount !== undefined" class="inline-flex items-center justify-center min-w-5 h-[18px] px-1 rounded-full bg-primary/10 text-primary text-[10px] font-semibold font-mono" :title="commit.fileCount + ' 个文件'">
                {{ commit.fileCount }}
              </span>
            </td>
            <td class="w-[120px]">
              <span v-if="commit.refs" class="flex gap-[3px] flex-wrap">
                <span
                  v-for="(ref, idx) in parseRefs(commit.refs)"
                  :key="idx"
                  class="text-[10px] px-1 py-[1px] rounded-sm whitespace-nowrap"
                  :class="ref.includes('HEAD') ? 'bg-green-500/20 text-green-500' : ref.includes('origin') ? 'bg-purple-500/20 text-purple-500' : 'bg-blue-500/20 text-blue-500'"
                >
                  {{ ref.replace('HEAD -> ', '') }}
                </span>
              </span>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="filteredLog.length === 0 && !loading" class="flex items-center justify-center h-[100px] text-base-content/60">
        <p>没有提交记录</p>
      </div>
    </div>

    <!-- 加载更多 (table view only) -->
    <div v-if="logViewMode === 'table' && hasMoreLog" class="flex justify-center p-1.5 border-t border-base-content/10 bg-base-100">
      <button class="btn btn-ghost btn-sm" @click="$emit('load-more-log')" :disabled="loading">
        加载更多 ({{ logCount }}/{{ logTotalEstimate }})
      </button>
    </div>

    <!-- ===== Git Graph 图形视图 ===== -->
    <div v-if="logViewMode === 'graph'" class="relative flex-1 overflow-hidden bg-[#1a1a2e] rounded">
      <canvas
        ref="graphCanvasRef"
        class="block w-full h-full cursor-pointer"
        @mousemove="$emit('on-graph-mouse-move', $event)"
        @click="$emit('on-graph-click', $event)"
      ></canvas>
      <div v-if="graphLoading" class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex flex-col items-center gap-3 text-base-content/60">
        <svg class="animate-spin" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10" />
          <polyline points="1 20 1 14 7 14" />
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
        <span>加载中...</span>
      </div>
      <div v-if="!graphLoading && graphLog.length === 0" class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex flex-col items-center gap-3 text-base-content/60">
        <p>没有提交记录</p>
      </div>
    </div>

    <!-- ===== Git Console 控制台 ===== -->
    <div v-if="logViewMode === 'console'" class="flex flex-col flex-1 overflow-hidden bg-[#0d1117] rounded font-mono">
      <div class="flex-1 overflow-y-auto p-2 px-3 text-xs leading-relaxed" ref="consoleOutputRef">
        <div
          v-for="(line, idx) in consoleHistory"
          :key="idx"
          class="mb-3 pb-2 border-b border-white/5"
        >
          <div class="flex items-center gap-1.5 text-blue-400 font-medium">
            <span class="text-green-500 font-bold">λ</span>
            <span :class="line.isError ? 'text-[#f85149]' : 'text-[#e6edf3]'">{{ line.command }}</span>
          </div>
          <pre v-if="line.output" class="ml-[18px] mt-1 p-2 bg-white/5 rounded whitespace-pre-wrap break-all text-[10px] max-h-[300px] overflow-y-auto" :class="line.isError ? 'text-[#f85149] bg-[rgba(248,81,73,0.05)]' : 'text-[#c9d1d9]'">{{ line.output }}</pre>
        </div>
        <div v-if="consoleHistory.length === 0" class="p-5 text-center text-[#8b949e]">
          <p>Git Console — 输入任意 git 命令</p>
          <p class="text-[11px]">例如: <code class="bg-white/10 px-1.5 py-0.5 rounded text-blue-400">status</code>, <code class="bg-white/10 px-1.5 py-0.5 rounded text-blue-400">log --oneline -10</code>, <code class="bg-white/10 px-1.5 py-0.5 rounded text-blue-400">branch -vv</code></p>
        </div>
      </div>
      <div class="flex items-center gap-2 p-2 px-3 bg-white/5 border-t border-white/10">
        <span class="text-green-500 font-bold text-[13px] shrink-0">λ git</span>
        <input
          :value="consoleInput"
          @input="$emit('update:consoleInput', ($event.target as HTMLInputElement).value)"
          ref="consoleInputRef"
          class="flex-1 bg-transparent border-0 outline-none text-[#e6edf3] font-mono text-xs placeholder:text-[#484f58]"
          placeholder="输入 git 命令参数..."
          @keydown.enter="$emit('exec-console-command')"
          @keydown.up="$emit('console-history-up')"
          @keydown.down="$emit('console-history-down')"
          spellcheck="false"
          autocomplete="off"
        />
      </div>
    </div>

    <!-- 提交详情面板 -->
    <div v-if="selectedCommit" class="border-t border-base-content/10 bg-base-100 max-h-[40%] overflow-y-auto shrink-0">
      <div class="flex items-center justify-between px-2.5 py-1.5 border-b border-base-content/10">
        <span class="font-semibold text-xs">提交详情</span>
        <button class="btn btn-ghost btn-xs" @click="$emit('update:selectedCommit', null)" title="关闭"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
      </div>
      <div class="p-2 px-2.5">
        <div class="flex gap-2 mb-1 text-xs">
          <span class="font-semibold text-base-content/60 min-w-[50px] shrink-0">Hash</span>
          <code class="text-base-content font-mono text-[11px]">{{ (selectedCommit as any).hash }}</code>
        </div>
        <div class="flex gap-2 mb-1 text-xs">
          <span class="font-semibold text-base-content/60 min-w-[50px] shrink-0">Author</span>
          <span class="text-base-content">{{ (selectedCommit as any).author }}</span>
        </div>
        <div class="flex gap-2 mb-1 text-xs">
          <span class="font-semibold text-base-content/60 min-w-[50px] shrink-0">Date</span>
          <span class="text-base-content">{{ formatFullDate((selectedCommit as any).date) }}</span>
        </div>
        <div class="flex gap-2 mb-1 text-xs">
          <span class="font-semibold text-base-content/60 min-w-[50px] shrink-0">Message</span>
          <span class="text-base-content block">{{ (selectedCommit as any).message }}</span>
        </div>
        <div class="flex gap-2 mb-1 text-xs" v-if="(selectedCommit as any).refs">
          <span class="font-semibold text-base-content/60 min-w-[50px] shrink-0">Refs</span>
          <span class="text-base-content">{{ (selectedCommit as any).refs }}</span>
        </div>
      </div>
      <div class="border-t border-base-content/10 p-2 px-2.5">
        <div class="flex items-center justify-between mb-1.5">
          <span class="font-semibold text-xs">Diff</span>
          <button class="btn btn-ghost btn-xs" @click="$emit('load-commit-diff')" :disabled="loadingDiff">
            {{ loadingDiff ? '加载中...' : '查看 Diff' }}
          </button>
        </div>
        <pre v-if="commitDiff" class="bg-base-200 p-2.5 rounded font-mono text-[11px] leading-relaxed overflow-x-auto max-h-[300px] whitespace-pre-wrap text-base-content">{{ commitDiff }}</pre>
        <div v-else-if="!loadingDiff" class="text-center text-base-content/60 text-xs p-4">点击"查看 Diff"加载变更详情</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  logViewMode: string
  logSearch: string
  logBranchFilter: string
  logDateFrom: string
  logDateTo: string
  showAuthorFilter: boolean
  logAuthors: string[]
  selectedAuthors: Set<string>
  filteredLog: any[]
  selectedLogCommits: Set<string>
  selectedCommit: any | null
  commitDiff: string | null
  loadingDiff: boolean
  loading: boolean
  hasMoreLog: boolean
  logCount: number
  logTotalEstimate: number
  graphLog: any[]
  graphLoading: boolean
  graphHoveredIndex: number
  graphSelectedCommit: any
  branchColors: string[]
  consoleHistory: any[]
  consoleInput: string
  localBranches: any[]
  getAuthorName: (author: string) => string
  formatRelativeDate: (date: any) => string
  formatFullDate: (date: any) => string
  parseRefs: (refs: string) => string[]
}>()

defineEmits<{
  'update:logViewMode': [value: string]
  'update:logSearch': [value: string]
  'update:logBranchFilter': [value: string]
  'update:logDateFrom': [value: string]
  'update:logDateTo': [value: string]
  'update:showAuthorFilter': [value: boolean]
  'update:selectedCommit': [value: any]
  'update:consoleInput': [value: string]
  'toggle-author': [author: string]
  'load-log': []
  'load-more-log': []
  'select-commit': [commit: any]
  'toggle-log-commit-select': [hash: string]
  'toggle-select-all-log-commits': []
  'log-context-menu': [payload: { event: MouseEvent; commit: any }]
  'exec-console-command': []
  'console-history-up': []
  'console-history-down': []
  'switch-to-graph-view': []
  'on-graph-mouse-move': [event: MouseEvent]
  'on-graph-click': [event: MouseEvent]
  'load-commit-diff': []
}>()

const graphCanvasRef = ref<HTMLCanvasElement | null>(null)
const consoleOutputRef = ref<HTMLElement | null>(null)
const consoleInputRef = ref<HTMLInputElement | null>(null)

defineExpose({
  graphCanvasRef,
  consoleOutputRef,
  consoleInputRef,
})
</script>

