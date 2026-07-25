<template>
  <div class="flex flex-col h-full gap-2">
    <div class="flex items-center justify-between gap-2 pb-2 border-b border-base-content/10">
      <div v-if="connection" class="flex items-center gap-1.5">
        <span class="text-xs px-2 py-[3px] rounded-md bg-primary/10 text-primary font-medium flex items-center gap-1">
          <SvgIcon :name="connection.type === 'redis' ? 'key' : (connection.type === 'sqlite' ? 'file' : 'database')" size="12" />
          {{ connection.name }}
        </span>
      </div>
      <div class="flex gap-1.5">
        <button
          @click="handleExecute"
          class="btn btn-primary btn-sm gap-1.5"
          :disabled="executing"
          title="执行 (Ctrl+Enter)"
        >
          <SvgIcon v-if="executing" name="refresh" size="14" class="animate-spin" />
          <SvgIcon v-else name="send" size="14" />
          {{ executing ? '执行中...' : '执行' }}
        </button>
        <button
          @click="handleFormat"
          class="btn btn-ghost btn-sm gap-1.5"
          title="格式化 (Ctrl+Shift+F)"
        >
          <SvgIcon name="menu" size="14" />
          格式化
        </button>
        <button @click="$emit('clear')" class="btn btn-ghost btn-sm btn-square" title="清空">
          <SvgIcon name="trash" size="14" />
        </button>
      </div>
    </div>

    <!-- Editor with syntax highlighting overlay -->
    <div class="relative flex-1 min-h-[120px] max-h-[300px] border border-base-content/10 rounded-lg overflow-hidden transition-[border-color] duration-150 focus-within:border-primary focus-within:shadow-[0_0_0_3px_rgba(59,130,246,0.1)]">
      <!-- Highlighted background layer -->
      <pre
        class="sql-highlight"
        aria-hidden="true"
      ><code ref="highlightCode" v-html="highlightedHtml"></code></pre>
      <!-- Foreground textarea for editing -->
      <textarea
        ref="textareaRef"
        v-model="sql"
        class="sql-textarea"
        placeholder="-- 输入 SQL 查询语句&#10;SELECT * FROM table_name LIMIT 100;"
        spellcheck="false"
        @scroll="syncScroll"
        @keydown="handleKeydown"
        @input="updateHighlight"
      ></textarea>
    </div>

    <!-- Status bar -->
    <div class="flex items-center justify-between px-2 py-1 text-[11px] text-base-content/60 bg-base-200 rounded-md border border-base-content/10">
      <div class="flex items-center gap-3">
        <span v-if="lastExecutionTime != null" class="flex items-center gap-1 text-primary font-medium">
          <SvgIcon name="clock" size="12" />
          {{ lastExecutionTime }}ms
        </span>
        <span v-if="lastRowCount != null" class="flex items-center gap-1 text-success">
          <SvgIcon name="barChart" size="12" />
          {{ lastRowCount }} 行
        </span>
        <span v-if="executedSql" class="flex items-center gap-1 max-w-[200px] overflow-hidden text-ellipsis whitespace-nowrap" :title="executedSql">
          已执行: {{ truncate(executedSql, 60) }}
        </span>
      </div>
      <div class="flex items-center gap-3">
        <span class="flex items-center gap-1">{{ charCount }} 字符</span>
        <span class="flex items-center gap-1">{{ lineCount }} 行</span>
        <span v-if="hasSelection" class="flex items-center gap-1 text-warning">已选中 {{ selectionLength }} 字符</span>
      </div>
    </div>

    <!-- Error display -->
    <div v-if="error" class="flex items-start gap-2 p-[10px_12px] rounded-lg bg-error/10 text-error text-[13px] leading-5">
      <SvgIcon name="alertCircle" size="16" class="mt-0.5" />
      <span>{{ error }}</span>
    </div>

    <!-- Query history -->
    <div v-if="history.length > 0" class="border-t border-base-content/10 pt-2">
      <div class="flex items-center justify-between mb-1.5 text-xs font-semibold text-base-content/60">
        <span>查询历史</span>
        <button @click="$emit('clear-history')" class="btn btn-ghost btn-xs">清空</button>
      </div>
      <div
        v-for="record in history.slice(0, 10)"
        :key="record.id"
        :class="[record.success ? 'text-success' : 'text-error']"
        class="px-2 py-1.5 rounded-md cursor-pointer text-xs font-mono transition-[background] duration-100 ease-in-out hover:bg-primary/10"
        @click="$emit('rerun', record.sql)"
      >
        <span class="block overflow-hidden text-ellipsis whitespace-nowrap">{{ truncate(record.sql, 80) }}</span>
        <span class="flex gap-3 text-[11px] text-base-content/60 font-inherit mt-0.5">
          <span>{{ formatTime(record.timestamp) }}</span>
          <span v-if="record.rowCount !== undefined">{{ record.rowCount }} 行</span>
          <span v-if="record.executionTime">{{ record.executionTime }}ms</span>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { format } from 'sql-formatter'
import hljs from 'highlight.js/lib/core'
import sqlLang from 'highlight.js/lib/languages/sql'
import type { QueryRecord, DBConnection } from '../../composables/useDBManager'

hljs.registerLanguage('sql', sqlLang)

const props = defineProps<{
  connection: DBConnection | null
  executing: boolean
  error: string | null
  history: QueryRecord[]
  initialSql?: string
}>()

const emit = defineEmits<{
  execute: [sql: string]
  clear: []
  rerun: [sql: string]
  'clear-history': []
}>()

const sql = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const highlightCode = ref<HTMLElement | null>(null)
const lastExecutionTime = ref<number | null>(null)
const lastRowCount = ref<number | null>(null)
const executedSql = ref('')

const highlightedHtml = ref('')

const charCount = computed(() => sql.value.length)
const lineCount = computed(() => sql.value.split('\n').length)
const hasSelection = computed(() => {
  const ta = textareaRef.value
  if (!ta) {return false}
  return ta.selectionStart !== ta.selectionEnd
})
const selectionLength = computed(() => {
  const ta = textareaRef.value
  if (!ta) {return 0}
  return ta.selectionEnd - ta.selectionStart
})

function truncate(str: string, len: number): string {
  return str.length > len ? str.slice(0, len) + '...' : str
}

function formatTime(iso: string): string {
  const d = new Date(iso)
  const h = d.getHours().toString().padStart(2, '0')
  const m = d.getMinutes().toString().padStart(2, '0')
  return `${h}:${m}`
}

// Syntax highlighting
function updateHighlight() {
  if (!sql.value) {
    highlightedHtml.value = ''
    return
  }
  try {
    const result = hljs.highlight(sql.value, { language: 'sql' })
    highlightedHtml.value = result.value
  } catch {
    highlightedHtml.value = escapeHtml(sql.value)
  }
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function syncScroll() {
  const pre = highlightCode.value?.parentElement
  const ta = textareaRef.value
  if (pre && ta) {
    pre.scrollTop = ta.scrollTop
    pre.scrollLeft = ta.scrollLeft
  }
}

// Execute: use selected text if any, otherwise full query
function handleExecute() {
  const ta = textareaRef.value
  if (!ta || !sql.value.trim()) {return}

  let queryText: string
  if (ta.selectionStart !== ta.selectionEnd) {
    queryText = sql.value.substring(ta.selectionStart, ta.selectionEnd).trim()
  } else {
    queryText = sql.value.trim()
  }

  if (!queryText) {return}
  executedSql.value = queryText
  emit('execute', queryText)
}

function handleFormat() {
  if (!sql.value.trim()) {return}
  try {
    sql.value = format(sql.value, {
      language: 'sql',
      keywordCase: 'upper',
      linesBetweenQueries: 2
    })
    nextTick(() => updateHighlight())
  } catch (e) {
    console.warn('SQL format failed:', e)
  }
}

function handleKeydown(e: KeyboardEvent) {
  // Ctrl+Enter or Cmd+Enter to execute
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    e.preventDefault()
    handleExecute()
    return
  }
  // Ctrl+Shift+F to format
  if (e.ctrlKey && e.shiftKey && e.key === 'F') {
    e.preventDefault()
    handleFormat()
    return
  }
  // Tab support for indentation
  if (e.key === 'Tab') {
    e.preventDefault()
    const ta = textareaRef.value
    if (!ta) {return}
    const start = ta.selectionStart
    const end = ta.selectionEnd
    sql.value = sql.value.substring(0, start) + '  ' + sql.value.substring(end)
    nextTick(() => {
      ta.selectionStart = ta.selectionEnd = start + 2
      updateHighlight()
    })
  }
}

// Record execution results when props change
watch(() => props.executing, (val) => {
  if (!val && lastExecutionTime.value != null) {
    // Execution finished
  }
})

watch(
  () => props.initialSql,
  (val) => {
    if (val) {
      sql.value = val
      nextTick(() => updateHighlight())
    }
  }
)

watch(sql, () => {
  updateHighlight()
})

onMounted(() => {
  if (sql.value) {
    nextTick(() => updateHighlight())
  }
})

// Expose methods for parent component to record execution stats
defineExpose({
  recordExecution: (time: number, rowCount: number) => {
    lastExecutionTime.value = time
    lastRowCount.value = rowCount
  }
})
</script>

<!-- Preserved: code editor syntax highlighting styles (SqlEditor specific) -->
<style>
.sql-highlight {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  margin: 0;
  padding: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-wrap: break-word;
  overflow: auto;
  pointer-events: none;
  color: transparent;
  background: var(--color-base-200);
}

.sql-highlight code {
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
}

/* highlight.js token colors for Catppuccin Mocha */
.sql-highlight :deep(.hljs-keyword) { color: #cba6f7; font-weight: 500; }
.sql-highlight :deep(.hljs-built_in) { color: #89b4fa; }
.sql-highlight :deep(.hljs-string) { color: #a6e3a1; }
.sql-highlight :deep(.hljs-number) { color: #fab387; }
.sql-highlight :deep(.hljs-comment) { color: #6c7086; font-style: italic; }
.sql-highlight :deep(.hljs-operator) { color: #94e2d5; }
.sql-highlight :deep(.hljs-variable) { color: #f38ba8; }
.sql-highlight :deep(.hljs-title) { color: #f9e2af; }
.sql-highlight :deep(.hljs-literal) { color: #fab387; }

.sql-textarea {
  position: relative;
  width: 100%;
  height: 100%;
  padding: 12px;
  border: none;
  background: transparent;
  color: var(--color-base-content);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  outline: none;
  white-space: pre-wrap;
  word-wrap: break-word;
  overflow: auto;
}

.sql-textarea::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.5;
}
</style>
