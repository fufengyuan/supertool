<template>
  <div class="flex-shrink-0 w-72 flex flex-col bg-base-200/30 rounded-lg">
    <!-- Column header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-base-content/10">
      <div class="flex items-center gap-2">
        <span :class="colorClass" class="w-2 h-2 rounded-full"></span>
        <span class="text-sm font-medium">{{ title }}</span>
        <span class="text-xs text-base-content/50">{{ tasks.length }}</span>
      </div>
    </div>

    <!-- Tasks list -->
    <div class="flex-1 overflow-y-auto p-2 space-y-2">
      <div 
        v-for="task in tasks" 
        :key="task.id"
        class="bg-base-100 rounded-lg p-2.5 cursor-pointer hover:ring-1 hover:ring-primary/30 transition-shadow"
        @click="$emit('task-click', task)"
      >
        <!-- Task header -->
        <div class="flex items-start justify-between gap-2">
          <span class="text-sm font-medium leading-tight flex-1">{{ task.title }}</span>
          <span v-if="task.priority" class="text-xs text-base-content/50 shrink-0">
            P{{ task.priority }}
          </span>
        </div>

        <!-- Assignee -->
        <div v-if="task.assignee" class="mt-1.5 flex items-center gap-1.5">
          <SvgIcon name="user" size="12" class="text-base-content/40" />
          <span class="text-xs text-base-content/60">{{ task.assignee }}</span>
        </div>

        <!-- Skills -->
        <div v-if="task.skills && task.skills.length > 0" class="mt-1.5 flex items-center gap-1.5">
          <SvgIcon name="skill" size="12" class="text-base-content/40" />
          <span class="text-xs text-base-content/50">{{ task.skills.join(', ') }}</span>
        </div>

        <!-- Actions -->
        <div class="mt-2 flex items-center gap-1">
          <button 
            v-if="status === 'in_progress'"
            class="btn btn-xs btn-ghost text-error"
            @click.stop="$emit('task-action', 'reclaim', task.id)"
            title="回收任务"
          >
            回收
          </button>
          <button 
            v-if="status === 'ready' || status === 'in_progress'"
            class="btn btn-xs btn-ghost text-success"
            @click.stop="$emit('task-action', 'complete', task.id)"
            title="完成任务"
          >
            完成
          </button>
          <button 
            v-if="status === 'ready' || status === 'in_progress'"
            class="btn btn-xs btn-ghost text-error"
            @click.stop="$emit('task-action', 'block', task.id, '需要人工介入')"
            title="阻塞任务"
          >
            阻塞
          </button>
          <button 
            v-if="status === 'blocked'"
            class="btn btn-xs btn-ghost text-success"
            @click.stop="$emit('task-action', 'unblock', task.id)"
            title="解除阻塞"
          >
            解除
          </button>
          <button 
            v-if="status === 'done'"
            class="btn btn-xs btn-ghost text-base-content/50"
            @click.stop="$emit('task-action', 'archive', task.id)"
            title="归档任务"
          >
            归档
          </button>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="tasks.length === 0" class="text-center py-8 text-xs text-base-content/40">
        暂无任务
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';

interface KanbanTask {
  id: string;
  title: string;
  status: string;
  assignee?: string;
  priority?: number;
  skills?: string[];
  createdBy?: string;
}

const props = defineProps<{
  title: string;
  status: string;
  tasks: KanbanTask[];
  color: string;
}>();

defineEmits<{
  (e: 'task-click', task: KanbanTask): void;
  (e: 'task-action', action: string, taskId: string, ...args: unknown[]): void;
}>();

const colorClass = computed(() => {
  switch (props.color) {
    case 'warning': return 'bg-warning';
    case 'info': return 'bg-info';
    case 'primary': return 'bg-primary';
    case 'error': return 'bg-error';
    case 'success': return 'bg-success';
    default: return 'bg-base-content/40';
  }
});
</script>