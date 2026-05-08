<template>
  <div class="stats-bar">
    <div class="stat-item">
      <span class="stat-value">{{ totalTasks }}</span>
      <span class="stat-label">{{ $t('stats.totalTasks') }}</span>
    </div>
    <div class="stat-divider"></div>
    <div class="stat-item">
      <span class="stat-value success">{{ completedToday }}</span>
      <span class="stat-label">{{ $t('stats.completedToday') }}</span>
      <span v-if="todayVsYesterday !== 0" class="trend" :class="todayVsYesterday > 0 ? 'up' : 'down'">
        {{ todayVsYesterday > 0 ? '↑' : '↓' }}{{ Math.abs(todayVsYesterday) }}
      </span>
    </div>
    <div class="stat-divider"></div>
    <div class="stat-item">
      <span class="stat-value">{{ completedThisWeek }}</span>
      <span class="stat-label">{{ $t('stats.completedThisWeek') }}</span>
    </div>
    <div class="stat-divider"></div>
    <div class="stat-item">
      <span class="stat-value">{{ completionRate }}%</span>
      <span class="stat-label">{{ $t('stats.completionRate') }}</span>
    </div>
    <div class="stat-divider"></div>
    <div class="stat-item compact">
      <span class="stat-label">🔥 Streak</span>
      <span class="stat-value streak">{{ streak }}天</span>
    </div>
    <div class="stat-divider"></div>
    <div class="stat-item compact">
      <span class="stat-label">本周进度</span>
      <div class="mini-progress">
        <div class="mini-progress-fill" :style="{ width: `${weeklyCompletionRate}%` }"></div>
      </div>
      <span class="stat-value mini">{{ weeklyCompletionRate }}%</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTodoStore } from '@/stores/todoStore';

const { t } = useI18n();
const todoStore = useTodoStore();

// 计算总任务数
const totalTasks = computed(() => todoStore.todos.length);

// 获取今天的日期字符串
const getTodayStr = (): string => {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  return today.toISOString();
};

const getYesterdayStr = (): string => {
  const yesterday = new Date();
  yesterday.setDate(yesterday.getDate() - 1);
  yesterday.setHours(0, 0, 0, 0);
  return yesterday.toISOString();
};

// 计算今天完成的任务数
const completedToday = computed(() => {
  const todayStr = getTodayStr();
  return todoStore.todos.filter((todo) => {
    if (!todo.completed || !todo.updatedAt) return false;
    const updatedDate = new Date(todo.updatedAt);
    updatedDate.setHours(0, 0, 0, 0);
    return updatedDate.toISOString() === todayStr;
  }).length;
});

// 计算昨天完成的任务数
const completedYesterday = computed(() => {
  const yesterdayStr = getYesterdayStr();
  return todoStore.todos.filter((todo) => {
    if (!todo.completed || !todo.updatedAt) return false;
    const updatedDate = new Date(todo.updatedAt);
    updatedDate.setHours(0, 0, 0, 0);
    return updatedDate.toISOString() === yesterdayStr;
  }).length;
});

// 今日 vs 昨日对比
const todayVsYesterday = computed(() => completedToday.value - completedYesterday.value);

// 计算本周完成的任务数
const completedThisWeek = computed(() => {
  const today = new Date();
  const firstDayOfWeek = new Date(today);
  firstDayOfWeek.setDate(today.getDate() - today.getDay());
  firstDayOfWeek.setHours(0, 0, 0, 0);

  return todoStore.todos.filter((todo) => {
    if (!todo.completed || !todo.updatedAt) return false;
    const updatedDate = new Date(todo.updatedAt);
    updatedDate.setHours(0, 0, 0, 0);
    return updatedDate >= firstDayOfWeek;
  }).length;
});

// 本周总任务数（本周创建的）
const totalThisWeek = computed(() => {
  const today = new Date();
  const firstDayOfWeek = new Date(today);
  firstDayOfWeek.setDate(today.getDate() - today.getDay());
  firstDayOfWeek.setHours(0, 0, 0, 0);

  return todoStore.todos.filter((todo) => {
    const createdDate = new Date(todo.createdAt);
    createdDate.setHours(0, 0, 0, 0);
    return createdDate >= firstDayOfWeek;
  }).length;
});

// 本周完成率
const weeklyCompletionRate = computed(() => {
  if (totalThisWeek.value === 0) return 0;
  return Math.round((completedThisWeek.value / totalThisWeek.value) * 100);
});

// 计算完成率
const completionRate = computed(() => {
  if (todoStore.todos.length === 0) return 0;
  const completed = todoStore.todos.filter((todo) => todo.completed).length;
  return Math.round((completed / todoStore.todos.length) * 100);
});

// Streak: 从今天往前数，连续有完成任务的天数
const streak = computed(() => {
  // Pre-compute a Set of completed dates for O(1) lookup
  const completedDates = new Set<string>();
  for (const todo of todoStore.todos) {
    if (todo.completed && todo.updatedAt) {
      const d = new Date(todo.updatedAt);
      d.setHours(0, 0, 0, 0);
      completedDates.add(d.toISOString());
    }
  }

  let count = 0;
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  // Check each day from today backwards
  for (let i = 0; i < 365; i++) {
    const checkDate = new Date(today);
    checkDate.setDate(today.getDate() - i);
    checkDate.setHours(0, 0, 0, 0);
    const dateStr = checkDate.toISOString();

    if (completedDates.has(dateStr)) {
      count++;
    } else {
      // If today has no completions, skip and start counting from yesterday
      if (i === 0) continue;
      break;
    }
  }

  return count;
});

// 计算各优先级任务数量
const highPriorityCount = computed(
  () => todoStore.todos.filter((todo) => todo.priority === 'high').length
);

const mediumPriorityCount = computed(
  () => todoStore.todos.filter((todo) => todo.priority === 'medium').length
);

const lowPriorityCount = computed(
  () => todoStore.todos.filter((todo) => todo.priority === 'low').length
);

// 计算各优先级占比
const highPriorityPercent = computed(() => {
  if (todoStore.todos.length === 0) return 0;
  return (highPriorityCount.value / todoStore.todos.length) * 100;
});

const mediumPriorityPercent = computed(() => {
  if (todoStore.todos.length === 0) return 0;
  return (mediumPriorityCount.value / todoStore.todos.length) * 100;
});

const lowPriorityPercent = computed(() => {
  if (todoStore.todos.length === 0) return 0;
  return (lowPriorityCount.value / todoStore.todos.length) * 100;
});

// 热门标签 Top 5（词云）
interface TagCloudItem {
  name: string;
  count: number;
  fontSize: number;
  opacity: number;
  color: string;
}

const tagColors = [
  'color-mix(in oklab, var(--color-primary) 10%, transparent)',
  'rgba(34, 197, 94, 0.15)',
  'rgba(245, 158, 11, 0.15)',
  'rgba(239, 68, 68, 0.15)',
  'rgba(139, 92, 246, 0.15)',
];

const topTags = computed<TagCloudItem[]>(() => {
  const tagMap: Record<string, number> = {};

  todoStore.todos.forEach((todo) => {
    const tag = todo.tag;
    if (!tag || tag === 'custom') return;
    if (!tagMap[tag]) {
      tagMap[tag] = 0;
    }
    tagMap[tag]++;
  });

  const sorted = Object.entries(tagMap)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 5);

  if (sorted.length === 0) return [];

  const maxCount = sorted[0][1];
  const minCount = sorted[sorted.length - 1][1];

  return sorted.map(([name, count], idx) => {
    const range = maxCount - minCount || 1;
    const ratio = (count - minCount) / range;
    const fontSize = 12 + ratio * 12; // 12px ~ 24px
    const opacity = 0.6 + ratio * 0.4;
    const color = tagColors[idx % tagColors.length];
    return { name, count, fontSize, opacity, color };
  });
});

// 今日激励语
const motivations = [
  '每一个完成的任务，都是通往目标的一步 🚀',
  '今天你做得很棒，继续保持！💪',
  '千里之行，始于足下。加油！✨',
  '高效的一天从完成任务开始！🌟',
  '你的坚持终将美好，继续前进！🎯',
  '小步快跑，积少成多，你正在变得更强！🏆',
  '每一次勾选，都是对自己的肯定 ✅',
  '不要等待机会，创造机会！🔥',
  '专注当下，完成眼前的每一件小事 🌈',
  '你已经走了这么远，别停下！🌻',
];

const dailyMotivation = computed(() => {
  const today = new Date();
  const dayOfYear = Math.floor(
    (today.getTime() - new Date(today.getFullYear(), 0, 0).getTime()) / (1000 * 60 * 60 * 24)
  );
  return motivations[dayOfYear % motivations.length];
});
</script>

<style scoped>
.stats-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  flex-wrap: wrap;
}

.stat-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
  white-space: nowrap;
}

.stat-item.compact {
  gap: 4px;
}

.stat-value {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
}

.stat-value.success {
  color: var(--color-success);
}

.stat-value.streak {
  color: var(--color-warning);
  font-size: 15px;
}

.stat-value.mini {
  font-size: 13px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  min-width: 32px;
}

.stat-label {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.trend {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 4px;
  border-radius: 3px;
}

.trend.up {
  color: var(--color-success);
  background: rgba(34, 197, 94, 0.1);
}

.trend.down {
  color: var(--color-error);
  background: rgba(239, 68, 68, 0.1);
}

.stat-divider {
  width: 1px;
  height: 20px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  flex-shrink: 0;
}

.mini-progress {
  width: 48px;
  height: 6px;
  background: var(--color-base-200);
  border-radius: 3px;
  overflow: hidden;
}

.mini-progress-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 3px;
  transition: width 0.4s ease;
}
</style>
