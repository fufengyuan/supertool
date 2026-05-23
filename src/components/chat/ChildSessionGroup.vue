<template>
  <div class="flex gap-2 w-full my-1">
    <!-- 子会话标识线 -->
    <div class="w-0.5 bg-info/40 rounded-full shrink-0 self-stretch"></div>
    <!-- 子会话折叠卡片 -->
    <div class="flex gap-2 w-full">
      <div class="flex h-6 w-6 items-center justify-center rounded-full bg-info/15 shrink-0 mt-0.5">
        <SvgIcon name="bot" size="12" class="text-info/80" />
      </div>
      <div class="max-w-[900px] flex-1">
        <!-- 折叠状态：显示摘要 -->
        <div 
          v-if="!expanded"
          class="bg-info/5 border border-info/15 rounded-lg px-2.5 py-1.5 cursor-pointer hover:bg-info/10 hover:border-info/25 transition-all group"
          @click="toggle"
        >
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <span class="text-[11px] text-info/80 font-medium shrink-0">子 Agent</span>
              <span class="text-[11px] text-base-content/40 shrink-0">{{ group.messageCount }}条</span>
              <span class="text-[11px] text-base-content/50 truncate">{{ previewText }}</span>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <span v-if="group.timestamp" class="text-[11px] text-base-content/35">{{ formatTime(group.timestamp) }}</span>
              <SvgIcon name="chevronRight" size="12" class="text-info/40 group-hover:text-info/60 transition-colors" />
            </div>
          </div>
        </div>
        <!-- 展开状态：显示完整消息 -->
        <div v-else class="animate-expand">
          <!-- 收起按钮 -->
          <div 
            class="bg-info/10 border border-info/20 rounded-lg px-2.5 py-1 cursor-pointer hover:bg-info/15 transition-colors mb-2 inline-flex items-center gap-1.5"
            @click="toggle"
          >
            <SvgIcon name="chevronDown" size="12" class="text-info/60" />
            <span class="text-[11px] text-info/80 font-medium">子 Agent</span>
            <span class="text-[11px] text-base-content/40">{{ group.messageCount }}条消息</span>
            <span v-if="group.timestamp" class="text-[11px] text-base-content/35">{{ formatTime(group.timestamp) }}</span>
          </div>
          <!-- 子会话消息列表 - 紧凑样式 -->
          <div class="space-y-1.5 pl-1 border-l-2 border-info/20">
            <div v-for="(msg, idx) in group.messages" :key="`${group.sessionId}-${idx}`" class="flex gap-2 items-start">
              <div class="flex h-5 w-5 items-center justify-center rounded-full shrink-0" :class="msg.role === 'user' ? 'bg-info/10' : 'bg-success/10'">
                <SvgIcon :name="msg.role === 'user' ? 'user' : 'bot'" size="10" :class="msg.role === 'user' ? 'text-info/70' : 'text-success/70'" />
              </div>
              <div class="flex-1 min-w-0">
                <div v-if="msg.role === 'user'" class="bg-info/5 border border-info/10 rounded-md px-2 py-1">
                  <VueMarkdown v-if="msg.content" :source="msg.content || ''" class="markdown-content text-xs text-base-content/80" :options="mdOptions" />
                </div>
                <div v-else class="bg-success/5 border border-success/10 rounded-md px-2 py-1">
                  <VueMarkdown v-if="msg.content" :source="msg.content" class="markdown-content text-xs text-base-content/80" :options="mdOptions" />
                  <!-- 工具调用显示 -->
                  <div v-if="msg.toolCalls && msg.toolCalls.length > 0" class="mt-1.5 space-y-1">
                    <ToolCallCard
                      v-for="(tc, tcIdx) in msg.toolCalls"
                      :key="tcIdx"
                      :tool="tc"
                      :expanded="isToolExpanded(idx, tcIdx)"
                      :icon="tc.isSubAgent ? 'bot' : 'tool'"
                      :title="tc.isSubAgent ? '子 Agent' : tc.name"
                      :summary="tc.isSubAgent 
                        ? (tc.args?.goal || tc.args?.task || tc.args?.prompt ? String(tc.args?.goal || tc.args?.task || tc.args?.prompt).slice(0, 100) + '...' : '执行任务')
                        : formatArgsSummary(tc.args || {})"
                      :formatResult="(r: string) => renderMarkdownSimple(r)"
                      @toggle="toggleToolCall(`${idx}-${tcIdx}`)"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
          <!-- 继续对话输入框 -->
          <div class="mt-2 pl-1 border-l-2 border-info/20">
            <div class="flex gap-1.5 items-center">
              <input
                v-model="continueInput"
                type="text"
                class="input input-xs input-bordered w-full bg-base-100 text-xs"
                placeholder="继续对话..."
                @keyup.enter="sendContinueMessage"
              />
              <button
                class="btn btn-xs btn-info gap-1"
                :disabled="!continueInput.trim()"
                @click="sendContinueMessage"
              >
                <SvgIcon name="send" size="10" />
              </button>
            </div>
          </div>
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

// 简单的 markdown 渲染函数（用于 ToolCallCard）
import MarkdownIt from 'markdown-it';
const md = new MarkdownIt(mdOptions);
const renderMarkdownSimple = (text: string): string => md.render(text);

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
  content?: string | null;
  toolCalls?: ToolCall[];
}

interface ChildSessionGroup {
  type: 'childSessionGroup';
  sessionId: string;
  messages: Message[];
  preview: string;
  messageCount: number;
  timestamp: number;
}

const props = defineProps<{
  group: ChildSessionGroup;
  isExpanded?: boolean;
  formatTime: (ts: number) => string;
}>();

const emit = defineEmits<{
  (e: 'toggle', sessionId: string): void;
  (e: 'continue', sessionId: string, message: string): void;
}>();

const expanded = computed(() => props.isExpanded || false);

// 继续对话输入
const continueInput = ref('');

const sendContinueMessage = () => {
  if (continueInput.value.trim()) {
    emit('continue', props.group.sessionId, continueInput.value.trim());
    continueInput.value = '';
  }
};

// 工具调用展开状态 (key: `${msgIdx}-${tcIdx}`)
const expandedTools = ref(new Set<string>());

const toggleToolCall = (key: string) => {
  if (expandedTools.value.has(key)) {
    expandedTools.value.delete(key);
  } else {
    expandedTools.value.add(key);
  }
};

const isToolExpanded = (msgIdx: number, tcIdx: number): boolean => {
  return expandedTools.value.has(`${msgIdx}-${tcIdx}`);
};

// 工具摘要生成
const formatArgsSummary = (args: Record<string, unknown>): string => {
  const keys = Object.keys(args);
  if (keys.length === 0) return '';
  const firstKey = keys[0];
  const value = args[firstKey];
  if (typeof value === 'string') {
    return value.length > 50 ? value.slice(0, 50) + '...' : value;
  }
  return `${firstKey}: ${JSON.stringify(value).slice(0, 30)}`;
};

const previewText = computed(() => {
  const text = props.group.preview;
  return text.length > 50 ? text.slice(0, 50) + '...' : text;
});

const toggle = () => emit('toggle', props.group.sessionId);
</script>

<style scoped>
.animate-expand {
  animation: expandIn 0.2s ease-out;
}

@keyframes expandIn {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.markdown-content {
  line-height: 1.5;
  word-break: break-word;
}

.markdown-content :deep(p) {
  margin: 0.3em 0;
}

.markdown-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5em 0;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid base-content/20;
  padding: 0.25em 0.5em;
  text-align: left;
}

.markdown-content :deep(th) {
  background: base-200;
  font-weight: bold;
}

.markdown-content :deep(thead) {
  border-bottom: 2px solid base-content/30;
}
</style>