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
  color: oklch(var(--er));
}

.badge-medium {
  background: rgba(223, 142, 29, 0.1);
  color: oklch(var(--wa));
}

.badge-low {
  background: rgba(64, 160, 43, 0.1);
  color: oklch(var(--su));
}

/* General variants */
.badge-tag {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

.badge-primary {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

.badge-default {
  background: oklch(var(--b2));
  color: oklch(var(--bc) / 0.6);
  border: 1px solid oklch(var(--bc) / 0.1);
}
</style>
