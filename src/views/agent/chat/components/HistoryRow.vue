<template>
  <div class="flex gap-2 w-full pl-0">
    <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-primary/10">
      <SvgIcon name="bot" size="14" class="text-primary/60" />
    </div>
    <div class="max-w-[80%]">
      <!-- Reasoning row -->
      <details
        v-if="variant === 'reasoning'"
        class="group/history mb-1"
      >
        <summary class="flex items-center gap-2 cursor-pointer text-xs text-base-content/50 hover:text-base-content/70 select-none">
          <SvgIcon
            name="chevronRight"
            size="10"
            class="transition-transform group-open/history:rotate-90"
          />
          <SvgIcon name="brain" size="12" />
          <span>Thinking</span>
          <span class="text-base-content/30">{{ lineCount }} {{ lineCount === 1 ? 'line' : 'lines' }}</span>
        </summary>
        <pre class="mt-1 p-2 rounded bg-base-200/40 border border-base-content/5 text-xs text-base-content/60 whitespace-pre-wrap overflow-x-auto max-h-64 overflow-y-auto">{{ reasoningText }}</pre>
      </details>

      <!-- Tool call row -->
      <details
        v-else-if="variant === 'tool-call'"
        class="group/history mb-1"
      >
        <summary class="flex items-center gap-2 cursor-pointer text-xs text-base-content/50 hover:text-base-content/70 select-none">
          <SvgIcon
            name="chevronRight"
            size="10"
            class="transition-transform group-open/history:rotate-90"
          />
          <SvgIcon name="tool" size="12" />
          <span>Tool call</span>
          <code class="px-1 py-0.5 rounded bg-primary/10 text-primary text-[10px]">{{ toolCallName }}</code>
          <span class="text-base-content/30 truncate max-w-[200px]">{{ argsSummary }}</span>
        </summary>
        <pre class="mt-1 p-2 rounded bg-base-200/40 border border-base-content/5 text-xs text-base-content/60 whitespace-pre-wrap overflow-x-auto max-h-64 overflow-y-auto font-mono">{{ toolCallArgs || '(no arguments)' }}</pre>
      </details>

      <!-- Tool result row -->
      <details
        v-else-if="variant === 'tool-result'"
        class="group/history mb-1"
      >
        <summary class="flex items-center gap-2 cursor-pointer text-xs text-base-content/50 hover:text-base-content/70 select-none">
          <SvgIcon
            name="chevronRight"
            size="10"
            class="transition-transform group-open/history:rotate-90"
          />
          <SvgIcon name="checkCircle" size="12" />
          <span>Tool result</span>
          <code class="px-1 py-0.5 rounded bg-success/10 text-success text-[10px]">{{ toolResultName }}</code>
          <span class="text-base-content/30">{{ toolResultLines }} {{ toolResultLines === 1 ? 'line' : 'lines' }}</span>
        </summary>
        <pre class="mt-1 p-2 rounded bg-base-200/40 border border-base-content/5 text-xs text-base-content/60 whitespace-pre-wrap overflow-x-auto max-h-64 overflow-y-auto">{{ toolResultContent || '(empty)' }}</pre>
      </details>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import type { ReasoningMessage, ToolCallMessage, ToolResultMessage } from '../types';

const props = defineProps<{
  variant: 'reasoning' | 'tool-call' | 'tool-result';
  msg: ReasoningMessage | ToolCallMessage | ToolResultMessage;
}>();

const reasoningText = computed(() =>
  props.variant === 'reasoning' ? (props.msg as ReasoningMessage).text : '',
);

const lineCount = computed(() => {
  const text = reasoningText.value;
  return text ? text.split('\n').length : 0;
});

const toolCallName = computed(() =>
  props.variant === 'tool-call' ? (props.msg as ToolCallMessage).name : '',
);

const toolCallArgs = computed(() =>
  props.variant === 'tool-call' ? (props.msg as ToolCallMessage).args : '',
);

const argsSummary = computed(() => {
  const flat = toolCallArgs.value.replace(/\s+/g, ' ').trim();
  if (flat.length <= 80) return flat;
  return flat.slice(0, 77) + '…';
});

const toolResultName = computed(() =>
  props.variant === 'tool-result' ? (props.msg as ToolResultMessage).name : '',
);

const toolResultContent = computed(() =>
  props.variant === 'tool-result' ? (props.msg as ToolResultMessage).content : '',
);

const toolResultLines = computed(() => {
  const content = toolResultContent.value;
  return content ? content.split('\n').length : 0;
});
</script>
