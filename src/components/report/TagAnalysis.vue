<template>
  <div class="tag-analysis">
    <h4>{{ title || $t('tag.distribution') }}</h4>
    <div class="tag-bar-chart">
      <div v-for="tag in Object.keys(stats.byTag)" :key="tag" class="tag-bar">
        <div class="tag-label">{{ tag }}</div>
        <div class="tag-count-bar">
          <div class="tag-fill" :style="{ width: getTagPercentage(tag) + '%' }"></div>
          <span class="tag-count">{{ stats.byTag[tag] }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
const props = defineProps({
  stats: { type: Object, required: true },
  title: { type: String, default: '标签分布' },
});

const getTagPercentage = (tag) => {
  if (props.stats.total === 0) return 0;
  return (props.stats.byTag[tag] / props.stats.total) * 100;
};
</script>

<style scoped>
.tag-analysis {
  margin-top: 20px;
}

.tag-analysis h4 {
  font-size: 15px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin-bottom: 12px;
}

.tag-bar-chart {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tag-bar {
  display: flex;
  align-items: center;
  gap: 12px;
}

.tag-label {
  min-width: 80px;
  font-size: 13px;
  color: oklch(var(--bc));
}

.tag-count-bar {
  flex: 1;
  height: 24px;
  background: oklch(var(--bc) / 0.1);
  border-radius: 12px;
  display: flex;
  align-items: center;
  padding: 4px;
  position: relative;
}

.tag-fill {
  height: 100%;
  background: linear-gradient(135deg, oklch(var(--p)), #4cc9f0);
  border-radius: 12px;
  transition: width 0.3s ease;
}

.tag-count {
  position: absolute;
  right: 8px;
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--bc));
}
</style>
