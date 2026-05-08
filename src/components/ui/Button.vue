<template>
  <component
    :is="tag"
    class="btn"
    :class="[btnVariant, { 'btn-sm': size === 'sm', 'btn-outline': variant === 'ghost' }]"
    :type="tag === 'button' ? buttonType : undefined"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <span v-if="loading" class="loading loading-spinner loading-xs"></span>
    <slot />
  </component>
</template>

<script setup lang="ts">
import { computed } from 'vue'

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
  disabled: { type: Boolean, default: false },
  loading: { type: Boolean, default: false },
})

defineEmits(['click'])

const btnVariant = computed(() => {
  const map: Record<string, string> = {
    'primary': 'btn-primary',
    'danger': 'btn-error',
    'ghost': 'btn-ghost',
    'success': 'btn-success',
    'warning': 'btn-warning',
  }
  return map[props.variant] || 'btn-primary'
})
</script>
