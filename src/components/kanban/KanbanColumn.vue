<template>
  <div class="flex-shrink-0 w-72 flex flex-col rounded-xl shadow-sm" :class="columnBgClass">
    <!-- Column header with colored bar -->
    <div class="flex items-center gap-2 px-3 py-2.5 rounded-t-xl" :class="headerBgClass">
      <span class="w-3 h-3 rounded-full" :class="colorDotClass"></span>
      <span class="text-sm font-semibold">{{ title }}</span>
      <span class="ml-auto text-xs px-1.5 py-0.5 rounded-full bg-base-content/10">{{ tasks.length }}</span>
    </div>

    <!-- Tasks list -->
    <div class="flex-1 overflow-y-auto p-2 space-y-2 min-h-[200px]">
      <div 
        v-for="task in tasks" 
        :key="task.id"
        class="bg-base-100 rounded-lg p-3 cursor-pointer border border-base-content/5 hover:border-primary/30 hover:shadow-md transition-all group"
        @click="$emit('task-click', task)"
      >
        <!-- Task header -->
        <div class="flex items-start justify-between gap-2">
          <span class="text-sm font-medium leading-snug flex-1 line-clamp-2">{{ task.title }}</span>
          <span v-if="task.priority" class="text-xs px-1.5 py-0.5 rounded bg-warning/20 text-warning shrink-0">
            P{{ task.priority }}
          </span>
        </div>

        <!-- Task meta -->
        <div class="mt-2 flex flex-wrap items-center gap-2 text-xs text-base-content/60">
          <span v-if="task.assignee" class="flex items-center gap-1">
            <SvgIcon name="user" size="12" />
            <span class="truncate max-w-[100px]">{{ task.assignee }}</span>
          </span>
          <span v-if="task.skills && task.skills.length > 0" class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-primary/10 text-primary">
            <SvgIcon name="skill" size="10" />
            <span class="truncate max-w-[80px]">{{ task.skills[0] }}</span>
          </span>
        </div>

        <!-- Actions (shown on hover) -->
        <div class="mt-2 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button 
            v-if="status === 'in_progress'"
            class="btn btn-xs btn-ghost px-2 text-warning hover:bg-warning/10"
            @click.stop="$emit('task-action', 'reclaim', task.id)"
            title="回收任务"
          >
            回收
          </button>
          <button 
            v-if="status === 'ready' || status === 'in_progress'"
            class="btn btn-xs btn-ghost px-2 text-success hover:bg-success/10"
            @click.stop="$emit('task-action', 'complete', task.id)"
            title="完成任务"
          >
            完成
          </button>
          <button 
            v-if="status === 'ready' || status === 'in_progress'"
            class="btn btn-xs btn-ghost px-2 text-error hover:bg-error/10"
            @click.stop="$emit('task-action', 'block', task.id, '需要人工介入')"
            title="阻塞任务"
          >
            阻塞
          </button>
          <button 
            v-if="status === 'blocked'"
            class="btn btn-xs btn-ghost px-2 text-success hover:bg-success/10"
            @click.stop="$emit('task-action', 'unblock', task.id)"
            title="解除阻塞"
          >
            解除
          </button>
          <button 
            v-if="status === 'done'"
            class="btn btn-xs btn-ghost px-2 text-base-content/50 hover:bg-base-content/5"
            @click.stop="$emit('task-action', 'archive', task.id)"
            title="归档任务"
          >
            归档
          </button>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="tasks.length === 0" class="flex flex-col items-center justify-center py-8 text-xs text-base-content/40">
        <SvgIcon name="inbox" size="24" class="mb-2 opacity-30" />
        <span>暂无任务</span>
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

// Column background (subtle tint)
const columnBgClass = computed(() => {
  switch (props.color) {
    case 'warning': return 'bg-warning/5';
    case 'info': return 'bg-info/5';
    case 'primary': return 'bg-primary/5';
    case 'error': return 'bg-error/5';
    case 'success': return 'bg-success/5';
    default: return 'bg-base-200/30';
  }
});

// Header background (stronger tint)
const headerBgClass = computed(() => {
  switch (props.color) {
    case 'warning': return 'bg-warning/15';
    case 'info': return 'bg-info/15';
    case 'primary': return 'bg-primary/15';
    case 'error': return 'bg-error/15';
    case 'success': return 'bg-success/15';
    default: return 'bg-base-200';
  }
});

// Color dot
const colorDotClass = computed(() => {
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