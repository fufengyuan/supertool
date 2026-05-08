<template>
  <component
    :is="tag"
    class="btn"
    :class="[`btn-${variant}`, { 'btn-sm': size === 'sm' }]"
    :type="tag === 'button' ? buttonType : undefined"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <span v-if="loading" class="btn-spinner"></span>
    <slot />
  </component>
</template>

<script setup lang="ts">
const props = defineProps({
  variant: {
    type: String,
    default: 'primary',
    validator: (v: string) => ['primary', 'danger', 'ghost', 'success', 'warning'].includes(v),
  },
  size: {
    type: String,
    default: 'md',
    validator: (v: string) => ['sm', 'md'].includes(v),
  },
  tag: {
    type: String,
    default: 'button',
  },
  buttonType: {
    type: String,
    default: 'button',
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  loading: {
    type: Boolean,
    default: false,
  },
});

defineEmits(['click']);
</script>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 20px;
  border: none;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
  font-family: inherit;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Variants */
.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: color-mix(in oklab, var(--color-primary) 80%, transparent);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(136, 57, 239, 0.3);
}

.btn-danger {
  background: var(--color-error);
  color: white;
}

.btn-danger:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-ghost {
  background: var(--color-base-200);
  color: var(--color-base-content);
  border: 1.5px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--color-base-100);
  border-color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.btn-success {
  background: var(--color-success);
  color: white;
}

.btn-success:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn-warning {
  background: var(--color-warning);
  color: white;
}

.btn-warning:hover:not(:disabled) {
  opacity: 0.9;
}

/* Sizes */
.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
  border-radius: 8px;
}

/* Spinner */
.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

.btn-ghost .btn-spinner {
  border-color: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-top-color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

@keyframes spin {
  from {
    transform: rotate(0);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
