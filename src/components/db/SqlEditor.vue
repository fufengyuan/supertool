<template>
  <div class="sql-editor">
    <div class="editor-toolbar">
      <div class="editor-connection-info" v-if="connection">
        <span class="editor-conn-badge">{{ dbTypeIcon(connection.type) }} {{ connection.name }}</span>
      </div>
      <div class="editor-actions">
        <button
          @click="handleExecute"
          class="btn btn-primary btn-sm"
          :disabled="executing"
          title="执行 (Ctrl+Enter)"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="5 3 19 12 5 21 5 3" />
          </svg>
          {{ executing ? '执行中...' : '执行' }}
        </button>
        <button
          @click="handleFormat"
          class="btn btn-ghost btn-sm"
          title="格式化 (Ctrl+Shift+F)"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="21" y1="10" x2="3" y2="10" />
            <line x1="21" y1="6" x2="3" y2="6" />
            <line x1="21" y1="14" x2="3" y2="14" />
            <line x1="21" y1="18" x2="3" y2="18" />
          </svg>
          格式化
        </button>
        <button @click="$emit('clear')" class="btn btn-ghost btn-sm" title="清空">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Editor with syntax highlighting overlay -->
    <div class="editor-container">
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
    <div class="editor-status">
      <div class="status-left">
        <span v-if="lastExecutionTime != null" class="status-item status-time">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <polyline points="12 6 12 12 16 14" />
          </svg>
          {{ lastExecutionTime }}ms
        </span>
        <span v-if="lastRowCount != null" class="status-item status-rows">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="3" y1="9" x2="21" y2="9" />
          </svg>
          {{ lastRowCount }} 行
        </span>
        <span v-if="executedSql" class="status-item status-sql" :title="executedSql">
          已执行: {{ truncate(executedSql, 60) }}
        </span>
      </div>
      <div class="status-right">
        <span class="status-item status-chars">{{ charCount }} 字符</span>
        <span class="status-item status-lines">{{ lineCount }} 行</span>
        <span v-if="hasSelection" class="status-item status-selection">已选中 {{ selectionLength }} 字符</span>
      </div>
    </div>

    <!-- Error display -->
    <div v-if="error" class="sql-error">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      <span>{{ error }}</span>
    </div>

    <!-- Query history -->
    <div v-if="history.length > 0" class="query-history">
      <div class="history-header">
        <span>查询历史</span>
        <button @click="$emit('clear-history')" class="btn btn-ghost btn-sm">清空</button>
      </div>
      <div
        v-for="record in history.slice(0, 10)"
        :key="record.id"
        class="history-item"
        :class="{ success: record.success, failed: !record.success }"
        @click="$emit('rerun', record.sql)"
      >
        <span class="history-sql">{{ truncate(record.sql, 80) }}</span>
        <span class="history-meta">
          <span>{{ formatTime(record.timestamp) }}</span>
          <span v-if="record.rowCount !== undefined">{{ record.rowCount }} 行</span>
          <span v-if="record.executionTime">{{ record.executionTime }}ms</span>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
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
  if (!ta) return false
  return ta.selectionStart !== ta.selectionEnd
})
const selectionLength = computed(() => {
  const ta = textareaRef.value
  if (!ta) return 0
  return ta.selectionEnd - ta.selectionStart
})

function dbTypeIcon(type: string): string {
  const icons: Record<string, string> = {
    mysql: '🐬',
    postgresql: '🐘',
    redis: '🔴',
    sqlite: '📄'
  }
  return icons[type] || '🗄️'
}

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
  if (!ta || !sql.value.trim()) return

  let queryText: string
  if (ta.selectionStart !== ta.selectionEnd) {
    queryText = sql.value.substring(ta.selectionStart, ta.selectionEnd).trim()
  } else {
    queryText = sql.value.trim()
  }

  if (!queryText) return
  executedSql.value = queryText
  emit('execute', queryText)
}

function handleFormat() {
  if (!sql.value.trim()) return
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
    if (!ta) return
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

<style scoped>
.sql-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 8px;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.editor-conn-badge {
  font-size: 12px;
  padding: 3px 8px;
  border-radius: 4px;
  background: var(--primary-light);
  color: var(--primary-color);
  font-weight: 500;
}

.editor-actions {
  display: flex;
  gap: 6px;
}

/* Editor container with overlay */
.editor-container {
  position: relative;
  flex: 1;
  min-height: 120px;
  max-height: 300px;
  border: 1.5px solid var(--input-border);
  border-radius: 8px;
  overflow: hidden;
  transition: border-color 0.15s ease;
}

.editor-container:focus-within {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-light);
}

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
  background: var(--input-bg);
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
  color: var(--main-text);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  outline: none;
  white-space: pre-wrap;
  word-wrap: break-word;
  overflow: auto;
  /* Text must be somewhat transparent to see highlights behind */
  color: var(--main-text);
}

.sql-textarea::placeholder {
  color: var(--main-text-secondary);
  opacity: 0.5;
}

/* Status bar */
.editor-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  font-size: 11px;
  color: var(--main-text-secondary);
  background: var(--input-bg);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.status-left, .status-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-time {
  color: var(--primary-color);
  font-weight: 500;
}

.status-rows {
  color: var(--success-color);
}

.status-sql {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-selection {
  color: var(--warning-color);
}

/* Error display */
.sql-error {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  background: rgba(210, 15, 57, 0.1);
  color: var(--danger-color);
  font-size: 13px;
  line-height: 1.5;
}

.sql-error svg {
  flex-shrink: 0;
  margin-top: 1px;
}

/* Query history */
.query-history {
  border-top: 1px solid var(--border-color);
  padding-top: 8px;
}

.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--main-text-secondary);
}

.history-item {
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  transition: background 0.1s ease;
}

.history-item:hover {
  background: var(--primary-light);
}

.history-item.success {
  color: var(--success-color);
}

.history-item.failed {
  color: var(--danger-color);
}

.history-sql {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-meta {
  display: flex;
  gap: 12px;
  font-size: 11px;
  color: var(--main-text-secondary);
  font-family: inherit;
  margin-top: 2px;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}
</style>
