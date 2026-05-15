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
              <span class="truncate text-xs text-base-content/50">{{ formatTime(session.startedAt) }}</span>
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
                <p class="text-sm text-base-content whitespace-pre-wrap">{{ msg.content }}</p>
                <!-- 工具调用显示 -->
                <div v-if="msg.toolCalls && msg.toolCalls.length > 0" class="mt-2 space-y-1">
                  <div v-for="tool in msg.toolCalls" :key="tool.name" class="flex items-center gap-2 text-xs">
                    <SvgIcon name="tool" size="12" class="text-warning" />
                    <span class="text-base-content/70">{{ tool.name }}</span>
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
              <p class="text-sm text-base-content whitespace-pre-wrap">{{ streamingText }}</p>
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
            @keydown.enter.shift.exact="() => {}"
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
import SvgIcon from '@/components/ui/SvgIcon.vue';

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
    sessions.value = result.sessions;
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
    // 工具开始，显示工具调用状态
    thinkingText.value = `调用工具: ${event.payload.name}...`;
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