<template>
  <div
    ref="containerRef"
    class="flex-1 overflow-y-auto px-4 py-3"
  >
    <div class="max-w-3xl mx-auto space-y-3">
      <template v-for="(msg, i) in messages" :key="msg.id">
        <!-- User message -->
        <div
          v-if="msg.kind === 'user' || !msg.kind"
          class="flex gap-2 w-full group justify-end"
        >
          <div class="max-w-[80%]">
            <div class="bg-primary/15 border border-primary/25 rounded-xl px-3 py-2 rounded-tr-sm">
              <div
                class="prose prose-sm max-w-none text-base-content"
                v-html="renderMarkdown(msg.content)"
              />
            </div>
            <!-- Timestamp + copy button -->
            <div class="flex items-center justify-end gap-2 mt-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                class="text-[10px] text-base-content/30 hover:text-base-content/60 transition-colors"
                title="复制"
                @click="copyMessage(msg.content)"
              >
                <SvgIcon name="copy" size="10" />
              </button>
              <span v-if="msg.timestamp" class="text-[10px] text-base-content/30">{{ formatTime(msg.timestamp) }}</span>
            </div>
          </div>
        </div>

        <!-- Assistant message -->
        <div
          v-else-if="msg.kind === 'assistant'"
          class="flex gap-2 w-full group"
        >
          <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-primary/20 mt-0.5">
            <SvgIcon name="bot" size="14" class="text-primary" />
          </div>
          <div class="max-w-[80%] min-w-0">
            <div
              v-if="msg.content"
              class="bg-base-200/60 border border-base-content/10 rounded-xl px-3 py-2 rounded-tl-sm"
            >
              <!-- Stopped badge -->
              <div v-if="msg.isStopped" class="mb-2 flex items-center gap-1 text-xs text-warning">
                <span class="inline-flex items-center gap-1 bg-warning/10 border border-warning/20 rounded px-1.5 py-0.5">
                  <SvgIcon name="playerStop" size="10" class="text-warning" />
                  生成已停止
                </span>
              </div>
              <div
                class="prose prose-sm max-w-none text-base-content"
                v-html="renderMarkdown(msg.content)"
              />
            </div>
            <!-- Timestamp + copy -->
            <div class="flex items-center gap-2 mt-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
              <span v-if="msg.timestamp" class="text-[10px] text-base-content/30">{{ formatTime(msg.timestamp) }}</span>
              <button
                v-if="msg.content"
                class="text-[10px] text-base-content/30 hover:text-base-content/60 transition-colors"
                title="复制"
                @click="copyMessage(msg.content)"
              >
                <SvgIcon name="copy" size="10" />
              </button>
            </div>
          </div>
        </div>

        <!-- Reasoning block (collapsible) -->
        <div
          v-else-if="msg.kind === 'reasoning'"
          class="flex gap-2 w-full group"
        >
          <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-base-200/50 mt-0.5">
            <SvgIcon name="brain" size="14" class="text-base-content/40" />
          </div>
          <div class="max-w-[80%] min-w-0">
            <button
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-base-content/50 hover:text-base-content/70 bg-base-200/20 rounded-lg border border-base-content/5 hover:bg-base-200/40 transition-colors"
              @click="toggleReasoning(msg.id)"
            >
              <SvgIcon name="brain" size="10" />
              <span>思考过程</span>
              <SvgIcon
                :name="expandedReasoning.has(msg.id) ? 'chevronDown' : 'chevronRight'"
                size="10"
              />
            </button>
            <div
              v-if="expandedReasoning.has(msg.id)"
              class="mt-1 px-3 py-2 text-xs text-base-content/50 bg-base-200/20 rounded-lg border border-base-content/5 whitespace-pre-wrap"
            >
              {{ msg.text || msg.content || '' }}
            </div>
          </div>
        </div>

        <!-- Tool call -->
        <div
          v-else-if="msg.kind === 'tool_call'"
          class="flex gap-2 w-full"
        >
          <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-base-200/50 mt-0.5">
            <SvgIcon name="terminal" size="14" class="text-base-content/40" />
          </div>
          <div class="max-w-[80%] min-w-0">
            <ToolCallCard :tool="mapToolCall(msg)" />
          </div>
        </div>

        <!-- Tool result (show inline, collapsed) -->
        <div
          v-else-if="msg.kind === 'tool_result'"
          class="flex gap-2 w-full"
        >
          <div class="w-8 shrink-0" />
          <div class="max-w-[80%] min-w-0">
            <button
              v-if="msg.content"
              class="flex items-center gap-1.5 px-2.5 py-1 text-[10px] text-base-content/40 hover:text-base-content/60 bg-base-200/20 rounded-lg border border-base-content/5 transition-colors"
              @click="toggleToolResult(msg.id)"
            >
              <SvgIcon name="clipboard" size="10" />
              <span>工具结果</span>
              <SvgIcon
                :name="expandedToolResults.has(msg.id) ? 'chevronDown' : 'chevronRight'"
                size="10"
              />
            </button>
            <div
              v-if="expandedToolResults.has(msg.id)"
              class="mt-1 px-2.5 py-2 text-[11px] text-base-content/50 bg-base-200/20 rounded border border-base-content/5 whitespace-pre-wrap font-mono max-h-[200px] overflow-y-auto"
            >
              {{ msg.content }}
            </div>
          </div>
        </div>
      </template>

      <!-- Streaming indicator (cursor blink at the end) -->
      <div
        v-if="isStreaming"
        class="flex gap-2 w-full"
      >
        <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-primary/20 mt-0.5">
          <SvgIcon name="bot" size="14" class="text-primary animate-pulse" />
        </div>
        <div class="flex items-center gap-1 px-3 py-2 rounded-xl bg-base-200/30 border border-base-content/5">
          <span class="w-1.5 h-1.5 rounded-full bg-primary/60 animate-bounce" style="animation-delay: 0ms" />
          <span class="w-1.5 h-1.5 rounded-full bg-primary/60 animate-bounce" style="animation-delay: 150ms" />
          <span class="w-1.5 h-1.5 rounded-full bg-primary/60 animate-bounce" style="animation-delay: 300ms" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import ToolCallCard from '@/components/chat/ToolCallCard.vue';
import { renderMarkdown } from '@/composables/useMarkdownRenderer';
import type { ChatMessage, ToolCallMessage } from './types';

defineProps<{
  messages: ChatMessage[];
  isStreaming: boolean;
}>();

const containerRef = ref<HTMLDivElement | null>(null);
const expandedReasoning = reactive(new Set<string>());
const expandedToolResults = reactive(new Set<string>());

function toggleReasoning(id: string) {
  if (expandedReasoning.has(id)) {
    expandedReasoning.delete(id);
  } else {
    expandedReasoning.add(id);
  }
}

function toggleToolResult(id: string) {
  if (expandedToolResults.has(id)) {
    expandedToolResults.delete(id);
  } else {
    expandedToolResults.add(id);
  }
}

function mapToolCall(msg: ChatMessage) {
  const tc = msg as ToolCallMessage;
  const info = tc.toolCallInfo;
  return {
    name: info?.name ?? tc.name ?? '',
    args: info?.args,
    status: info?.status ?? (tc.callId ? 'completed' : 'running'),
    durationMs: info?.durationMs,
    emoji: info?.emoji,
    label: info?.label,
  };
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

async function copyMessage(content: string) {
  try {
    await navigator.clipboard.writeText(content);
  } catch {
    // Clipboard API may fail in Tauri; silently ignore
  }
}
</script>
