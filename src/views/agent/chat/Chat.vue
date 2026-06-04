<template>
  <div class="flex flex-col h-full bg-base-100">
    <!-- Header -->
    <ChatHeader
      :session-id="hermesSessionId"
      :usage="usage"
      :fast-mode="fastMode"
      :has-messages="messages.length > 0"
      :context-folder="contextFolder"
      :show-context-folder="true"
      @pick-folder="pickContextFolder"
      @clear-folder="clearContextFolder"
      @toggle-fast="handleToggleFast"
      @new-chat="startNewChat"
      @clear="clearChat"
    />

    <!-- Messages area -->
    <div ref="containerRef" class="flex-1 overflow-y-auto min-h-0">
      <template v-if="messages.length > 0">
        <MessageList
          ref="messageListRef"
          :messages="messages"
          :is-loading="isLoading"
          :tool-progress="toolProgress"
          :on-approve="handleApprove"
          :on-deny="handleDeny"
        />
      </template>
      <template v-else>
        <ChatEmptyState @select-suggestion="handleSuggestion" />
      </template>
      <div ref="bottomRef" />
    </div>

    <!-- Input area -->
    <div class="border-t border-base-content/10 bg-base-100/80 backdrop-blur-sm">
      <!-- Approval mode indicator -->
      <div
        v-if="isApprovalMode"
        class="px-4 py-1.5 bg-primary/10 border-t border-primary/20 flex items-center gap-2"
      >
        <SvgIcon name="clock" size="12" class="text-primary animate-pulse" />
        <span class="text-xs text-primary/70">等待审批中...</span>
      </div>

      <div class="px-4 py-3">
        <div class="max-w-3xl mx-auto">
          <!-- Model picker row -->
          <div class="flex items-center justify-between mb-2">
            <ModelPicker
              :current-model="currentModel"
              :current-provider="currentProvider"
              :current-base-url="currentBaseUrl"
              :model-groups="modelGroups"
              :display-model="displayModel"
              @open="reload"
              @select-model="handleSelectModel"
            />
            <div class="flex items-center gap-2 text-[10px] text-base-content/30">
              <span
                v-if="fastMode"
                class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-warning/10 text-warning/70"
              >
                <SvgIcon name="zap" size="8" />
                优先模式
              </span>
              <span>⌘ ↵ 发送</span>
            </div>
          </div>

          <!-- ChatInput -->
          <SimpleChatInput
            ref="chatInputRef"
            :model-value="currentInput"
            :is-loading="isLoading"
            :is-approval-mode="isApprovalMode"
            :placeholder="inputPlaceholder"
            @update:model-value="currentInput = $event"
            @send="handleSendInput"
            @abort="handleAbort"
            @approve="handleApprove"
            @deny="handleDeny"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

import SvgIcon from '@/components/ui/SvgIcon.vue';

import ChatHeader from './components/ChatHeader.vue';
import ChatEmptyState from './components/ChatEmptyState.vue';
import MessageList from './components/MessageList.vue';
import ModelPicker from './ModelPicker.vue';
import SimpleChatInput from './SimpleChatInput.vue';

import { useChatIPC } from './composables/useChatIPC';
import { useChatActions } from './composables/useChatActions';
import { useChatScroll } from './composables/useChatScroll';
import { useFastMode } from './composables/useFastMode';
import { useInputHistory } from './composables/useInputHistory';
import { useModelConfig } from './composables/useModelConfig';
import { useLocalCommands } from './composables/useLocalCommands';

import type { ChatMessage, UsageState } from './types';
import { hermesMessagesToChatMessages } from './sessionHistory';

import { useAgentModeStore } from '@/stores/agentModeStore';

const route = useRoute();
const agentModeStore = useAgentModeStore();

// ── Core state ───────────────────────────────────────────────────────────────
const messages = ref<ChatMessage[]>([]);
const hermesSessionId = ref<string | null>(null);
const isLoading = ref(false);
const toolProgress = ref<string | null>(null);
const usage = ref<UsageState | null>(null);
const contextFolder = ref<string | null>(null);
const currentInput = ref('');
const chatInputRef = ref<{ clear: () => void; focus: () => void } | null>(null);
const messageListRef = ref<InstanceType<typeof MessageList> | null>(null);

// Claw 模式状态
const clawInitialized = ref(false);
const isClawMode = computed(() => agentModeStore.mode === 'claw');

// 监听模式切换：清空消息、重新初始化
watch(() => agentModeStore.mode, async (newMode, oldMode) => {
  console.log(`[Chat] 🔄 Mode changed: ${oldMode} → ${newMode}`);
  messages.value = [];
  usage.value = null;
  toolProgress.value = null;
  hermesSessionId.value = null;

  if (newMode === 'claw') {
    clawInitialized.value = false;
    await ensureClawChat();
  } else {
    await loadSessionHistory();
  }
});

// 监听路由 query 变化（用户在 Sessions 页面点击会话后导航到这里）
watch(
  () => route.query.session,
  async (newSessionId, oldSessionId) => {
    console.log(`[Chat] 🔀 Route session changed: ${oldSessionId} → ${newSessionId}, isClawMode=${isClawMode.value}`);
    // 仅当确实变化且组件已挂载时处理
    if (newSessionId === oldSessionId) return;
    if (isClawMode.value) {
      // Claw 模式：重新初始化 session
      clawInitialized.value = false;
      messages.value = [];
      usage.value = null;
      toolProgress.value = null;
      hermesSessionId.value = null;
      await ensureClawChat();
    } else {
      // Hermes 模式：加载对应 session 的消息
      await loadSessionHistory();
    }
  },
);

function setMessages(msgs: ChatMessage[]) {
  messages.value = msgs;
}

// ── Helper: add an agent-side text message ────────────────────────────────────
function addAgentMessage(content: string) {
  setMessages([
    ...messages.value,
    { id: `agent-${Date.now()}`, role: 'agent', content },
  ]);
}

function startNewChat() {
  if (messages.value.length > 0 && !window.confirm('确认开始新对话？当前对话内容将保留。')) return;
  messages.value = [];
  hermesSessionId.value = null;
  usage.value = null;
  toolProgress.value = null;
  currentInput.value = '';
}

function clearChat() {
  if (!window.confirm('确认清空当前对话内容？')) return;
  messages.value = [];
  usage.value = null;
  toolProgress.value = null;
}

// ── Fast mode ────────────────────────────────────────────────────────────────
const { fastMode, toggle: doToggleFast } = useFastMode();

// ── Local commands (must be set up before chat actions) ──────────────────────
const localCommands = useLocalCommands({
  usage,
  fastMode,
  setFastMode: async (next: boolean) => {
    await invoke('hermes_set_config', {
      key: 'agent.service_tier',
      value: next ? 'fast' : 'normal',
    });
    fastMode.value = next;
  },
  onNewChat: startNewChat,
  onClear: clearChat,
  addAgentMessage,
});

// ── Scroll management ────────────────────────────────────────────────────────
const { containerRef, bottomRef, scrollToBottom } = useChatScroll(messages);

// ── Chat IPC listeners ───────────────────────────────────────────────────────
useChatIPC({
  messages,
  setMessages,
  hermesSessionId,
  toolProgress,
  isLoading,
  usage,
  scrollToBottom,
  isClawMode,
});

function handleToggleFast() {
  doToggleFast();
}

// ── Input history ────────────────────────────────────────────────────────────
const { push: pushHistory, recallPrev, recallNext } = useInputHistory({
  currentInput,
  applyText: (text: string) => {
    currentInput.value = text;
    chatInputRef.value?.focus();
  },
});

// ── Model config ─────────────────────────────────────────────────────────────
const {
  currentModel,
  currentProvider,
  currentBaseUrl,
  modelGroups,
  displayModel,
  reload,
  selectModel,
} = useModelConfig();

// ── Chat actions (depends on localCommands, scrollToBottom, etc.) ────────────
const {
  handleSend: doSend,
  handleAbort,
  handleApprove,
  handleDeny,
} = useChatActions({
  hermesSessionId,
  messages,
  setMessages,
  isLoading,
  onSessionStarted: scrollToBottom,
  localCommands,
  contextFolder,
  scrollToBottom,
  inputRef: chatInputRef,
});

// ── Derived ──────────────────────────────────────────────────────────────────
const isApprovalMode = computed(() => {
  if (messages.value.length === 0) return false;
  const last = messages.value[messages.value.length - 1];
  return last.kind === 'tool_call' && last.name === 'ask_user_question';
});

const inputPlaceholder = computed(() => {
  if (isApprovalMode.value) return '输入回复内容或审批操作（/approve /deny）...';
  if (isLoading.value) return 'Agent 正在处理...';
  return '输入消息，Ctrl+Enter 发送...';
});

// ── Handlers ─────────────────────────────────────────────────────────────────

/** 添加用户消息到列表 */
function pushUser(content: string) {
  setMessages([
    ...messages.value,
    { id: `user-${Date.now()}`, role: 'user', content },
  ])
}

/** Claw 模式：初始化连接（可恢复历史会话） */
async function ensureClawChat() {
  console.log(`[Chat] 🐾 ensureClawChat() called, clawInitialized=${clawInitialized.value}`);
  if (clawInitialized.value) {
    console.log('[Chat] ⏭ Claw already initialized, skipping');
    return;
  }
  try {
    const sessionId = route.query.session as string | undefined
    console.log(`[Chat] 🐾 claw_chat_init sessionId=${sessionId}`);
    const result = await invoke<{
      sessionId: string;
      restored: boolean;
      messageCount: number;
      messages: Array<{ role: string; content: string }>;
    }>('claw_chat_init', {
      sessionId: sessionId || null,
      cwd: null as string | null,
    })
    clawInitialized.value = true
    console.log(`[Chat] 🐾 claw_chat_init result: sessionId=${result.sessionId}, restored=${result.restored}, messages=${result.messages?.length ?? 0}`);
    if (result.restored && result.messages?.length > 0) {
      const converted: ChatMessage[] = result.messages.map((m, i) => ({
        id: `msg-${Date.now()}-${i}`,
        role: m.role === 'user' ? 'user' : 'agent',
        content: m.content,
      }))
      setMessages(converted)
    } else {
      addAgentMessage('Claw 编码助手已就绪')
    }
  } catch (e: any) {
    addAgentMessage(`Claw 初始化失败: ${e?.message || String(e)}`)
    isLoading.value = false
  }
}

/** Claw 模式：发送消息 */
async function clawSend(text: string) {
  isLoading.value = true
  await ensureClawChat()
  try {
    await invoke('claw_chat_send', { message: text })
  } catch (e: any) {
    addAgentMessage(`发送失败: ${e?.message || String(e)}`)
    isLoading.value = false
  }
}

async function handleSendInput() {
  const text = currentInput.value.trim();
  if (!text) return;

  // Claw 模式 → 走 direct LLM API
  if (isClawMode.value) {
    chatInputRef.value?.clear()
    pushUser(text)
    await clawSend(text)
    pushHistory(text)
    return
  }

  if (isApprovalMode.value) {
    if (/^\/approve$/i.test(text)) {
      chatInputRef.value?.clear();
      handleApprove();
    } else if (/^\/deny$/i.test(text)) {
      chatInputRef.value?.clear();
      handleDeny();
    } else {
      await doSend(text);
      chatInputRef.value?.clear();
    }
  } else {
    await doSend(text);
    chatInputRef.value?.clear();
  }

  pushHistory(text);
}

function handleSuggestion(text: string) {
  currentInput.value = text;
  handleSendInput();
}

async function handleSelectModel(provider: string, model: string, baseUrl: string) {
  await selectModel(provider, model, baseUrl);
  try {
    await invoke('agent_set_config', {
      config: { provider, model, ...(baseUrl ? { baseUrl } : {}) },
    });
  } catch {
    // Best-effort sync to backend
  }
}

async function pickContextFolder() {
  try {
    const selected = await open({ directory: true });
    if (typeof selected === 'string') {
      contextFolder.value = selected;
    }
  } catch {
    // User cancelled
  }
}

function clearContextFolder() {
  contextFolder.value = null;
}

// ── Keyboard ─────────────────────────────────────────────────────────────────
function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
    e.preventDefault();
    handleSendInput();
    return;
  }

  if (e.key === 'ArrowUp') {
    const prev = recallPrev();
    if (prev) e.preventDefault();
  } else if (e.key === 'ArrowDown') {
    const next = recallNext();
    if (next) e.preventDefault();
  }
}

async function loadSessionHistory() {
  console.log(`[Chat] 📥 loadSessionHistory() called, isClawMode=${isClawMode.value}, route.query.session=${route.query.session}`);
  // Claw mode: session history is managed by Claw runtime, not Hermes DB
  if (isClawMode.value) {
    console.log('[Chat] ⏭ Skipping loadSessionHistory in Claw mode');
    return;
  }
  const sessionId = route.query.session as string | undefined;
  if (!sessionId) {
    console.log('[Chat] ⏭ No sessionId in route query, skipping');
    return;
  }
  try {
    hermesSessionId.value = sessionId;
    isLoading.value = true;
    console.log(`[Chat] 📡 Calling agent_list_messages(sessionId=${sessionId})`);
    const result = await invoke<{
      success: boolean;
      messages: any[];
      sessionId: string;
    }>('agent_list_messages', { sessionId });
    console.log(`[Chat] 📨 agent_list_messages returned: success=${result.success}, messages=${result.messages?.length ?? 0}`);
    if (result.messages?.length > 0) {
      console.log('[Chat] 📝 First message sample:', JSON.stringify(result.messages[0]).slice(0, 200));
    }
    if (result.success && result.messages?.length) {
      const converted = hermesMessagesToChatMessages(result.messages);
      console.log(`[Chat] 🔄 hermesMessagesToChatMessages → ${converted.length} ChatMessages`);
      setMessages(converted);
      console.log(`[Chat] ✅ Messages set, current count: ${messages.value.length}`);
    } else {
      console.log('[Chat] ⚠️ No messages to display (success=' + result.success + ', count=' + (result.messages?.length ?? 0) + ')');
    }
  } catch (e) {
    console.error('[Chat] ❌ Failed to load session history:', e);
  } finally {
    isLoading.value = false;
  }
}

onMounted(async () => {
  console.log(`[Chat] 🚀 onMounted: isClawMode=${isClawMode.value}, route.query.session=${route.query.session}, messages.length=${messages.value.length}`);
  if (isClawMode.value) {
    await ensureClawChat();
  } else {
    await loadSessionHistory();
  }
  document.addEventListener('keydown', handleKeydown);
  chatInputRef.value?.focus();
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
  // Don't close Claw session on unmount — preserve messages for navigation
});
</script>
