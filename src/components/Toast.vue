<template>
  <div class="toast" :class="`toast-${type}`" role="alert">
    <div class="toast-icon">
      <svg v-if="type === 'success'" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      <svg v-else-if="type === 'error'" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
      <svg v-else-if="type === 'warning'" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
      <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
    </div>
    <span class="toast-message">{{ message }}</span>
    <button class="toast-close" @click="$emit('close')" title="关闭">
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
    </button>
    <div v-if="duration > 0" class="toast-progress">
      <div class="toast-progress-bar" :style="{ width: `${progress}%` }"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps({
  message: {
    type: String,
    required: true,
  },
  type: {
    type: String,
    default: 'info',
    validator: (v: string) => ['success', 'error', 'warning', 'info'].includes(v),
  },
  duration: {
    type: Number,
    default: 3000,
  },
  progress: {
    type: Number,
    default: 100,
  },
});

defineEmits(['close']);
</script>

<style scoped>
.toast {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-radius: 10px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  box-shadow: 0 4px 6px rgba(0,0,0,0.1);
  min-width: 280px;
  max-width: 400px;
  position: relative;
  overflow: hidden;
  animation: toastSlideIn 0.3s ease;
}

.toast-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.toast-message {
  flex: 1;
  font-size: 14px;
  color: oklch(var(--bc));
  line-height: 1.4;
  word-break: break-word;
}

.toast-close {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: oklch(var(--bc) / 0.6);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.toast-close:hover {
  background: oklch(var(--p) / 0.1);
  color: oklch(var(--p));
}

.toast-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: transparent;
}

.toast-progress-bar {
  height: 100%;
  border-radius: 0 0 10px 10px;
  transition: width 50ms linear;
}

/* Type variants */
.toast-success .toast-icon { color: oklch(var(--su)); }
.toast-success .toast-progress-bar { background: oklch(var(--su)); }

.toast-error .toast-icon { color: oklch(var(--er)); }
.toast-error .toast-progress-bar { background: oklch(var(--er)); }

.toast-warning .toast-icon { color: oklch(var(--wa)); }
.toast-warning .toast-progress-bar { background: oklch(var(--wa)); }

.toast-info .toast-icon { color: oklch(var(--p)); }
.toast-info .toast-progress-bar { background: oklch(var(--p)); }

@keyframes toastSlideIn {
  from {
    opacity: 0;
    transform: translateX(100%);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes toastSlideOut {
  from {
    opacity: 1;
    transform: translateX(0);
  }
  to {
    opacity: 0;
    transform: translateX(100%);
  }
}
</style>
