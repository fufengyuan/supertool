<template>
  <!-- Bubble messages filtered to non-empty, history rows always shown -->
  <template v-for="(msg, i) in visibleMessages" :key="msg.id">
    <!-- Reasoning row -->
    <HistoryRow v-if="msg.kind === 'reasoning'" :msg="msg" variant="reasoning" />
    <!-- Tool call row -->
    <HistoryRow v-else-if="msg.kind === 'tool_call'" :msg="msg" variant="tool-call" />
    <!-- Tool result row -->
    <HistoryRow v-else-if="msg.kind === 'tool_result'" :msg="msg" variant="tool-result" />
    <!-- Bubble message -->
    <MessageRow
      v-else
      :msg="msg"
      :is-last="i === visibleMessages.length - 1"
      :is-loading="isLoading"
      :on-approve="onApprove"
      :on-deny="onDeny"
    />
  </template>

  <!-- Typing indicator -->
  <div
    v-if="isLoading && !lastMessageIsAgent"
    class="flex gap-2 w-full"
  >
    <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-primary/20">
      <SvgIcon name="bot" size="14" class="text-primary" />
    </div>
    <div class="bg-base-200/60 border border-base-content/10 rounded-xl px-3 py-2 rounded-tl-sm">
      <div v-if="toolProgress" class="text-sm text-base-content/60">{{ toolProgress }}</div>
      <div v-else class="flex gap-1 items-center h-5">
        <span class="w-2 h-2 bg-base-content/30 rounded-full animate-bounce [animation-delay:-0.3s]" />
        <span class="w-2 h-2 bg-base-content/30 rounded-full animate-bounce [animation-delay:-0.15s]" />
        <span class="w-2 h-2 bg-base-content/30 rounded-full animate-bounce" />
      </div>
    </div>
  </div>

  <!-- Inline tool progress when agent is last message -->
  <div
    v-if="isLoading && toolProgress && lastMessageIsAgent"
    class="text-xs text-base-content/50 pl-10 py-1"
  >
    {{ toolProgress }}
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import MessageRow from './MessageRow.vue';
import HistoryRow from './HistoryRow.vue';
import type { ChatMessage, ChatBubbleMessage } from '../types';

const props = defineProps<{
  messages: ChatMessage[];
  isLoading: boolean;
  toolProgress: string | null;
  onApprove: () => void;
  onDeny: () => void;
}>();

function isBubble(m: ChatMessage): m is ChatBubbleMessage {
  const k = (m as { kind?: string }).kind;
  return !k || k === 'user' || k === 'assistant';
}

const visibleMessages = computed(() =>
  props.messages.filter((m) => {
    if (!isBubble(m)) return true;
    return ((m.content as string) || '').trim().length > 0;
  }),
);

const lastMessageIsAgent = computed(() => {
  const lastBubble = [...props.messages].reverse().find(isBubble);
  return !!lastBubble && lastBubble.role === 'agent';
});
</script>
