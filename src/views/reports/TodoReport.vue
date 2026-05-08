<template>
  <div class="w-full px-4 py-3">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-bold text-base-content m-0">工作报表</h2>
      <div class="flex items-center gap-2 shrink-0">
        <button @click="previousWeek" class="btn btn-primary btn-sm">← 上一周</button>
        <span class="text-[13px] font-semibold text-base-content">{{ currentWeekLabel }}</span>
        <button @click="nextWeek" class="btn btn-primary btn-sm">下一周 →</button>
      </div>
    </div>

    <!-- 本周统计 + 标签分布 -->
    <div class="grid grid-cols-[2fr_1fr] gap-3 mb-3">
      <div class="mb-3 p-3.5 bg-base-100 rounded-lg border border-base-content/10">
        <h3 class="text-sm font-semibold text-primary mb-2.5">本周工作内容</h3>
        <StatsSummary
          :stats="currentWeekStats"
          :tasks="currentWeekTasks"
          :format-date="formatDate"
          label="完成任务数"
          list-title="本周完成任务"
          empty-text="本周暂无完成任务"
        />
      </div>

      <div class="mb-3 p-3.5 bg-base-100 rounded-lg border border-base-content/10">
        <TagAnalysis :stats="currentWeekStats" title="标签分布" />
      </div>
    </div>

    <!-- 上周对比 -->
    <div class="mb-3 p-3.5 bg-base-100 rounded-lg border border-base-content/10 bg-gradient-to-br from-success/10 to-base-200">
      <h3 class="text-sm font-semibold text-primary mb-2.5">与上周对比</h3>
      <ProjectAnalysis
        :current-stats="currentWeekStats"
        :previous-stats="lastWeekStats"
        current-label="本周完成"
        previous-label="上周完成"
      />

      <!-- 上周任务列表 -->
      <div class="mt-5">
        <h4 class="text-[15px] font-semibold text-base-content mb-3">上周完成任务</h4>
        <ul class="list-none p-0 max-h-[200px] overflow-y-auto">
          <li v-for="task in lastWeekTasks" :key="task.id" class="flex justify-between items-center px-3 py-2.5 bg-base-100 rounded-lg mb-2 border border-base-content/10">
            <span class="text-sm text-base-content">{{ task.text }}</span>
            <span class="text-xs text-base-content/40 bg-success/10 px-2 py-1 rounded-md">{{ formatDate(task.completedAt || task.updatedAt) }}</span>
          </li>
        </ul>
        <p v-if="lastWeekTasks.length === 0" class="text-center text-base-content/40 text-sm p-5">上周暂无完成任务</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed } from 'vue';
import { useTodoStore } from '@/stores/todoStore';
import StatsSummary from '@/views/reports/StatsSummary.vue';
import TagAnalysis from '@/views/reports/TagAnalysis.vue';
import ProjectAnalysis from '@/views/reports/ProjectAnalysis.vue';

const todoStore = useTodoStore();

// 当前查看的周偏移量
const weekOffset = ref(0);

// 获取周的起止日期
const getWeekRange = (offset) => {
  const now = new Date();
  const dayOfWeek = now.getDay();
  const startOfWeek = new Date(now);
  startOfWeek.setDate(now.getDate() - dayOfWeek + offset * 7);
  startOfWeek.setHours(0, 0, 0, 0);

  const endOfWeek = new Date(startOfWeek);
  endOfWeek.setDate(startOfWeek.getDate() + 6);
  endOfWeek.setHours(23, 59, 59, 999);

  return { start: startOfWeek, end: endOfWeek };
};

// 当前周的标签
const currentWeekLabel = computed(() => {
  const range = getWeekRange(weekOffset.value);
  const startMonth = range.start.getMonth() + 1;
  const startDay = range.start.getDate();
  const endMonth = range.end.getMonth() + 1;
  const endDay = range.end.getDate();

  if (weekOffset.value === 0) return `本周 (${startMonth}/${startDay} - ${endMonth}/${endDay})`;
  if (weekOffset.value === -1) return `上周 (${startMonth}/${startDay} - ${endMonth}/${endDay})`;
  return `${startMonth}/${startDay} - ${endMonth}/${endDay}`;
});

// 获取指定周的任务
const getWeekTasks = (offset) => {
  const range = getWeekRange(offset);
  return todoStore.todos.filter((todo) => {
    if (!todo.completed) return false;
    const completedDate = new Date(todo.completedAt || todo.updatedAt);
    return completedDate >= range.start && completedDate <= range.end;
  });
};

// 当前周任务
const currentWeekTasks = computed(() => getWeekTasks(weekOffset.value));

// 上周任务
const lastWeekTasks = computed(() => getWeekTasks(weekOffset.value - 1));

// 计算统计数据
const calculateStats = (tasks) => {
  const stats = {
    total: tasks.length,
    byPriority: { high: 0, medium: 0, low: 0 },
    byTag: {},
  };

  tasks.forEach((task) => {
    const priority = task.priority || 'medium';
    stats.byPriority[priority]++;

    const tag = task.tag || '未分类';
    stats.byTag[tag] = (stats.byTag[tag] || 0) + 1;
  });

  return stats;
};

// 当前周统计
const currentWeekStats = computed(() => calculateStats(currentWeekTasks.value));

// 上周统计
const lastWeekStats = computed(() => calculateStats(lastWeekTasks.value));

// 格式化日期
const formatDate = (dateString) => {
  if (!dateString) return '';
  const date = new Date(dateString);
  const month = date.getMonth() + 1;
  const day = date.getDate();
  return `${month}/${day}`;
};

// 切换周
const previousWeek = () => {
  weekOffset.value--;
};

const nextWeek = () => {
  if (weekOffset.value < 1) weekOffset.value++;
};
</script>
