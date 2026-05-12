<template>
  <div class="diff-viewer">
    <!-- 文件列表 -->
    <div v-if="files && files.length > 0" class="file-list">
      <div class="file-list-header">
        <span class="text-xs font-semibold text-base-content/60">变更文件</span>
        <span class="text-xs text-base-content/40">{{ files.length }} 个文件</span>
      </div>
      <div class="file-list-content">
        <div
          v-for="(file, idx) in files"
          :key="file.path"
          class="file-item"
          :class="{ selected: selectedFileIdx === idx }"
          @click="selectFile(idx)"
        >
          <span class="file-status" :class="getStatusClass(file.status)">
            {{ getStatusLabel(file.status) }}
          </span>
          <span class="file-path" :title="file.path">{{ getFileName(file.path) }}</span>
          <span class="file-changes text-xs text-base-content/40">{{ file.changes }}</span>
        </div>
      </div>
    </div>

    <!-- Diff 内容 -->
    <div class="diff-content">
      <div v-if="loading" class="loading-state">
        <SvgIcon name="refresh" size="16" class="animate-spin" />
        <span class="text-xs">加载中...</span>
      </div>
      <div v-else-if="!diff" class="empty-state">
        <span class="text-xs text-base-content/40">点击文件查看变更</span>
      </div>
      <div v-else class="diff-lines">
        <div
          v-for="(line, idx) in parsedLines"
          :key="idx"
          class="diff-line"
          :class="getLineClass(line)"
        >
          <span class="line-number">{{ line.oldNum || '' }}</span>
          <span class="line-number new">{{ line.newNum || '' }}</span>
          <span class="line-prefix">{{ line.prefix }}</span>
          <span class="line-content">{{ line.content }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const props = defineProps<{
  files: any[] | null
  diff: string | null
  loading: boolean
}>()

const selectedFileIdx = ref<number | null>(null)

// Parse diff into structured lines
const parsedLines = computed(() => {
  if (!props.diff) return []
  
  const lines = props.diff.split('\n')
  const result: any[] = []
  let oldNum = 0
  let newNum = 0
  
  for (const line of lines) {
    // Skip file header lines
    if (line.startsWith('diff --git') || line.startsWith('index ') || 
        line.startsWith('--- ') || line.startsWith('+++ ') || line.startsWith('@@')) {
      result.push({
        prefix: '',
        content: line,
        type: 'header',
        oldNum: '',
        newNum: ''
      })
      continue
    }
    
    const prefix = line[0] || ''
    const content = line.slice(1)
    
    if (prefix === '+') {
      newNum++
      result.push({ prefix, content, type: 'add', oldNum: '', newNum: newNum })
    } else if (prefix === '-') {
      oldNum++
      result.push({ prefix, content, type: 'remove', oldNum: oldNum, newNum: '' })
    } else if (prefix === ' ') {
      oldNum++
      newNum++
      result.push({ prefix, content, type: 'context', oldNum: oldNum, newNum: newNum })
    } else {
      result.push({ prefix: '', content: line, type: 'meta', oldNum: '', newNum: '' })
    }
  }
  
  return result.slice(0, 500) // Limit to 500 lines for performance
})

function selectFile(idx: number) {
  selectedFileIdx.value = idx
}

function getStatusClass(status: string) {
  return {
    'status-added': status === 'added',
    'status-deleted': status === 'deleted',
    'status-modified': status === 'modified'
  }
}

function getStatusLabel(status: string) {
  return status === 'added' ? 'A' : status === 'deleted' ? 'D' : 'M'
}

function getFileName(path: string) {
  return path.split('/').pop() || path
}

function getLineClass(line: any) {
  return {
    'line-header': line.type === 'header',
    'line-add': line.type === 'add',
    'line-remove': line.type === 'remove',
    'line-context': line.type === 'context',
    'line-meta': line.type === 'meta'
  }
}
</script>

<style scoped>
.diff-viewer {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-height: 100%;
  overflow: hidden;
}

.file-list {
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 0.5rem;
  background: var(--color-base-200);
  max-height: 150px;
  overflow-y: auto;
}

.file-list-header {
  display: flex;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.file-list-content {
  padding: 0.25rem;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.5rem;
  border-radius: 0.25rem;
  cursor: pointer;
  transition: background 0.1s;
}

.file-item:hover {
  background: color-mix(in oklab, var(--color-base-content) 5%, transparent);
}

.file-item.selected {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.file-status {
  display: inline-flex;
  justify-content: center;
  width: 1.25rem;
  height: 1.25rem;
  font-size: 0.65rem;
  font-weight: bold;
  border-radius: 0.25rem;
}

.status-added {
  background: color-mix(in oklab, var(--color-green) 20%, transparent);
  color: var(--color-green);
}

.status-deleted {
  background: color-mix(in oklab, var(--color-red) 20%, transparent);
  color: var(--color-red);
}

.status-modified {
  background: color-mix(in oklab, var(--color-amber) 20%, transparent);
  color: var(--color-amber);
}

.file-path {
  flex: 1;
  font-size: 0.7rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diff-content {
  flex: 1;
  overflow-y: auto;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 0.5rem;
  background: var(--color-base-100);
}

.loading-state, .empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
}

.diff-lines {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.7rem;
  line-height: 1.4;
  white-space: pre;
}

.diff-line {
  display: flex;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 2%, transparent);
}

.line-number {
  width: 2.5rem;
  text-align: right;
  padding: 0 0.25rem;
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
  font-size: 0.65rem;
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 5%, transparent);
}

.line-prefix {
  width: 1rem;
  text-align: center;
  font-weight: bold;
}

.line-content {
  flex: 1;
  padding: 0 0.25rem;
  overflow: hidden;
  text-overflow: ellipsis;
}

.line-header {
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.line-add {
  background: color-mix(in oklab, var(--color-green) 8%, transparent);
}

.line-add .line-prefix {
  color: var(--color-green);
}

.line-remove {
  background: color-mix(in oklab, var(--color-red) 8%, transparent);
}

.line-remove .line-prefix {
  color: var(--color-red);
}

.line-context {
  background: transparent;
}

.line-meta {
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
}
</style>