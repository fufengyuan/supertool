<template>
  <div class="diff-tool">
    <h3>文本对比</h3>

    <div class="tool-row">
      <label class="tool-label" style="align-self: center;">对比模式</label>
      <div class="tool-btn-group">
        <button class="tool-btn" :class="{ active: diffMode === 'lines' }" @click="diffMode = 'lines'">行对比</button>
        <button class="tool-btn" :class="{ active: diffMode === 'words' }" @click="diffMode = 'words'">词对比</button>
      </div>

      <label class="tool-label" style="align-self: center;">显示方式</label>
      <div class="tool-btn-group">
        <button class="tool-btn" :class="{ active: viewMode === 'unified' }" @click="viewMode = 'unified'">统一视图</button>
        <button class="tool-btn" :class="{ active: viewMode === 'sidebyside' }" @click="viewMode = 'sidebyside'">并排视图</button>
      </div>

      <button class="tool-btn primary" @click="runDiff" :disabled="!left || !right">对比</button>
      <button class="tool-btn" @click="clearAll">清空</button>
    </div>

    <div class="input-section" :class="{ 'side-by-side': viewMode === 'sidebyside' }">
      <div class="tool-section">
        <h4>原文（左）</h4>
        <textarea v-model="left" class="tool-textarea" placeholder="输入原文..." rows="8"></textarea>
      </div>
      <div class="tool-section">
        <h4>新文（右）</h4>
        <textarea v-model="right" class="tool-textarea" placeholder="输入新文..." rows="8"></textarea>
      </div>
    </div>

    <div class="tool-section" v-if="diffResult.length > 0">
      <h4>对比结果</h4>
      <div v-if="viewMode === 'unified'" class="diff-unified">
        <div v-for="(line, idx) in diffResult" :key="idx" :class="['diff-line', `diff-${line.type}`]">
          <span class="diff-marker">{{ line.marker }}</span>
          <span class="diff-content" v-html="highlightWords(line)"></span>
        </div>
      </div>

      <div v-else class="diff-sidebyside">
        <div class="diff-panel">
          <div class="diff-panel-header">原文</div>
          <div v-for="(line, idx) in leftDiffLines" :key="idx" :class="['diff-line', `diff-${line.type}`]">
            <span class="diff-marker">{{ line.marker }}</span>
            <span class="diff-content" v-html="highlightWords(line)"></span>
          </div>
        </div>
        <div class="diff-panel">
          <div class="diff-panel-header">新文</div>
          <div v-for="(line, idx) in rightDiffLines" :key="idx" :class="['diff-line', `diff-${line.type}`]">
            <span class="diff-marker">{{ line.marker }}</span>
            <span class="diff-content" v-html="highlightWords(line)"></span>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-section" v-if="diffResult.length === 0 && hasRun">
      <div class="tool-result result-equal">
        ✅ 两段文本完全相同
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { diffLines, diffWords, type Change } from 'diff'
// styles in <style scoped>
import { useToast } from '@/composables/useToast'

const toast = useToast()

const left = ref('')
const right = ref('')
const diffMode = ref<'lines' | 'words'>('lines')
const viewMode = ref<'unified' | 'sidebyside'>('unified')
const hasRun = ref(false)

interface DiffLine {
  type: 'added' | 'removed' | 'equal' | 'header'
  marker: string
  value: string
  added?: boolean
  removed?: boolean
}

const diffResult = ref<DiffLine[]>([])

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
    for (const change of changes) {
      const type = change.added ? 'added' : change.removed ? 'removed' : 'equal'
      const marker = change.added ? '+' : change.removed ? '-' : ' '
      result.push({
        type,
        marker,
        value: change.value.replace(/\n$/, ''),
        added: change.added,
        removed: change.removed,
      })
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
  if (line.added) return `<span class="diff-added-word">${escaped}</span>`
  if (line.removed) return `<span class="diff-removed-word">${escaped}</span>`
  return escaped
}

const leftDiffLines = computed(() => {
  return diffResult.value.filter(l => l.type === 'removed' || l.type === 'equal')
})

const rightDiffLines = computed(() => {
  return diffResult.value.filter(l => l.type === 'added' || l.type === 'equal')
})

function clearAll() {
  left.value = ''
  right.value = ''
  diffResult.value = []
  hasRun.value = false
}
</script>

<style scoped>

.input-section {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.input-section.side-by-side {
  flex-direction: row;
  gap: 12px;
}

.input-section.side-by-side .tool-section {
  flex: 1;
}

.diff-unified {
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  overflow: auto;
  max-height: 500px;
  background: var(--color-base-200);
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.5;
}

.diff-line {
  padding: 2px 8px;
  display: flex;
  gap: 8px;
  white-space: pre-wrap;
  word-break: break-all;
}

.diff-added {
  background: rgba(34, 197, 94, 0.15);
}

.diff-removed {
  background: rgba(239, 68, 68, 0.15);
}

.diff-equal {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.diff-marker {
  width: 16px;
  flex-shrink: 0;
  user-select: none;
}

.diff-added .diff-marker {
  color: #22c55e;
}

.diff-removed .diff-marker {
  color: #ef4444;
}

.diff-added-word {
  background: rgba(34, 197, 94, 0.3);
  color: #16a34a;
}

.diff-removed-word {
  background: rgba(239, 68, 68, 0.3);
  color: #dc2626;
}

.diff-sidebyside {
  display: flex;
  gap: 12px;
}

.diff-panel {
  flex: 1;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  overflow: hidden;
  background: var(--color-base-200);
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.5;
  max-height: 500px;
  overflow-y: auto;
}

.diff-panel-header {
  padding: 8px 12px;
  font-weight: 600;
  color: var(--color-base-content);
  background: var(--color-base-100);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  position: sticky;
  top: 0;
}

.result-equal {
  color: #22c55e;
  text-align: center;
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--color-base-content); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--color-primary); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-input:focus { border-color: var(--color-primary); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: var(--color-base-100); color: var(--color-base-content); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.tool-btn.primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: var(--color-primary); color: white; border-color: var(--color-primary); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: var(--color-base-200); border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: var(--color-base-content); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-select:focus { border-color: var(--color-primary); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--color-base-content); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); margin: 20px 0; }
</style>
