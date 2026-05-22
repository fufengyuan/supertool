<template>
  <div class="bg-base-200/40 rounded-lg border border-base-content/10">
    <!-- 外层：工具名 + 参数摘要 -->
    <div 
      class="px-2.5 py-1.5 cursor-pointer hover:bg-base-200/60 transition-colors"
      @click="toggleExpand"
    >
      <div class="flex items-center gap-2">
        <SvgIcon :name="icon" size="12" class="text-base-content/50" />
        <span class="text-base-content/60 text-xs font-medium">{{ title }}</span>
        <span class="text-base-content/50 text-xs truncate flex-1">{{ summary }}</span>
        <!-- 状态标识 -->
        <span v-if="tool.status === 'completed'" class="text-success text-xs">✓</span>
        <span v-else-if="tool.status === 'running'" class="text-base-content/40 text-xs animate-pulse">○</span>
        <span v-else-if="tool.status === 'error'" class="text-error/60 text-xs">✕</span>
        <!-- 执行时长 -->
        <span v-if="tool.durationMs" class="text-base-content/30 text-xs">{{ (tool.durationMs / 1000).toFixed(1) }}s</span>
        <SvgIcon 
          :name="isExpanded ? 'chevronDown' : 'chevronRight'" 
          size="10" 
          class="text-base-content/30"
        />
      </div>
    </div>
    <!-- 预览：折叠时展示格式化结果 -->
    <div v-if="tool.result && !isExpanded && showPreview" class="px-2.5 pb-1.5 text-xs">
      <slot name="preview" :result="tool.result"></slot>
    </div>
    <!-- 折叠内容：详细结果 -->
    <div v-if="isExpanded" class="px-2.5 py-1.5 bg-base-200/20 text-xs">
      <!-- 参数 -->
      <div v-if="tool.args && Object.keys(tool.args).length > 0" class="mb-1">
        <span class="text-base-content/40">参数：</span>
        <pre class="bg-base-200/50 rounded p-1.5 mt-1 overflow-auto text-xs max-h-24">{{ JSON.stringify(tool.args, null, 2) }}</pre>
      </div>
      <!-- 结果 -->
      <div v-if="tool.result" class="mt-1">
        <span class="text-base-content/40">结果：</span>
        <slot name="result" :result="tool.result">
          <pre class="bg-base-200/50 rounded p-1.5 mt-1 overflow-auto text-xs max-h-32 whitespace-pre-wrap font-mono">{{ tool.result }}</pre>
        </slot>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import SvgIcon from '../SvgIcon.vue';

interface ToolCall {
  name: string;
  args?: Record<string, unknown>;
  result?: string;
  status?: string;
  durationMs?: number;
  isSubAgent?: boolean;
}

const props = defineProps<{
  tool: ToolCall;
  expanded?: boolean;
  icon?: string;
  title?: string;
  summary?: string;
  showPreview?: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle'): void;
}>();

const isExpanded = ref(props.expanded || false);

const icon = computed(() => props.icon || 'tool');
const title = computed(() => props.title || props.tool.name);
const summary = computed(() => props.summary || '');
const showPreview = computed(() => props.showPreview ?? true);

const toggleExpand = () => {
  isExpanded.value = !isExpanded.value;
  emit('toggle');
};
</script>