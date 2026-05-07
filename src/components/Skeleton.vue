<template>
  <div class="skeleton" :class="`skeleton-${variant}`">
    <template v-if="variant === 'text'">
      <div
        v-for="i in rows"
        :key="i"
        class="skeleton-line skeleton-text"
        :style="{
          width: i === rows && lastLineWidth ? lastLineWidth : '100%',
          height: typeof lineHeight === 'number' ? `${lineHeight}px` : lineHeight,
        }"
      ></div>
    </template>

    <template v-else-if="variant === 'list'">
      <div v-for="i in rows" :key="i" class="skeleton-list-item">
        <div class="skeleton-circle"></div>
        <div class="skeleton-list-content">
          <div class="skeleton-line skeleton-text" style="width: 70%"></div>
          <div class="skeleton-line skeleton-text skeleton-sm" style="width: 50%"></div>
        </div>
      </div>
    </template>

    <template v-else>
      <div
        v-for="i in rows"
        :key="i"
        class="skeleton-block"
        :style="{ height: typeof blockHeight === 'number' ? `${blockHeight}px` : blockHeight }"
      ></div>
    </template>
  </div>
</template>

<script setup lang="ts">
defineProps({
  variant: {
    type: String,
    default: 'text',
    validator: (v: string) => ['text', 'card', 'list'].includes(v),
  },
  rows: {
    type: Number,
    default: 3,
  },
  lineHeight: {
    type: [String, Number],
    default: '16px',
  },
  lastLineWidth: {
    type: String,
    default: '60%',
  },
  blockHeight: {
    type: [String, Number],
    default: '120px',
  },
});
</script>

<style scoped>
.skeleton {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}

.skeleton-line {
  border-radius: 6px;
  background: linear-gradient(
    90deg,
    var(--border-color) 25%,
    var(--card-bg) 50%,
    var(--border-color) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}

.skeleton-text {
  height: 16px;
}

.skeleton-sm {
  height: 12px;
  margin-top: 8px;
}

.skeleton-block {
  border-radius: 10px;
  background: linear-gradient(
    90deg,
    var(--border-color) 25%,
    var(--card-bg) 50%,
    var(--border-color) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}

.skeleton-circle {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  flex-shrink: 0;
  background: linear-gradient(
    90deg,
    var(--border-color) 25%,
    var(--card-bg) 50%,
    var(--border-color) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}

.skeleton-list-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
}

.skeleton-list-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
</style>
