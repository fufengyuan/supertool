<template>
  <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 bg-base-100/80 backdrop-blur-sm shrink-0">
    <!-- Left: title + usage -->
    <div class="flex items-center gap-3 min-w-0">
      <h1 class="text-sm font-semibold text-base-content/80 truncate">
        {{ sessionId ? `对话 ${sessionId.slice(-6)}` : 'Hermes Agent' }}
      </h1>
      <span
        v-if="usage"
        class="text-[10px] text-base-content/40 shrink-0"
        :title="`Prompt: ${usage.promptTokens.toLocaleString()} | Completion: ${usage.completionTokens.toLocaleString()}${usage.cost != null ? ` | Cost: $${usage.cost.toFixed(4)}` : ''}`"
      >
        {{ usage.totalTokens.toLocaleString() }} tokens
        <template v-if="usage.cost != null"> · ${{ usage.cost.toFixed(4) }}</template>
      </span>
    </div>

    <!-- Right: action buttons -->
    <div class="flex items-center gap-1 shrink-0">
      <!-- Context folder chip -->
      <template v-if="showContextFolder">
        <button
          v-if="contextFolder"
          class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs border border-info/20 bg-info/10 text-info/80 hover:bg-info/20 transition-colors max-w-[160px]"
          title="点击更改上下文文件夹"
          @click="$emit('pickFolder')"
        >
          <SvgIcon name="folderOpen" size="12" class="shrink-0" />
          <span class="truncate">{{ folderName(contextFolder) }}</span>
          <SvgIcon
            name="x"
            size="10"
            class="shrink-0 opacity-60 hover:opacity-100"
            @click.stop="$emit('clearFolder')"
          />
        </button>
        <button
          v-else
          class="p-1.5 rounded-lg text-base-content/50 hover:bg-base-200/60 hover:text-base-content/70 transition-colors"
          title="设置上下文文件夹"
          @click="$emit('pickFolder')"
        >
          <SvgIcon name="folderOpen" size="14" />
        </button>
      </template>

      <!-- Fast mode toggle -->
      <button
        class="relative p-1.5 rounded-lg transition-colors"
        :class="
          fastMode
            ? 'text-warning bg-warning/10 hover:bg-warning/20'
            : 'text-base-content/50 hover:bg-base-200/60 hover:text-base-content/70'
        "
        :title="fastMode ? '优先模式已开启 — Agent 将跳过确认提示' : '优先模式'"
        @click="$emit('toggleFast')"
      >
        <SvgIcon name="zap" size="14" />
      </button>

      <!-- New chat button -->
      <button
        class="p-1.5 rounded-lg text-base-content/50 hover:bg-base-200/60 hover:text-base-content/70 transition-colors"
        title="新对话"
        @click="$emit('newChat')"
      >
        <SvgIcon name="plus" size="14" />
      </button>

      <!-- Clear button (only when messages exist) -->
      <button
        v-if="hasMessages"
        class="p-1.5 rounded-lg text-base-content/50 hover:bg-error/10 hover:text-error transition-colors"
        title="清空对话"
        @click="$emit('clear')"
      >
        <SvgIcon name="trash" size="14" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue';
import type { UsageState } from './types';

defineProps<{
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
</script>
