<template>
  <div class="h-full flex flex-col bg-base-200/30">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
      <div>
        <h2 class="text-sm font-semibold">Kanban</h2>
        <p class="text-xs text-base-content/50">Task board for hermes agents</p>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="btn btn-sm btn-ghost"
          @click="refreshTasks"
          :disabled="actionBusy !== null"
          title="Refresh"
        >
          <SvgIcon name="refresh" size="14" />
          <span class="ml-1 text-xs hidden sm:inline">Refresh</span>
        </button>
        <button
          class="btn btn-sm btn-ghost"
          @click="handleDispatch"
          :disabled="actionBusy !== null"
          title="Trigger dispatch pass"
        >
          <SvgIcon name="zap" size="14" />
          <span class="ml-1 text-xs hidden sm:inline">Dispatch</span>
        </button>
        <button class="btn btn-sm btn-primary" @click="showCreateTask = true">
          <SvgIcon name="plus" size="14" />
          <span class="ml-1 text-xs">New Task</span>
        </button>
      </div>
    </div>

    <!-- Error banner -->
    <div
      v-if="error"
      class="flex items-center justify-between px-4 py-2 bg-error/10 text-error text-xs border-b border-error/20 shrink-0"
    >
      <span class="truncate">{{ error }}</span>
      <button class="hover:opacity-70 shrink-0 ml-2" @click="error = ''">
        <SvgIcon name="close" size="12" />
      </button>
    </div>

    <!-- Board chips -->
    <div
      v-if="boards.length > 0"
      class="flex items-center gap-1.5 px-4 py-2 border-b border-base-content/10 overflow-x-auto shrink-0"
    >
      <button
        v-for="board in boards"
        :key="board.slug"
        class="kanban-board-chip"
        :class="{ 'kanban-board-chip-active': board.is_current }"
        :disabled="actionBusy !== null"
        :title="board.description || board.slug"
        @click="switchBoard(board.slug)"
      >
        <span v-if="board.is_current" class="kanban-board-dot" />
        <span class="text-xs font-medium">{{ board.name || board.slug }}</span>
        <span class="kanban-board-count">{{ boardTotal(board) }}</span>
      </button>
      <button
        class="kanban-board-chip kanban-board-chip-add"
        @click="showNewBoard = true"
        title="Create a new board"
      >
        <SvgIcon name="plus" size="12" />
        <span class="text-xs">New Board</span>
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <span class="loading loading-spinner loading-md" />
    </div>

    <!-- Columns (via KanbanColumn component) -->
    <div v-else class="flex-1 overflow-hidden">
      <div class="h-full flex gap-3 p-3 overflow-x-auto">
        <KanbanColumn
          v-for="col in columns"
          :key="col.key"
          :title="col.title"
          :status="col.key"
          :tasks="tasksByStatus[col.key] || []"
          :color="col.color"
          :busy-task-ids="busyTaskIds"
          :dragging-task-id="draggingTaskId"
          :drag-over-col="dragOverCol"
          :can-drop-here="canDropOnCol(col.key)"
          @task-click="showTaskDetail"
          @task-action="handleColumnAction"
          @drag-start="onDragStart"
          @drag-end="onDragEnd"
          @drag-over="onDragOver"
          @drag-leave="onDragLeave"
          @drop="onDrop"
        />
      </div>
    </div>

    <!-- Task detail drawer -->
    <TaskDetailDrawer
      v-if="selectedTask"
      :task="selectedTask"
      :assignees="assignees"
      @close="selectedTask = null"
      @refresh="refreshTasks"
      @action="handleTaskAction"
    />

    <!-- Create task modal -->
    <CreateTaskModal
      v-if="showCreateTask"
      :assignees="assignees"
      @close="showCreateTask = false"
      @create="createTask"
    />

    <!-- New board modal -->
    <div v-if="showNewBoard" class="fixed inset-0 bg-base-content/20 z-50 flex items-center justify-center" @click.self="showNewBoard = false">
      <div class="bg-base-100 rounded-lg shadow-xl w-80 max-h-[90vh] flex flex-col">
        <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
          <span class="text-sm font-medium">New Board</span>
          <button class="btn btn-sm btn-ghost btn-circle" @click="showNewBoard = false">
            <SvgIcon name="close" size="14" />
          </button>
        </div>
        <div class="p-4 space-y-3">
          <div>
            <label class="text-xs text-base-content/50 block mb-1">Slug *</label>
            <input
              v-model="newBoardSlug"
              type="text"
              class="input input-sm input-bordered w-full"
              placeholder="board-slug"
              autofocus
              @keydown.enter="handleCreateBoard"
            />
          </div>
          <div>
            <label class="text-xs text-base-content/50 block mb-1">Display Name</label>
            <input
              v-model="newBoardName"
              type="text"
              class="input input-sm input-bordered w-full"
              placeholder="My Board"
              @keydown.enter="handleCreateBoard"
            />
          </div>
        </div>
        <div class="px-4 py-3 border-t border-base-content/10 flex items-center justify-end gap-2">
          <button class="btn btn-sm btn-ghost" @click="showNewBoard = false">Cancel</button>
          <button
            class="btn btn-sm btn-primary"
            :disabled="!newBoardSlug.trim() || actionBusy === 'board-create'"
            @click="handleCreateBoard"
          >
            {{ actionBusy === 'board-create' ? 'Creating...' : 'Create' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'KanbanBoard' })
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import KanbanColumn from './KanbanColumn.vue'
import TaskDetailDrawer from './TaskDetailDrawer.vue'
import CreateTaskModal from './CreateTaskModal.vue'

// ── Types ────────────────────────────────────────────────
interface KanbanBoard {
  slug: string
  name: string
  description?: string
  icon?: string
  color?: string
  archived: boolean
  is_current: boolean
  db_path?: string
  counts: Record<string, number>
}

interface KanbanTask {
  id: string
  title: string
  body?: string
  status: string
  assignee?: string
  priority?: number
  skills?: string[]
  created_by?: string
  created_at?: number
  started_at?: number
  completed_at?: number
  tenant?: string
  workspace_kind?: string
  workspace_path?: string
  result?: string
  session_id?: string
}

interface KanbanTaskDetail {
  task: KanbanTask
  latest_summary?: string
  parents: string[]
  children: string[]
  comments: Array<{ id: number; author: string; body: string; created_at: number }>
  events: Array<{ kind: string; payload: Record<string, unknown>; created_at: number; run_id?: number }>
  runs: Array<{
    id: number; profile: string; step_key?: string; status?: string
    outcome?: string; summary?: string; error?: string
    started_at: number; ended_at?: number
  }>
}

// ── Column definitions ───────────────────────────────────
const columns = [
  { key: 'triage', title: 'Triage', color: 'secondary' },
  { key: 'todo', title: 'Todo', color: 'warning' },
  { key: 'ready', title: 'Ready', color: 'info' },
  { key: 'running', title: 'Running', color: 'primary' },
  { key: 'blocked', title: 'Blocked', color: 'error' },
  { key: 'done', title: 'Done', color: 'success' },
] as const

// ── State ────────────────────────────────────────────────
const boards = ref<KanbanBoard[]>([])
const tasks = ref<KanbanTask[]>([])
const loading = ref(true)
const error = ref('')
const actionBusy = ref<string | null>(null)
const showCreateTask = ref(false)
const showNewBoard = ref(false)
const selectedTask = ref<KanbanTaskDetail | null>(null)
const assignees = ref<Array<{ name: string; on_disk: boolean; counts: Record<string, number> }>>([])

const newBoardSlug = ref('')
const newBoardName = ref('')

// Drag & drop
const draggingTaskId = ref<string | null>(null)
const dragOverCol = ref<string | null>(null)

let pollTimer: ReturnType<typeof setInterval> | null = null

// ── Computed ─────────────────────────────────────────────
const currentBoard = computed(() => boards.value.find(b => b.is_current) ?? null)

const busyTaskIds = computed(() => {
  if (!actionBusy.value) {return []}
  if (actionBusy.value === 'board-switch' || actionBusy.value === 'dispatch' || actionBusy.value === 'board-create') {return []}
  return [actionBusy.value]
})

const tasksByStatus = computed(() => {
  const grouped: Record<string, KanbanTask[]> = {}
  for (const col of columns) {grouped[col.key] = []}
  for (const task of tasks.value) {
    const key = columns.some(c => c.key === task.status) ? task.status : 'todo'
    ;(grouped[key] = grouped[key] || []).push(task)
  }
  for (const k of Object.keys(grouped)) {
    grouped[k].sort((a, b) => {
      if ((b.priority || 0) !== (a.priority || 0)) {return (b.priority || 0) - (a.priority || 0)}
      return (a.created_at || 0) - (b.created_at || 0)
    })
  }
  return grouped
})

// ── Helpers ──────────────────────────────────────────────
function boardTotal(b: KanbanBoard): number {
  if (b.counts && typeof b.counts === 'object') {
    return Object.values(b.counts).reduce((sum: number, v: number) => sum + v, 0)
  }
  return 0
}

function isValidDragTransition(from: string, to: string): boolean {
  if (from === to) {return false}
  if (to === 'done') {return true}
  if (to === 'blocked' && (from === 'todo' || from === 'ready' || from === 'running')) {return true}
  if (to === 'ready' && from === 'blocked') {return true}
  return false
}

function canDropOnCol(colKey: string): boolean {
  if (!draggingTaskId.value) {return false}
  const task = tasks.value.find(t => t.id === draggingTaskId.value)
  if (!task) {return false}
  return isValidDragTransition(task.status, colKey)
}

// ── Data loading ─────────────────────────────────────────
async function loadBoards() {
  try { boards.value = await invoke<KanbanBoard[]>('kanban_list_boards') } catch (e) { console.error(e) }
}

async function loadTasks() {
  try { tasks.value = await invoke<KanbanTask[]>('kanban_list_tasks', { board: null, status: null, assignee: null }) } catch (e) { console.error(e) }
}

async function loadAssignees() {
  try { assignees.value = await invoke('kanban_list_assignees') } catch (e) { console.error(e) }
}

async function refreshTasks() {
  error.value = ''
  await Promise.all([loadBoards(), loadTasks()])
}

// ── Board operations ─────────────────────────────────────
async function switchBoard(slug: string) {
  if (currentBoard.value?.slug === slug) {return}
  actionBusy.value = 'board-switch'
  try {
    await invoke('kanban_switch_board', { slug })
    await loadBoards()
    await loadTasks()
    selectedTask.value = null
  } catch (e) {
    error.value = `Failed to switch board: ${e}`
  } finally {
    actionBusy.value = null
  }
}

async function handleCreateBoard() {
  if (!newBoardSlug.value.trim()) {return}
  actionBusy.value = 'board-create'
  try {
    await invoke('kanban_create_board', {
      slug: newBoardSlug.value.trim(),
      name: newBoardName.value.trim() || undefined,
      description: null, icon: null, color: null,
    })
    showNewBoard.value = false
    newBoardSlug.value = ''
    newBoardName.value = ''
    await loadBoards()
    await loadTasks()
  } catch (e) {
    error.value = `Failed to create board: ${e}`
  } finally {
    actionBusy.value = null
  }
}

async function handleDispatch() {
  actionBusy.value = 'dispatch'
  try {
    await invoke('kanban_dispatch', { dryRun: false, maxSpawns: null })
    await refreshTasks()
  } catch (e) {
    error.value = `Dispatch failed: ${e}`
  } finally {
    actionBusy.value = null
  }
}

// ── Task operations ──────────────────────────────────────
async function showTaskDetail(task: KanbanTask) {
  try {
    selectedTask.value = await invoke<KanbanTaskDetail>('kanban_show_task', { taskId: task.id })
  } catch (e) { console.error(e) }
}

// Handles actions from both columns and detail drawer
async function handleTaskAction(action: string, taskId: string, ...args: unknown[]) {
  try {
    switch (action) {
      case 'assign':
        await invoke('kanban_assign_task', { taskId, assignee: args[0] }); break
      case 'reclaim':
        await invoke('kanban_reclaim_task', { taskId }); break
      case 'complete':
        await invoke('kanban_complete_task', { taskId, summary: args[0] || null }); break
      case 'block': {
        const reason = window.prompt('Block reason:', String(args[0] || 'Needs attention'))
        if (reason === null) {return}
        await invoke('kanban_block_task', { taskId, reason }); break
      }
      case 'unblock':
        await invoke('kanban_unblock_task', { taskId }); break
      case 'archive':
        await invoke('kanban_archive_task', { taskId }); break
      case 'comment':
        await invoke('kanban_add_comment', { taskId, body: args[0] }); break
    }
    await loadTasks()
    if (selectedTask.value?.task.id === taskId) {
      selectedTask.value = await invoke<KanbanTaskDetail>('kanban_show_task', { taskId })
    }
  } catch (e) { console.error(e) }
}

// Column action handler - wraps handleTaskAction with move logic
async function handleColumnAction(action: string, taskId: string, ...args: unknown[]) {
  const task = tasks.value.find(t => t.id === taskId)
  if (!task) {return}

  if (action === 'complete') {
    if (!window.confirm(`Mark "${task.title}" as done?`)) {return}
  } else if (action === 'archive') {
    if (!window.confirm(`Archive "${task.title}"?`)) {return}
  }
  await handleTaskAction(action, taskId, ...args)
}

// ── Create task ──────────────────────────────────────────
async function createTask(data: {
  title: string; body?: string; assignee?: string; parents?: string[]; priority?: number
}) {
  try {
    await invoke('kanban_create_task', {
      title: data.title,
      body: data.body || null,
      assignee: data.assignee || null,
      parents: data.parents || null,
      priority: data.priority ?? null,
      board: null,
    })
    showCreateTask.value = false
    await loadTasks()
  } catch (e) {
    error.value = `Failed to create task: ${e}`
  }
}

// ── Drag & drop ──────────────────────────────────────────
function onDragStart(taskId: string) {
  draggingTaskId.value = taskId
}

function onDragEnd(_taskId: string) {
  draggingTaskId.value = null
  dragOverCol.value = null
}

function onDragOver(colKey: string) {
  dragOverCol.value = colKey
}

function onDragLeave(colKey: string) {
  if (dragOverCol.value === colKey) {dragOverCol.value = null}
}

function onDrop(colKey: string) {
  dragOverCol.value = null
  if (!draggingTaskId.value) {return}
  const task = tasks.value.find(t => t.id === draggingTaskId.value)
  if (!task) {return}
  if (!isValidDragTransition(task.status, colKey)) {return}

  handleTaskAction(
    colKey === 'done' ? 'complete' : colKey === 'blocked' ? 'block' : 'unblock',
    task.id,
    colKey === 'blocked' ? 'Blocked' : null,
  )
}

// ── Lifecycle ────────────────────────────────────────────
onMounted(async () => {
  loading.value = true
  await Promise.all([loadBoards(), loadTasks(), loadAssignees()]).catch(e => {
    error.value = `Failed to load: ${e}`
  })
  loading.value = false
  pollTimer = setInterval(() => { loadTasks().catch(() => {}) }, 6000)
})

onUnmounted(() => {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
})
</script>

<style scoped>
/* ── Board chips ───────────────────────────────────────── */
.kanban-board-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem 0.625rem;
  border-radius: 9999px;
  border: 1px solid var(--fallback-bc, oklch(0.4 0 0 / 0.12));
  background: var(--fallback-b2, oklch(0.98 0 0));
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
  font-size: 0.75rem;
  color: var(--fallback-bc, oklch(0.4 0 0 / 0.7));
}
.kanban-board-chip:hover {
  background: var(--fallback-b3, oklch(0.94 0 0));
  border-color: var(--fallback-p, oklch(0.55 0.2 250 / 0.3));
}
.kanban-board-chip:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.kanban-board-chip-active {
  border-color: var(--fallback-p, oklch(0.55 0.2 250 / 0.5));
  background: var(--fallback-p, oklch(0.55 0.2 250 / 0.08));
  color: var(--fallback-p, oklch(0.55 0.2 250));
  font-weight: 600;
}
.kanban-board-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--fallback-su, oklch(0.65 0.2 140));
  flex-shrink: 0;
}
.kanban-board-count {
  font-size: 0.65rem;
  padding: 0 0.3rem;
  border-radius: 9999px;
  background: var(--fallback-bc, oklch(0.4 0 0 / 0.08));
  color: var(--fallback-bc, oklch(0.4 0 0 / 0.5));
  margin-left: 0.125rem;
}
.kanban-board-chip-add {
  border-style: dashed;
  opacity: 0.6;
}
.kanban-board-chip-add:hover {
  opacity: 1;
}
</style>
