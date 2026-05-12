<template>
  <div class="flex flex-col flex-1 min-w-[280px] overflow-hidden bg-base-200">
    <!-- IDEA 风格日志头部 -->
    <div class="flex items-center justify-between px-2 py-1 border-b border-base-content/8 bg-base-200 shrink-0 h-[24px]">
      <span class="font-medium text-[11px]">日志</span>
      <div class="flex items-center gap-1">
        <!-- View mode toggle -->
        <div class="flex items-center gap-0 p-0.5 bg-base-100 rounded border border-base-content/10">
          <button
            class="flex items-center justify-center w-[24px] h-[18px] p-0 border-0 bg-transparent text-base-content/50 rounded cursor-pointer hover:text-base-content hover:bg-[var(--hover-bg)]"
            :class="{ '!text-primary !bg-primary/10': logViewMode === 'table' }"
            @click="$emit('update:logViewMode', 'table')"
            title="表格视图"
          >
            <SvgIcon name="grid" :size="12" />
          </button>
          <button
            class="flex items-center justify-center w-[24px] h-[18px] p-0 border-0 bg-transparent text-base-content/50 rounded cursor-pointer hover:text-base-content hover:bg-[var(--hover-bg)]"
            :class="{ '!text-primary !bg-primary/10': logViewMode === 'graph' }"
            @click="$emit('switch-to-graph-view')"
            title="图形视图"
          >
            <SvgIcon name="gitCommit" :size="12" />
          </button>
          <button
            class="flex items-center justify-center w-[24px] h-[18px] p-0 border-0 bg-transparent text-base-content/50 rounded cursor-pointer hover:text-base-content hover:bg-[var(--hover-bg)]"
            :class="{ '!text-primary !bg-primary/10': logViewMode === 'console' }"
            @click="$emit('update:logViewMode', 'console')"
            title="Git Console"
          >
            <SvgIcon name="terminal" :size="12" />
          </button>
        </div>
        <!-- 搜索框 -->
        <div class="relative">
          <SvgIcon name="search" :size="10" class="absolute left-1.5 top-1/2 -translate-y-1/2 text-base-content/50 pointer-events-none" />
          <input
            :value="logSearch"
            @input="$emit('update:logSearch', ($event.target as HTMLInputElement).value)"
            class="py-0.5 px-1.5 pl-[20px] border border-base-content/10 rounded bg-base-100 text-base-content text-[10px] outline-none w-[120px] transition-[border-color,width] duration-150 focus:border-primary focus:w-[160px] h-[18px]"
            placeholder="搜索..."
            spellcheck="false"
          />
        </div>
        <!-- 分支过滤 -->
        <select :value="logBranchFilter" @change="$emit('update:logBranchFilter', ($event.target as HTMLSelectElement).value)" class="py-0.5 px-1 border border-base-content/10 rounded bg-base-100 text-base-content text-[10px] outline-none cursor-pointer h-[18px]">
          <option value="">所有分支</option>
          <option v-for="b in localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
        </select>
        <!-- 日期过滤 -->
        <input :value="logDateFrom" @input="$emit('update:logDateFrom', ($event.target as HTMLInputElement).value)" type="date" class="py-0.5 px-1 border border-base-content/10 rounded bg-base-100 text-base-content text-[10px] outline-none w-[100px] h-[18px]" title="From" />
        <input :value="logDateTo" @input="$emit('update:logDateTo', ($event.target as HTMLInputElement).value)" type="date" class="py-0.5 px-1 border border-base-content/10 rounded bg-base-100 text-base-content text-[10px] outline-none w-[100px] h-[18px]" title="To" />
        <!-- 作者过滤 -->
        <button class="btn btn-ghost btn-xs h-[18px] min-h-[18px] px-1" @click="$emit('update:showAuthorFilter', !showAuthorFilter)" :class="{ '!text-primary !bg-primary/10': (selectedAuthors as Set<string>).size > 0 }" title="作者">
          <SvgIcon name="user" :size="10" />
        </button>
        <div v-if="showAuthorFilter" class="absolute top-full right-0 z-[100] bg-base-100 border border-base-content/8 rounded shadow-lg min-w-[160px] max-h-[180px] overflow-y-auto p-1 mt-0.5" @click.stop>
          <div v-for="a in logAuthors" :key="a" class="flex items-center gap-1 px-2 py-0.5 text-[10px] cursor-pointer hover:bg-[var(--hover-bg)]" @click.stop="$emit('toggle-author', a)">
            <input type="checkbox" :checked="(selectedAuthors as Set<string>).has(a)" class="accent-primary w-[10px] h-[10px]" />
            <span>{{ getAuthorName(a) }}</span>
          </div>
          <div v-if="logAuthors.length === 0" class="p-2 text-center text-[10px] text-base-content/50">无数据</div>
        </div>
        <!-- 刷新 -->
        <button class="btn btn-ghost btn-xs h-[18px] min-h-[18px] px-1" @click="$emit('load-log')" :disabled="loading" title="刷新">
          <SvgIcon name="refresh" :size="10" :class="{ 'animate-spin': loading }" />
        </button>
      </div>
    </div>

    <!-- 提交历史表格 - IDEA 风格紧凑 -->
    <div v-if="logViewMode === 'table'" class="flex-1 overflow-y-auto overflow-x-hidden">
      <table class="w-full border-collapse text-[11px]">
        <thead class="sticky top-0 bg-base-200 border-b border-base-content/8">
          <tr class="h-[20px]">
            <th class="w-[24px] text-center px-1"><input type="checkbox" :checked="(selectedLogCommits as Set<string>).size > 0 && (selectedLogCommits as Set<string>).size === filteredLog.length" @click="$emit('toggle-select-all-log-commits')" class="accent-primary w-[10px] h-[10px] cursor-pointer" title="全选" /></th>
            <th class="w-[60px] px-1 text-left font-medium">Hash</th>
            <th class="w-[100px] px-1 text-left font-medium">Author</th>
            <th class="w-[80px] px-1 text-left font-medium">Date</th>
            <th class="min-w-[150px] px-1 text-left font-medium">Message</th>
            <th class="w-[40px] px-1 text-center font-medium">Files</th>
            <th class="w-[100px] px-1 text-left font-medium">Refs</th>
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
              <span class="text-base-content">{{ getAuthorName(commit) }}</span>
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
        <SvgIcon name="refresh" :size="24" class="animate-spin" />
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
        <button class="btn btn-ghost btn-xs" @click="$emit('update:selectedCommit', null)" title="关闭"><SvgIcon name="x" :size="14" class="inline-block" /></button>
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
      <div class="border-t border-base-content/10 p-2">
        <DiffViewer
          :files="commitDiff?.files || null"
          :diff="commitDiff?.diff || null"
          :loading="loadingDiff"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import DiffViewer from '@/components/ui/DiffViewer.vue'
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
  commitDiff: { hash: string; author: string; authorEmail: string; date: string; message: string; files: any[]; diff: string } | null
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
  getAuthorName: (commit: any) => string
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

