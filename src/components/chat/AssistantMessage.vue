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
        <VueMarkdown v-if="message.content" :source="message.content" class="prose prose-sm max-w-none" :options="mdOptions" />
        
        <!-- 工具调用卡片 -->
        <div v-if="message.toolCalls && message.toolCalls.length > 0" class="space-y-1.5">
          <ToolCallCard
            v-for="(tool, tIdx) in message.toolCalls"
            :key="`${tool.name}-${tIdx}`"
            :tool="tool"
            :icon="tool.isSubAgent ? 'bot' : getToolIcon(tool.name).icon"
            :title="tool.isSubAgent ? '子 Agent' : tool.name"
            :summary="tool.isSubAgent 
              ? (tool.args?.goal || tool.args?.task || tool.args?.prompt ? String(tool.args?.goal || tool.args?.task || tool.args?.prompt).slice(0, 80) + '...' : '')
              : (tool.name === 'todo' ? formatTodoArgsSummary(tool.args || {}) : formatArgsSummary(tool.args || {}))"
          />
        </div>
      </div>
      <!-- 时间戳和操作按钮 -->
      <div class="mt-1 flex items-center justify-between">
        <span v-if="message.timestamp" class="text-xs text-base-content/40">
          {{ formatTime(message.timestamp) }}
        </span>
        <div class="flex items-center gap-1">
          <!-- 复制消息内容 -->
          <button
            v-if="message.content"
            class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
            :title="copied ? '已复制' : '复制消息'"
            @click="copyContent"
          >
            <SvgIcon :name="copied ? 'clipboardCheck' : 'clipboard'" size="12" :class="copied ? 'text-success' : ''" />
          </button>
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
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import VueMarkdown from 'vue-markdown-render';
import hljs from 'highlight.js';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import ToolCallCard from './ToolCallCard.vue';
import { formatTodoArgsSummary } from '@/composables/useToolFormatter';

const copied = ref(false);

async function copyContent() {
  if (!props.message.content) return;
  try {
    await navigator.clipboard.writeText(props.message.content);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch {
    // 降级：部分 Tauri 环境可能不支持 clipboard API
    const ta = document.createElement('textarea');
    ta.value = props.message.content;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  }
}

// markdown-it 配置（代码高亮）
const mdOptions = {
  html: true,
  linkify: true,
  typographer: true,
  breaks: true,
  highlight: (str: string, lang: string) => {
    if (lang && hljs.getLanguage(lang)) {
      return hljs.highlight(str, { language: lang }).value;
    }
    return hljs.highlightAuto(str).value;
  },
};

interface ToolCall {
  name: string;
  args?: Record<string, unknown>;
  result?: string;
  status?: string;
  durationMs?: number;
  isSubAgent?: boolean;
  label?: string; // Hermes API 返回的友好标签
  emoji?: string; // Hermes API 返回的 emoji
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
  getToolIcon: (name: string) => ToolIconInfo;
  formatArgsSummary: (args: Record<string, unknown>) => string;
  isThinkingExpanded: (idx: number) => boolean;
  onToggleThinking: (idx: number) => void;
  onRetry: (content: string) => void;
}>();

const thinkingExpanded = computed(() => props.isThinkingExpanded(props.messageIndex));

const hasContent = computed(() => 
  props.message.content || 
  (props.message.toolCalls && props.message.toolCalls.length > 0) || 
  props.message.isStopped
);

const toggleThinking = () => props.onToggleThinking(props.messageIndex);
const retryMessage = (content: string) => props.onRetry(content);
</script>