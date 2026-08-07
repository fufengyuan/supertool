<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- 顶部标题栏 -->
    <header class="shrink-0 flex items-center gap-3 px-5 py-3 border-b border-base-content/10 bg-base-100/80 backdrop-blur-sm sticky top-0 z-20">
      <button
        class="btn btn-ghost btn-sm btn-circle shrink-0"
        title="返回工具列表"
        @click="$emit('back')"
      >
        <SvgIcon name="arrowLeft" size="16" />
      </button>
      <span
        class="shrink-0 w-9 h-9 rounded-lg bg-primary/15 text-primary flex items-center justify-center"
      >
        <SvgIcon :name="icon" size="18" />
      </span>
      <div class="min-w-0 flex-1">
        <h2 class="text-sm font-bold text-base-content leading-tight flex items-center gap-2">
          {{ name }}
          <span
            v-if="!offline"
            class="text-[10px] font-normal text-warning flex items-center gap-0.5 border border-warning/30 rounded-full px-1.5 py-px"
            title="需要联网"
          >
            <SvgIcon name="globe" size="9" /> 需联网
          </span>
        </h2>
        <p class="text-[11px] text-base-content/50 leading-tight truncate">{{ description }}</p>
      </div>
    </header>

    <!-- 工具内容区 -->
    <div class="flex-1 min-h-0 overflow-y-auto px-5 py-4">
      <div class="max-w-[1100px] mx-auto flex flex-col gap-4">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'

defineProps<{
  icon: string
  name: string
  description: string
  offline?: boolean
}>()

defineEmits<{ back: [] }>()
</script>
