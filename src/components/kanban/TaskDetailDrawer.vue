<template>
  <div class="fixed inset-y-0 right-0 w-96 bg-base-100 border-l border-base-content/10 shadow-xl z-50 flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
      <div class="flex items-center gap-2">
        <span :class="statusColorClass" class="w-2.5 h-2.5 rounded-full"></span>
        <span class="text-sm font-medium">{{ task.task.title }}</span>
      </div>
      <button class="btn btn-sm btn-ghost btn-circle" @click="$emit('close')">
        <SvgIcon name="close" size="14" />
      </button>
    </div>

    <!-- Task info -->
    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      <!-- Status & Assignee -->
      <div class="flex items-center gap-4">
        <div>
          <span class="text-xs text-base-content/50">状态</span>
          <div class="text-sm capitalize">{{ formatStatus(task.task.status) }}</div>
        </div>
        <div>
          <span class="text-xs text-base-content/50">执行者</span>
          <div class="text-sm">{{ task.task.assignee || '未分配' }}</div>
        </div>
        <div>
          <span class="text-xs text-base-content/50">优先级</span>
          <div class="text-sm">{{ task.task.priority ? `P${task.task.priority}` : '默认' }}</div>
        </div>
      </div>

      <!-- Body -->
      <div v-if="task.task.body">
        <span class="text-xs text-base-content/50 block mb-1">描述</span>
        <div class="text-sm bg-base-200/50 rounded p-2 whitespace-pre-wrap">{{ task.task.body }}</div>
      </div>

      <!-- Dependencies -->
      <div v-if="task.parents.length > 0">
        <span class="text-xs text-base-content/50 block mb-1">依赖任务</span>
        <div class="flex flex-wrap gap-1">
          <span v-for="p in task.parents" :key="p" class="badge badge-sm badge-ghost">
            {{ p }}
          </span>
        </div>
      </div>

      <!-- Assign selector -->
      <div>
        <span class="text-xs text-base-content/50 block mb-1">分配给</span>
        <select 
          class="select select-sm select-bordered w-full"
          :value="task.task.assignee || ''"
          @change="handleAssign"
        >
          <option value="">未分配</option>
          <option v-for="a in assignees" :key="a.name" :value="a.name">
            {{ a.name }} ({{ Object.values(a.counts).reduce((sum, c) => sum + c, 0) }} 个任务)
          </option>
        </select>
      </div>

      <!-- Runs history -->
      <div v-if="task.runs.length > 0">
        <span class="text-xs text-base-content/50 block mb-2">执行历史</span>
        <div class="space-y-2">
          <div v-for="run in task.runs" :key="run.id" class="bg-base-200/50 rounded p-2">
            <div class="flex items-center justify-between">
              <span class="text-xs text-base-content/60">{{ run.profile }}</span>
              <span class="text-xs text-base-content/50">{{ formatTime(run.startedAt) }}</span>
            </div>
            <div class="text-xs mt-1">
              <span v-if="run.status || run.outcome" :class="outcomeClass(run.status || run.outcome)">
                {{ run.status || run.outcome }}
              </span>
              <span v-if="run.error" class="text-error ml-2">{{ run.error.slice(0, 30) }}...</span>
              <span v-if="run.summary" class="text-base-content/60 ml-2">
                {{ run.summary.slice(0, 50) }}...
              </span>
            </div>
          </div>
        </div>
        
        <!-- Load log button -->
        <button 
          class="btn btn-sm btn-ghost mt-3 w-full"
          @click="loadTaskLog"
          :disabled="loadingLog"
        >
          <SvgIcon v-if="loadingLog" name="refresh" size="12" class="animate-spin mr-1" />
          <SvgIcon v-else name="terminal" size="12" class="mr-1" />
          {{ loadingLog ? '加载中...' : '查看执行日志' }}
        </button>
      </div>

      <!-- Execution log -->
      <div v-if="taskLog">
        <span class="text-xs text-base-content/50 block mb-1">执行日志</span>
        <div class="bg-base-300 rounded-lg p-3 text-xs font-mono whitespace-pre-wrap overflow-x-auto max-h-80 overflow-y-auto leading-relaxed">
          <LogContent :content="taskLog" />
        </div>
      </div>

      <!-- Comments -->
      <div>
        <span class="text-xs text-base-content/50 block mb-2">评论 ({{ task.comments.length }})</span>
        <div class="space-y-2">
          <div v-for="c in task.comments" :key="c.id" class="bg-base-200/50 rounded p-2">
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium">{{ c.author }}</span>
              <span class="text-xs text-base-content/50">{{ formatTime(c.createdAt) }}</span>
            </div>
            <div class="text-sm mt-1 whitespace-pre-wrap">{{ c.body }}</div>
          </div>
        </div>

        <!-- Add comment -->
        <div class="mt-3">
          <textarea 
            v-model="newComment"
            class="textarea textarea-sm textarea-bordered w-full"
            placeholder="添加评论..."
            rows="2"
          ></textarea>
          <button 
            class="btn btn-sm btn-ghost mt-1"
            :disabled="!newComment.trim()"
            @click="handleAddComment"
          >
            发送
          </button>
        </div>
      </div>
    </div>

    <!-- Actions footer -->
    <div class="px-4 py-3 border-t border-base-content/10 flex items-center gap-2">
      <button 
        v-if="task.task.status === 'ready' || task.task.status === 'in_progress'"
        class="btn btn-sm btn-success flex-1"
        @click="$emit('action', 'complete', task.task.id)"
      >
        完成
      </button>
      <button 
        v-if="task.task.status === 'in_progress'"
        class="btn btn-sm btn-error flex-1"
        @click="$emit('action', 'reclaim', task.task.id)"
      >
        回收
      </button>
      <button 
        v-if="task.task.status === 'ready' || task.task.status === 'in_progress'"
        class="btn btn-sm btn-warning flex-1"
        @click="$emit('action', 'block', task.task.id, '需要人工介入')"
      >
        阻塞
      </button>
      <button 
        v-if="task.task.status === 'blocked'"
        class="btn btn-sm btn-success flex-1"
        @click="$emit('action', 'unblock', task.task.id)"
      >
        解除阻塞
      </button>
      <button 
        v-if="task.task.status === 'done'"
        class="btn btn-sm btn-ghost flex-1"
        @click="$emit('action', 'archive', task.task.id)"
      >
        归档
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import LogContent from './LogContent.vue';

interface KanbanTaskDetail {
  task: {
    id: string;
    title: string;
    status: string;
    assignee?: string;
    priority?: number;
    body?: string;
    tenant?: string;
    skills?: string[];
    createdBy?: string;
    createdAt?: number;
    startedAt?: number;
    completedAt?: number;
    result?: string;
  };
  latestSummary?: string;
  parents: string[];
  children: string[];
  comments: Array<{ id: number; author: string; body: string; createdAt: number }>;
  events: Array<{ kind: string; payload: Record<string, unknown>; createdAt: number; runId?: number }>;
  runs: Array<{
    id: number;
    profile: string;
    stepKey?: string;
    status?: string;
    outcome?: string;
    summary?: string;
    error?: string;
    startedAt: number;
    endedAt?: number;
  }>;
}

const props = defineProps<{
  task: KanbanTaskDetail;
  assignees: Array<{ name: string; on_disk: boolean; counts: Record<string, number> }>;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'refresh'): void;
  (e: 'action', action: string, taskId: string, ...args: unknown[]): void;
}>();

const newComment = ref('');
const taskLog = ref('');
const loadingLog = ref(false);

async function loadTaskLog() {
  loadingLog.value = true;
  try {
    const log = await invoke<string>('kanban_get_task_log', { taskId: props.task.task.id });
    taskLog.value = log;
  } catch (e) {
    console.error('Failed to load task log:', e);
    taskLog.value = `加载日志失败: ${e}`;
  } finally {
    loadingLog.value = false;
  }
}

const statusColorClass = computed(() => {
  switch (props.task.task.status) {
    case 'todo': return 'bg-warning';
    case 'ready': return 'bg-info';
    case 'in_progress': return 'bg-primary animate-pulse';
    case 'blocked': return 'bg-error';
    case 'done': return 'bg-success';
    default: return 'bg-base-content/40';
  }
});

function formatStatus(status: string): string {
  const map: Record<string, string> = {
    todo: '待办',
    ready: '就绪',
    in_progress: '进行中',
    blocked: '阻塞',
    scheduled: '等待',
    done: '完成',
    archived: '归档',
  };
  return map[status] || status;
}

function formatTime(time: number | string): string {
  const timestamp = typeof time === 'number' ? time * 1000 : new Date(time).getTime();
  return new Date(timestamp).toLocaleString('zh-CN', {
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function outcomeClass(outcome: string | undefined): string {
  if (!outcome) return 'text-base-content/60';
  if (outcome === 'completed') return 'text-success';
  if (outcome === 'crashed' || outcome === 'timed_out') return 'text-error';
  if (outcome === 'blocked') return 'text-warning';
  return 'text-base-content/60';
}

function handleAssign(e: Event) {
  const target = e.target as HTMLSelectElement;
  const assignee = target.value;
  if (assignee !== props.task.task.assignee) {
    emit('action', 'assign', props.task.task.id, assignee);
  }
}

function handleAddComment() {
  if (newComment.value.trim()) {
    emit('action', 'comment', props.task.task.id, newComment.value.trim());
    newComment.value = '';
  }
}
</script>