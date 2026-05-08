<template>
  <div class="stats-summary">
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-value">{{ stats.total }}</div>
        <div class="stat-label">{{ label || $t('stats.completedCount') }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.byPriority.high }}</div>
        <div class="stat-label">{{ $t('stats.highPriority') }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.byPriority.medium }}</div>
        <div class="stat-label">{{ $t('stats.mediumPriority') }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.byPriority.low }}</div>
        <div class="stat-label">{{ $t('stats.lowPriority') }}</div>
      </div>
    </div>

    <!-- 任务列表 -->
    <div class="task-list-section">
      <h4>{{ listTitle || $t('stats.taskList') }}</h4>
      <ul class="report-task-list">
        <li v-for="task in tasks" :key="task.id" class="report-task-item">
          <span class="task-text">{{ task.text }}</span>
          <span class="task-date">{{ formatDate(task.completedAt || task.updatedAt) }}</span>
        </li>
      </ul>
      <p v-if="tasks.length === 0" class="empty-text">{{ emptyText || $t('stats.empty') }}</p>
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

<style scoped>
.stats-summary {
  margin-bottom: 20px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 16px;
  margin-bottom: 20px;
}

.stat-card {
  background: var(--color-base-100);
  padding: 16px;
  border-radius: 12px;
  text-align: center;
  border: 2px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  transition: all 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--color-primary);
  margin-bottom: 8px;
}

.stat-label {
  font-size: 14px;
  color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
}

.task-list-section {
  margin-top: 20px;
}

.task-list-section h4 {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-base-content);
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
  background: var(--color-base-100);
  border-radius: 8px;
  margin-bottom: 8px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.task-text {
  font-size: 14px;
  color: var(--color-base-content);
}

.task-date {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
  background: color-mix(in oklab, var(--color-success) 10%, transparent);
  padding: 4px 8px;
  border-radius: 6px;
}

.empty-text {
  text-align: center;
  color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
  font-size: 14px;
  padding: 20px;
}
</style>
