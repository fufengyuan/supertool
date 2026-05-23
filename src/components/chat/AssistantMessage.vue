<template>
  <div class="flex gap-2 w-full group">
    <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-primary/20">
      <SvgIcon name="bot" size="14" class="text-primary" />
    </div>
    <div class="max-w-[900px]">
      <!-- 思考过程（如果有）- 可折叠 -->
      <div 
        v-if="message.thinking" 
        class="mb-2 bg-base-200/30 rounded-lg px-3 py-2 text-xs text-base-content/50 italic border border-base-content/10 cursor-pointer"
        @click="toggleThinking"
      >
        <div class="flex items-center justify-between">
          <span>💭 思考过程</span>
          <SvgIcon :name="thinkingExpanded ? 'chevronDown' : 'chevronRight'" size="10" />
        </div>
        <div v-if="thinkingExpanded" class="mt-2 whitespace-pre-wrap">
          {{ message.thinking }}
        </div>
      </div>
      
      <!-- 气泡主体：有内容时才渲染 -->
      <div
        v-if="hasContent"
        class="bg-base-100 border border-base-300 rounded-xl px-3 py-2"
      >
        <!-- 已停止徽章 -->
        <div v-if="message.isStopped" class="mb-2 flex items-center gap-1 text-xs text-warning">
          <span class="inline-flex items-center gap-1 bg-warning/10 border border-warning/20 rounded px-1.5 py-0.5">
            <SvgIcon name="stop" size="10" class="text-warning" />
            已停止
          </span>
        </div>
        <!-- Markdown 渲染的消息内容 -->
        <div v-if="message.content" class="markdown-content text-sm text-base-content" v-html="renderMarkdown(message.content)"></div>
        
        <!-- 工具调用卡片 -->
        <div v-if="message.toolCalls && message.toolCalls.length > 0" class="space-y-1.5">
          <ToolCallCard
            v-for="(tool, tIdx) in message.toolCalls"
            :key="`${tool.name}-${tIdx}`"
            :tool="tool"
            :expanded="isToolExpanded(tIdx)"
            :icon="tool.isSubAgent ? 'bot' : getToolIcon(tool.name).icon"
            :title="tool.isSubAgent ? '子 Agent' : tool.name"
            :summary="tool.isSubAgent 
              ? (tool.args?.goal || tool.args?.task || tool.args?.prompt ? String(tool.args?.goal || tool.args?.task || tool.args?.prompt).slice(0, 100) + '...' : '执行任务')
              : (tool.name === 'todo' ? '待办任务' : formatArgsSummary(tool.args || {}))"
            :resultLabel="tool.isSubAgent ? '原始结果' : (tool.name === 'todo' ? '原始结果' : '结果')"
            :formatResult="(result: string) => tool.name === 'todo' ? `<pre class='whitespace-pre-wrap font-mono'>${result}</pre>` : formatToolResult(tool.name, result)"
            @toggle="toggleToolCall(tIdx)"
          />
        </div>
      </div>
      <!-- 时间戳和重试按钮 -->
      <div class="mt-1 flex items-center justify-between">
        <span v-if="message.timestamp" class="text-xs text-base-content/40">
          {{ formatTime(message.timestamp) }}
        </span>
        <!-- 错误消息重试按钮 -->
        <button 
          v-if="message.isError && message.retryContent"
          class="btn btn-ghost btn-xs text-xs text-error hover:bg-error/10"
          @click="retryMessage(message.retryContent)"
        >
          <SvgIcon name="refresh" size="10" />
          重试
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import ToolCallCard from './ToolCallCard.vue';

interface ToolCall {
  name: string;
  args?: Record<string, unknown>;
  result?: string;
  status?: string;
  durationMs?: number;
  isSubAgent?: boolean;
}

interface Message {
  role: string;
  content: string | null;
  thinking?: string;
  toolCalls?: ToolCall[];
  timestamp: number | null;
  isStopped?: boolean;
  isError?: boolean;
  retryContent?: string;
  toolName: string | null;
}

interface ToolIconInfo {
  icon: string;
  color: string;
}

const props = defineProps<{
  message: Message;
  messageIndex: number;
  formatTime: (ts: number) => string;
  renderMarkdown: (content: string) => string;
  getToolIcon: (name: string) => ToolIconInfo;
  formatArgsSummary: (args: Record<string, unknown>) => string;
  formatToolResult: (name: string, result: string) => string;
  formatTodoResult: (result: string) => string;
  isThinkingExpanded: (idx: number) => boolean;
  isToolCallExpanded: (key: string) => boolean;
  onToggleThinking: (idx: number) => void;
  onToggleToolCall: (key: string) => void;
  onRetry: (content: string) => void;
}>();

const thinkingExpanded = computed(() => props.isThinkingExpanded(props.messageIndex));

const hasContent = computed(() => 
  props.message.content || 
  (props.message.toolCalls && props.message.toolCalls.length > 0) || 
  props.message.isStopped
);

const isToolExpanded = (tIdx: number) => props.isToolCallExpanded(`${props.messageIndex}-${tIdx}`);

const toggleThinking = () => props.onToggleThinking(props.messageIndex);

const toggleToolCall = (tIdx: number) => props.onToggleToolCall(`${props.messageIndex}-${tIdx}`);

const retryMessage = (content: string) => props.onRetry(content);
</script>

<style scoped>
.markdown-content {
  line-height: 1.6;
  word-break: break-word;
}

.markdown-content :deep(p) {
  margin: 0.4em 0;
}

.markdown-content :deep(pre) {
  margin: 0.5em 0;
  padding: 0.75em;
  background: var(--fallback-b2, oklch(var(--b2) / 1));
  border-radius: 0.5em;
  overflow-x: auto;
}

.markdown-content :deep(code) {
  font-size: 0.85em;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin: 0.4em 0;
  padding-left: 1.5em;
}
</style>