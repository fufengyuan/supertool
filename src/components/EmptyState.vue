<template>
  <div class="empty-state">
    <svg
      v-if="!$slots.icon"
      viewBox="0 0 24 24"
      width="48"
      height="48"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
    </svg>
    <slot name="icon" />
    <p class="empty-state-text">{{ text }}</p>
    <p v-if="subtext" class="empty-state-subtext">{{ subtext }}</p>
    <slot name="action" />
  </div>
</template>

<script setup lang="ts">
defineProps({
  text: {
    type: String,
    default: '暂无内容',
  },
  subtext: {
    type: String,
    default: '',
  },
});
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px 20px;
  gap: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  text-align: center;
}

.empty-state svg {
  opacity: 0.3;
}

.empty-state-text {
  font-size: 14px;
  margin: 0;
}

.empty-state-subtext {
  font-size: 13px;
  margin: 0;
  opacity: 0.7;
}

.empty-state-action {
  margin-top: 8px;
}
</style>
