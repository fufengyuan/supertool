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
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(136, 57, 239, 0.3);
}

.btn-danger {
  background: var(--danger-color);
  color: white;
}

.btn-danger:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-ghost {
  background: var(--input-bg);
  color: var(--main-text);
  border: 1.5px solid var(--input-border);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--card-bg);
  border-color: var(--main-text-secondary);
}

.btn-success {
  background: var(--success-color);
  color: white;
}

.btn-success:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn-warning {
  background: var(--warning-color);
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
  border-color: var(--border-color);
  border-top-color: var(--main-text-secondary);
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
