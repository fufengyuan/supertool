<template>
  <div class="split-diff-viewer">
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

    <!-- Split Diff 内容 -->
    <div class="split-diff-content">
      <div v-if="loading" class="loading-state">
        <SvgIcon name="refresh" size="16" class="animate-spin" />
        <span class="text-xs">加载中...</span>
      </div>
      <div v-else-if="!diff" class="empty-state">
        <span class="text-xs text-base-content/40">点击文件查看变更</span>
      </div>
      <div v-else class="split-container">
        <!-- 左侧：旧文件 -->
        <div class="diff-pane left">
          <div class="pane-header">
            <div class="pane-header-left">
              <span class="text-xs font-semibold">旧版本</span>
              <span class="text-xs text-base-content/40">{{ oldFileName }}</span>
            </div>
            <button 
              class="copy-btn" 
              @click="copyOldContent"
              :title="copiedOld ? '已复制' : '复制旧版本内容'"
            >
              <SvgIcon :name="copiedOld ? 'check' : 'copy'" size="12" />
            </button>
          </div>
          <div class="pane-content">
            <div
              v-for="(line, idx) in oldLines"
              :key="'old-' + idx"
              class="diff-line"
              :class="getLineClass(line, 'old')"
            >
              <span class="line-number">{{ line.oldNum || '' }}</span>
              <span class="line-content">{{ line.content }}</span>
            </div>
          </div>
        </div>
        
        <!-- 右侧：新文件 -->
        <div class="diff-pane right">
          <div class="pane-header">
            <div class="pane-header-left">
              <span class="text-xs font-semibold">新版本</span>
              <span class="text-xs text-base-content/40">{{ newFileName }}</span>
            </div>
            <button 
              class="copy-btn" 
              @click="copyNewContent"
              :title="copiedNew ? '已复制' : '复制新版本内容'"
            >
              <SvgIcon :name="copiedNew ? 'check' : 'copy'" size="12" />
            </button>
          </div>
          <div class="pane-content">
            <div
              v-for="(line, idx) in newLines"
              :key="'new-' + idx"
              class="diff-line"
              :class="getLineClass(line, 'new')"
            >
              <span class="line-number">{{ line.newNum || '' }}</span>
              <span class="line-content">{{ line.content }}</span>
            </div>
          </div>
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
const copiedOld = ref(false)
const copiedNew = ref(false)

// 从 diff 中解析文件名（需要跳过 commit header）
const oldFileName = computed(() => {
  if (!props.diff) {return ''}
  // 找到 diff 开始的位置
  const diffStart = props.diff.indexOf('diff --git')
  if (diffStart === -1) {return '旧文件'}
  const diffPart = props.diff.slice(diffStart)
  const match = diffPart.match(/^--- (.+)/m)
  return match ? match[1].replace('a/', '') : '旧文件'
})

const newFileName = computed(() => {
  if (!props.diff) {return ''}
  // 找到 diff 开始的位置
  const diffStart = props.diff.indexOf('diff --git')
  if (diffStart === -1) {return '新文件'}
  const diffPart = props.diff.slice(diffStart)
  const match = diffPart.match(/^\+\+\+ (.+)/m)
  return match ? match[1].replace('b/', '') : '新文件'
})

// 解析 diff 为左右两边的行
// 策略：将删除行放左边，新增行放右边，上下文行两边都放
// 优化：仅缩进变化的行对标记为 whitespace 类型，不显示为删除/新增
const oldLines = computed(() => {
  if (!props.diff) {return []}
  return parseDiffLines(props.diff, 'old')
})

const newLines = computed(() => {
  if (!props.diff) {return []}
  return parseDiffLines(props.diff, 'new')
})

// 从 diff 中提取旧版本的完整内容（用于复制）
const oldContent = computed(() => {
  if (!props.diff) {return ''}
  return extractContentFromDiff(props.diff, 'old')
})

// 从 diff 中提取新版本的完整内容（用于复制）
const newContent = computed(() => {
  if (!props.diff) {return ''}
  return extractContentFromDiff(props.diff, 'new')
})

// 从 diff 提取完整文件内容
function extractContentFromDiff(diff: string, side: 'old' | 'new') {
  const lines = diff.split('\n')
  const result: string[] = []
  
  // 找到 diff 开始的位置
  let diffStartIndex = 0
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].startsWith('diff --git')) {
      diffStartIndex = i
      break
    }
  }
  
  const diffLines = lines.slice(diffStartIndex)
  
  for (const line of diffLines) {
    // 跳过元数据行
    if (line.startsWith('diff --git') || line.startsWith('index ') ||
        line.startsWith('--- ') || line.startsWith('+++ ') || line.startsWith('@@')) {
      continue
    }
    
    const prefix = line[0] || ''
    const content = line.slice(1)
    
    if (side === 'old') {
      // 旧版本：保留上下文行和删除行
      if (prefix === ' ' || prefix === '-') {
        result.push(content)
      }
      // 跳过新增行（prefix === '+')
    } else {
      // 新版本：保留上下文行和新增行
      if (prefix === ' ' || prefix === '+') {
        result.push(content)
      }
      // 跳过删除行（prefix === '-')
    }
  }
  
  return result.join('\n')
}

function parseDiffLines(diff: string, side: 'old' | 'new') {
  const lines = diff.split('\n')
  const result: any[] = []
  let oldNum = 0
  let newNum = 0
  
  // 从 @@ header 解析起始行号
  let oldStart = 0
  let newStart = 0
  
  // 找到 diff 开始的位置（跳过 git show 的 commit header）
  let diffStartIndex = 0
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].startsWith('diff --git')) {
      diffStartIndex = i
      break
    }
  }
  
  // 只处理 diff 部分
  const diffLines = lines.slice(diffStartIndex)
  
  // 收集相邻的删除/新增行对，用于检测仅缩进变化
  const pendingRemoves: { content: string; oldNum: number }[] = []
  const pendingAdds: { content: string; newNum: number }[] = []
  
  function flushPendingLines() {
    // 检查是否有仅缩进变化的行对（内容相同，仅空白差异）
    const pairs: Array<{ oldIdx: number; newIdx: number }> = []
    const usedNewIdx = new Set<number>()
    
    for (let i = 0; i < pendingRemoves.length; i++) {
      const oldContent = pendingRemoves[i].content
      // 找第一个未被匹配的、内容相同的新行
      for (let j = 0; j < pendingAdds.length; j++) {
        if (usedNewIdx.has(j)) {continue}
        const newContent = pendingAdds[j].content
        // 去除空白后比较内容
        if (oldContent.trim() === newContent.trim()) {
          pairs.push({ oldIdx: i, newIdx: j })
          usedNewIdx.add(j)
          break
        }
      }
    }
    
    // 处理删除行
    for (let i = 0; i < pendingRemoves.length; i++) {
      const pair = pairs.find(p => p.oldIdx === i)
      if (pair) {
        // 仅缩进变化：标记为 whitespace，使用淡色显示
        if (side === 'old') {
          result.push({ 
            content: pendingRemoves[i].content, 
            type: 'whitespace', 
            oldNum: pendingRemoves[i].oldNum, 
            newNum: '',
            pairedNewNum: pendingAdds[pair.newIdx].newNum
          })
        } else {
          // 右侧显示 placeholder
          result.push({ content: '', type: 'placeholder', oldNum: '', newNum: '' })
        }
      } else {
        // 真正的删除
        if (side === 'old') {
          result.push({ content: pendingRemoves[i].content, type: 'remove', oldNum: pendingRemoves[i].oldNum, newNum: '' })
        } else {
          result.push({ content: '', type: 'placeholder', oldNum: '', newNum: '' })
        }
      }
    }
    
    // 处理新增行
    for (let j = 0; j < pendingAdds.length; j++) {
      const pair = pairs.find(p => p.newIdx === j)
      if (pair) {
        // 仅缩进变化：标记为 whitespace
        if (side === 'new') {
          result.push({ 
            content: pendingAdds[j].content, 
            type: 'whitespace', 
            oldNum: '', 
            newNum: pendingAdds[j].newNum,
            pairedOldNum: pendingRemoves[pair.oldIdx].oldNum
          })
        } else {
          result.push({ content: '', type: 'placeholder', oldNum: '', newNum: '' })
        }
      } else {
        // 真正的新增
        if (side === 'new') {
          result.push({ content: pendingAdds[j].content, type: 'add', oldNum: '', newNum: pendingAdds[j].newNum })
        } else {
          result.push({ content: '', type: 'placeholder', oldNum: '', newNum: '' })
        }
      }
    }
    
    // 清空待处理列表
    pendingRemoves.length = 0
    pendingAdds.length = 0
  }
  
  for (const line of diffLines) {
    // 解析 @@ -1,10 +1,15 @@ 格式
    if (line.startsWith('@@')) {
      // 先处理之前积累的行
      flushPendingLines()
      const match = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/)
      if (match) {
        oldStart = parseInt(match[1])
        newStart = parseInt(match[2])
        oldNum = oldStart
        newNum = newStart
      }
      continue
    }
    
    // 跳过 diff --git, index, ---, +++ 等元数据行
    if (line.startsWith('diff --git') || line.startsWith('index ') ||
        line.startsWith('--- ') || line.startsWith('+++ ')) {
      continue
    }
    
    const prefix = line[0] || ''
    const content = line.slice(1)
    
    if (prefix === '-') {
      // 删除行：先积累
      oldNum++
      pendingRemoves.push({ content, oldNum })
    } else if (prefix === '+') {
      // 新增行：先积累
      newNum++
      pendingAdds.push({ content, newNum })
    } else if (prefix === ' ') {
      // 上下文行：先处理积累的行，再输出上下文
      flushPendingLines()
      oldNum++
      newNum++
      result.push({ content, type: 'context', oldNum, newNum })
    } else {
      // 其他行（如空行）：先处理积累的行
      flushPendingLines()
      if (line === '') {
        oldNum++
        newNum++
        result.push({ content: '', type: 'context', oldNum, newNum })
      } else {
        result.push({ content: line, type: 'meta', oldNum: '', newNum: '' })
      }
    }
  }
  
  // 处理最后积累的行
  flushPendingLines()
  
  return result
}

// 复制旧版本内容
async function copyOldContent() {
  try {
    await navigator.clipboard.writeText(oldContent.value)
    copiedOld.value = true
    setTimeout(() => copiedOld.value = false, 2000)
  } catch (e) {
    console.error('复制失败:', e)
  }
}

// 复制新版本内容
async function copyNewContent() {
  try {
    await navigator.clipboard.writeText(newContent.value)
    copiedNew.value = true
    setTimeout(() => copiedNew.value = false, 2000)
  } catch (e) {
    console.error('复制失败:', e)
  }
}

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

function getLineClass(line: any, side: 'old' | 'new') {
  return {
    'line-header': line.type === 'header',
    'line-remove': line.type === 'remove',
    'line-add': line.type === 'add',
    'line-context': line.type === 'context',
    'line-placeholder': line.type === 'placeholder',
    'line-whitespace': line.type === 'whitespace',
    'line-meta': line.type === 'meta'
  }
}
</script>

<style scoped>
.split-diff-viewer {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  height: 100%;
  min-height: 400px;
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

.split-diff-content {
  flex: 1;
  overflow: hidden;
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

.split-container {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.diff-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.diff-pane.left {
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.pane-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--color-base-200);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.pane-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.copy-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 0.25rem;
  background: transparent;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 15%, transparent);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
  transition: all 0.15s;
}

.copy-btn:hover {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  color: var(--color-base-content);
}

.copy-btn:active {
  transform: scale(0.95);
}

.pane-content {
  flex: 1;
  overflow-y: auto;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.7rem;
  line-height: 1.4;
}

.diff-line {
  display: flex;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 2%, transparent);
  min-height: 1.4rem;
}

.line-number {
  width: 3rem;
  text-align: right;
  padding: 0 0.5rem;
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
  font-size: 0.65rem;
  border-right: 1px solid color-mix(in oklab, var(--color-base-content) 5%, transparent);
  user-select: none;
}

.line-content {
  flex: 1;
  padding: 0 0.5rem;
  white-space: pre;
  overflow: hidden;
  text-overflow: ellipsis;
}

.line-header {
  background: #dbeafe;
  color: #2563eb;
}

.line-header .line-content {
  font-weight: 600;
}

.line-remove {
  background: #fee2e2;
}

.line-remove .line-content {
  color: #dc2626;
  font-weight: 600;
}

.line-add {
  background: #dbeafe;
}

.line-add .line-content {
  color: #2563eb;
  font-weight: 600;
}

.line-context {
  background: transparent;
}

/* 仅缩进变化的行 - 使用淡灰色，不明显标记为变更 */
.line-whitespace {
  background: color-mix(in oklab, var(--color-base-content) 5%, transparent);
}

.line-whitespace .line-content {
  color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
}

.line-placeholder {
  background: color-mix(in oklab, var(--color-base-content) 3%, transparent);
}

.line-placeholder .line-number {
  background: color-mix(in oklab, var(--color-base-content) 5%, transparent);
}

.line-meta {
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
}
</style>