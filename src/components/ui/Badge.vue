<template>
  <span class="badge" :class="[`badge-${variant}`, { 'badge-sm': size === 'sm' }]">
    <slot />
  </span>
</template>

<script setup lang="ts">
defineProps({
  variant: {
    type: String,
    default: 'tag',
    validator: (v: string) => ['high', 'medium', 'low', 'tag', 'primary', 'default'].includes(v),
  },
  size: {
    type: String,
    default: 'md',
    validator: (v: string) => ['sm', 'md'].includes(v),
  },
});
</script>

<style scoped>
.badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.badge-sm {
  font-size: 11px;
  padding: 1px 6px;
}

/* Priority variants */
.badge-high {
  background: rgba(210, 15, 57, 0.1);
  color: var(--high-priority);
}

.badge-medium {
  background: rgba(223, 142, 29, 0.1);
  color: var(--medium-priority);
}

.badge-low {
  background: rgba(64, 160, 43, 0.1);
  color: var(--low-priority);
}

/* General variants */
.badge-tag {
  background: var(--primary-light);
  color: var(--primary-color);
}

.badge-primary {
  background: var(--primary-light);
  color: var(--primary-color);
}

.badge-default {
  background: var(--input-bg);
  color: var(--main-text-secondary);
  border: 1px solid var(--border-color);
}
</style>
