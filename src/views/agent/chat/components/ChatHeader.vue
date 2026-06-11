<template>
  <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 bg-base-100/80">
    <div class="flex items-center gap-3 min-w-0">
      <div class="text-sm font-medium text-base-content truncate">
        {{ sessionId ? `Session ${sessionId.slice(-6)}` : (props.isClawMode ? 'Claw' : 'Hermes Chat') }}
      </div>
      <span v-if="usage" class="text-xs text-base-content/50 shrink-0" :title="usageTooltip">
        {{ usage.totalTokens.toLocaleString() }} tokens
        <span v-if="usage.cost != null"> · ${{ usage.cost.toFixed(4) }}</span>
      </span>
      <!-- Goal mode status indicator -->
      <span
        v-if="goalMode && goalStatus === 'active'"
        class="text-xs text-info/70 shrink-0"
        title="目标模式已激活"
      >
        🎯 {{ goalTurnsUsed }}/{{ goalMaxTurns }}
        <span v-if="goalLastVerdict === 'continue'" class="text-warning/70">⏳</span>
      </span>
      <span
        v-else-if="goalMode && goalStatus === 'done'"
        class="text-xs text-success/70 shrink-0"
        title="目标已完成"
      >
        🎯 ✓ 完成
      </span>
      <span
        v-else-if="goalMode && goalStatus === 'paused'"
        class="text-xs text-warning/70 shrink-0"
        title="目标已暂停"
      >
        🎯 ⏸ 已暂停
      </span>
    </div>
    <div class="flex items-center gap-1">
      <template v-if="showContextFolder">
        <button
          v-if="contextFolder"
          class="btn btn-ghost btn-xs gap-1"
          :title="`工作目录: ${contextFolder}`"
          @click="$emit('pickFolder')"
        >
          <SvgIcon name="folder" size="14" />
          <span class="text-xs max-w-[120px] truncate">{{ folderName(contextFolder) }}</span>
        </button>
        <button
          v-if="contextFolder"
          class="btn btn-ghost btn-xs btn-square"
          title="移除工作目录"
          @click="$emit('clearFolder')"
        >
          <SvgIcon name="x" size="12" />
        </button>
        <button
          v-if="!contextFolder"
          class="btn btn-ghost btn-xs"
          title="设置工作目录"
          @click="$emit('pickFolder')"
        >
          <SvgIcon name="folder" size="14" />
        </button>
      </template>
      <button
        class="btn btn-ghost btn-xs"
        :class="{ 'text-amber-500': fastMode }"
        :title="fastMode ? '快速模式: 已开启' : '快速模式: 已关闭'"
        @click="$emit('toggleFast')"
      >
        <SvgIcon name="zap" size="14" />
      </button>
      <button class="btn btn-ghost btn-xs" title="新建对话" @click="$emit('newChat')">
        <SvgIcon name="plus" size="16" />
      </button>
      <button
        v-if="hasMessages"
        class="btn btn-ghost btn-xs"
        :class="{ 'loading': compacting }"
        :disabled="compacting"
        title="压缩会话（把较早的消息总结成摘要）"
        @click="$emit('compact')"
      >
        <SvgIcon v-if="!compacting" name="compress" size="14" />
      </button>
      <button
        v-if="hasMessages"
        class="btn btn-ghost btn-xs"
        title="分叉会话"
        @click="$emit('fork')"
      >
        <SvgIcon name="copy" size="14" />
      </button>
      <button
        class="btn btn-ghost btn-xs"
        :class="{ 'text-success': planMode, 'text-base-content/50': !planMode }"
        :title="planMode ? '计划模式: 已开启 — 仅读取' : '计划模式: 已关闭'"
        @click="$emit('togglePlan')"
      >
        <SvgIcon name="clipboard" size="14" />
      </button>
      <button
        class="btn btn-ghost btn-xs"
        :class="{ 'text-info': goalMode, 'text-base-content/50': !goalMode }"
        :title="goalMode ? `目标模式: 已开启 — ${goalText}` : '目标模式: 已关闭'"
        @click="$emit('toggleGoal')"
      >
        <SvgIcon name="crosshair" size="14" />
      </button>
      <!-- Pause/Resume button (only visible when goal is active/paused) -->
      <button
        v-if="goalMode && (goalStatus === 'active' || goalStatus === 'paused')"
        class="btn btn-ghost btn-xs"
        :title="goalStatus === 'paused' ? '恢复目标' : '暂停目标'"
        @click="$emit('toggleGoalPause')"
      >
        <SvgIcon :name="goalStatus === 'paused' ? 'play' : 'pause'" size="14" />
      </button>
      <button
        class="btn btn-ghost btn-xs"
        :class="{ 'text-success': loopMode, 'text-base-content/30': !loopMode }"
        :title="loopMode ? '循环模式: 已开启 — 每次回复后自动重发。按 Esc 暂停' : '循环模式: 已关闭 — 每次一轮'"
        @click="$emit('toggleLoop')"
      >
        <SvgIcon name="refresh" size="14" />
      </button>
      <button
        v-if="hasMessages"
        class="btn btn-ghost btn-xs"
        title="清空对话"
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
  compacting?: boolean;
  isClawMode?: boolean;
  planMode: boolean;
  goalMode: boolean;
  goalText: string;
  goalStatus: string;
  goalTurnsUsed: number;
  goalMaxTurns: number;
  goalLastVerdict: string | null;
  loopMode: boolean;
}>();

defineEmits<{
  pickFolder: [];
  clearFolder: [];
  toggleFast: [];
  newChat: [];
  clear: [];
  compact: [];
  fork: [];
  togglePlan: [];
  toggleGoal: [];
  toggleGoalPause: [];
  toggleLoop: [];
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
