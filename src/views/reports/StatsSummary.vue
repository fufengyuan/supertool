<template>
  <div class="mb-5">
    <div class="grid grid-cols-[repeat(auto-fit,minmax(120px,1fr))] gap-4 mb-5">
      <div class="bg-base-100 p-4 rounded-xl text-center border-2 border-base-content/10 transition-all duration-300 hover:-translate-y-1 hover:shadow-sm">
        <div class="text-[28px] font-bold text-primary mb-2">{{ stats.total }}</div>
        <div class="text-sm text-base-content/40">{{ label || $t('stats.completedCount') }}</div>
      </div>
      <div class="bg-base-100 p-4 rounded-xl text-center border-2 border-base-content/10 transition-all duration-300 hover:-translate-y-1 hover:shadow-sm">
        <div class="text-[28px] font-bold text-primary mb-2">{{ stats.byPriority.high }}</div>
        <div class="text-sm text-base-content/40">{{ $t('stats.highPriority') }}</div>
      </div>
      <div class="bg-base-100 p-4 rounded-xl text-center border-2 border-base-content/10 transition-all duration-300 hover:-translate-y-1 hover:shadow-sm">
        <div class="text-[28px] font-bold text-primary mb-2">{{ stats.byPriority.medium }}</div>
        <div class="text-sm text-base-content/40">{{ $t('stats.mediumPriority') }}</div>
      </div>
      <div class="bg-base-100 p-4 rounded-xl text-center border-2 border-base-content/10 transition-all duration-300 hover:-translate-y-1 hover:shadow-sm">
        <div class="text-[28px] font-bold text-primary mb-2">{{ stats.byPriority.low }}</div>
        <div class="text-sm text-base-content/40">{{ $t('stats.lowPriority') }}</div>
      </div>
    </div>

    <!-- 任务列表 -->
    <div class="mt-5">
      <h4 class="text-[15px] font-semibold text-base-content mb-3">{{ listTitle || $t('stats.taskList') }}</h4>
      <ul class="list-none p-0 max-h-[200px] overflow-y-auto">
        <li v-for="task in tasks" :key="task.id" class="flex justify-between items-center px-3 py-2.5 bg-base-100 rounded-lg mb-2 border border-base-content/10">
          <span class="text-sm text-base-content">{{ task.text }}</span>
          <span class="text-xs text-base-content/40 bg-success/10 px-2 py-1 rounded-md">{{ formatDate(task.completedAt || task.updatedAt) }}</span>
        </li>
      </ul>
      <p v-if="tasks.length === 0" class="text-center text-base-content/40 text-sm p-5">{{ emptyText || $t('stats.empty') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps({
  stats: { type: Object as () => { total: number; byPriority: { high: number; medium: number; low: number } }, required: true },
  tasks: { type: Array as () => any[], required: true },
  formatDate: { type: Function, required: true },
  label: { type: String, default: '完成任务数' },
  listTitle: { type: String, default: '完成任务' },
  emptyText: { type: String, default: '暂无完成任务' },
});
</script>
