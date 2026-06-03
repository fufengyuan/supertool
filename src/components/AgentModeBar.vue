<template>
  <div class="flex items-center gap-0 border-b border-base-300 bg-base-100">
    <button
      v-for="item in items"
      :key="item.value"
      class="relative px-4 py-2 text-xs font-medium transition-colors"
      :class="mode === item.value
        ? 'text-primary'
        : 'text-base-content/50 hover:text-base-content/80'"
      @click="agentModeStore.setMode(item.value)"
    >
      {{ item.label }}
      <!-- 激活下划线 -->
      <span
        v-if="mode === item.value"
        class="absolute bottom-0 left-2 right-2 h-0.5 bg-primary rounded-full"
      />
    </button>

    <!-- 右侧：claw 状态指示 -->
    <span
      v-if="mode === 'claw'"
      class="ml-auto mr-3 text-[10px] text-base-content/30"
    >
      终端模式
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAgentModeStore, type AgentMode } from '@/stores/agentModeStore'

const agentModeStore = useAgentModeStore()
const mode = computed(() => agentModeStore.mode)

const items: { label: string; value: AgentMode }[] = [
  { label: 'Hermes', value: 'hermes' },
  { label: 'Claw', value: 'claw' },
]
</script>
