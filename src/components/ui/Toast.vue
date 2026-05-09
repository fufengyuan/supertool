<template>
  <div class="alert shadow-lg" :class="alertVariant" role="alert">
    <div class="flex items-center gap-2 w-full">
      <span v-if="type === 'success'" class="text-success">
        <SvgIcon name="checkCircle" size="20" class="text-success" />
      </span>
      <span v-else-if="type === 'error'" class="text-error">
        <SvgIcon name="xCircle" size="20" class="text-error" />
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
