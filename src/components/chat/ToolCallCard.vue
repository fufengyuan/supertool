<template>
  <div class="flex flex-col">
    <div 
      class="flex items-center gap-2 px-2 py-1 bg-base-200/40 rounded-lg border border-base-content/10 cursor-pointer hover:bg-base-200/60 transition-colors"
      @click="toggleExpanded"
    >
      <!-- 工具 emoji（Hermes API 返回） -->
      <span v-if="tool.emoji" class="text-sm">{{ tool.emoji }}</span>
      <SvgIcon v-else :name="iconName" size="12" class="text-base-content/50" />
      
      <!-- 工具名称 -->
      <span class="text-base-content/70 text-xs font-medium">{{ titleText }}</span>
      
      <!-- 参数摘要 -->
      <span v-if="summaryText" class="text-base-content/50 text-xs truncate flex-1">{{ summaryText }}</span>
      
      <!-- 状态标识 -->
      <SvgIcon v-if="tool.status === 'completed'" name="check" size="10" class="text-success" />
      <SvgIcon v-else-if="tool.status === 'running'" name="loader" size="10" class="text-base-content/40 animate-spin" />
      <SvgIcon v-else-if="tool.status === 'error'" name="x" size="10" class="text-error" />
      
      <!-- 执行时长 -->
      <span v-if="tool.durationMs" class="text-base-content/30 text-xs">{{ (tool.durationMs / 1000).toFixed(1) }}s</span>
      
      <!-- 展开/折叠按钮 -->
      <SvgIcon 
        v-if="tool.result && tool.status === 'completed'"
        :name="expanded ? 'chevronDown' : 'chevronRight'" 
        size="10" 
        class="text-base-content/40"
      />
    </div>
    
    <!-- 展开的结果内容 -->
    <div v-if="expanded && tool.result" class="mt-1 ml-6 px-2 py-2 bg-base-300/30 rounded border border-base-content/10 text-xs">
      <div v-if="isJson" v-html="highlightedResult" class="whitespace-pre-wrap font-mono" />
      <div v-else class="whitespace-pre-wrap text-base-content/70">{{ tool.result }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import hljs from 'highlight.js';
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

const expanded = ref(false);

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

const toggleExpanded = () => {
  if (props.tool.result && props.tool.status === 'completed') {
    expanded.value = !expanded.value;
  }
};

const isJson = computed(() => {
  if (!props.tool.result) {return false;}
  try {
    JSON.parse(props.tool.result);
    return true;
  } catch {
    return false;
  }
});

const highlightedResult = computed(() => {
  if (!props.tool.result) {return '';}
  if (isJson.value) {
    const formatted = JSON.stringify(JSON.parse(props.tool.result), null, 2);
    return hljs.highlight(formatted, { language: 'json' }).value;
  }
  return hljs.highlightAuto(props.tool.result).value;
});
</script>