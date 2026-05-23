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
        <SvgIcon name="check" size="14" class="align-text-bottom" /> 两段文本完全相同
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
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

function clearAll() {
  left.value = ''
  right.value = ''
  diffResult.value = []
  hasRun.value = false
}
</script>


