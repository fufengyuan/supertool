<template>
  <div
    class="fixed inset-y-0 right-0 w-96 bg-base-100 border-l border-base-content/10 shadow-xl z-50 flex flex-col animate-slide-in"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
      <div class="flex items-center gap-2 min-w-0">
        <span :class="statusColorClass" class="w-2.5 h-2.5 rounded-full shrink-0"></span>
        <span class="text-sm font-medium truncate">{{ taskInfo.title }}</span>
      </div>
      <button class="btn btn-sm btn-ghost btn-circle shrink-0" @click="$emit('close')">
        <SvgIcon name="close" size="14" />
      </button>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      <!-- Meta pills -->
      <div class="flex flex-wrap items-center gap-1.5">
        <span class="kanban-pill kanban-pill-status">
          {{ formatStatus(taskInfo.status) }}
        </span>
        <span v-if="taskInfo.assignee" class="kanban-pill">
          @{{ taskInfo.assignee }}
        </span>
        <span v-if="taskInfo.tenant" class="kanban-pill">
          {{ taskInfo.tenant }}
        </span>
        <span class="kanban-pill kanban-pill-id" :title="taskInfo.id">
          {{ taskInfo.id.slice(0, 8) }}
        </span>
      </div>

      <!-- Schedule meta -->
      <div v-if="taskInfo.created_at || taskInfo.started_at || taskInfo.completed_at" class="flex flex-wrap gap-4 text-xs text-base-content/50">
        <span v-if="taskInfo.created_at">
          Created {{ ageLabel(taskInfo.created_at) }} ago
        </span>
        <span v-if="taskInfo.started_at && taskInfo.status === 'running'" class="text-info">
          Running {{ startedAgo(taskInfo.started_at) }}
        </span>
        <span v-if="taskInfo.completed_at" class="text-success">
          Completed {{ ageLabel(taskInfo.completed_at) }} ago
        </span>
      </div>

      <!-- Assign selector -->
      <div>
        <label class="text-xs text-base-content/50 block mb-1">Assign to</label>
        <select
          class="select select-sm select-bordered w-full"
          :value="taskInfo.assignee || ''"
          @change="handleAssign"
        >
          <option value="">Unassigned</option>
          <option v-for="a in assignees" :key="a.name" :value="a.name">
            {{ a.name }}
          </option>
        </select>
      </div>

      <!-- Priority -->
      <div v-if="taskInfo.priority !== undefined">
        <label class="text-xs text-base-content/50 block mb-1">Priority</label>
        <span class="text-sm">{{ priorityLabel(taskInfo.priority) || 'Normal' }}</span>
      </div>

      <!-- Body -->
      <div v-if="taskInfo.body">
        <label class="text-xs text-base-content/50 block mb-1">Description</label>
        <pre class="text-sm bg-base-200/50 rounded p-2 whitespace-pre-wrap break-words text-base-content/80">{{ taskInfo.body }}</pre>
      </div>

      <!-- Latest summary -->
      <div v-if="task.latest_summary">
        <label class="text-xs text-base-content/50 block mb-1">Latest Summary</label>
        <pre class="text-sm bg-base-200/50 rounded p-2 whitespace-pre-wrap break-words text-base-content/80">{{ task.latest_summary }}</pre>
      </div>

      <!-- Result -->
      <div v-if="taskInfo.result">
        <label class="text-xs text-base-content/50 block mb-1">Result</label>
        <pre class="text-sm bg-base-200/50 rounded p-2 whitespace-pre-wrap break-words text-base-content/80">{{ taskInfo.result }}</pre>
      </div>

      <!-- Dependencies -->
      <div v-if="parents.length > 0 || children.length > 0">
        <label class="text-xs text-base-content/50 block mb-1">Dependencies</label>
        <div v-if="parents.length > 0" class="mb-1.5">
          <span class="text-xs text-base-content/40">Parents: </span>
          <span v-for="p in parents" :key="p" class="badge badge-sm badge-ghost mr-1 font-mono text-xs">{{ p.slice(0, 8) }}</span>
        </div>
        <div v-if="children.length > 0">
          <span class="text-xs text-base-content/40">Children: </span>
          <span v-for="c in children" :key="c" class="badge badge-sm badge-ghost mr-1 font-mono text-xs">{{ c.slice(0, 8) }}</span>
        </div>
      </div>

      <!-- Runs -->
      <div v-if="runs.length > 0">
        <label class="text-xs text-base-content/50 block mb-2">Runs ({{ runs.length }})</label>
        <div class="space-y-2">
          <div v-for="run in runs" :key="run.id" class="bg-base-200/50 rounded p-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5">
                <span class="text-xs font-medium">{{ run.profile }}</span>
                <span
                  v-if="run.outcome"
                  class="text-xs px-1 rounded"
                  :class="outcomeClass(run.outcome)"
                >{{ run.outcome }}</span>
              </div>
              <span class="text-xs text-base-content/40">{{ formatTime(run.started_at) }}</span>
            </div>
            <div v-if="run.error" class="text-xs text-error mt-1 truncate">{{ run.error.slice(0, 60) }}</div>
            <div v-if="run.summary" class="text-xs text-base-content/60 mt-1 truncate">{{ run.summary.slice(0, 80) }}</div>
          </div>
        </div>

        <!-- Load log -->
        <button
          class="btn btn-sm btn-ghost mt-2 w-full text-xs"
          @click="loadTaskLog"
          :disabled="loadingLog"
        >
          <SvgIcon v-if="loadingLog" name="refresh" size="12" class="animate-spin" />
          <SvgIcon v-else name="terminal" size="12" />
          <span class="ml-1">{{ loadingLog ? 'Loading...' : 'View Execution Log' }}</span>
        </button>

        <!-- Log output -->
        <div v-if="taskLog" class="mt-3">
          <label class="text-xs text-base-content/50 block mb-1">Execution Log</label>
          <div class="bg-base-300 rounded-lg p-3 text-xs font-mono whitespace-pre-wrap overflow-x-auto max-h-80 overflow-y-auto leading-relaxed">
            <LogContent :content="taskLog" />
          </div>
        </div>
      </div>

      <!-- Events (last 12, newest first) -->
      <div v-if="events.length > 0">
        <label class="text-xs text-base-content/50 block mb-2">Events ({{ events.length }})</label>
        <div class="space-y-1">
          <div
            v-for="ev in events.slice(-12).reverse()"
            :key="(ev as any).id || ev.created_at"
            class="flex items-center gap-2 text-xs"
          >
            <span class="kanban-pill">{{ ev.kind }}</span>
            <span class="text-base-content/40">{{ ageLabel(ev.created_at) }} ago</span>
          </div>
        </div>
      </div>

      <!-- Comments -->
      <div>
        <label class="text-xs text-base-content/50 block mb-2">
          Comments ({{ task.comments?.length || 0 }})
        </label>
        <div class="space-y-2">
          <div v-for="c in (task.comments || [])" :key="c.id" class="bg-base-200/50 rounded p-2">
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium">{{ c.author }}</span>
              <span class="text-xs text-base-content/40">{{ formatTime(c.created_at) }}</span>
            </div>
            <div class="text-sm mt-1 whitespace-pre-wrap break-words">{{ c.body }}</div>
          </div>
        </div>

        <!-- Add comment -->
        <div class="mt-3">
          <textarea
            v-model="newComment"
            class="textarea textarea-sm textarea-bordered w-full text-xs"
            placeholder="Add comment..."
            rows="2"
            @keydown.enter.meta="handleAddComment"
          ></textarea>
          <button
            class="btn btn-sm btn-ghost mt-1 text-xs"
            :disabled="!newComment.trim()"
            @click="handleAddComment"
          >
            <SvgIcon name="send" size="12" />
            <span class="ml-1">Send</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Footer actions -->
    <div class="px-4 py-3 border-t border-base-content/10 flex items-center gap-2 flex-wrap">
      <button
        v-if="taskInfo.status === 'ready' || taskInfo.status === 'running'"
        class="btn btn-sm btn-success flex-1 min-w-0"
        @click="$emit('action', 'complete', taskInfo.id)"
      >
        Complete
      </button>
      <button
        v-if="taskInfo.status === 'running'"
        class="btn btn-sm btn-error flex-1 min-w-0"
        @click="$emit('action', 'reclaim', taskInfo.id)"
      >
        Reclaim
      </button>
      <button
        v-if="taskInfo.status === 'todo' || taskInfo.status === 'ready' || taskInfo.status === 'running'"
        class="btn btn-sm btn-warning flex-1 min-w-0"
        @click="$emit('action', 'block', taskInfo.id, 'Needs attention')"
      >
        Block
      </button>
      <button
        v-if="taskInfo.status === 'blocked'"
        class="btn btn-sm btn-success flex-1 min-w-0"
        @click="$emit('action', 'unblock', taskInfo.id)"
      >
        Unblock
      </button>
      <button
        v-if="taskInfo.status === 'done'"
        class="btn btn-sm btn-ghost flex-1 min-w-0"
        @click="$emit('action', 'archive', taskInfo.id)"
      >
        Archive
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import LogContent from './LogContent.vue'

// ── Types ────────────────────────────────────────────────
interface KanbanTaskDetail {
  task: {
    id: string
    title: string
    status: string
    assignee?: string
    priority?: number
    body?: string
    tenant?: string
    skills?: string[]
    created_by?: string
    created_at?: number
    started_at?: number
    completed_at?: number
    result?: string
  }
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

// ── Props & emits ────────────────────────────────────────
const props = defineProps<{
  task: KanbanTaskDetail
  assignees: Array<{ name: string; on_disk: boolean; counts: Record<string, number> }>
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'refresh'): void
  (e: 'action', action: string, taskId: string, ...args: unknown[]): void
}>()

// ── Computed shortcuts ──────────────────────────────────
const taskInfo = computed(() => props.task.task)
const parents = computed(() => props.task.parents)
const children = computed(() => props.task.children)
const runs = computed(() => props.task.runs)
const events = computed(() => props.task.events)

// ── Local state ──────────────────────────────────────────
const newComment = ref('')
const taskLog = ref('')
const loadingLog = ref(false)

// ── Helpers ──────────────────────────────────────────────
function ageLabel(createdAt: number | undefined): string {
  if (!createdAt) {return ''}
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - createdAt))
  if (seconds < 60) {return `${seconds}s`}
  if (seconds < 3600) {return `${Math.floor(seconds / 60)}m`}
  if (seconds < 86400) {return `${Math.floor(seconds / 3600)}h`}
  return `${Math.floor(seconds / 86400)}d`
}

function startedAgo(startedAt: number | undefined): string {
  if (!startedAt) {return ''}
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - startedAt))
  if (seconds < 60) {return `${seconds}s`}
  if (seconds < 3600) {return `${Math.floor(seconds / 60)}m`}
  if (seconds < 86400) {return `${Math.floor(seconds / 3600)}h`}
  return `${Math.floor(seconds / 86400)}d`
}

function priorityLabel(p: number | undefined): string {
  if (!p) {return ''}
  if (p >= 10) {return 'P0 (Urgent)'}
  if (p >= 5) {return 'P1 (High)'}
  if (p > 0) {return 'P2 (Low)'}
  return 'Normal'
}

function formatStatus(status: string): string {
  const map: Record<string, string> = {
    triage: 'Triage', todo: 'Todo', ready: 'Ready',
    running: 'Running', blocked: 'Blocked', done: 'Done',
    in_progress: 'Running', scheduled: 'Scheduled',
    archived: 'Archived',
  }
  return map[status] || status
}

function formatTime(time: number): string {
  return new Date(time * 1000).toLocaleString('zh-CN', {
    month: 'numeric', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}

function outcomeClass(outcome: string): string {
  switch (outcome) {
    case 'completed': return 'text-success bg-success/10'
    case 'crashed':
    case 'timed_out': return 'text-error bg-error/10'
    case 'blocked': return 'text-warning bg-warning/10'
    default: return 'text-base-content/60'
  }
}

const statusColorClass = computed(() => {
  switch (taskInfo.value.status) {
    case 'triage': return 'bg-secondary'
    case 'todo': return 'bg-warning'
    case 'ready': return 'bg-info'
    case 'running':
    case 'in_progress': return 'bg-primary animate-pulse'
    case 'blocked': return 'bg-error'
    case 'done': return 'bg-success'
    default: return 'bg-base-content/40'
  }
})

// ── Actions ──────────────────────────────────────────────
function handleAssign(e: Event) {
  const target = e.target as HTMLSelectElement
  const assignee = target.value
  if (assignee !== (taskInfo.value.assignee || '')) {
    emit('action', 'assign', taskInfo.value.id, assignee)
  }
}

function handleAddComment() {
  if (newComment.value.trim()) {
    emit('action', 'comment', taskInfo.value.id, newComment.value.trim())
    newComment.value = ''
  }
}

async function loadTaskLog() {
  loadingLog.value = true
  try {
    taskLog.value = await invoke<string>('kanban_get_task_log', { taskId: taskInfo.value.id })
  } catch (e) {
    taskLog.value = `Failed to load log: ${e}`
  } finally {
    loadingLog.value = false
  }
}
</script>

<style scoped>
@keyframes slide-in {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}
.animate-slide-in {
  animation: slide-in 0.2s ease-out;
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
.kanban-pill-id {
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 0.6rem;
}
</style>
