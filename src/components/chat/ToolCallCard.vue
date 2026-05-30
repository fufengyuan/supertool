<template>
  <div class="flex items-center gap-2 px-2 py-1 bg-base-200/40 rounded-lg border border-base-content/10">
    <!-- 工具 emoji（Hermes API 返回） -->
    <span v-if="tool.emoji" class="text-sm">{{ tool.emoji }}</span>
    <SvgIcon v-else :name="iconName" size="12" class="text-base-content/50" />
    
    <!-- 工具名称 -->
    <span class="text-base-content/70 text-xs font-medium">{{ titleText }}</span>
    
    <!-- 参数摘要 -->
    <span v-if="summaryText" class="text-base-content/50 text-xs truncate flex-1">{{ summaryText }}</span>
    
    <!-- 状态标识 -->
    <span v-if="tool.status === 'completed'" class="text-success text-xs">✓</span>
    <span v-else-if="tool.status === 'running'" class="text-base-content/40 text-xs animate-pulse">○</span>
    <span v-else-if="tool.status === 'error'" class="text-error/60 text-xs">✕</span>
    
    <!-- 执行时长 -->
    <span v-if="tool.durationMs" class="text-base-content/30 text-xs">{{ (tool.durationMs / 1000).toFixed(1) }}s</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';

interface ToolCall {
  name: string;
  args?: Record<string, unknown>;
  result?: string;
  status?: string;
  durationMs?: number;
  isSubAgent?: boolean;
  label?: string;
  emoji?: string;
}

const props = defineProps<{
  tool: ToolCall;
  icon?: string;
  title?: string;
  summary?: string;
}>();

const iconName = computed(() => props.icon || (props.tool.isSubAgent ? 'bot' : 'tool'));

const titleText = computed(() => {
  if (props.tool.emoji && props.tool.label) {
    return props.tool.label;
  }
  if (props.tool.label) {
    return props.tool.label;
  }
  return props.title || (props.tool.isSubAgent ? '子 Agent' : props.tool.name);
});

const summaryText = computed(() => {
  if (props.summary) {return props.summary;}
  if (props.tool.isSubAgent) {
    const args = props.tool.args as Record<string, unknown> | undefined;
    const goal = args?.goal || args?.task || args?.prompt;
    return goal ? String(goal).slice(0, 80) + '...' : '';
  }
  return '';
});
</script>