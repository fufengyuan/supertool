<template>
  <div class="flex flex-col h-full">
    <h3 class="text-lg font-bold text-base-content mb-4">文本对比</h3>

    <div class="flex flex-wrap gap-2.5 mb-4 items-center">
      <label class="text-xs text-base-content/60">对比模式</label>
      <div class="join">
        <button class="btn btn-ghost join-item btn-sm" :class="{ 'btn-active': diffMode === 'lines' }" @click="diffMode = 'lines'">行对比</button>
        <button class="btn btn-ghost join-item btn-sm" :class="{ 'btn-active': diffMode === 'words' }" @click="diffMode = 'words'">词对比</button>
      </div>

      <label class="text-xs text-base-content/60 ml-2">显示方式</label>
      <div class="join">
        <button class="btn btn-ghost join-item btn-sm" :class="{ 'btn-active': viewMode === 'unified' }" @click="viewMode = 'unified'">统一视图</button>
        <button class="btn btn-ghost join-item btn-sm" :class="{ 'btn-active': viewMode === 'sidebyside' }" @click="viewMode = 'sidebyside'">并排视图</button>
      </div>

      <button class="btn btn-primary btn-sm" @click="runDiff" :disabled="!left && !right">对比</button>
      <button class="btn btn-ghost btn-sm" @click="clearAll">清空</button>
      <button v-if="diffResult.length > 0" class="btn btn-ghost btn-sm" @click="swapInputs" title="交换左右文本">⇄ 交换</button>
    </div>

    <div class="grid gap-4 mb-4" :class="viewMode === 'sidebyside' ? 'grid-cols-2' : 'grid-cols-1'">
      <div>
        <h4 class="text-sm font-semibold text-base-content mb-1.5">原文（左）</h4>
        <textarea v-model="left" class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px] resize-none" placeholder="输入原文..." rows="8"></textarea>
      </div>
      <div v-if="viewMode === 'sidebyside'">
        <h4 class="text-sm font-semibold text-base-content mb-1.5">新文（右）</h4>
        <textarea v-model="right" class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px] resize-none" placeholder="输入新文..." rows="8"></textarea>
      </div>
      <div v-else>
        <h4 class="text-sm font-semibold text-base-content mb-1.5">新文（右）</h4>
        <textarea v-model="right" class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px] resize-none" placeholder="输入新文..." rows="8"></textarea>
      </div>
    </div>

    <div v-if="diffResult.length > 0" class="flex flex-col flex-1 min-h-0">
      <h4 class="text-sm font-semibold text-base-content mb-2">对比结果
        <span class="text-xs font-normal text-base-content/50 ml-2">
          +{{ statAdded }} / -{{ statRemoved }} / ={{ statEqual }}
        </span>
      </h4>

      <div v-if="viewMode === 'unified'" class="diff-unified flex-1 overflow-auto bg-base-200 border border-base-content/10 rounded-box">
        <div v-for="(line, idx) in diffResult" :key="idx" :class="['diff-row', `diff-${line.type}`]">
          <span class="diff-line-no">{{ line.lineNo ?? '' }}</span>
          <span class="diff-marker">{{ line.marker }}</span>
          <span class="diff-content" v-html="highlightWords(line)"></span>
        </div>
      </div>

      <div v-else class="grid grid-cols-2 gap-2 flex-1 overflow-hidden">
        <div class="diff-panel overflow-auto bg-base-200 border border-base-content/10 rounded-box">
          <div class="diff-panel-header">原文</div>
          <div v-for="(line, idx) in leftDiffLines" :key="idx" :class="['diff-row', `diff-${line.type}`]">
            <span class="diff-line-no">{{ line.lineNo ?? '' }}</span>
            <span class="diff-marker">{{ line.marker }}</span>
            <span class="diff-content" v-html="highlightWords(line)"></span>
          </div>
        </div>
        <div class="diff-panel overflow-auto bg-base-200 border border-base-content/10 rounded-box">
          <div class="diff-panel-header">新文</div>
          <div v-for="(line, idx) in rightDiffLines" :key="idx" :class="['diff-row', `diff-${line.type}`]">
            <span class="diff-line-no">{{ line.lineNo ?? '' }}</span>
            <span class="diff-marker">{{ line.marker }}</span>
            <span class="diff-content" v-html="highlightWords(line)"></span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="diffResult.length === 0 && hasRun" class="alert alert-success">
      <SvgIcon name="check" size="16" class="align-text-bottom" />
      <span>两段文本完全相同</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed } from 'vue'
import { diffLines, diffWords, type Change } from 'diff'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const left = ref('')
const right = ref('')
const diffMode = ref<'lines' | 'words'>('lines')
const viewMode = ref<'unified' | 'sidebyside'>('unified')
const hasRun = ref(false)

interface DiffLine {
  type: 'added' | 'removed' | 'equal'
  marker: string
  value: string
  added?: boolean
  removed?: boolean
  lineNo?: number
}

const diffResult = ref<DiffLine[]>([])

const statAdded = computed(() => diffResult.value.filter(l => l.type === 'added').length)
const statRemoved = computed(() => diffResult.value.filter(l => l.type === 'removed').length)
const statEqual = computed(() => diffResult.value.filter(l => l.type === 'equal').length)

function runDiff() {
  hasRun.value = true

  if (!left.value && !right.value) {
    toast.warning('请输入要对比的文本')
    return
  }

  try {
    let changes: Change[]

    if (diffMode.value === 'lines') {
      changes = diffLines(left.value, right.value, { newlineIsToken: true })
    } else {
      changes = diffWords(left.value, right.value)
    }

    const result: DiffLine[] = []
    let leftLineNo = 0
    let rightLineNo = 0

    for (const change of changes) {
      const type = change.added ? 'added' : change.removed ? 'removed' : 'equal'
      const marker = change.added ? '+' : change.removed ? '-' : ' '
      const value = change.value.replace(/\n$/, '')
      // 按行拆分，每行单独一条记录，便于显示行号
      const lines = value.split('\n')
      for (const line of lines) {
        if (type === 'removed') {
          leftLineNo++
          result.push({ type, marker, value: line, added: change.added, removed: change.removed, lineNo: leftLineNo })
        } else if (type === 'added') {
          rightLineNo++
          result.push({ type, marker, value: line, added: change.added, removed: change.removed, lineNo: rightLineNo })
        } else {
          leftLineNo++
          rightLineNo++
          result.push({ type, marker, value: line, lineNo: rightLineNo })
        }
      }
    }

    diffResult.value = result
  } catch (e: any) {
    toast.error(`对比失败: ${e.message}`)
  }
}

function highlightWords(line: DiffLine): string {
  const escaped = line.value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
  if (line.added) {return `<span class="diff-added-word">${escaped}</span>`}
  if (line.removed) {return `<span class="diff-removed-word">${escaped}</span>`}
  return escaped
}

const leftDiffLines = computed(() => {
  return diffResult.value.filter(l => l.type === 'removed' || l.type === 'equal')
})

const rightDiffLines = computed(() => {
  return diffResult.value.filter(l => l.type === 'added' || l.type === 'equal')
})

function swapInputs() {
  const tmp = left.value
  left.value = right.value
  right.value = tmp
  if (hasRun.value) {runDiff()}
}

function clearAll() {
  left.value = ''
  right.value = ''
  diffResult.value = []
  hasRun.value = false
}
</script>

<style scoped>
.diff-row {
  display: flex;
  align-items: flex-start;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  line-height: 1.6;
  min-height: 19px;
}

.diff-line-no {
  flex: 0 0 40px;
  text-align: right;
  padding-right: 8px;
  color: rgba(0, 0, 0, 0.3);
  user-select: none;
  border-right: 1px solid rgba(0, 0, 0, 0.05);
  margin-right: 6px;
}

.diff-marker {
  flex: 0 0 16px;
  user-select: none;
}

.diff-content {
  flex: 1;
  white-space: pre-wrap;
  word-break: break-all;
  padding-right: 8px;
}

.diff-added {
  background-color: rgba(34, 197, 94, 0.12);
}

.diff-added .diff-marker,
.diff-added .diff-content {
  color: rgb(22, 101, 52);
}

.diff-removed {
  background-color: rgba(239, 68, 68, 0.12);
}

.diff-removed .diff-marker,
.diff-removed .diff-content {
  color: rgb(127, 29, 29);
}

.diff-equal .diff-marker,
.diff-equal .diff-content {
  color: inherit;
}

.diff-added-word {
  background-color: rgba(34, 197, 94, 0.3);
  border-radius: 2px;
}

.diff-removed-word {
  background-color: rgba(239, 68, 68, 0.3);
  border-radius: 2px;
}

.diff-panel-header {
  position: sticky;
  top: 0;
  background-color: rgba(0, 0, 0, 0.05);
  padding: 4px 8px;
  font-size: 12px;
  font-weight: 600;
  z-index: 1;
}

:global([data-theme="dark"]) .diff-line-no {
  color: rgba(255, 255, 255, 0.3);
  border-right-color: rgba(255, 255, 255, 0.08);
}

:global([data-theme="dark"]) .diff-added {
  background-color: rgba(34, 197, 94, 0.18);
}

:global([data-theme="dark"]) .diff-added .diff-marker,
:global([data-theme="dark"]) .diff-added .diff-content {
  color: rgb(134, 239, 172);
}

:global([data-theme="dark"]) .diff-removed {
  background-color: rgba(239, 68, 68, 0.18);
}

:global([data-theme="dark"]) .diff-removed .diff-marker,
:global([data-theme="dark"]) .diff-removed .diff-content {
  color: rgb(252, 165, 165);
}

:global([data-theme="dark"]) .diff-panel-header {
  background-color: rgba(255, 255, 255, 0.06);
}
</style>
