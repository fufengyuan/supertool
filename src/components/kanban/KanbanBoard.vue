<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10">
      <div class="flex items-center gap-3">
        <h1 class="text-sm font-medium">{{ currentBoard?.name || '看板' }}</h1>
        <select 
          v-if="boards.length > 1"
          v-model="selectedBoardSlug"
          class="select select-sm select-bordered bg-base-200"
          @change="switchBoard"
        >
          <option v-for="board in boards" :key="board.slug" :value="board.slug">
            {{ board.name }}
          </option>
        </select>
      </div>
      <div class="flex items-center gap-2">
        <button class="btn btn-sm btn-ghost" @click="refreshTasks">
          <SvgIcon name="refresh" size="14" />
        </button>
        <button class="btn btn-sm btn-primary" @click="showCreateTask = true">
          新建任务
        </button>
      </div>
    </div>

    <!-- Stats bar -->
    <div v-if="stats" class="flex items-center gap-3 px-4 py-2 bg-base-200/50 text-xs border-b border-base-content/10">
      <span class="font-medium">统计</span>
      <div class="flex items-center gap-2 ml-2">
        <span class="px-2 py-0.5 rounded-full bg-warning/20 text-warning">Todo {{ stats.todo || 0 }}</span>
        <span class="px-2 py-0.5 rounded-full bg-info/20 text-info">Ready {{ stats.ready || 0 }}</span>
        <span class="px-2 py-0.5 rounded-full bg-primary/20 text-primary">进行 {{ stats.in_progress || 0 }}</span>
        <span class="px-2 py-0.5 rounded-full bg-error/20 text-error">阻塞 {{ stats.blocked || 0 }}</span>
        <span class="px-2 py-0.5 rounded-full bg-success/20 text-success">完成 {{ stats.done || 0 }}</span>
      </div>
      <span class="ml-auto text-base-content/50">共 {{ totalTasks }} 项</span>
    </div>

    <!-- Kanban columns -->
    <div class="flex-1 overflow-hidden">
      <div class="h-full flex gap-3 p-3 overflow-x-auto">
        <!-- Todo column -->
        <KanbanColumn
          title="待办"
          status="todo"
          :tasks="todoTasks"
          color="warning"
          @task-click="showTaskDetail"
          @task-action="handleTaskAction"
        />
        <!-- Ready column -->
        <KanbanColumn
          title="就绪"
          status="ready"
          :tasks="readyTasks"
          color="info"
          @task-click="showTaskDetail"
          @task-action="handleTaskAction"
        />
        <!-- In Progress column -->
        <KanbanColumn
          title="进行中"
          status="in_progress"
          :tasks="inProgressTasks"
          color="primary"
          @task-click="showTaskDetail"
          @task-action="handleTaskAction"
        />
        <!-- Blocked column -->
        <KanbanColumn
          title="阻塞"
          status="blocked"
          :tasks="blockedTasks"
          color="error"
          @task-click="showTaskDetail"
          @task-action="handleTaskAction"
        />
        <!-- Done column -->
        <KanbanColumn
          title="完成"
          status="done"
          :tasks="doneTasks"
          color="success"
          @task-click="showTaskDetail"
          @task-action="handleTaskAction"
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import KanbanColumn from './KanbanColumn.vue';
import TaskDetailDrawer from './TaskDetailDrawer.vue';
import CreateTaskModal from './CreateTaskModal.vue';

// Types
interface KanbanBoard {
  slug: string;
  name: string;
  description?: string;
  archived: boolean;
  isCurrent: boolean;
  total: number;
}

interface KanbanTask {
  id: string;
  title: string;
  body?: string;
  status: string;
  assignee?: string;
  priority?: number;
  skills?: string[];
  createdBy?: string;
  createdAt?: number;
  startedAt?: number;
  completedAt?: number;
  workspaceKind?: string;
  workspacePath?: string;
  branchName?: string;
  result?: string;
  sessionId?: string;
}

interface KanbanTaskDetail {
  task: KanbanTask;
  latestSummary?: string;
  parents: string[];
  children: string[];
  comments: Array<{ id: number; author: string; body: string; createdAt: number }>;
  events: Array<{ kind: string; payload: unknown; createdAt: number; runId?: number }>;
  runs: Array<{ id: number; profile: string; stepKey?: string; status?: string; outcome?: string; summary?: string; error?: string; startedAt: number; endedAt?: number }>;
}

// State
const boards = ref<KanbanBoard[]>([]);
const currentBoard = ref<KanbanBoard | null>(null);
const selectedBoardSlug = ref('');
const tasks = ref<KanbanTask[]>([]);
const stats = ref<Record<string, number> | null>(null);
const assignees = ref<Array<{ name: string; on_disk: boolean; counts: Record<string, number> }>>([]);
const selectedTask = ref<KanbanTaskDetail | null>(null);
const showCreateTask = ref(false);

// Computed
const todoTasks = computed(() => tasks.value.filter(t => t.status === 'todo'));
const readyTasks = computed(() => tasks.value.filter(t => t.status === 'ready'));
const inProgressTasks = computed(() => tasks.value.filter(t => t.status === 'in_progress'));
const blockedTasks = computed(() => tasks.value.filter(t => t.status === 'blocked'));
const doneTasks = computed(() => tasks.value.filter(t => t.status === 'done'));
const totalTasks = computed(() => tasks.value.length);

// Methods
async function loadBoards() {
  try {
    boards.value = await invoke('kanban_list_boards');
    currentBoard.value = boards.value.find(b => b.isCurrent) || boards.value[0];
    selectedBoardSlug.value = currentBoard.value?.slug || '';
  } catch (e) {
    console.error('Failed to load boards:', e);
  }
}

async function loadTasks() {
  try {
    tasks.value = await invoke('kanban_list_tasks', { board: null, status: null, assignee: null });
  } catch (e) {
    console.error('Failed to load tasks:', e);
  }
}

async function loadStats() {
  try {
    const result = await invoke<{ by_status: Record<string, number>; by_assignee: Record<string, number> }>('kanban_get_stats', { board: null });
    stats.value = result?.by_status || {};
  } catch (e) {
    console.error('Failed to load stats:', e);
  }
}

async function loadAssignees() {
  try {
    assignees.value = await invoke('kanban_list_assignees');
  } catch (e) {
    console.error('Failed to load assignees:', e);
  }
}

async function refreshTasks() {
  await Promise.all([loadTasks(), loadStats()]);
}

async function switchBoard() {
  if (selectedBoardSlug.value) {
    try {
      await invoke('kanban_switch_board', { slug: selectedBoardSlug.value });
      await loadBoards();
      await refreshTasks();
    } catch (e) {
      console.error('Failed to switch board:', e);
    }
  }
}

async function showTaskDetail(task: KanbanTask) {
  try {
    selectedTask.value = await invoke('kanban_show_task', { taskId: task.id });
  } catch (e) {
    console.error('Failed to load task detail:', e);
  }
}

async function handleTaskAction(action: string, taskId: string, ...args: unknown[]) {
  try {
    switch (action) {
      case 'assign':
        await invoke('kanban_assign_task', { taskId, assignee: args[0] });
        break;
      case 'reclaim':
        await invoke('kanban_reclaim_task', { taskId });
        break;
      case 'complete':
        await invoke('kanban_complete_task', { taskId, summary: args[0] || null });
        break;
      case 'block':
        await invoke('kanban_block_task', { taskId, reason: args[0] || '需要人工介入' });
        break;
      case 'unblock':
        await invoke('kanban_unblock_task', { taskId });
        break;
      case 'archive':
        await invoke('kanban_archive_task', { taskId });
        break;
      case 'comment':
        await invoke('kanban_add_comment', { taskId, body: args[0] });
        break;
    }
    await refreshTasks();
    if (selectedTask.value?.task.id === taskId) {
      selectedTask.value = await invoke('kanban_show_task', { taskId });
    }
  } catch (e) {
    console.error(`Failed to ${action} task:`, e);
  }
}

async function createTask(data: { title: string; body?: string; assignee?: string; parents?: string[] }) {
  try {
    await invoke('kanban_create_task', {
      title: data.title,
      body: data.body,
      assignee: data.assignee,
      parents: data.parents,
      priority: null,
      board: null,
    });
    showCreateTask.value = false;
    await refreshTasks();
  } catch (e) {
    console.error('Failed to create task:', e);
  }
}

onMounted(async () => {
  await Promise.all([loadBoards(), loadTasks(), loadStats(), loadAssignees()]);
});
</script>