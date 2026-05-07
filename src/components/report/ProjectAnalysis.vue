<template>
  <div class="project-analysis comparison">
    <h3 class="section-title">{{ title || $t('comparison.lastWeekTitle') }}</h3>
    <div class="comparison-grid">
      <div class="comparison-item">
        <div class="comparison-label">{{ currentLabel || $t('comparison.thisWeek') }}</div>
        <div class="comparison-value">{{ currentStats.total }}</div>
      </div>
      <div class="comparison-item">
        <div class="comparison-label">{{ previousLabel || $t('comparison.lastWeek') }}</div>
        <div class="comparison-value">{{ previousStats.total }}</div>
      </div>
      <div class="comparison-item highlight">
        <div class="comparison-label">{{ $t('comparison.change') }}</div>
        <div class="comparison-value" :class="changeClass">
          {{ changeText }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps({
  currentStats: { type: Object, required: true },
  previousStats: { type: Object, required: true },
  title: { type: String, default: '与上周对比' },
  currentLabel: { type: String, default: '本周完成' },
  previousLabel: { type: String, default: '上周完成' },
});

const changeValue = computed(() => props.currentStats.total - props.previousStats.total);

const changeText = computed(() => {
  const change = changeValue.value;
  if (change > 0) return `+${change}`;
  if (change < 0) return `${change}`;
  return '0';
});

const changeClass = computed(() => {
  const change = changeValue.value;
  if (change > 0) return 'positive';
  if (change < 0) return 'negative';
  return 'neutral';
});
</script>

<style scoped>
.project-analysis {
  margin-bottom: 32px;
  padding: 20px;
  background: linear-gradient(135deg, var(--completed-bg), var(--input-bg));
  border-radius: 12px;
  border: 2px solid var(--border-color);
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--primary-color);
  margin-bottom: 16px;
}

.comparison-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.comparison-item {
  background: var(--card-bg);
  padding: 16px;
  border-radius: 12px;
  text-align: center;
  border: 2px solid var(--border-color);
}

.comparison-item.highlight {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.comparison-label {
  font-size: 13px;
  color: var(--empty-color);
  margin-bottom: 8px;
}

.comparison-item.highlight .comparison-label {
  color: rgba(255, 255, 255, 0.8);
}

.comparison-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--main-text);
}

.comparison-item.highlight .comparison-value {
  color: white;
}

.comparison-value.positive {
  color: var(--success-color);
}

.comparison-value.negative {
  color: var(--high-priority);
}

.comparison-value.neutral {
  color: var(--empty-color);
}
</style>
