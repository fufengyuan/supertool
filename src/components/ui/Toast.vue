<template>
  <div class="alert shadow-lg" :class="alertVariant" role="alert">
    <div class="flex items-center gap-2 w-full">
      <span v-if="type === 'success'" class="text-success">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      </span>
      <span v-else-if="type === 'error'" class="text-error">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
      </span>
      <span v-else-if="type === 'warning'" class="text-warning">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
      </span>
      <span v-else class="text-info">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
      </span>
      <span class="text-sm flex-1">{{ message }}</span>
      <button class="btn btn-xs btn-ghost btn-circle opacity-60 hover:opacity-100" @click="$emit('close')" title="关闭"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
    </div>
    <div v-if="duration > 0" class="absolute bottom-0 left-0 right-0 h-0.5 bg-base-300/50">
      <div class="h-full rounded-full transition-all duration-100 ease-linear" :class="progressBarVariant" :style="{ width: `${progress}%` }"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  message: { type: String, required: true },
  type: {
    type: String,
    default: 'info',
    validator: (v: string) => ['success', 'error', 'warning', 'info'].includes(v),
  },
  duration: { type: Number, default: 3000 },
  progress: { type: Number, default: 100 },
})

defineEmits(['close'])

const alertVariant = computed(() => {
  const map: Record<string, string> = {
    success: 'alert-success',
    error: 'alert-error',
    warning: 'alert-warning',
    info: 'alert-info',
  }
  return map[props.type] || 'alert-info'
})

const progressBarVariant = computed(() => {
  const map: Record<string, string> = {
    success: 'bg-success',
    error: 'bg-error',
    warning: 'bg-warning',
    info: 'bg-info',
  }
  return map[props.type] || 'bg-info'
})
</script>
