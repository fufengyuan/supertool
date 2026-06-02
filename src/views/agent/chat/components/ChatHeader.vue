<template>
  <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 bg-base-100/80">
    <div class="flex items-center gap-3 min-w-0">
      <div class="text-sm font-medium text-base-content truncate">
        {{ sessionId ? `Session ${sessionId.slice(-6)}` : 'Hermes Chat' }}
      </div>
      <span v-if="usage" class="text-xs text-base-content/50 shrink-0" :title="usageTooltip">
        {{ usage.totalTokens.toLocaleString() }} tokens
        <span v-if="usage.cost != null"> · ${{ usage.cost.toFixed(4) }}</span>
      </span>
    </div>
    <div class="flex items-center gap-1">
      <template v-if="showContextFolder">
        <button
          v-if="contextFolder"
          class="btn btn-ghost btn-xs gap-1"
          :title="`Working folder: ${contextFolder}`"
          @click="$emit('pickFolder')"
        >
          <SvgIcon name="folder" size="14" />
          <span class="text-xs max-w-[120px] truncate">{{ folderName(contextFolder) }}</span>
        </button>
        <button
          v-if="contextFolder"
          class="btn btn-ghost btn-xs btn-square"
          title="Remove context folder"
          @click="$emit('clearFolder')"
        >
          <SvgIcon name="x" size="12" />
        </button>
        <button
          v-if="!contextFolder"
          class="btn btn-ghost btn-xs"
          title="Set working folder"
          @click="$emit('pickFolder')"
        >
          <SvgIcon name="folder" size="14" />
        </button>
      </template>
      <button
        class="btn btn-ghost btn-xs"
        :class="{ 'text-amber-500': fastMode }"
        :title="fastMode ? 'Fast mode: ON' : 'Fast mode: OFF'"
        @click="$emit('toggleFast')"
      >
        <SvgIcon name="zap" size="14" />
      </button>
      <button class="btn btn-ghost btn-xs" title="New chat" @click="$emit('newChat')">
        <SvgIcon name="plus" size="16" />
      </button>
      <button
        v-if="hasMessages"
        class="btn btn-ghost btn-xs"
        title="Clear chat"
        @click="$emit('clear')"
      >
        <SvgIcon name="trash" size="16" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import type { UsageState } from '../types';

const props = defineProps<{
  sessionId: string | null;
  usage: UsageState | null;
  fastMode: boolean;
  hasMessages: boolean;
  contextFolder: string | null;
  showContextFolder: boolean;
}>();

defineEmits<{
  pickFolder: [];
  clearFolder: [];
  toggleFast: [];
  newChat: [];
  clear: [];
}>();

function folderName(p: string): string {
  const parts = p.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] || p;
}

const usageTooltip = computed(() => {
  if (!props.usage) return '';
  const u = props.usage;
  return `Prompt: ${u.promptTokens.toLocaleString()} | Completion: ${u.completionTokens.toLocaleString()}${u.cost != null ? ` | Cost: $${u.cost.toFixed(4)}` : ''}`;
});
</script>
