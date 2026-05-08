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
  color: var(--color-error);
}

.badge-medium {
  background: rgba(223, 142, 29, 0.1);
  color: var(--color-warning);
}

.badge-low {
  background: rgba(64, 160, 43, 0.1);
  color: var(--color-success);
}

/* General variants */
.badge-tag {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
}

.badge-primary {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
}

.badge-default {
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}
</style>
