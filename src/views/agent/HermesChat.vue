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
        <button class="btn btn-primary btn-sm w-full gap-1.5" @click="startNewChat">
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
        <div class="flex items-center gap-2">
          <button class="btn btn-ghost btn-xs btn-circle" @click="router.back()" title="返回">
            <SvgIcon name="arrowLeft" size="14" />
          </button>
          <SvgIcon name="bot" size="16" class="text-primary" />
          <span class="text-sm font-semibold text-base-content">
            {{ currentSession?.title || '新对话' }}
          </span>
          <span v-if="currentSession" class="badge badge-ghost badge-xs">
            {{ currentSession.model }}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <button v-if="isStreaming" class="btn btn-error btn-xs gap-1" @click="abortChat">
            <SvgIcon name="stop" size="12" />
            停止
          </button>
          <button v-if="currentSession" class="btn btn-ghost btn-xs" @click="deleteCurrentSession">
            <SvgIcon name="trash" size="12" />
          </button>
        </div>
      </div>

      <!-- 消息列表 -->
      <div ref="messagesContainer" class="flex-1 overflow-y-auto px-4 py-4 space-y-3">
        <!-- 加载消息状态 -->
        <div v-if="loadingMessages" class="flex items-center justify-center py-8">
          <SvgIcon name="refresh" size="16" class="animate-spin text-base-content/40" />
        </div>

        <!-- 消息列表 -->
        <template v-else-if="messages.length > 0">
          <div v-for="(msg, idx) in messages" :key="idx" class="flex gap-3">
            <!-- 用户消息 -->
            <div v-if="msg.role === 'user'" class="flex gap-3 w-full">
              <div class="flex h-8 w-8 items-center justify-center rounded-full bg-base-200 shrink-0">
                <SvgIcon name="user" size="14" class="text-base-content/60" />
              </div>
              <div class="flex-1 bg-base-200 rounded-xl px-3 py-2">
                <p class="text-sm text-base-content whitespace-pre-wrap">{{ msg.content }}</p>
              </div>
            </div>

            <!-- Assistant 消息 -->
            <div v-else class="flex gap-3 w-full">
              <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20 shrink-0">
                <SvgIcon name="bot" size="14" class="text-primary" />
              </div>
              <div class="flex-1 bg-primary/5 border border-primary/10 rounded-xl px-3 py-2">
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
        <div v-else class="flex gap-2">
          <textarea
            ref="inputRef"
            v-model="inputText"
            class="textarea textarea-bordered w-full resize-none text-sm"
            rows="2"
            placeholder="输入消息..."
            :disabled="isStreaming"
            @keydown.enter.exact.prevent="sendMessage"
          ></textarea>
          <button
            class="btn btn-primary self-end"
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
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
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

// Refs
const messagesContainer = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLTextAreaElement | null>(null);

// Event listeners
let unlistenDelta: UnlistenFn | null = null;
let unlistenToolStart: UnlistenFn | null = null;
let unlistenToolComplete: UnlistenFn | null = null;
let unlistenThinking: UnlistenFn | null = null;
let unlistenError: UnlistenFn | null = null;
let currentToolCalls: { name: string; durationMs: number }[] = [];

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

// Markdown 渲染函数
const renderMarkdown = (text: string | null): string => {
  if (!text) return '';
  try {
    const html = marked.parse(text) as string;
    return DOMPurify.sanitize(html, {
      ADD_ATTR: ['target'], // 允许链接的 target 属性
      ADD_TAGS: ['mark'], // 允许 mark 标签
    });
  } catch {
    return text;
  }
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

const sendMessage = async () => {
  if (!inputText.value.trim() || isStreaming.value) return;

  const text = inputText.value.trim();
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
    const result = await invoke<{ response: string; session_id: string; message_count: number }>('agent_chat', {
      message: text,
      sessionId: currentSessionId.value,
      model: null,
      toolsets: null,
    });

    // 更新 session ID
    if (result.session_id && !currentSessionId.value) {
      currentSessionId.value = result.session_id;
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
    messages.value.push({
      role: 'assistant',
      content: `错误: ${e}`,
      timestamp: Date.now() / 1000,
      toolName: null,
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
});

// Watch streamingText to auto-scroll
watch(streamingText, () => {
  scrollToBottom();
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

.markdown-content :deep(code) {
  background: rgba(0, 0, 0, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

.markdown-content :deep(pre) {
  background: rgba(0, 0, 0, 0.05);
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 0.8em 0;
}

.markdown-content :deep(pre code) {
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