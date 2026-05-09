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
        <SvgIcon name="alertTriangle" size="20" />
      </span>
      <span v-else class="text-info">
        <SvgIcon name="info" size="20" />
      </span>
      <span class="text-sm flex-1">{{ message }}</span>
      <button class="btn btn-xs btn-ghost btn-circle opacity-60 hover:opacity-100" @click="$emit('close')" title="关闭"><SvgIcon name="x" size="14" /></button>
    </div>
    <div v-if="duration > 0" class="absolute bottom-0 left-0 right-0 h-0.5 bg-base-300/50">
      <div class="h-full rounded-full transition-all duration-100 ease-linear" :class="progressBarVariant" :style="{ width: `${progress}%` }"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
