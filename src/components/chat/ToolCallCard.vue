<template>
  <div
    class="px-3 py-2 rounded-xl border transition-colors"
    :class="statusClass"
  >
    <div class="flex items-center gap-2">
      <!-- Status indicator -->
      <span class="w-1.5 h-1.5 rounded-full shrink-0" :class="statusDotClass" />

      <!-- Tool name + label -->
      <span class="text-xs font-medium truncate">
        <template v-if="tool.emoji">{{ tool.emoji }} </template>
        {{ tool.label || tool.name }}
      </span>

      <!-- Duration -->
      <span v-if="tool.durationMs" class="text-[10px] text-base-content/30 ml-auto shrink-0">
        {{ formatDuration(tool.durationMs) }}
      </span>
    </div>

    <!-- Args (collapsible) -->
    <div v-if="hasArgs" class="mt-1.5">
      <button
        class="flex items-center gap-1 text-[10px] text-base-content/40 hover:text-base-content/60 transition-colors"
        @click="expanded = !expanded"
      >
        <SvgIcon :name="expanded ? 'chevronDown' : 'chevronRight'" size="8" />
        <span>参数</span>
      </button>
      <pre
        v-if="expanded"
        class="mt-1 px-2 py-1.5 text-[10px] text-base-content/50 bg-base-200/30 rounded border border-base-content/5 overflow-x-auto whitespace-pre-wrap font-mono"
      >{{ argsText }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'

export interface ToolInfo {
  name: string
  args?: Record<string, unknown>
  status?: string
  durationMs?: number
  emoji?: string
  label?: string
}

const props = defineProps<{
  tool: ToolInfo
}>()

const expanded = ref(false)

const hasArgs = computed(() => {
  return props.tool.args && Object.keys(props.tool.args).length > 0
})

const argsText = computed(() => {
  if (!props.tool.args) return ''
  return JSON.stringify(props.tool.args, null, 2)
})

const isRunning = computed(() => props.tool.status === 'running')
const isCompleted = computed(() => props.tool.status === 'completed' || !props.tool.status)

const statusClass = computed(() => {
  if (isRunning.value) return 'bg-info/5 border-info/20 text-info/80'
  return 'bg-base-200/30 border-base-content/10 text-base-content/60'
})

const statusDotClass = computed(() => {
  if (isRunning.value) return 'bg-info animate-pulse'
  return 'bg-success/60'
})

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  const s = (ms / 1000).toFixed(1)
  return `${s}s`
}
</script>
