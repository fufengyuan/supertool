<template>
  <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
    <!-- Filter Bar -->
    <FilterBar
      :columns="columns"
      @apply="onFilterApply"
      @clear="onFilterClear"
    />

    <!-- Header: tabs + actions -->
    <div class="flex items-center justify-between px-3 py-1.5 border-b border-base-content/10 bg-base-100 shrink-0 gap-2 min-h-10">
      <div class="flex gap-0.5">
        <button class="btn btn-xs" :class="viewMode === 'table' ? 'btn-primary' : 'btn-ghost'" @click="viewMode = 'table'">
          <SvgIcon name="grid" size="14" />
          表格
        </button>
        <button class="btn btn-xs" :class="viewMode === 'json' ? 'btn-primary' : 'btn-ghost'" @click="viewMode = 'json'">
          <SvgIcon name="code" size="14" />
          JSON
        </button>
      </div>
      <div class="flex items-center gap-2">
        <span v-if="rows.length > 0" class="text-xs text-base-content/60 px-2 py-0.5 bg-base-200 rounded-md border border-base-content/10">
          共 {{ total }} 条，显示 {{ (page - 1) * pageSize + 1 }}-{{ Math.min(page * pageSize, total) }}
        </span>
        <template v-if="dirtyRows.size > 0">
          <button class="btn btn-success btn-xs" @click="saveAllDirty">
            <SvgIcon name="download" size="14" />  保存 ({{ dirtyRows.size }} 行)
          </button>
          <button class="btn btn-ghost btn-xs" @click="discardAllDirty">取消修改</button>
        </template>
        <button class="btn btn-ghost btn-xs" @click="addNewRow">
          <SvgIcon name="plus" size="14" />  新增行
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="rows.length === 0 && !loading" class="flex flex-col items-center justify-center p-12 text-base-content/60 gap-3">
      <SvgIcon name="grid" size="40" />
      <p>暂无数据</p>
      <button class="btn btn-primary btn-sm" @click="addNewRow">新增第一行</button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex items-center justify-center p-8 text-base-content/60 text-sm">加载中...</div>

    <!-- Table view -->
    <div v-if="viewMode === 'table' && (rows.length > 0 || newRowData)" class="flex-1 overflow-auto min-h-0 rounded-b-lg"
         @contextmenu.prevent="onTableContext($event)">
      <table class="border-collapse w-max min-w-full">
        <thead>
          <tr>
            <th class="w-14 min-w-14 text-center bg-base-200 text-base-content/60 text-[11px] font-mono sticky left-0 top-0 z-40 border-r-2 border-b border-base-content/10 px-3.5 py-2 align-middle whitespace-nowrap">#</th>
            <th v-for="col in columns" :key="col"
                class="bg-base-200 border-t border-b-2 border-r border-base-content/10 sticky top-0 z-20 select-none font-semibold text-[11px] tracking-wider min-w-[60px] px-0 py-0 align-middle whitespace-nowrap cursor-pointer group"
                :class="{ 'text-primary': sortColumn === col }"
                @click.stop="() => toggleSort(col)"
                :title="columnComments && columnComments[col] ? `${col}: ${columnComments[col]}` : col">
              <div class="flex flex-col items-start gap-px px-3 py-2 min-h-8">
                <div class="flex items-center gap-1 w-full">
                  <span class="font-semibold text-[11px] uppercase tracking-wider flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap"
                        :class="{ 'text-primary': sortColumn === col }">{{ col }}</span>
                  <span v-if="sortColumn === col" class="text-primary font-bold text-[10px] shrink-0">{{ sortDirection === 'asc' ? '▲' : '▼' }}</span>
                  <span v-else class="text-base-content/60 opacity-30 text-xs shrink-0 group-hover:text-primary group-hover:opacity-70 transition-all duration-150">⇅</span>
                </div>
                <span v-if="columnComments && columnComments[col]" class="text-[9px] text-base-content/60 opacity-50 max-w-[120px] overflow-hidden text-ellipsis whitespace-nowrap font-normal normal-case tracking-normal leading-tight" :title="columnComments[col]">{{ columnComments[col] }}</span>
              </div>
            </th>
            <th v-if="dirtyRows.size > 0 || newRowData" class="w-15 text-center bg-base-200 border-r border-b border-base-content/10 px-3.5 py-2 align-middle whitespace-nowrap font-semibold text-[11px] tracking-wider">操作</th>
          </tr>
        </thead>
        <tbody>
          <!-- Existing rows -->
          <tr v-for="(row, idx) in rows" :key="`row-${idx}-${getRowHash(row)}`"
              :class="dirtyRows.has(idx)
                ? 'bg-warning/5 border-l-[3px] border-l-warning hover:bg-warning/10'
                : 'even:bg-black/[0.015] hover:bg-primary/10'"
              @contextmenu.prevent="onRowContext($event, row, idx)">
            <td class="w-14 min-w-14 text-center bg-base-200 text-base-content/60 text-[11px] font-mono sticky left-0 z-30 border-r-2 border-b border-base-content/10 px-3.5 py-2 align-middle whitespace-nowrap">
              <span v-if="dirtyRows.has(idx)" class="text-warning font-bold text-sm" title="已修改">*</span>
              <span v-else>{{ (page - 1) * pageSize + idx + 1 }}</span>
            </td>
            <td v-for="col in columns" :key="col"
                class="px-3 py-1.5 border-r border-b border-base-content/10 text-left align-middle text-xs leading-5 whitespace-nowrap max-w-[400px] overflow-hidden text-ellipsis relative cursor-[cell] transition-colors duration-100 hover:bg-primary/10 hover:outline hover:outline-1 hover:outline-base-content/10 hover:-outline-offset-1"
                :class="{ 'font-semibold !text-primary !bg-primary/5': primaryKeyColumns.includes(col) }"
                @dblclick="startEdit(idx, col)"
                :title="String(formatValue(getDisplayValue(idx, col)) ?? 'NULL')">
              <!-- Editing cell -->
              <template v-if="editingCell.row === idx && editingCell.col === col">
                <input v-if="editingCell.isDatetime" v-model="editingValue"
                       type="datetime-local"
                       class="input input-sm w-full font-mono min-w-[180px]"
                       @blur="finishEdit" @keydown.enter="finishEdit"
                       @keydown.escape="cancelEdit" @keydown.tab="handleEditTab($event, col)" />
                <input v-else-if="!isComplexType(getDisplayValue(idx, col))" ref="editInput" v-model="editingValue"
                       class="input input-sm w-full"
                       @blur="finishEdit" @keydown.enter="finishEdit"
                       @keydown.escape="cancelEdit" @keydown.tab="handleEditTab($event, col)" />
                <textarea v-else ref="editInput" v-model="editingValue"
                          class="textarea textarea-sm w-full min-h-[80px] resize-y font-mono"
                          @blur="finishEdit" @keydown.ctrl.enter="finishEdit"
                          @keydown.escape="cancelEdit"></textarea>
              </template>
              <!-- Normal cell -->
              <template v-else>
                <span v-if="getDisplayValue(idx, col) === null || getDisplayValue(idx, col) === undefined" class="text-base-content/60 opacity-45 italic text-[11px] font-mono tracking-wider">NULL</span>
                <span v-else class="inline font-mono text-xs text-base-content">{{ formatValue(getDisplayValue(idx, col)) }}</span>
              </template>
            </td>
            <td v-if="dirtyRows.size > 0 || newRowData" class="w-15 text-center bg-base-200 border-r border-b border-base-content/10 px-3.5 py-2 align-middle whitespace-nowrap">
              <button class="btn btn-ghost btn-xs" @click.stop="confirmDeleteRow(row, idx)" title="删除行"><SvgIcon name="trash" size="14" /> </button>
            </td>
          </tr>

          <!-- New row -->
          <tr v-if="newRowData" class="bg-success/5 border-l-[3px] border-l-success hover:bg-success/10">
            <td class="w-14 min-w-14 text-center bg-base-200 text-base-content/60 text-[11px] font-mono sticky left-0 z-30 border-r-2 border-b border-base-content/10 px-3.5 py-2 align-middle whitespace-nowrap">
              <span class="text-success font-bold text-sm">+</span>
            </td>
            <td v-for="col in columns" :key="col"
                class="px-3 py-1.5 border-r border-b border-base-content/10 text-left align-middle text-xs leading-5 whitespace-nowrap max-w-[400px] overflow-hidden text-ellipsis relative cursor-[cell] transition-colors duration-100 hover:bg-primary/10 hover:outline hover:outline-1 hover:outline-base-content/10 hover:-outline-offset-1"
                @dblclick="startNewEdit(col)">
              <template v-if="editingCell.row === -1 && editingCell.col === col">
                <input v-if="!isComplexType(newRowData[col])" ref="editInput" v-model="editingValue"
                       class="input input-sm w-full"
                       @blur="finishNewEdit" @keydown.enter="finishNewEdit"
                       @keydown.escape="cancelNewEdit" @keydown.tab="handleEditTab($event, col)" />
                <textarea v-else ref="editInput" v-model="editingValue"
                          class="textarea textarea-sm w-full min-h-[80px] resize-y font-mono"
                          @blur="finishNewEdit" @keydown.ctrl.enter="finishNewEdit"
                          @keydown.escape="cancelNewEdit"></textarea>
              </template>
              <template v-else>
                <span v-if="newRowData[col] === null || newRowData[col] === undefined" class="text-base-content/60 opacity-45 italic text-[11px] font-mono tracking-wider">NULL</span>
                <span v-else class="inline font-mono text-xs text-base-content">{{ formatValue(newRowData[col]) }}</span>
              </template>
            </td>
            <td v-if="newRowData" class="w-15 text-center bg-base-200 border-r border-b border-base-content/10 px-3.5 py-2 align-middle whitespace-nowrap">
              <button class="btn btn-ghost btn-xs" @click.stop="saveNewRow" title="保存新行"><SvgIcon name="download" size="14" /> </button>
              <button class="btn btn-ghost btn-xs" @click.stop="cancelNewRow" title="取消新增">✖</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- JSON view -->
    <pre v-if="viewMode === 'json' && rows.length > 0" class="flex-1 p-4 m-0 overflow-auto font-mono text-xs leading-5 bg-base-200 text-base-content">{{ formatJson(rows) }}</pre>

    <!-- Pagination -->
    <div v-if="total > pageSize" class="flex items-center justify-center gap-3 px-3 py-2 border-t border-base-content/10 bg-base-100 shrink-0 min-h-[38px]">
      <button class="btn btn-ghost btn-sm" :disabled="page <= 1" @click="handlePrevPage">
        ‹ 上一页
      </button>
      <span class="text-xs text-base-content/60">第 {{ page }} / {{ totalPages }} 页</span>
      <button class="btn btn-ghost btn-sm" :disabled="page >= totalPages" @click="handleNextPage">
        下一页 ›
      </button>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="contextMenu.visible" class="fixed z-[9999] bg-base-100 border border-base-content/10 rounded-lg shadow-lg p-1 min-w-[180px]"
           :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }" @click.stop>
        <div v-for="(item, i) in contextMenu.items" :key="i"
             class="flex items-center gap-2 px-3 py-2 text-xs text-base-content rounded-md cursor-pointer transition-colors duration-100"
             :class="item.disabled
               ? '!opacity-40 !cursor-not-allowed'
               : 'hover:bg-base-200'"
             @click="item.disabled ? null : item.action()">
          <span v-html="item.icon"></span>
          <span>{{ item.label }}</span>
        </div>
      </div>
    </Teleport>
    <div v-if="contextMenu.visible" class="fixed inset-0 z-[9998]" @click="closeContextMenu"></div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, nextTick } from 'vue'
import { useToast } from '../../composables/useToast'
import FilterBar, { type FilterCondition } from './FilterBar.vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
      { icon: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 2l4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="M7 22l-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/></svg> ', label: '查看完整值', action: () => { toast.info(String(displayRow)); closeContextMenu() } },
      { icon: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> ', label: '复制行 JSON', action: () => { navigator.clipboard?.writeText(JSON.stringify(displayRow)); toast.info('已复制'); closeContextMenu() } },
      { icon: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg> ', label: '删除此行', action: () => { confirmDeleteRow(displayRow, idx); closeContextMenu() } },
    ]
  }
}

function onTableContext(event: MouseEvent) {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    items: [
      { icon: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg> ', label: '新增行', action: () => { addNewRow(); closeContextMenu() } },
      { icon: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg> ', label: '刷新数据', action: () => { emit('refresh'); closeContextMenu() } },
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
    // NOTE: Do NOT format numeric strings as timestamps — they are likely IDs or identifiers.
    // Real timestamp columns are typically numeric types (INT/BIGINT), not TEXT/VARCHAR.

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
