<template>
  <div class="flex h-full">
    <!-- 左侧会话列表 -->
    <div class="w-64 border-r border-base-content/10 flex flex-col bg-base-100">
      <!-- 会话列表头部 -->
      <div class="flex items-center justify-between px-3 py-2 border-b border-base-content/10">
        <span class="text-sm font-semibold text-base-content">会话</span>
        <button class="btn btn-ghost btn-xs" @click="refreshSessions" :disabled="loadingSessions">
          <SvgIcon name="refresh" size="12" :class="{ 'animate-spin': loadingSessions }" />
        </button>
      </div>

      <!-- 新会话按钮 -->
      <div class="px-2 py-2">
        <button class="btn btn-primary btn-sm w-full gap-1.5" @click="startNewChat" title="快捷键: Cmd+K">
          <SvgIcon name="plus" size="14" />
          新对话
        </button>
      </div>

      <!-- 会话列表 -->
      <div class="flex-1 overflow-y-auto">
        <div v-if="loadingSessions" class="flex items-center justify-center py-8">
          <SvgIcon name="refresh" size="16" class="animate-spin text-base-content/40" />
        </div>
        <div v-else-if="sessions.length === 0" class="flex flex-col items-center justify-center py-8 text-center">
          <SvgIcon name="chat" size="24" class="text-base-content/30" />
          <p class="mt-2 text-xs text-base-content/50">暂无会话</p>
        </div>
        <div v-else class="flex flex-col gap-1 px-2 py-1">
          <div
            v-for="session in sessions"
            :key="session.id"
            class="group flex items-center gap-2 px-2 py-1.5 rounded-lg cursor-pointer transition-colors"
            :class="currentSessionId === session.id ? 'bg-primary/10 text-primary' : 'hover:bg-base-200'"
            @click="selectSession(session)"
          >
            <SvgIcon :name="sourceIcon(session.source)" size="14" class="shrink-0" />
            <div class="flex flex-col min-w-0 flex-1">
              <span class="truncate text-xs font-medium">{{ session.title || session.preview || '新会话' }}</span>
              <span class="truncate text-xs text-base-content/50">{{ formatTime(session.lastActive || session.startedAt) }}</span>
            </div>
            <span class="text-xs text-base-content/40 shrink-0">{{ session.messageCount }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧聊天区域 -->
    <div class="flex-1 flex flex-col">
      <!-- 聊天头部 -->
      <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 bg-base-100">
        <div class="flex items-center gap-2 flex-1">
          <button class="btn btn-ghost btn-xs btn-circle" @click="router.back()" title="返回">
            <SvgIcon name="arrowLeft" size="14" />
          </button>
          <SvgIcon name="bot" size="16" class="text-primary" />
          <!-- 标题显示/编辑 -->
          <template v-if="isEditingTitle">
            <input
              ref="titleInputRef"
              v-model="editingTitle"
              class="input input-sm input-bordered w-48 text-sm"
              placeholder="输入标题..."
              @keydown.enter.exact="saveTitle"
              @keydown.escape="cancelEditTitle"
              @blur="saveTitle"
            />
          </template>
          <template v-else>
            <span class="text-sm font-semibold text-base-content cursor-pointer hover:opacity-80" @click="startEditTitle">
              {{ currentSession?.title || '新对话' }}
            </span>
            <button v-if="currentSession" class="btn btn-ghost btn-xs btn-square" @click="startEditTitle">
              <SvgIcon name="edit" size="12" />
            </button>
          </template>
          <span v-if="currentSession" class="badge badge-ghost badge-xs">
            {{ currentSession.model }}
          </span>
          <!-- 会话统计 -->
          <span v-if="messages.length > 0" class="text-xs text-base-content/40">
            {{ sessionStats.totalMessages }} 条消息
          </span>
        </div>
        <div class="flex items-center gap-2">
          <button v-if="isStreaming" class="btn btn-error btn-xs gap-1" @click="abortChat">
            <SvgIcon name="stop" size="12" />
            停止
          </button>
          <button v-if="currentSession && messages.length > 0" class="btn btn-ghost btn-xs" @click="exportSession" title="导出 (Cmd+S)">
            <SvgIcon name="download" size="12" />
          </button>
          <button v-if="currentSession" class="btn btn-ghost btn-xs" @click="deleteCurrentSession" title="删除">
            <SvgIcon name="trash" size="12" />
          </button>
          <!-- 搜索按钮 -->
          <div v-if="messages.length > 0" class="relative">
            <input
              v-model="searchQuery"
              type="text"
              class="input input-xs input-bordered w-20 focus:w-40 transition-all"
              placeholder="搜索..."
            />
            <button
              v-if="searchQuery"
              class="btn btn-ghost btn-xs btn-square absolute right-0"
              @click="clearSearch"
            >
              <SvgIcon name="close" size="10" />
            </button>
          </div>
        </div>
      </div>

      <!-- 消息列表 -->
      <div ref="messagesContainer" class="flex-1 overflow-y-auto px-4 py-4 space-y-3">
        <!-- 加载消息状态 - 骨架屏 -->
        <div v-if="loadingMessages" class="space-y-3">
          <div class="flex gap-3">
            <div class="h-8 w-8 rounded-full bg-base-200 shrink-0 animate-pulse"></div>
            <div class="flex-1 space-y-2">
              <div class="h-4 bg-base-200 rounded w-3/4 animate-pulse"></div>
              <div class="h-4 bg-base-200 rounded w-1/2 animate-pulse"></div>
            </div>
          </div>
          <div class="flex gap-3">
            <div class="h-8 w-8 rounded-full bg-primary/20 shrink-0 animate-pulse"></div>
            <div class="flex-1 space-y-2">
              <div class="h-4 bg-primary/10 rounded w-full animate-pulse"></div>
              <div class="h-4 bg-primary/10 rounded w-2/3 animate-pulse"></div>
            </div>
          </div>
        </div>

        <!-- 消息列表 -->
        <template v-else-if="messages.length > 0">
          <div v-for="(msg, idx) in (searchQuery ? filteredMessages : messages)" :key="idx" class="flex gap-3">
            <!-- 用户消息 -->
            <div v-if="msg.role === 'user'" class="flex gap-3 w-full group">
              <div class="flex h-8 w-8 items-center justify-center rounded-full bg-base-200 shrink-0">
                <SvgIcon name="user" size="14" class="text-base-content/60" />
              </div>
              <div class="flex-1">
                <div class="bg-base-200 rounded-xl px-3 py-2">
                  <!-- 搜索时高亮显示 -->
                  <p v-if="searchQuery" class="text-sm text-base-content whitespace-pre-wrap" v-html="highlightText(msg.content, searchQuery)"></p>
                  <p v-else class="text-sm text-base-content whitespace-pre-wrap">{{ msg.content }}</p>
                </div>
                <!-- 消息时间和操作按钮 -->
                <div class="flex items-center gap-2 mt-1">
                  <span v-if="msg.timestamp" class="text-xs text-base-content/40">
                    {{ formatMessageTime(msg.timestamp) }}
                  </span>
                  <!-- 复制按钮 -->
                  <button
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
                    @click="copyMessageContent(msg.content)"
                    title="复制"
                  >
                    <SvgIcon name="copy" size="12" />
                  </button>
                  <!-- 引用按钮 -->
                  <button
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
                    @click="quoteMessage(msg.content)"
                    title="引用"
                  >
                    <SvgIcon name="quote" size="12" />
                  </button>
                  <!-- 编辑按钮 -->
                  <button
                    v-if="!isStreaming"
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
                    @click="editUserMessage(idx)"
                    title="编辑并重新发送"
                  >
                    <SvgIcon name="edit" size="12" />
                  </button>
                </div>
              </div>
            </div>

            <!-- Assistant 消息 -->
            <div v-else class="flex gap-3 w-full group">
              <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20 shrink-0">
                <SvgIcon name="bot" size="14" class="text-primary" />
              </div>
              <div class="flex-1">
                <div class="bg-primary/5 border border-primary/10 rounded-xl px-3 py-2">
                  <!-- Markdown 渲染的消息内容 -->
                  <div class="markdown-content text-sm text-base-content" v-html="renderMarkdown(msg.content)"></div>
                  <!-- 工具调用显示 -->
                  <div v-if="msg.toolCalls && msg.toolCalls.length > 0" class="mt-2 space-y-1">
                    <div v-for="(tool, idx) in msg.toolCalls" :key="`${tool.name}-${idx}`" class="flex items-center gap-2 text-xs">
                      <!-- 子 agent 使用不同图标 -->
                      <SvgIcon v-if="tool.name === 'delegate_task'" name="bot" size="12" class="text-info" />
                      <SvgIcon v-else name="tool" size="12" class="text-warning" />
                      <!-- 子 agent 显示不同文字 -->
                      <span v-if="tool.name === 'delegate_task'" class="text-info">子 Agent</span>
                      <span v-else class="text-base-content/70">{{ tool.name }}</span>
                      <span class="text-base-content/40">{{ tool.durationMs }}ms</span>
                    </div>
                  </div>
                </div>
                <!-- 消息时间和操作按钮 -->
                <div class="flex items-center gap-2 mt-1 ml-1">
                  <span v-if="msg.timestamp" class="text-xs text-base-content/40">
                    {{ formatMessageTime(msg.timestamp) }}
                  </span>
                  <!-- 复制按钮 -->
                  <button
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
                    @click="copyMessageContent(msg.content)"
                    title="复制"
                  >
                    <SvgIcon name="copy" size="12" />
                  </button>
                  <!-- 引用按钮 -->
                  <button
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
                    @click="quoteMessage(msg.content)"
                    title="引用"
                  >
                    <SvgIcon name="quote" size="12" />
                  </button>
                  <!-- 删除按钮 -->
                  <button
                    v-if="!isStreaming"
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity text-error"
                    @click="deleteMessage(idx)"
                    title="删除"
                  >
                    <SvgIcon name="trash" size="12" />
                  </button>
                  <!-- 重试按钮 - 仅对错误消息显示 -->
                  <button
                    v-if="msg.isError && msg.retryContent"
                    class="btn btn-xs btn-ghost text-error"
                    @click="retryMessage(msg.retryContent!)"
                    :disabled="isStreaming"
                  >
                    <SvgIcon name="refresh" size="12" class="mr-1" />
                    重试
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 思考动画 - 显示在流式输出前 -->
          <div v-if="isStreaming && thinkingText && !streamingText" class="flex gap-3">
            <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20 shrink-0">
              <SvgIcon name="bot" size="14" class="text-primary animate-pulse" />
            </div>
            <div class="flex-1 bg-primary/5 border border-primary/10 rounded-xl px-3 py-2">
              <p class="text-sm text-base-content/60 animate-pulse">{{ thinkingText }}</p>
            </div>
          </div>

          <!-- 流式输出中的临时消息 -->
          <div v-if="isStreaming && streamingText" class="flex gap-3">
            <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20 shrink-0">
              <SvgIcon name="bot" size="14" class="text-primary animate-pulse" />
            </div>
            <div class="flex-1 bg-primary/5 border border-primary/10 rounded-xl px-3 py-2">
              <!-- 流式输出实时渲染 markdown -->
              <div class="markdown-content text-sm text-base-content" v-html="renderMarkdown(streamingText)"></div>
            </div>
          </div>
        </template>

        <!-- 空状态 -->
        <div v-else class="flex flex-col items-center justify-center py-16 text-center">
          <SvgIcon name="chat" size="32" class="text-base-content/30" />
          <p class="mt-2 text-sm text-base-content/50">开始对话</p>
          <p class="text-xs text-base-content/40">输入消息与 Hermes Agent 交流</p>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="border-t border-base-content/10 px-4 py-3 bg-base-100">
        <!-- Hermes 未安装提示 -->
        <div v-if="!hermesAvailable" class="flex items-center justify-center gap-2 py-2">
          <SvgIcon name="warning" size="14" class="text-warning" />
          <span class="text-xs text-base-content/60">Hermes 未安装或不可用</span>
          <button class="btn btn-ghost btn-xs" @click="checkHermes">检测</button>
        </div>

        <!-- 正常输入 -->
        <div v-else class="space-y-2">
          <!-- 模型选择、工具集和引用消息显示 -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <!-- 模型选择 -->
              <select
                v-model="selectedModel"
                class="select select-bordered select-xs w-auto"
                :disabled="isStreaming"
              >
                <option value="">默认模型</option>
                <option v-for="model in availableModels" :key="model" :value="model">{{ model }}</option>
              </select>
              <!-- 工具集选择 -->
              <div class="flex items-center gap-1">
                <button
                  v-for="toolset in availableToolsets"
                  :key="toolset"
                  class="btn btn-xs"
                  :class="selectedToolsets.includes(toolset) ? 'btn-primary' : 'btn-ghost'"
                  @click="toggleToolset(toolset)"
                  :disabled="isStreaming"
                  :title="toolsetLabels[toolset]"
                >
                  {{ toolsetLabels[toolset] }}
                </button>
              </div>
            </div>
            <!-- 引用消息提示 -->
            <div v-if="quotedMessage" class="flex items-center gap-1 text-xs text-base-content/60">
              <SvgIcon name="quote" size="12" />
              <span class="truncate max-w-32">{{ quotedMessage.slice(0, 30) }}...</span>
              <button class="btn btn-ghost btn-xs btn-square" @click="quotedMessage = null">
                <SvgIcon name="close" size="10" />
              </button>
            </div>
          </div>
          <!-- 输入框 -->
          <div class="flex gap-2">
            <textarea
              ref="inputRef"
              v-model="inputText"
              class="textarea textarea-bordered w-full resize-none text-sm"
              style="min-height: 52px; max-height: 200px;"
              :placeholder="quotedMessage ? '回复引用的消息...' : '输入消息...'"
              :disabled="isStreaming"
              @keydown.enter.exact.prevent="sendMessage"
            ></textarea>
            <!-- 操作按钮组 -->
            <div class="flex items-center gap-1 self-end">
              <!-- 撤回按钮 -->
              <button
                v-if="messages.length > 0 && !isStreaming"
                class="btn btn-ghost btn-sm"
                @click="undoLastMessage"
                title="撤回最后一条消息"
              >
                <SvgIcon name="undo" size="14" />
              </button>
              <!-- 发送按钮 -->
              <button
                class="btn btn-primary"
                :disabled="!inputText.trim() || isStreaming"
                @click="sendMessage"
              >
                <SvgIcon v-if="isStreaming" name="refresh" size="14" class="animate-spin" />
                <SvgIcon v-else name="send" size="14" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/core';
import javascript from 'highlight.js/lib/languages/javascript';
import python from 'highlight.js/lib/languages/python';
import rust from 'highlight.js/lib/languages/rust';
import bash from 'highlight.js/lib/languages/bash';
import json from 'highlight.js/lib/languages/json';
import sql from 'highlight.js/lib/languages/sql';
import typescript from 'highlight.js/lib/languages/typescript';
import SvgIcon from '@/components/ui/SvgIcon.vue';

// 注册代码高亮语言
hljs.registerLanguage('javascript', javascript);
hljs.registerLanguage('python', python);
hljs.registerLanguage('rust', rust);
hljs.registerLanguage('bash', bash);
hljs.registerLanguage('json', json);
hljs.registerLanguage('sql', sql);
hljs.registerLanguage('typescript', typescript);

// 配置 marked 使用 highlight.js
marked.setOptions({
  highlight: function(code: string, lang: string) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(code, { language: lang }).value;
      } catch {}
    }
    return hljs.highlightAuto(code).value;
  },
  breaks: true, // 支持 GFM 换行
  gfm: true, // GitHub Flavored Markdown
});

const route = useRoute();
const router = useRouter();

interface Session {
  id: string;
  title: string | null;
  model: string;
  source: string;
  startedAt: number;
  endedAt: number | null;
  messageCount: number;
  preview: string;
  lastActive?: number; // 最近活跃时间（可选）
}

interface Message {
  role: string;
  content: string | null;
  timestamp: number | null;
  toolName: string | null;
  toolCalls?: { name: string; durationMs: number }[];
  isError?: boolean; // 是否是错误消息
  retryContent?: string; // 用于重试的原始消息内容
  tokens?: { input: number; output: number }; // token 使用量
}

// Raw message from backend (matches MessageInfo in Rust)
interface RawMessage {
  role: string;
  content: string | null;
  timestamp: number | null;
  toolName: string | null;
}

// State
const sessions = ref<Session[]>([]);
const currentSessionId = ref<string | null>(null);
const currentSession = ref<Session | null>(null);
const messages = ref<Message[]>([]);
const inputText = ref('');
const loadingSessions = ref(false);
const loadingMessages = ref(false);
const isStreaming = ref(false);
const streamingText = ref('');
const thinkingText = ref(''); // 思考动画文本
const hermesAvailable = ref(false);

// 模型选择
const selectedModel = ref('');
const availableModels = ref([
  'glm-4',
  'glm-4-flash',
  'glm-5',
  'qwen-plus',
  'qwen-max',
  'claude-sonnet-4',
  'claude-opus-4',
]);
const quotedMessage = ref<string | null>(null); // 引用的消息

// 工具集选择
const availableToolsets = ref([
  'web',       // 网络搜索
  'terminal',  // 终端命令
  'file',      // 文件操作
  'browser',   // 浏览器操作
  'vision',    // 图像分析
]);
const selectedToolsets = ref<string[]>([]);
const toolsetLabels: Record<string, string> = {
  web: '搜索',
  terminal: '终端',
  file: '文件',
  browser: '浏览器',
  vision: '图像',
};

// 切换工具集选择
const toggleToolset = (toolset: string) => {
  const index = selectedToolsets.value.indexOf(toolset);
  if (index === -1) {
    selectedToolsets.value.push(toolset);
  } else {
    selectedToolsets.value.splice(index, 1);
  }
};

// 搜索状态
const searchQuery = ref('');
const filteredMessages = ref<Message[]>([]);

// Refs
const messagesContainer = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLTextAreaElement | null>(null);
const titleInputRef = ref<HTMLInputElement | null>(null);

// 标题编辑状态
const isEditingTitle = ref(false);
const editingTitle = ref('');

// 复制代码功能（全局函数）
const copyCode = (codeId: string) => {
  const codeElement = document.getElementById(codeId);
  if (codeElement) {
    const text = codeElement.textContent || '';
    navigator.clipboard.writeText(text).then(() => {
      // 显示复制成功提示
      const btn = codeElement.closest('.code-block-wrapper')?.querySelector('.copy-btn');
      if (btn) {
        btn.classList.add('copied');
        setTimeout(() => btn.classList.remove('copied'), 2000);
      }
    });
  }
};
// 挂载到 window 以便 onclick 调用
if (typeof window !== 'undefined') {
  (window as any).copyCode = copyCode;
}

// Event listeners
let unlistenDelta: UnlistenFn | null = null;
let unlistenToolStart: UnlistenFn | null = null;
let unlistenToolComplete: UnlistenFn | null = null;
let unlistenThinking: UnlistenFn | null = null;
let unlistenError: UnlistenFn | null = null;
let currentToolCalls: { name: string; durationMs: number }[] = [];

// 自动调整输入框高度
const adjustTextareaHeight = () => {
  if (inputRef.value) {
    inputRef.value.style.height = 'auto';
    // 限制最大高度为 200px（约 8 行）
    const maxHeight = 200;
    const newHeight = Math.min(inputRef.value.scrollHeight, maxHeight);
    inputRef.value.style.height = `${newHeight}px`;
  }
};

// Computed
const sourceIcon = (source: string) => {
  const icons: Record<string, string> = {
    cli: 'terminal',
    feishu: 'message',
    telegram: 'message',
    discord: 'message',
    slack: 'message',
    cron: 'clock',
  };
  return icons[source] || 'chat';
};

// Markdown 渲染函数 - 添加代码块复制按钮
const renderMarkdown = (text: string | null): string => {
  if (!text) return '';
  try {
    // 自定义渲染器，为代码块添加复制按钮
    const renderer = new marked.Renderer();
    renderer.code = function(code: string, language: string | undefined) {
      const lang = language || 'plaintext';
      const highlighted = lang && hljs.getLanguage(lang) 
        ? hljs.highlight(code, { language: lang }).value 
        : hljs.highlightAuto(code).value;
      
      // 生成唯一 ID 用于复制功能
      const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;
      
      return `<div class="code-block-wrapper">
        <div class="code-header">
          <span class="code-lang">${lang}</span>
          <button class="copy-btn" onclick="copyCode('${codeId}')" title="复制代码">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
          </button>
        </div>
        <pre><code id="${codeId}" class="hljs">${highlighted}</code></pre>
      </div>`;
    };
    
    marked.setOptions({ renderer });
    const html = marked.parse(text) as string;
    return DOMPurify.sanitize(html, {
      ADD_ATTR: ['target', 'onclick', 'id', 'title'],
      ADD_TAGS: ['button', 'svg', 'rect', 'path'],
    });
  } catch {
    return text;
  }
};

// 格式化消息时间（显示具体时间）
const formatMessageTime = (ts: number | null): string => {
  if (!ts) return '';
  const date = new Date(ts * 1000);
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
};

const formatTime = (ts: number | null) => {
  if (!ts) return '';
  const date = new Date(ts * 1000);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  if (days === 0) {
    const hours = Math.floor(diff / (1000 * 60 * 60));
    if (hours === 0) {
      const mins = Math.floor(diff / (1000 * 60));
      return mins <= 1 ? '刚刚' : `${mins}分钟前`;
    }
    return `${hours}小时前`;
  } else if (days === 1) {
    return '昨天';
  } else if (days < 7) {
    return `${days}天前`;
  }
  return date.toLocaleDateString();
};

// Methods
const refreshSessions = async () => {
  loadingSessions.value = true;
  try {
    const result = await invoke<{ sessions: Session[]; total: number }>('agent_list_sessions', { limit: 50 });
    // 按 lastActive 降序排序（最近活跃的在前）
    sessions.value = result.sessions.sort((a, b) => {
      const aTime = a.lastActive || a.startedAt;
      const bTime = b.lastActive || b.startedAt;
      return bTime - aTime;
    });
  } catch (e) {
    console.error('Failed to list sessions:', e);
  }
  loadingSessions.value = false;
};

const selectSession = async (session: Session) => {
  currentSessionId.value = session.id;
  currentSession.value = session;
  loadingMessages.value = true;
  messages.value = [];

  try {
    const result = await invoke<{ session_id: string; messages: RawMessage[] }>('agent_get_session', {
      sessionId: session.id,
    });
    // 直接使用返回的消息，无需过滤
    messages.value = result.messages.map((m: RawMessage) => ({
      role: m.role,
      content: m.content,
      timestamp: m.timestamp,
      toolName: m.toolName,
      toolCalls: [],
    }));
  } catch (e) {
    console.error('Failed to get session:', e);
  }

  loadingMessages.value = false;
  scrollToBottom();
};

const startNewChat = () => {
  currentSessionId.value = null;
  currentSession.value = null;
  messages.value = [];
  inputText.value = '';
  streamingText.value = '';
  thinkingText.value = '';
  isStreaming.value = false;
};

// 自动生成会话标题（基于第一条用户消息）
const generateSessionTitle = (firstMessage: string): string => {
  // 截取前30个字符作为标题
  let title = firstMessage.trim().slice(0, 30);
  // 如果截断，添加省略号
  if (firstMessage.trim().length > 30) {
    title += '...';
  }
  return title;
};

const sendMessage = async () => {
  if (!inputText.value.trim() || isStreaming.value) return;

  // 构建消息（包含引用）
  let text = inputText.value.trim();
  if (quotedMessage.value) {
    text = `> ${quotedMessage.value}\n\n${text}`;
    quotedMessage.value = null; // 清除引用
  }
  inputText.value = '';

  // 添加用户消息
  messages.value.push({
    role: 'user',
    content: text,
    timestamp: Date.now() / 1000,
    toolName: null,
  });
  scrollToBottom();

  // 开始流式输出
  isStreaming.value = true;
  streamingText.value = '';
  thinkingText.value = '';
  currentToolCalls = [];

  try {
    // 使用选择的模型（如果有）
    const modelToUse = selectedModel.value || null;
    // 使用选择的工具集（如果有）
    const toolsetsToUse = selectedToolsets.value.length > 0 ? selectedToolsets.value : null;
    
    const result = await invoke<{ response: string; session_id: string; message_count: number }>('agent_chat', {
      message: text,
      sessionId: currentSessionId.value,
      model: modelToUse,
      toolsets: toolsetsToUse,
    });

    // 更新 session ID
    if (result.session_id && !currentSessionId.value) {
      currentSessionId.value = result.session_id;
      // 自动生成标题（如果是第一条消息）
      const autoTitle = generateSessionTitle(text);
      // 尝试重命名会话
      try {
        await invoke('agent_rename_session', {
          sessionId: result.session_id,
          newTitle: autoTitle,
        });
        // 更新本地 session 信息
        currentSession.value = {
          id: result.session_id,
          title: autoTitle,
          model: modelToUse || 'unknown',
          startedAt: Date.now() / 1000,
          lastActive: Date.now() / 1000,
        };
      } catch (e) {
        console.warn('Auto-title failed:', e);
      }
      // 刷新会话列表
      refreshSessions();
    }

    // 添加 assistant 消息 - 使用流式文本或完整响应
    const finalContent = streamingText.value || result.response;
    if (finalContent && !messages.value.some(m => m.role === 'assistant' && m.content === finalContent)) {
      messages.value.push({
        role: 'assistant',
        content: finalContent,
        timestamp: Date.now() / 1000,
        toolName: null,
        toolCalls: currentToolCalls,
      });
    }
    streamingText.value = '';
    thinkingText.value = '';
  } catch (e) {
    console.error('Chat error:', e);
    // 添加错误消息，保存原始内容以便重试
    messages.value.push({
      role: 'assistant',
      content: `错误: ${e}`,
      timestamp: Date.now() / 1000,
      toolName: null,
      isError: true,
      retryContent: text, // 保存原始消息用于重试
    });
  }

  isStreaming.value = false;
  scrollToBottom();
  // 自动聚焦输入框，方便继续输入
  inputRef.value?.focus();
};

const abortChat = async () => {
  try {
    await invoke('agent_abort_chat');
    isStreaming.value = false;
    streamingText.value = '';
    thinkingText.value = '';
  } catch (e) {
    console.error('Abort error:', e);
  }
};

// 重试发送消息
const retryMessage = async (retryContent: string) => {
  if (isStreaming.value || !retryContent.trim()) return;

  // 移除最后一条错误消息
  if (messages.value.length > 0 && messages.value[messages.value.length - 1].isError) {
    messages.value.pop();
  }

  // 设置输入文本并重新发送
  inputText.value = retryContent;
  await sendMessage();
};

// 复制消息内容
const copyMessageContent = async (content: string | null) => {
  if (!content) return;
  try {
    await navigator.clipboard.writeText(content);
    // 可选：显示复制成功提示（用 toast 或临时状态）
  } catch (e) {
    console.error('Copy failed:', e);
  }
};

// 高亮搜索匹配文本
const highlightText = (text: string | null, query: string): string => {
  if (!text || !query.trim()) return text || '';
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(escapedQuery, 'gi');
  return text.replace(regex, '<mark class="bg-warning/30 text-inherit px-0.5 rounded">$&</mark>');
};

// 计算会话统计
const sessionStats = computed(() => {
  const userMessages = messages.value.filter(m => m.role === 'user');
  const assistantMessages = messages.value.filter(m => m.role === 'assistant');
  
  const totalInputTokens = messages.value.reduce((sum, m) => sum + (m.tokens?.input || 0), 0);
  const totalOutputTokens = messages.value.reduce((sum, m) => sum + (m.tokens?.output || 0), 0);
  
  return {
    userCount: userMessages.length,
    assistantCount: assistantMessages.length,
    totalMessages: messages.value.length,
    totalTokens: totalInputTokens + totalOutputTokens,
    inputTokens: totalInputTokens,
    outputTokens: totalOutputTokens,
  };
});

// 引用消息
const quoteMessage = (content: string | null) => {
  if (!content) return;
  quotedMessage.value = content.slice(0, 200); // 限制引用长度
  inputRef.value?.focus();
};

// 搜索消息
const searchMessages = () => {
  if (!searchQuery.value.trim()) {
    filteredMessages.value = messages.value;
    return;
  }
  const query = searchQuery.value.toLowerCase();
  filteredMessages.value = messages.value.filter(msg => 
    msg.content?.toLowerCase().includes(query)
  );
};

// 清除搜索
const clearSearch = () => {
  searchQuery.value = '';
  filteredMessages.value = messages.value;
};

// 编辑用户消息并重新发送
const editUserMessage = (msgIndex: number) => {
  if (isStreaming.value) return;
  
  const msg = messages.value[msgIndex];
  if (!msg || msg.role !== 'user' || !msg.content) return;
  
  // 删除该消息及其后所有消息
  messages.value = messages.value.slice(0, msgIndex);
  
  // 设置输入框内容
  inputText.value = msg.content;
  inputRef.value?.focus();
};

// 全局快捷键处理
const handleGlobalKeydown = (e: KeyboardEvent) => {
  // Cmd/Ctrl + K: 新对话
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault();
    startNewChat();
    return;
  }
  
  // Cmd/Ctrl + S: 保存/导出当前会话
  if ((e.metaKey || e.ctrlKey) && e.key === 's') {
    e.preventDefault();
    exportSession();
    return;
  }
  
  // Escape: 如果正在编辑标题，取消编辑
  if (e.key === 'Escape' && isEditingTitle.value) {
    cancelEditTitle();
    return;
  }
  
  // Escape: 如果正在流式输出，停止
  if (e.key === 'Escape' && isStreaming.value) {
    abortChat();
    return;
  }
};

// 导出会话为 Markdown
const exportSession = () => {
  if (messages.value.length === 0) return;
  
  const title = currentSession.value?.title || '新对话';
  const timestamp = new Date().toISOString().split('T')[0];
  
  let markdown = `# ${title}\n\n`;
  markdown += `> 导出时间: ${timestamp}\n> 模型: ${currentSession.value?.model || 'unknown'}\n\n---\n\n`;
  
  for (const msg of messages.value) {
    const time = msg.timestamp ? formatMessageTime(msg.timestamp) : '';
    if (msg.role === 'user') {
      markdown += `## 用户 (${time})\n\n${msg.content || ''}\n\n`;
    } else if (msg.role === 'assistant') {
      markdown += `## Hermes (${time})\n\n${msg.content || ''}\n\n`;
      if (msg.toolCalls && msg.toolCalls.length > 0) {
        markdown += `**工具调用:**\n`;
        for (const tool of msg.toolCalls) {
          markdown += `- ${tool.name} (${tool.durationMs}ms)\n`;
        }
        markdown += '\n';
      }
    }
  }
  
  // 下载文件
  const blob = new Blob([markdown], { type: 'text/markdown' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${title.replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, '_')}_${timestamp}.md`;
  a.click();
  URL.revokeObjectURL(url);
};

const deleteCurrentSession = async () => {
  if (!currentSessionId.value) return;

  // 简单确认对话框
  if (!confirm('确定要删除当前会话吗？此操作不可撤销。')) return;

  try {
    await invoke('agent_delete_session', { sessionId: currentSessionId.value });
    sessions.value = sessions.value.filter(s => s.id !== currentSessionId.value);
    startNewChat();
  } catch (e) {
    console.error('Delete error:', e);
  }
};

// 删除单条消息
const deleteMessage = (msgIndex: number) => {
  if (isStreaming.value) return;
  if (msgIndex < 0 || msgIndex >= messages.value.length) return;
  
  // 删除该消息
  messages.value.splice(msgIndex, 1);
};

// 撤回最后一条消息
const undoLastMessage = () => {
  if (messages.value.length === 0 || isStreaming.value) return;
  messages.value.pop();
};

// 标题编辑功能
const startEditTitle = () => {
  if (!currentSession.value) return;
  isEditingTitle.value = true;
  editingTitle.value = currentSession.value.title || '';
  nextTick(() => {
    titleInputRef.value?.focus();
  });
};

const cancelEditTitle = () => {
  isEditingTitle.value = false;
  editingTitle.value = '';
};

const saveTitle = async () => {
  if (!isEditingTitle.value || !currentSessionId.value) return;
  
  const newTitle = editingTitle.value.trim();
  if (!newTitle) {
    cancelEditTitle();
    return;
  }
  
  // 如果标题没有变化，直接取消编辑
  if (newTitle === currentSession.value?.title) {
    cancelEditTitle();
    return;
  }
  
  isEditingTitle.value = false;
  
  try {
    await invoke('agent_rename_session', {
      sessionId: currentSessionId.value,
      title: newTitle,
    });
    
    // 更新本地状态
    if (currentSession.value) {
      currentSession.value.title = newTitle;
    }
    // 更新会话列表中的标题
    const session = sessions.value.find(s => s.id === currentSessionId.value);
    if (session) {
      session.title = newTitle;
    }
  } catch (e) {
    console.error('Rename error:', e);
    // 恢复原标题
    editingTitle.value = currentSession.value?.title || '';
  }
};

const checkHermes = async () => {
  try {
    const result = await invoke<{ available: boolean; error: string | null }>('agent_check_available');
    hermesAvailable.value = result.available;
  } catch (e) {
    hermesAvailable.value = false;
  }
};

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
  });
};

// Lifecycle
onMounted(async () => {
  // 全局快捷键
  document.addEventListener('keydown', handleGlobalKeydown);
  
  // 监听流式事件
  unlistenDelta = await listen<string>('agent-delta', (event) => {
    // 收到实际内容时清空思考动画
    thinkingText.value = '';
    streamingText.value += event.payload;
    scrollToBottom();
  });

  unlistenToolStart = await listen<{ name: string; args: unknown }>('agent-tool-start', (event) => {
    // 工具开始，显示工具调用状态（区分子 agent）
    const toolName = event.payload.name;
    if (toolName === 'delegate_task') {
      thinkingText.value = '启动子 Agent 处理任务...';
    } else {
      thinkingText.value = `调用工具: ${toolName}...`;
    }
  });

  unlistenToolComplete = await listen<{ name: string; result: string; duration_ms: number }>('agent-tool-complete', (event) => {
    thinkingText.value = '';
    currentToolCalls.push({
      name: event.payload.name,
      durationMs: event.payload.duration_ms,
    });
  });

  // 思考动画事件
  unlistenThinking = await listen<string>('agent-thinking', (event) => {
    thinkingText.value = event.payload;
    scrollToBottom();
  });

  unlistenError = await listen<string>('agent-error', (event) => {
    thinkingText.value = '';
    streamingText.value += `\n[错误: ${event.payload}]`;
  });

  // 初始化
  await checkHermes();
  await refreshSessions();

  // 如果URL有sessionId参数，自动选择该会话
  const sessionIdFromQuery = route.query.sessionId as string;
  if (sessionIdFromQuery) {
    const session = sessions.value.find(s => s.id === sessionIdFromQuery);
    if (session) {
      selectSession(session);
    } else {
      // 会话不存在，尝试直接加载
      try {
        const result = await invoke<{ session_id: string; messages: Message[] }>('agent_get_session', {
          sessionId: sessionIdFromQuery,
        });
        currentSessionId.value = sessionIdFromQuery;
        currentSession.value = {
          id: sessionIdFromQuery,
          title: null,
          model: 'unknown',
          source: 'unknown',
          startedAt: Date.now() / 1000,
          endedAt: null,
          messageCount: result.messages.length,
          preview: '',
        };
        messages.value = result.messages;
        scrollToBottom();
      } catch (e) {
        console.error('Failed to load session from query:', e);
      }
    }
  }

  // 自动聚焦输入框，方便立即开始对话
  inputRef.value?.focus();
});

onUnmounted(() => {
  // Properly clean up event listeners
  unlistenDelta?.();
  unlistenToolStart?.();
  unlistenToolComplete?.();
  unlistenThinking?.();
  unlistenError?.();
  // 移除快捷键监听
  document.removeEventListener('keydown', handleGlobalKeydown);
});

// Watch streamingText to auto-scroll
watch(streamingText, () => {
  scrollToBottom();
});

// Watch inputText to auto-adjust textarea height
watch(inputText, () => {
  adjustTextareaHeight();
});

// Watch messages to update filteredMessages
watch(messages, () => {
  filteredMessages.value = messages.value;
}, { immediate: true });

// Watch searchQuery to filter messages
watch(searchQuery, () => {
  searchMessages();
});
</script>

<style scoped>
/* Markdown 内容样式 */
.markdown-content {
  line-height: 1.6;
}

.markdown-content :deep(p) {
  margin: 0.5em 0;
}

/* 行内代码样式 */
.markdown-content :deep(code:not(.hljs)) {
  background: rgba(0, 0, 0, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

/* 代码块包装器 */
.markdown-content :deep(.code-block-wrapper) {
  position: relative;
  margin: 0.8em 0;
  border-radius: 8px;
  overflow: hidden;
}

/* 代码块头部 */
.markdown-content :deep(.code-header) {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  background: rgba(0, 0, 0, 0.08);
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

/* 语言标签 */
.markdown-content :deep(.code-lang) {
  font-size: 0.75em;
  color: var(--color-base-content, #666);
  opacity: 0.7;
  text-transform: uppercase;
  font-weight: 500;
}

/* 复制按钮 */
.markdown-content :deep(.copy-btn) {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  background: transparent;
  border: 1px solid rgba(0, 0, 0, 0.1);
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-base-content, #666);
}

.markdown-content :deep(.copy-btn:hover) {
  background: rgba(0, 0, 0, 0.05);
}

.markdown-content :deep(.copy-btn.copied) {
  background: rgba(76, 175, 80, 0.2);
  border-color: #4caf50;
  color: #4caf50;
}

/* 代码块内容 */
.markdown-content :deep(.code-block-wrapper pre) {
  margin: 0;
  padding: 12px;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 0;
  overflow-x: auto;
}

.markdown-content :deep(.code-block-wrapper pre code) {
  background: transparent;
  padding: 0;
  font-size: 0.85em;
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

/* 旧版 pre 样式（兼容无包装器的代码块） */
.markdown-content :deep(pre:not(.code-block-wrapper pre)) {
  background: rgba(0, 0, 0, 0.05);
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 0.8em 0;
}

.markdown-content :deep(pre:not(.code-block-wrapper pre) code) {
  background: transparent;
  padding: 0;
  font-size: 0.85em;
}

/* 代码高亮主题 - 适配 daisyUI 主题 */
.markdown-content :deep(.hljs-keyword),
.markdown-content :deep(.hljs-selector-tag) {
  color: #e91e63;
}

.markdown-content :deep(.hljs-string),
.markdown-content :deep(.hljs-attr) {
  color: #4caf50;
}

.markdown-content :deep(.hljs-number),
.markdown-content :deep(.hljs-literal) {
  color: #2196f3;
}

.markdown-content :deep(.hljs-comment) {
  color: #9e9e9e;
}

.markdown-content :deep(.hljs-function),
.markdown-content :deep(.hljs-title) {
  color: #ff9800;
}

.markdown-content :deep(.hljs-variable),
.markdown-content :deep(.hljs-params) {
  color: #673ab7;
}

/* 链接样式 */
.markdown-content :deep(a) {
  color: var(--color-primary);
  text-decoration: underline;
}

.markdown-content :deep(a:hover) {
  opacity: 0.8;
}

/* 列表样式 */
.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}

.markdown-content :deep(li) {
  margin: 0.3em 0;
}

/* 表格样式 */
.markdown-content :deep(table) {
  border-collapse: collapse;
  margin: 0.8em 0;
  width: 100%;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid var(--color-base-content, #ccc);
  padding: 6px 12px;
  text-align: left;
}

.markdown-content :deep(th) {
  background: rgba(0, 0, 0, 0.05);
  font-weight: 600;
}

/* 引用块样式 */
.markdown-content :deep(blockquote) {
  border-left: 3px solid var(--color-primary);
  padding-left: 1em;
  margin: 0.8em 0;
  color: var(--color-base-content);
  opacity: 0.8;
}

/* 标题样式 */
.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3),
.markdown-content :deep(h4) {
  margin: 1em 0 0.5em;
  font-weight: 600;
}

.markdown-content :deep(h1) { font-size: 1.4em; }
.markdown-content :deep(h2) { font-size: 1.2em; }
.markdown-content :deep(h3) { font-size: 1.1em; }
</style>