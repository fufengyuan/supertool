<template>
  <div class="report-container">
    <div class="report-header">
      <h2 class="report-title">工作报表</h2>
      <div class="week-selector">
        <button @click="previousWeek" class="week-btn">← 上一周</button>
        <span class="week-label">{{ currentWeekLabel }}</span>
        <button @click="nextWeek" class="week-btn">下一周 →</button>
      </div>
    </div>

    <!-- 本周统计 + 标签分布 -->
    <div class="report-grid">
      <div class="report-section">
        <h3 class="section-title">本周工作内容</h3>
        <StatsSummary
          :stats="currentWeekStats"
          :tasks="currentWeekTasks"
          :format-date="formatDate"
          label="完成任务数"
          list-title="本周完成任务"
          empty-text="本周暂无完成任务"
        />
      </div>

      <div class="report-section">
        <TagAnalysis :stats="currentWeekStats" title="标签分布" />
      </div>
    </div>

    <!-- 上周对比 -->
    <div class="report-section comparison-section">
      <h3 class="section-title">与上周对比</h3>
      <ProjectAnalysis
        :current-stats="currentWeekStats"
        :previous-stats="lastWeekStats"
        current-label="本周完成"
        previous-label="上周完成"
      />

      <!-- 上周任务列表 -->
      <div class="task-list-section">
        <h4>上周完成任务</h4>
        <ul class="report-task-list">
          <li v-for="task in lastWeekTasks" :key="task.id" class="report-task-item">
            <span class="task-text">{{ task.text }}</span>
            <span class="task-date">{{ formatDate(task.completedAt || task.updatedAt) }}</span>
          </li>
        </ul>
        <p v-if="lastWeekTasks.length === 0" class="empty-text">上周暂无完成任务</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed } from 'vue';
import { useTodoStore } from '@/stores/todoStore';
import StatsSummary from '@/components/report/StatsSummary.vue';
import TagAnalysis from '@/components/report/TagAnalysis.vue';
import ProjectAnalysis from '@/components/report/ProjectAnalysis.vue';

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

<style scoped>
.report-container {
  width: 100%;
  padding: 12px 16px;
}

.report-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.report-title {
  font-size: 18px;
  font-weight: 700;
  color: oklch(var(--bc));
  margin: 0;
}

.week-selector {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.week-btn {
  padding: 5px 12px;
  font-size: 12px;
  background: oklch(var(--p));
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.week-btn:hover {
  background: oklch(var(--p) / 0.8);
}

.week-label {
  font-size: 13px;
  font-weight: 600;
  color: oklch(var(--bc));
}

.report-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 12px;
  margin-bottom: 12px;
}

.report-section {
  margin-bottom: 12px;
  padding: 14px;
  background: oklch(var(--b1));
  border-radius: 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--p));
  margin-bottom: 10px;
}

.comparison-section {
  background: linear-gradient(135deg, oklch(var(--su) / 0.1), oklch(var(--b2)));
}

.task-list-section {
  margin-top: 20px;
}

.task-list-section h4 {
  font-size: 15px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin-bottom: 12px;
}

.report-task-list {
  list-style: none;
  padding: 0;
  max-height: 200px;
  overflow-y: auto;
}

.report-task-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: oklch(var(--b1));
  border-radius: 8px;
  margin-bottom: 8px;
  border: 1px solid oklch(var(--bc) / 0.1);
}

.task-text {
  font-size: 14px;
  color: oklch(var(--bc));
}

.task-date {
  font-size: 12px;
  color: oklch(var(--bc) / 0.4);
  background: oklch(var(--su) / 0.1);
  padding: 4px 8px;
  border-radius: 6px;
}

.empty-text {
  text-align: center;
  color: oklch(var(--bc) / 0.4);
  font-size: 14px;
  padding: 20px;
}
</style>
