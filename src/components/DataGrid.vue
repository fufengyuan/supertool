<template>
  <div class="data-grid">
    <!-- Filter Bar -->
    <FilterBar
      :columns="columns"
      @apply="onFilterApply"
      @clear="onFilterClear"
    />

    <!-- Header: tabs + actions -->
    <div class="grid-header">
      <div class="grid-tabs">
        <button class="grid-tab" :class="{ active: viewMode === 'table' }" @click="viewMode = 'table'">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="3" y1="9" x2="21" y2="9" /><line x1="3" y1="15" x2="21" y2="15" /><line x1="9" y1="3" x2="9" y2="21" />
          </svg>
          表格
        </button>
        <button class="grid-tab" :class="{ active: viewMode === 'json' }" @click="viewMode = 'json'">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" />
          </svg>
          JSON
        </button>
      </div>
      <div class="grid-actions">
        <span class="grid-info" v-if="rows.length > 0">
          共 {{ total }} 条，显示 {{ (page - 1) * pageSize + 1 }}-{{ Math.min(page * pageSize, total) }}
        </span>
        <template v-if="dirtyRows.size > 0">
          <button class="btn btn-success btn-xs" @click="saveAllDirty">
            💾 保存 ({{ dirtyRows.size }} 行)
          </button>
          <button class="btn btn-ghost btn-xs" @click="discardAllDirty">取消修改</button>
        </template>
        <button class="btn btn-ghost btn-xs" @click="addNewRow">
          ➕ 新增行
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="rows.length === 0 && !loading" class="grid-empty">
      <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        <line x1="3" y1="9" x2="21" y2="9" /><line x1="9" y1="21" x2="9" y2="9" />
      </svg>
      <p>暂无数据</p>
      <button class="btn btn-primary btn-sm" @click="addNewRow">新增第一行</button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="grid-loading">加载中...</div>

    <!-- Table view -->
    <div v-if="viewMode === 'table' && (rows.length > 0 || newRowData)" class="grid-table-wrapper"
         @contextmenu.prevent="onTableContext($event)">
      <table class="grid-table">
        <thead>
          <tr>
            <th class="row-num">#</th>
            <th v-for="col in columns" :key="col" class="grid-col-th"
                :class="{ 'sortable': true, 'sort-asc': sortColumn === col && sortDirection === 'asc', 'sort-desc': sortColumn === col && sortDirection === 'desc' }"
                @click.stop="() => toggleSort(col)"
                :title="columnComments && columnComments[col] ? `${col}: ${columnComments[col]}` : col">
              <div class="grid-col">
                <div class="col-name-row">
                  <span class="col-name">{{ col }}</span>
                  <span v-if="sortColumn === col" class="sort-icon sort-icon-active">{{ sortDirection === 'asc' ? '▲' : '▼' }}</span>
                  <span v-else class="sort-icon sort-icon-hint">⇅</span>
                </div>
                <span v-if="columnComments && columnComments[col]" class="col-comment" :title="columnComments[col]">{{ columnComments[col] }}</span>
              </div>
            </th>
            <th class="row-actions" v-if="dirtyRows.size > 0 || newRowData">操作</th>
          </tr>
        </thead>
        <tbody>
          <!-- Existing rows -->
          <tr v-for="(row, idx) in rows" :key="`row-${idx}-${getRowHash(row)}`"
              :class="{ 'dirty-row': dirtyRows.has(idx), 'new-row': false }"
              @contextmenu.prevent="onRowContext($event, row, idx)">
            <td class="row-num">
              <span v-if="dirtyRows.has(idx)" class="dirty-indicator" title="已修改">*</span>
              <span v-else>{{ (page - 1) * pageSize + idx + 1 }}</span>
            </td>
            <td v-for="col in columns" :key="col" class="grid-cell"
                :class="{ 'is-pk': primaryKeyColumns.includes(col) }"
                @dblclick="startEdit(idx, col)"
                :title="String(formatValue(getDisplayValue(idx, col)) ?? 'NULL')">
              <!-- Editing cell -->
              <template v-if="editingCell.row === idx && editingCell.col === col">
                <input v-if="editingCell.isDatetime" v-model="editingValue"
                       type="datetime-local" class="cell-editor cell-editor-datetime"
                       @blur="finishEdit" @keydown.enter="finishEdit"
                       @keydown.escape="cancelEdit" @keydown.tab="handleEditTab($event, col)" />
                <input v-else-if="!isComplexType(getDisplayValue(idx, col))" ref="editInput" v-model="editingValue"
                       class="cell-editor" @blur="finishEdit" @keydown.enter="finishEdit"
                       @keydown.escape="cancelEdit" @keydown.tab="handleEditTab($event, col)" />
                <textarea v-else ref="editInput" v-model="editingValue"
                          class="cell-editor cell-editor-multiline"
                          @blur="finishEdit" @keydown.ctrl.enter="finishEdit"
                          @keydown.escape="cancelEdit"></textarea>
              </template>
              <!-- Normal cell -->
              <template v-else>
                <span v-if="getDisplayValue(idx, col) === null || getDisplayValue(idx, col) === undefined" class="null-value">NULL</span>
                <span v-else class="cell-value">{{ formatValue(getDisplayValue(idx, col)) }}</span>
              </template>
            </td>
            <td class="row-actions" v-if="dirtyRows.size > 0 || newRowData">
              <button class="row-action-btn delete-btn" @click.stop="confirmDeleteRow(row, idx)" title="删除行">🗑️</button>
            </td>
          </tr>

          <!-- New row -->
          <tr v-if="newRowData" class="new-row">
            <td class="row-num"><span class="new-indicator">+</span></td>
            <td v-for="col in columns" :key="col" class="grid-cell"
                @dblclick="startNewEdit(col)">
              <template v-if="editingCell.row === -1 && editingCell.col === col">
                <input v-if="!isComplexType(newRowData[col])" ref="editInput" v-model="editingValue"
                       class="cell-editor" @blur="finishNewEdit" @keydown.enter="finishNewEdit"
                       @keydown.escape="cancelNewEdit" @keydown.tab="handleEditTab($event, col)" />
                <textarea v-else ref="editInput" v-model="editingValue"
                          class="cell-editor cell-editor-multiline"
                          @blur="finishNewEdit" @keydown.ctrl.enter="finishNewEdit"
                          @keydown.escape="cancelNewEdit"></textarea>
              </template>
              <template v-else>
                <span v-if="newRowData[col] === null || newRowData[col] === undefined" class="null-value">NULL</span>
                <span v-else class="cell-value">{{ formatValue(newRowData[col]) }}</span>
              </template>
            </td>
            <td class="row-actions" v-if="newRowData">
              <button class="row-action-btn save-btn" @click.stop="saveNewRow" title="保存新行">💾</button>
              <button class="row-action-btn delete-btn" @click.stop="cancelNewRow" title="取消新增">✖</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- JSON view -->
    <pre v-if="viewMode === 'json' && rows.length > 0" class="grid-json"
    >{{ formatJson(rows) }}</pre>

    <!-- Pagination -->
    <div v-if="total > pageSize" class="grid-pagination">
      <button class="btn btn-ghost btn-sm" :disabled="page <= 1" @click="handlePrevPage">
        ‹ 上一页
      </button>
      <span class="page-info">第 {{ page }} / {{ totalPages }} 页</span>
      <button class="btn btn-ghost btn-sm" :disabled="page >= totalPages" @click="handleNextPage">
        下一页 ›
      </button>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="contextMenu.visible" class="context-menu"
           :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }" @click.stop>
        <div v-for="(item, i) in contextMenu.items" :key="i" class="context-menu-item"
             :class="{ disabled: item.disabled }" @click="item.disabled ? null : item.action()">
          <span class="context-menu-icon">{{ item.icon }}</span>
          <span class="context-menu-label">{{ item.label }}</span>
        </div>
      </div>
    </Teleport>
    <div v-if="contextMenu.visible" class="context-menu-overlay" @click="closeContextMenu"></div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, nextTick } from 'vue'
import { useToast } from '@/composables/useToast'
import FilterBar, { type FilterCondition } from './FilterBar.vue'

const props = defineProps<{
  rows: Record<string, unknown>[]
  total: number
  page: number
  pageSize: number
  loading: boolean
  primaryKeyColumns?: string[]  // e.g. ['id']
  columnComments?: Record<string, string>  // column name -> comment
  sortColumn?: string | null
  sortDirection?: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  'prev-page': []
  'next-page': []
  'filter': [conditions: FilterCondition[]]
  'filter-clear': []
  'update-row': [index: number, oldRow: Record<string, unknown>, newRow: Record<string, unknown>]
  'insert-row': [row: Record<string, unknown>]
  'delete-row': [row: Record<string, unknown>, index: number]
  'refresh': []
  'batch-update': [updates: Array<{ oldRow: Record<string, unknown>; newRow: Record<string, unknown> }>]
  'sort': [column: string, direction: 'asc' | 'desc']
}>()

function getRowHash(row: Record<string, unknown>): string {
  return JSON.stringify(row).slice(0, 50)
}

// Clear dirty state on page change (page-local indices become invalid)
function handlePrevPage() {
  dirtyRows.value.clear()
  originalRows.value.clear()
  localEdits.value.clear()
  emit('prev-page')
}

function handleNextPage() {
  dirtyRows.value.clear()
  originalRows.value.clear()
  localEdits.value.clear()
  emit('next-page')
}

const viewMode = ref<'table' | 'json'>('table')
const toast = useToast()

function toggleSort(col: string) {
  const currentCol = props.sortColumn
  const currentDir = props.sortDirection || 'asc'
  let nextCol: string | null = col
  let nextDir: 'asc' | 'desc' = 'asc'
  
  if (currentCol === col) {
    nextDir = currentDir === 'asc' ? 'desc' : 'asc'
  }
  emit('sort', nextCol, nextDir)
}

const columns = computed(() => {
  const keySet = new Set<string>()
  if (props.rows.length > 0) {
    for (const row of props.rows) {
      for (const key of Object.keys(row)) keySet.add(key)
    }
  } else if (newRowData.value) {
    for (const key of Object.keys(newRowData.value)) keySet.add(key)
  }
  return [...keySet]
})

const totalPages = computed(() => Math.max(1, Math.ceil(props.total / props.pageSize)))

// ============ Dirty tracking ============
const dirtyRows = ref(new Set<number>())
const originalRows = ref<Map<number, Record<string, unknown>>>(new Map())
const localEdits = ref<Map<number, Record<string, unknown>>>(new Map())

// Get the display value for a cell (including local edits)
function getDisplayValue(rowIdx: number, col: string): unknown {
  const edit = localEdits.value.get(rowIdx)
  if (edit && col in edit) return edit[col]
  return props.rows[rowIdx]?.[col]
}

// Mark a row as dirty (for highlight purposes)
function markDirty(idx: number) {
  if (!originalRows.value.has(idx)) {
    originalRows.value.set(idx, JSON.parse(JSON.stringify(props.rows[idx])))
  }
  dirtyRows.value.add(idx)
}

// ============ Cell Editing ============
const editingCell = ref<{ row: number; col: string; isDatetime: boolean }>({ row: -1, col: '', isDatetime: false })
const editingValue = ref('')
const editingOriginal = ref<unknown>(null)  // store original value for comparison
const editInput = ref<HTMLInputElement[] | null>(null)

// Check if a value looks like a datetime (for datetime-local picker)
function isDatetimeValue(val: unknown): boolean {
  if (val instanceof Date && !isNaN(val.getTime())) return true
  if (typeof val === 'string' && /^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}/.test(val)) return true
  return false
}

// Convert value to datetime-local input format (YYYY-MM-DDTHH:mm)
function toDatetimeLocal(val: unknown): string {
  const d = val instanceof Date ? val : new Date(val as string)
  if (isNaN(d.getTime())) return ''
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`
}

// Convert datetime-local input value back to appropriate type
function fromDatetimeLocal(input: string, originalValue: unknown): unknown {
  if (!input) return null
  const d = new Date(input)
  if (isNaN(d.getTime())) return input
  // Return same type as original
  if (originalValue instanceof Date) return d
  return d
}

// Check if value actually changed (deep comparison for primitives and dates)
function valueChanged(newVal: unknown, oldVal: unknown): boolean {
  if (newVal === oldVal) return false
  if (newVal === null && oldVal === undefined) return false
  if (newVal === undefined && oldVal === null) return false
  if (newVal === '' && (oldVal === null || oldVal === undefined)) return false
  if ((newVal === null || newVal === undefined || newVal === '') && oldVal === '') return false
  if (newVal instanceof Date && oldVal instanceof Date) return newVal.getTime() !== oldVal.getTime()
  if (typeof newVal === 'number' && typeof oldVal === 'number') return newVal !== oldVal
  if (typeof newVal === 'boolean' && typeof oldVal === 'boolean') return newVal !== oldVal
  return String(newVal) !== String(oldVal)
}

function startEdit(idx: number, col: string) {
  const val = getDisplayValue(idx, col)
  const isDt = isDatetimeValue(val)
  editingCell.value = { row: idx, col, isDatetime: isDt }
  editingValue.value = isDt ? toDatetimeLocal(val) : (val === null || val === undefined ? '' : formatValue(val))
  editingOriginal.value = val  // store for comparison
  // DON'T mark dirty yet — wait until finishEdit to check if value actually changed
  nextTick(() => {
    const inputs = editInput.value
    if (inputs && inputs.length > 0) {
      const el = inputs.find(i => i)
      el?.focus()
      if (el && el instanceof HTMLInputElement && !isDt) el.select()
    }
  })
}

function finishEdit() {
  const { row, col, isDatetime } = editingCell.value
  if (row < 0 || !col) return

  const oldRow = props.rows[row]
  const originalValue = editingOriginal.value ?? oldRow?.[col]

  let parsedValue: unknown
  if (isDatetime) {
    parsedValue = fromDatetimeLocal(editingValue.value, originalValue)
  } else {
    parsedValue = parseCellValue(editingValue.value, originalValue)
  }

  // Only mark dirty if value actually changed
  if (valueChanged(parsedValue, originalValue)) {
    if (!originalRows.value.has(row)) {
      originalRows.value.set(row, JSON.parse(JSON.stringify(oldRow)))
    }
    const edit = localEdits.value.get(row) || { ...oldRow }
    edit[col] = parsedValue
    localEdits.value.set(row, edit)
    dirtyRows.value.add(row)
  } else {
    // Value unchanged — remove this column's edit
    const edit = localEdits.value.get(row)
    if (edit) {
      delete edit[col]
      // If no more edits in this row, clean up
      if (Object.keys(edit).length === 0) {
        localEdits.value.delete(row)
        dirtyRows.value.delete(row)
        originalRows.value.delete(row)
      }
    }
  }

  editingCell.value = { row: -1, col: '', isDatetime: false }
  editingOriginal.value = null
}

function discardAllDirty() {
  dirtyRows.value.forEach(idx => {
    localEdits.value.delete(idx)
  })
  dirtyRows.value.clear()
  originalRows.value.clear()
  toast.info('已取消所有修改')
}

function cancelEdit() {
  editingCell.value = { row: -1, col: '', isDatetime: false }
  editingOriginal.value = null
}

function handleEditTab(event: KeyboardEvent, currentCol: string) {
  event.preventDefault()
  const colIdx = columns.value.indexOf(currentCol)
  if (colIdx < 0) return

  const currentRow = editingCell.value.row  // capture before finishEdit resets it
  const nextColIdx = event.shiftKey
    ? (colIdx - 1 + columns.value.length) % columns.value.length
    : (colIdx + 1) % columns.value.length

  finishEdit()

  if (nextColIdx === 0 && !event.shiftKey) {
    // Wrapped to first column, don't continue editing
  } else {
    startEdit(currentRow, columns.value[nextColIdx])
  }
}

function isComplexType(val: unknown): boolean {
  if (val === null || val === undefined) return false
  // Date objects are simple (handled by datetime picker)
  if (val instanceof Date) return false
  if (typeof val === 'object') return true
  if (typeof val === 'string') {
    try {
      const parsed = JSON.parse(val)
      if (typeof parsed === 'boolean' || typeof parsed === 'number' || parsed === null) return false
      if (typeof parsed === 'object') return true
    } catch {
      // Not JSON
    }
  }
  if (typeof val === 'string' && val.includes('\n')) return true
  if (typeof val === 'string' && val.length > 200) return true
  return false
}

function parseCellValue(input: string, originalValue: unknown): unknown {
  if (input === '' || input.toLowerCase() === 'null') return null

  // Date objects — parse as Date
  if (originalValue instanceof Date) {
    const d = new Date(input)
    return isNaN(d.getTime()) ? input : d
  }

  if (typeof originalValue === 'number') {
    if (input.toLowerCase() === 'true' || input === '1') return 1
    if (input.toLowerCase() === 'false' || input === '0') return 0
    const num = Number(input)
    return isNaN(num) ? input : num
  }
  if (typeof originalValue === 'boolean') {
    return input.toLowerCase() === 'true' || input === '1'
  }
  if (typeof originalValue === 'object' && originalValue !== null) {
    try { return JSON.parse(input) } catch { return input }
  }
  return input
}

// ============ New Row ============
const newRowData = ref<Record<string, unknown> | null>(null)

function addNewRow() {
  if (newRowData.value) {
    toast.warning('请先保存或取消当前新增行')
    return
  }
  if (columns.value.length === 0) {
    toast.info('无可用列，无法新增')
    return
  }
  const emptyRow: Record<string, unknown> = {}
  for (const col of columns.value) emptyRow[col] = null
  newRowData.value = emptyRow
}

function startNewEdit(col: string) {
  if (!newRowData.value) return
  editingCell.value = { row: -1, col, isDatetime: false }
  const val = newRowData.value[col]
  editingValue.value = val === null || val === undefined ? '' : String(val)
  nextTick(() => {
    const el = editInput.value?.find(i => i)
    el?.focus()
  })
}

function finishNewEdit() {
  const { col } = editingCell.value
  if (!col || !newRowData.value) return
  newRowData.value[col] = parseCellValue(editingValue.value, newRowData.value[col])
  editingCell.value = { row: -1, col: '', isDatetime: false }
}

function cancelNewEdit() {
  editingCell.value = { row: -1, col: '', isDatetime: false }
}

function saveNewRow() {
  if (!newRowData.value) return
  emit('insert-row', { ...newRowData.value })
  newRowData.value = null
  editingCell.value = { row: -1, col: '', isDatetime: false }
}

function cancelNewRow() {
  newRowData.value = null
  editingCell.value = { row: -1, col: '', isDatetime: false }
}

// ============ Delete ============
function confirmDeleteRow(row: Record<string, unknown>, idx: number) {
  emit('delete-row', row, idx)
}

// ============ Save All Dirty ============
function saveAllDirty() {
  const toSave: number[] = []
  dirtyRows.value.forEach(idx => toSave.push(idx))
  toSave.sort((a, b) => a - b)

  const updates: Array<{ oldRow: Record<string, unknown>; newRow: Record<string, unknown> }> = []
  for (const idx of toSave) {
    const original = originalRows.value.get(idx)
    const edit = localEdits.value.get(idx)
    if (edit && original) {
      // Only include if there are actual differences
      const hasChanges = Object.keys(edit).some(key => valueChanged(edit[key], original[key]))
      if (hasChanges) {
        updates.push({ oldRow: { ...original }, newRow: { ...edit } })
      }
    }
  }

  if (updates.length > 0) {
    emit('batch-update', updates)
  } else {
    toast.info('没有需要保存的修改')
  }

  // Clear dirty state after emitting
  dirtyRows.value.clear()
  originalRows.value.clear()
  localEdits.value.clear()
}

// ============ Context Menu ============
interface ContextMenuItem {
  icon: string
  label: string
  action: () => void
  disabled?: boolean
}

const contextMenu = ref<{
  visible: boolean
  x: number
  y: number
  items: ContextMenuItem[]
}>({ visible: false, x: 0, y: 0, items: [] })

function closeContextMenu() {
  contextMenu.value.visible = false
}

function onRowContext(event: MouseEvent, row: Record<string, unknown>, idx: number) {
  const displayRow = localEdits.value.has(idx)
    ? { ...row, ...localEdits.value.get(idx) }
    : row
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    items: [
      { icon: '👁️', label: '查看完整值', action: () => { toast.info(String(displayRow)); closeContextMenu() } },
      { icon: '📋', label: '复制行 JSON', action: () => { navigator.clipboard?.writeText(JSON.stringify(displayRow)); toast.info('已复制'); closeContextMenu() } },
      { icon: '🗑️', label: '删除此行', action: () => { confirmDeleteRow(displayRow, idx); closeContextMenu() } },
    ]
  }
}

function onTableContext(event: MouseEvent) {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    items: [
      { icon: '➕', label: '新增行', action: () => { addNewRow(); closeContextMenu() } },
      { icon: '🔄', label: '刷新数据', action: () => { emit('refresh'); closeContextMenu() } },
    ]
  }
}

// ============ Utilities ============
function formatValue(val: unknown): string {
  if (val === null || val === undefined) return ''
  if (typeof val === 'boolean') return val ? 'true' : 'false'
  if (val instanceof Boolean) return val.valueOf() ? 'true' : 'false'

  // Date objects from database drivers
  if (val instanceof Date) {
    if (isNaN(val.getTime())) return String(val)
    const pad = (n: number) => String(n).padStart(2, '0')
    return `${val.getFullYear()}-${pad(val.getMonth() + 1)}-${pad(val.getDate())} ${pad(val.getHours())}:${pad(val.getMinutes())}:${pad(val.getSeconds())}`
  }

  // BigInt (common for large IDs)
  if (typeof val === 'bigint') return val.toString()

  // Buffer / Uint8Array (binary data, show hex preview)
  if (val instanceof Uint8Array) {
    const hex = Array.from((val as Uint8Array).slice(0, 8)).map(b => b.toString(16).padStart(2, '0')).join('')
    return (val as Uint8Array).length > 8 ? `0x${hex}… (${(val as Uint8Array).length} bytes)` : `0x${hex}`
  }

  // Unix timestamps (seconds or milliseconds)
  if (typeof val === 'number') {
    if (val > 1e11 && val < 1e15) {
      // Millisecond timestamp
      const d = new Date(val)
      if (!isNaN(d.getTime())) {
        const pad = (n: number) => String(n).padStart(2, '0')
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
      }
    }
    if (val > 1e8 && val < 1e11) {
      // Second timestamp
      const d = new Date(val * 1000)
      if (!isNaN(d.getTime())) {
        const pad = (n: number) => String(n).padStart(2, '0')
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
      }
    }
    return String(val)
  }

  if (typeof val === 'string') {
    // Format ISO date/time strings (e.g. 2024-01-15T08:30:00Z, 2024-01-15 08:30:00)
    if (/^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}/.test(val)) {
      const d = new Date(val)
      if (!isNaN(d.getTime())) {
        const pad = (n: number) => String(n).padStart(2, '0')
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
      }
    }
    // Format date-only strings (e.g. 2024-01-15)
    if (/^\d{4}-\d{2}-\d{2}$/.test(val)) {
      const d = new Date(val)
      if (!isNaN(d.getTime())) {
        const pad = (n: number) => String(n).padStart(2, '0')
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
      }
    }
    // Numeric strings that look like timestamps (e.g. "1705305000")
    const numStr = Number(val)
    if (!isNaN(numStr) && val.trim().length >= 10) {
      const ts = numStr > 1e11 ? numStr : numStr * 1000
      const d = new Date(ts)
      if (!isNaN(d.getTime()) && d.getFullYear() >= 2000 && d.getFullYear() <= 2100) {
        const pad = (n: number) => String(n).padStart(2, '0')
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
      }
    }

    try {
      let parsed = JSON.parse(val)
      if (Array.isArray(parsed)) {
        parsed = parsed.map(item => {
          if (typeof item === 'string') {
            try { return JSON.parse(item) } catch { return item }
          }
          return item
        })
      }
      if (typeof parsed === 'boolean') return parsed ? 'true' : 'false'
      if (parsed === null) return 'NULL'
      if (typeof parsed === 'number') return String(parsed)
      if (typeof parsed === 'object') {
        const json = JSON.stringify(parsed)
        if (json.length > 100) return json.slice(0, 100) + '…'
        return json
      }
    } catch {
      // Not JSON
    }
  }
  return String(val)
}

function formatJson(rows: Record<string, unknown>[]): string {
  return JSON.stringify(rows, null, 2)
}

// ============ Filter Bar ============
function onFilterApply(conditions: FilterCondition[]) {
  emit('filter', conditions)
}

function onFilterClear() {
  emit('filter-clear')
}
</script>

<style scoped>
.data-grid {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.grid-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  flex-shrink: 0;
  gap: 8px;
  min-height: 40px;
}

.grid-tabs {
  display: flex;
  gap: 2px;
}

.grid-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: 12px;
  border: none;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.grid-tab:hover {
  background: oklch(var(--b2));
  color: oklch(var(--bc));
}

.grid-tab.active {
  background: oklch(var(--p));
  color: white;
}

.grid-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.grid-info {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  padding: 2px 8px;
  background: oklch(var(--b2));
  border-radius: 4px;
  border: 1px solid oklch(var(--bc) / 0.1);
}

.grid-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: oklch(var(--bc) / 0.6);
  gap: 12px;
}

.grid-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: oklch(var(--bc) / 0.6);
  font-size: 14px;
}

.grid-table-wrapper {
  flex: 1;
  overflow: auto;
  min-height: 0;
  border-radius: 0 0 8px 8px;
}

/* 自定义滚动条 */
.grid-table-wrapper::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}
.grid-table-wrapper::-webkit-scrollbar-track {
  background: oklch(var(--b2));
}
.grid-table-wrapper::-webkit-scrollbar-thumb {
  background: oklch(var(--bc) / 0.1);
  border-radius: 4px;
}
.grid-table-wrapper::-webkit-scrollbar-thumb:hover {
  background: oklch(var(--bc) / 0.6);
}

.grid-table {
  border-collapse: collapse;
  width: max-content;
  min-width: 100%;
}

.grid-table th,
.grid-table td {
  border-right: 1px solid oklch(var(--bc) / 0.1);
  border-bottom: 1px solid oklch(var(--bc) / 0.1);
  padding: 8px 14px;
  text-align: left;
  vertical-align: middle;
  font-size: 13px;
  line-height: 1.5;
  white-space: nowrap;
}

.grid-table td {
  white-space: nowrap;
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-num {
  width: 56px;
  min-width: 56px;
  text-align: center;
  background: oklch(var(--b2));
  color: oklch(var(--bc) / 0.6);
  font-size: 11px;
  font-family: 'SF Mono', 'Fira Code', monospace;
  position: sticky;
  left: 0;
  z-index: 3;
  border-right: 2px solid oklch(var(--bc) / 0.1);
}

.grid-col-th {
  background: oklch(var(--b2));
  border-top: 1px solid oklch(var(--bc) / 0.1);
  border-bottom: 2px solid oklch(var(--bc) / 0.1);
  position: sticky;
  top: 0;
  z-index: 2;
  user-select: none;
  white-space: nowrap;
  font-weight: 600;
  font-size: 11px;
  letter-spacing: 0.5px;
  min-width: 60px;
}

.grid-col-th .grid-col {
  padding: 8px 12px;
  min-height: 32px;
}

.grid-col-th.sortable {
  cursor: pointer;
}

.grid-col-th.sortable:hover {
  background: oklch(var(--bc) / 0.1);
}

.grid-col {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1px;
  padding: 4px 12px 6px;
  min-height: 32px;
}

.col-name-row {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
}

.col-name {
  font-weight: 600;
  font-size: 11px;
  color: oklch(var(--bc));
  text-transform: uppercase;
  letter-spacing: 0.5px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.col-comment {
  font-size: 9px;
  color: oklch(var(--bc) / 0.6);
  opacity: 0.5;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 400;
  text-transform: none;
  letter-spacing: normal;
  line-height: 1.2;
}

.sort-icon {
  font-size: 11px;
  flex-shrink: 0;
  transition: all 0.15s ease;
}

.sort-icon-hint {
  color: oklch(var(--bc) / 0.6);
  opacity: 0.3;
  font-size: 12px;
}

.grid-col-th.sortable:hover .sort-icon-hint {
  opacity: 0.7;
  color: oklch(var(--p));
}

.sort-icon-active {
  color: oklch(var(--p));
  font-weight: 700;
  font-size: 10px;
}

.grid-col-th.sort-asc .col-name,
.grid-col-th.sort-desc .col-name {
  color: oklch(var(--p));
}

.grid-cell {
  padding: 6px 12px;
  position: relative;
  cursor: cell;
  transition: background 0.1s ease;
}

.grid-cell:hover {
  background: oklch(var(--p) / 0.1);
  outline: 1px solid oklch(var(--bc) / 0.1);
  outline-offset: -1px;
}

.grid-cell.is-pk {
  font-weight: 600;
  color: oklch(var(--p));
  background: rgba(var(--primary-rgb, 100, 100, 255), 0.05);
}

.dirty-row {
  background: rgba(245, 158, 11, 0.06);
  border-left: 3px solid oklch(var(--wa));
}

.dirty-row:hover {
  background: rgba(245, 158, 11, 0.12);
}

.new-row {
  background: rgba(34, 197, 94, 0.06);
  border-left: 3px solid oklch(var(--su));
}

.new-row:hover {
  background: rgba(34, 197, 94, 0.12);
}

.grid-table tbody tr:hover {
  background: oklch(var(--p) / 0.1);
}

.grid-table tbody tr:nth-child(even):not(.dirty-row):not(.new-row) {
  background: rgba(0, 0, 0, 0.015);
}

.grid-table tbody tr:nth-child(even):not(.dirty-row):not(.new-row):hover {
  background: oklch(var(--p) / 0.1);
}

.dirty-indicator {
  color: oklch(var(--wa));
  font-weight: 700;
  font-size: 14px;
}

.new-row {
  background: rgba(34, 197, 94, 0.08);
}

.new-indicator {
  color: oklch(var(--su));
  font-weight: 700;
  font-size: 14px;
}

.cell-value {
  display: inline;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12.5px;
  color: oklch(var(--bc));
}

.null-value {
  color: oklch(var(--bc) / 0.6);
  opacity: 0.45;
  font-style: italic;
  font-size: 11px;
  font-family: 'SF Mono', 'Fira Code', monospace;
  letter-spacing: 0.5px;
}

.cell-editor {
  width: 100%;
  padding: 4px 6px;
  border: 2px solid oklch(var(--p));
  border-radius: 4px;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
  font-size: 13px;
  font-family: inherit;
  outline: none;
  box-sizing: border-box;
}

.cell-editor-datetime {
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 12px;
  min-width: 180px;
}

.cell-editor-datetime::-webkit-calendar-picker-indicator {
  cursor: pointer;
  opacity: 0.6;
}

.cell-editor-datetime::-webkit-calendar-picker-indicator:hover {
  opacity: 1;
}

.cell-editor-multiline {
  min-height: 80px;
  resize: vertical;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
}

.row-actions {
  width: 60px;
  text-align: center;
  background: oklch(var(--b2));
}

.row-action-btn {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  padding: 4px;
  border-radius: 4px;
  transition: background 0.15s;
}

.row-action-btn:hover {
  background: oklch(var(--bc) / 0.1);
}

.grid-pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 8px 12px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b1));
  flex-shrink: 0;
  min-height: 38px;
}

.page-info {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.grid-json {
  flex: 1;
  padding: 16px;
  margin: 0;
  overflow: auto;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.5;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
}

/* Context Menu */
.context-menu {
  position: fixed;
  z-index: 9999;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
  padding: 4px;
  min-width: 180px;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  color: oklch(var(--bc));
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}

.context-menu-item:hover {
  background: oklch(var(--b2));
}

.context-menu-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.context-menu-overlay {
  position: fixed;
  inset: 0;
  z-index: 9998;
}
</style>
