<template>
  <div
    class="flex-shrink-0 w-72 flex flex-col kanban-column"
    :class="[
      columnBgClass,
      isDragOver && canDropHere ? 'kanban-column-drop' : '',
    ]"
    @dragover.prevent="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <!-- Column header -->
    <div
      class="flex items-center gap-2 px-3 py-2.5 rounded-t-lg border-b"
      :class="colHeaderBorderClass"
    >
      <span class="w-2 h-2 rounded-full shrink-0" :class="colorDotClass" />
      <span class="text-xs font-semibold uppercase tracking-wide text-base-content/70">{{ title }}</span>
      <span class="ml-auto text-xs px-1.5 py-0.5 rounded-full bg-base-content/10 font-mono">
        {{ tasks.length }}
      </span>
    </div>

    <!-- Column body -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1.5 min-h-[100px]">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="kanban-card group"
        :class="{
          'kanban-card-dragging': isDragging && draggingTaskId === task.id,
          'opacity-50 pointer-events-none': busyTaskIds?.includes(task.id),
        }"
        :draggable="!busyTaskIds?.includes(task.id)"
        @dragstart="onCardDragStart($event, task)"
        @dragend="$emit('drag-end', task.id)"
        @click="$emit('task-click', task)"
      >
        <!-- Title -->
        <div class="kanban-card-title">{{ task.title }}</div>

        <!-- Pills row -->
        <div class="kanban-card-meta">
          <span class="kanban-pill kanban-pill-status">
            {{ colStatusLabel }}
          </span>
          <span v-if="taskPriorityLabel(task.priority)" class="kanban-pill kanban-pill-prio">
            {{ taskPriorityLabel(task.priority) }}
          </span>
          <span v-if="task.assignee" class="kanban-pill" title="Assignee">
            @{{ task.assignee }}
          </span>
          <span v-if="taskAgeLabel(task.created_at)" class="kanban-pill kanban-pill-age" title="Age">
            {{ taskAgeLabel(task.created_at) }}
          </span>
          <span v-if="task.status === 'running' && task.started_at && taskStartedAgo(task.started_at)" class="kanban-pill" title="Running for">
            {{ taskStartedAgo(task.started_at) }}
          </span>
        </div>

        <!-- Hover-reveal actions -->
        <div class="kanban-card-actions">
          <!-- Ready: complete -->
          <button
            v-if="task.status === 'ready'"
            class="kanban-card-action text-success"
            title="Mark done"
            @click.stop="$emit('task-action', 'complete', task.id)"
          >
            <SvgIcon name="check" size="14" />
          </button>
          <!-- Running: reclaim -->
          <button
            v-if="task.status === 'running'"
            class="kanban-card-action text-warning"
            title="Reclaim"
            @click.stop="$emit('task-action', 'reclaim', task.id)"
          >
            <SvgIcon name="undo" size="14" />
          </button>
          <!-- Blocked: unblock -->
          <button
            v-if="task.status === 'blocked'"
            class="kanban-card-action text-success"
            title="Unblock"
            @click.stop="$emit('task-action', 'unblock', task.id)"
          >
            <SvgIcon name="undo" size="14" />
          </button>
          <!-- Todo/Ready: block -->
          <button
            v-if="task.status === 'todo' || task.status === 'ready'"
            class="kanban-card-action text-error"
            title="Block"
            @click.stop="$emit('task-action', 'block', task.id, 'Needs attention')"
          >
            <SvgIcon name="ban" size="14" />
          </button>
          <!-- Running: block -->
          <button
            v-if="task.status === 'running'"
            class="kanban-card-action text-error"
            title="Block"
            @click.stop="$emit('task-action', 'block', task.id, 'Needs attention')"
          >
            <SvgIcon name="ban" size="14" />
          </button>
          <!-- Archive (always visible) -->
          <button
            class="kanban-card-action kanban-card-action-danger text-base-content/40 hover:text-error"
            title="Archive"
            @click.stop="$emit('task-action', 'archive', task.id)"
          >
            <SvgIcon name="trash" size="12" />
          </button>
        </div>
      </div>

      <!-- Empty state -->
      <div
        v-if="tasks.length === 0"
        class="flex items-center justify-center py-8 text-xs text-base-content/30 italic"
      >
        &mdash;
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

interface KanbanTask {
  id: string
  title: string
  status: string
  assignee?: string
  priority?: number
  skills?: string[]
  created_by?: string
  created_at?: number
  started_at?: number
  completed_at?: number
  tenant?: string
}

const props = defineProps<{
  title: string
  status: string
  tasks: KanbanTask[]
  color: string
  busyTaskIds?: string[]
  draggingTaskId?: string | null
  dragOverCol?: string | null
  canDropHere?: boolean
}>()

const emit = defineEmits<{
  (e: 'task-click', task: KanbanTask): void
  (e: 'task-action', action: string, taskId: string, ...args: unknown[]): void
  (e: 'drag-start', taskId: string): void
  (e: 'drag-end', taskId: string): void
  (e: 'drag-over', colKey: string): void
  (e: 'drag-leave', colKey: string): void
  (e: 'drop', colKey: string): void
}>()

const isDragging = computed(() => props.draggingTaskId !== null && props.draggingTaskId !== undefined)
const isDragOver = computed(() => props.dragOverCol === props.status)

// ── Style helpers ────────────────────────────────────────
const columnBgClass = computed(() => {
  const map: Record<string, string> = {
    secondary: 'bg-secondary/5', warning: 'bg-warning/5',
    info: 'bg-info/5', primary: 'bg-primary/5',
    error: 'bg-error/5', success: 'bg-success/5',
  }
  return map[props.color] || 'bg-base-200/30'
})

const colHeaderBorderClass = computed(() => {
  const map: Record<string, string> = {
    secondary: 'border-secondary/20', warning: 'border-warning/20',
    info: 'border-info/20', primary: 'border-primary/20',
    error: 'border-error/20', success: 'border-success/20',
  }
  return map[props.color] || 'border-base-content/10'
})

const colorDotClass = computed(() => {
  const map: Record<string, string> = {
    secondary: 'bg-secondary', warning: 'bg-warning',
    info: 'bg-info', primary: 'bg-primary',
    error: 'bg-error', success: 'bg-success',
  }
  return map[props.color] || 'bg-base-content/40'
})

// ── Per-task helpers (called from template) ──────────────
const colStatusLabel = computed(() => {
  const map: Record<string, string> = {
    triage: 'triage', todo: 'todo', ready: 'ready',
    running: 'running', blocked: 'blocked', done: 'done',
  }
  return map[props.status] || props.status
})

function taskPriorityLabel(p: number | undefined): string {
  if (!p) {return ''}
  if (p >= 10) {return 'P0'}
  if (p >= 5) {return 'P1'}
  if (p > 0) {return 'P2'}
  return ''
}

function taskAgeLabel(createdAt: number | undefined): string {
  if (!createdAt) {return ''}
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - createdAt))
  if (seconds < 60) {return `${seconds}s`}
  if (seconds < 3600) {return `${Math.floor(seconds / 60)}m`}
  if (seconds < 86400) {return `${Math.floor(seconds / 3600)}h`}
  return `${Math.floor(seconds / 86400)}d`
}

function taskStartedAgo(startedAt: number | undefined): string {
  if (!startedAt) {return ''}
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - startedAt))
  if (seconds < 60) {return `${seconds}s`}
  if (seconds < 3600) {return `${Math.floor(seconds / 60)}m`}
  if (seconds < 86400) {return `${Math.floor(seconds / 3600)}h`}
  return `${Math.floor(seconds / 86400)}d`
}

// ── Drag & drop handlers ─────────────────────────────────
function onCardDragStart(e: DragEvent, task: KanbanTask) {
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', task.id)
  }
  emit('drag-start', task.id)
}

function onDragOver(e: DragEvent) {
  if (!isDragging.value) {return}
  if (!props.canDropHere) {return}
  if (e.dataTransfer) {e.dataTransfer.dropEffect = 'move'}
  emit('drag-over', props.status)
}

function onDragLeave(e: DragEvent) {
  const target = e.currentTarget as HTMLElement | null
  if (target && e.relatedTarget instanceof Node && target.contains(e.relatedTarget)) {return}
  emit('drag-leave', props.status)
}

function onDrop(e: DragEvent) {
  e.preventDefault()
  emit('drop', props.status)
}
</script>

<style scoped>
/* ── Column ─────────────────────────────────────────────── */
.kanban-column {
  transition: box-shadow 0.15s ease;
}
.kanban-column-drop {
  box-shadow: 0 0 0 2px var(--fallback-p, oklch(0.55 0.2 250 / 0.4));
}

/* ── Cards ──────────────────────────────────────────────── */
.kanban-card {
  background: var(--fallback-b1, oklch(1 0 0));
  border: 1px solid oklch(0.4 0 0 / 0.06);
  border-radius: 0.5rem;
  padding: 0.625rem 0.75rem;
  cursor: pointer;
  transition: all 0.15s ease;
  position: relative;
  user-select: none;
}
.kanban-card:hover {
  border-color: var(--fallback-p, oklch(0.55 0.2 250 / 0.25));
  box-shadow: 0 1px 6px oklch(0 0 0 / 0.08);
}
.kanban-card-dragging {
  opacity: 0.4;
  transform: rotate(2deg);
}
.kanban-card-title {
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.3;
  margin-bottom: 0.375rem;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* ── Pills ──────────────────────────────────────────────── */
.kanban-card-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.25rem;
}
.kanban-pill {
  display: inline-flex;
  align-items: center;
  padding: 0.0625rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 0.625rem;
  font-weight: 500;
  line-height: 1.4;
  white-space: nowrap;
  background: oklch(0.4 0 0 / 0.06);
  color: oklch(0.4 0 0 / 0.6);
}
.kanban-pill-status {
  background: var(--fallback-p, oklch(0.55 0.2 250 / 0.1));
  color: var(--fallback-p, oklch(0.55 0.2 250));
}
.kanban-pill-prio {
  background: var(--fallback-wa, oklch(0.75 0.15 80 / 0.12));
  color: var(--fallback-wa, oklch(0.65 0.15 80));
  font-weight: 700;
}
.kanban-pill-age {
  color: oklch(0.4 0 0 / 0.4);
  margin-left: auto;
}

/* ── Hover-reveal actions ───────────────────────────────── */
.kanban-card-actions {
  display: flex;
  align-items: center;
  gap: 0.125rem;
  margin-top: 0.375rem;
  opacity: 0;
  transition: opacity 0.12s ease;
}
.kanban-card:hover .kanban-card-actions {
  opacity: 1;
}
.kanban-card-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 0.25rem;
  border: none;
  background: transparent;
  cursor: pointer;
  transition: all 0.12s ease;
  padding: 0;
}
.kanban-card-action:hover {
  background: oklch(0.4 0 0 / 0.08);
}
.kanban-card-action-danger:hover {
  background: var(--fallback-er, oklch(0.6 0.2 20 / 0.1));
}
</style>
