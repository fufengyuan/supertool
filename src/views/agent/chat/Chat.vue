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
      :is-claw-mode="isClawMode"
      :plan-mode="planMode"
      :goal-mode="goalMode"
      :goal-text="goalText"
      :goal-status="goalStatus"
      :goal-turns-used="goalTurnsUsed"
      :goal-max-turns="goalMaxTurns"
      :goal-last-verdict="goalLastVerdict"
      :loop-mode="loopMode"
      @pick-folder="pickContextFolder"
      @clear-folder="clearContextFolder"
      @toggle-fast="handleToggleFast"
      @new-chat="startNewChat"
      @clear="clearChat"
      @fork="handleFork"
      @toggle-plan="handleTogglePlan"
      @toggle-goal="handleToggleGoal"
      @toggle-goal-pause="handleToggleGoalPause"
      @toggle-loop="handleToggleLoop"
    />
    
    <!-- Mode Status Bar (Plan / Goal / Loop) -->
    <ModeBar
      :plan-mode="planMode"
      :goal-mode="goalMode"
      :goal-text="goalText"
      :goal-status="goalStatus"
      :goal-turns-used="goalTurnsUsed"
      :goal-max-turns="goalMaxTurns"
      :tokens-used="goalTokensUsed"
      :token-budget="goalTokenBudget"
      :loop-mode="loopMode"
      :loop-iterations="loopIterations"
      :loop-max-iterations="loopMaxIterations"
      :loop-prompt="loopPrompt"
      @exit-plan="handleTogglePlan"
      @resume-goal="handleResumeGoal"
      @pause-goal="handleToggleGoalPause"
      @drop-goal="handleDropGoal"
      @pause-loop="handlePauseLoop"
      @stop-loop="handleToggleLoop"
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
        <ChatEmptyState
          :is-claw-mode="isClawMode"
          @select-suggestion="handleSuggestion"
        />
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

          <!-- Goal input prompt -->
          <div
            v-if="showGoalInput"
            class="mb-3 p-3 rounded-lg border border-info/20 bg-info/5"
          >
            <div class="text-xs text-info/70 mb-2">输入目标内容，按回车确认 / Esc 取消</div>
            <div class="flex gap-2">
              <input
                ref="goalInputRef"
                v-model="goalInputText"
                type="text"
                class="input input-sm input-bordered flex-1 text-sm"
                placeholder="输入目标描述..."
                @keydown.enter="confirmGoalInput"
                @keydown.esc="cancelGoalInput"
              />
              <button class="btn btn-sm btn-primary" @click="confirmGoalInput">确认</button>
              <button class="btn btn-sm btn-ghost" @click="cancelGoalInput">取消</button>
            </div>
          </div>

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
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

import { getTauriAPI } from '@/utils/tauri-api';
import SvgIcon from '@/components/ui/SvgIcon.vue';

import ChatHeader from './components/ChatHeader.vue';
import ChatEmptyState from './components/ChatEmptyState.vue';
import MessageList from './components/MessageList.vue';
import ModeBar from './components/ModeBar.vue';
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
const router = useRouter();
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
const goalInputRef = ref<HTMLInputElement | null>(null);
const showGoalInput = ref(false);
const goalInputText = ref('');

// Claw 模式状态
const clawInitialized = ref(false);
const isClawMode = computed(() => agentModeStore.mode === 'claw');

// Plan mode state
const planMode = ref(false);

// Goal mode state
const goalMode = ref(false);
const goalText = ref('');
const goalStatus = ref('inactive');
const goalTurnsUsed = ref(0);
const goalMaxTurns = ref(20);
const goalTokensUsed = ref(0);
const goalTokenBudget = ref<number | null>(null);
const goalLastVerdict = ref<string | null>(null);
const goalLastReason = ref<string | null>(null);

// Loop mode state
const loopMode = ref(false); // default: off (opt-in via /loop)
const loopPrompt = ref<string | null>(null);
const loopIterations = ref(0);
const loopMaxIterations = ref(0); // 0 = unlimited
let loopResendTimer: ReturnType<typeof setTimeout> | null = null;

function cancelLoopResend() {
  if (loopResendTimer) {
    clearTimeout(loopResendTimer);
    loopResendTimer = null;
  }
}

/** Schedule auto-resend of loop prompt after 800ms delay (matching oh-my-pi) */
function scheduleLoopResend() {
  cancelLoopResend();
  if (!loopMode.value || !loopPrompt.value) return;
  // Check iteration limit
  if (loopMaxIterations.value > 0 && loopIterations.value >= loopMaxIterations.value) {
    loopMode.value = false;
    addAgentMessage(`🔁 Loop finished — ${loopIterations.value} iterations completed`);
    invoke('claw_chat_set_loop_mode', { active: false }).catch(() => {});
    return;
  }
  loopResendTimer = setTimeout(async () => {
    loopResendTimer = null;
    if (!loopMode.value || !loopPrompt.value) return;
    loopIterations.value++;
    const prompt = loopPrompt.value;
    await clawSend(prompt);
    // Schedule next loop iteration after this turn completes
    scheduleLoopResend();
  }, 800);
}

// Plan mode toggle — calls Tauri backend set/get
async function handleTogglePlan() {
  if (!isClawMode.value) return;
  const newState = !planMode.value;
  planMode.value = newState;
  try {
    await invoke('claw_chat_set_plan_mode', { active: newState });
  } catch (e) {
    console.error('[Chat] Failed to set plan mode:', e);
  }
}

// Fetch plan mode state on init
async function fetchPlanMode() {
  if (!isClawMode.value) return;
  try {
    const res = await invoke<{ active: boolean }>('claw_chat_get_plan_mode');
    planMode.value = res.active;
  } catch (e) {
    planMode.value = false;
  }
}

// Goal mode toggle — button: ON→OFF clears goal; OFF→show inline input
async function handleToggleGoal() {
  if (!isClawMode.value) return;
  if (goalMode.value) {
    // Turn off — clear goal
    goalMode.value = false;
    goalText.value = '';
    goalStatus.value = 'inactive';
    goalTurnsUsed.value = 0;
    goalLastVerdict.value = null;
    goalLastReason.value = null;
    try {
      await invoke('claw_chat_set_goal_status', { status: 'clear' });
    } catch (e) {
      console.error('[Chat] Failed to clear goal mode:', e);
    }
  } else {
    // Show inline goal input
    goalInputText.value = '';
    showGoalInput.value = true;
    await nextTick();
    goalInputRef.value?.focus();
  }
}

function confirmGoalInput() {
  const text = goalInputText.value.trim();
  if (!text) return;
  showGoalInput.value = false;
  goalInputText.value = '';
  goalMode.value = true;
  goalText.value = text;
  goalStatus.value = 'active';
  goalTurnsUsed.value = 0;
  goalLastVerdict.value = null;
  goalLastReason.value = null;
  invoke('claw_chat_set_goal_mode', { active: true, goalText: text })
    .catch((e: unknown) => console.error('[Chat] Failed to set goal mode:', e));
}

function cancelGoalInput() {
  showGoalInput.value = false;
  goalInputText.value = '';
}

// Fetch goal mode state on init
async function fetchGoalMode() {
  if (!isClawMode.value) return;
  try {
    const res = await invoke<{
      active: boolean;
      goalText: string;
      status: string;
      turnsUsed: number;
      maxTurns: number;
      tokensUsed: number;
      tokenBudget: number | null;
      timeUsedSeconds: number;
      mode: string;
    }>('claw_chat_get_goal_mode');
    goalMode.value = res.active;
    goalText.value = res.goalText || '';
    goalStatus.value = res.status || 'inactive';
    goalTurnsUsed.value = res.turnsUsed || 0;
    goalMaxTurns.value = res.maxTurns || 20;
    goalTokensUsed.value = res.tokensUsed || 0;
    goalTokenBudget.value = res.tokenBudget ?? null;
  } catch (e) {
    goalMode.value = false;
    goalText.value = '';
    goalStatus.value = 'inactive';
    goalTokensUsed.value = 0;
    goalTokenBudget.value = null;
  }
}

// Loop mode toggle
async function handleToggleLoop() {
  if (!isClawMode.value) return;
  const newState = !loopMode.value;
  loopMode.value = newState;
  if (!newState) {
    // Disabling: clean up loop state
    cancelLoopResend();
    loopPrompt.value = null;
    loopIterations.value = 0;
  }
  try {
    await invoke('claw_chat_set_loop_mode', { active: newState });
  } catch (e) {
    console.error('[Chat] Failed to set loop mode:', e);
  }
}

// Goal pause/resume toggle
async function handleToggleGoalPause() {
  if (!isClawMode.value || !goalMode.value) return;
  const isPaused = goalStatus.value === 'paused';
  const newStatus = isPaused ? 'resume' : 'pause';
  try {
    await invoke('claw_chat_set_goal_status', { status: newStatus });
    goalStatus.value = isPaused ? 'active' : 'paused';
    goalMode.value = !isPaused; // paused = goal_mode off for backend
  } catch (e) {
    console.error('[Chat] Failed to toggle goal pause:', e);
  }
}

/** Resume a paused goal */
async function handleResumeGoal() {
  if (!isClawMode.value) return;
  try {
    await invoke('claw_chat_set_goal_status', { status: 'resume' });
    goalStatus.value = 'active';
    goalMode.value = true;
  } catch (e) {
    console.error('[Chat] Failed to resume goal:', e);
  }
}

/** Drop the current goal */
async function handleDropGoal() {
  if (!isClawMode.value) return;
  try {
    await invoke('claw_chat_set_goal_status', { status: 'drop' });
    goalMode.value = false;
    goalText.value = '';
    goalStatus.value = 'inactive';
    goalTurnsUsed.value = 0;
    goalTokensUsed.value = 0;
  } catch (e) {
    console.error('[Chat] Failed to drop goal:', e);
  }
}

/** Pause loop (keep mode, clear prompt) */
function handlePauseLoop() {
  cancelLoopResend();
  loopPrompt.value = null;
}

// Fetch loop mode state on init
async function fetchLoopMode() {
  if (!isClawMode.value) return;
  try {
    const res = await invoke<{ active: boolean }>('claw_chat_get_loop_mode');
    loopMode.value = res.active;
  } catch (e) {
    loopMode.value = false;
  }
}

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
    await fetchPlanMode();
    await fetchGoalMode();
    await fetchLoopMode();
  } else {
    planMode.value = false;
    goalMode.value = false;
    goalText.value = '';
    loopMode.value = false;
    await loadSessionHistory();
  }
});

// 监听路由 query 变化（用户在 Sessions 页面点击会话后导航到这里）
// 使用 deep watch 确保 query 变化能被捕获
watch(
  () => ({ ...route.query }),
  async (newQuery, oldQuery) => {
    const newSessionId = newQuery.session as string | undefined;
    const oldSessionId = oldQuery?.session as string | undefined;
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
  { deep: true },
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
  onGoalModeChange: (active, text) => {
    goalMode.value = active;
    if (text != null) goalText.value = text;
  },
  onLoopModeChange: (active, maxIterations) => {
    loopMode.value = active;
    if (!active) {
      cancelLoopResend();
      loopPrompt.value = null;
      loopIterations.value = 0;
    }
    if (maxIterations != null) {
      loopMaxIterations.value = maxIterations;
    }
  },
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
  handleAbort: handleAbortRaw,
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
  isClawMode,
});

/** Wrapper around handleAbort that also pauses loop mode (matching oh-my-pi Esc → pauseLoop) */
async function handleAbort() {
  // Pause loop: cancel pending resend, clear prompt, but keep mode enabled
  cancelLoopResend();
  loopPrompt.value = null;
  await handleAbortRaw();
}

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
  if (clawInitialized.value) return;
  try {
    const sessionId = route.query.session as string | undefined
    const result = await invoke<{
      sessionId: string;
      restored: boolean;
      messageCount: number;
      messages: Array<{ role: string; content: string; kind?: string; callId?: string; name?: string; args?: any }>;
    }>('claw_chat_init', {
      sessionId: sessionId || null,
      cwd: null as string | null,
    })
    clawInitialized.value = true

    if (result.restored && result.messages?.length > 0) {
      // Reset usage when switching to a restored session
      usage.value = null;
      console.log(`[Chat] 🐾 Restoring ${result.messages.length} messages from session ${result.sessionId}`);
      const converted: ChatMessage[] = result.messages.map((m: any, i: number) => {
        if (m.kind === 'tool_call') {
          return {
            id: `tc-${m.callId || Date.now()}-${i}`,
            kind: 'tool_call',
            role: 'agent',
            callId: m.callId || '',
            name: m.name || '',
            args: typeof m.args === 'string' ? m.args : JSON.stringify(m.args || {}, null, 2),
          }
        }
        if (m.kind === 'tool_result') {
          return {
            id: `tr-${m.callId || Date.now()}-${i}`,
            kind: 'tool_result',
            role: 'agent',
            callId: m.callId || '',
            name: m.name || '',
            content: m.content || '',
          }
        }
        return {
          id: `msg-${Date.now()}-${i}`,
          role: m.role === 'user' ? 'user' : 'agent',
          content: m.content || '',
        }
      })
      setMessages(converted)
    } else {
      // New session — reset all state
      messages.value = [];
      usage.value = null;
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
    const result = await invoke<{
      sessionId: string
      messageCount: number
      autoCompaction: number | null
      goalCompleted?: boolean
      goalPaused?: boolean
      goalTurnsUsed?: number
      goalMaxTurns?: number
    }>('claw_chat_send', { message: text })
    // Update session metadata from return value
    if (result?.sessionId) {
      hermesSessionId.value = result.sessionId
    }
    if (result?.autoCompaction && result.autoCompaction > 0) {
      console.log(`[Chat] ⚠️ Session compacted: ${result.autoCompaction} messages removed`)
    }
    // Update goal tracking from response
    if (result?.goalCompleted) {
      goalStatus.value = 'done';
      addAgentMessage(`✅ **Goal completed!** (${result.goalTurnsUsed ?? 0} turns used)`);
    } else if (result?.goalPaused) {
      goalStatus.value = 'paused';
      goalMode.value = false;
      addAgentMessage(`⏸ **Goal paused** — turn budget of ${result.goalMaxTurns ?? 20} exhausted. Resume to continue.`);
    }
    if (result?.goalTurnsUsed != null) {
      goalTurnsUsed.value = result.goalTurnsUsed;
    }
    // Reset loading state — agent-done event also resets this,
    // but we do it here as safety net in case event doesn't fire
    isLoading.value = false
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
    if (isLoading.value) return; // prevent concurrent sends
    // Capture loop prompt if loop mode is ON
    if (loopMode.value) {
      loopPrompt.value = text;
      loopIterations.value = 0;
    }
    chatInputRef.value?.clear()
    pushUser(text)
    await clawSend(text)
    // Schedule loop auto-resend after turn completes
    scheduleLoopResend();
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
  if (isClawMode.value) {
    // Claw 模式：只填充输入框，不自动发送
    chatInputRef.value?.focus();
  } else {
    handleSendInput();
  }
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
  // Claw mode: also update active session model
  if (isClawMode.value && hermesSessionId.value) {
    try {
      const api = getTauriAPI();
      await api.clawChatSetModel(hermesSessionId.value, model);
    } catch (e) {
      console.warn('[Chat] Failed to set session model:', e);
    }
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

async function handleFork() {
  if (!hermesSessionId.value) return;
  const branchName = prompt('分支名称（可选）:', '');
  // Cancel = null, empty = ok (no name)
  if (branchName === null) return;
  try {
    const tauri = getTauriAPI();
    const result = await tauri.clawChatFork(hermesSessionId.value, branchName || null);
    addAgentMessage(`🔀 已派生新会话: ${result.newSessionId.slice(-8)}${result.branchName ? ` (分支: ${result.branchName})` : ''}`);
  } catch (e: any) {
    addAgentMessage(`❌ 派生失败: ${e?.message || String(e)}`);
  }
}

// ── Keyboard ─────────────────────────────────────────────────────────────────
function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
    e.preventDefault();
    handleSendInput();
    return;
  }

  // Esc during loop gap (800ms between iterations) → cancel pending resend
  if (e.key === 'Escape' && loopResendTimer) {
    cancelLoopResend();
    loopPrompt.value = null;
    e.preventDefault();
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
  if (isClawMode.value) return;
  const sessionId = route.query.session as string | undefined;
  if (!sessionId) return;
  try {
    hermesSessionId.value = sessionId;
    isLoading.value = true;
    const result = await invoke<{
      success: boolean;
      messages: any[];
      sessionId: string;
    }>('agent_list_messages', { sessionId });
    if (result.success && result.messages?.length) {
      setMessages(hermesMessagesToChatMessages(result.messages));
    }
  } catch (e) {
    console.error('[Chat] Failed to load session history:', e);
  } finally {
    isLoading.value = false;
  }
}

onMounted(async () => {
  if (isClawMode.value) {
    await ensureClawChat();
    await fetchPlanMode();
    await fetchGoalMode();
    await fetchLoopMode();
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
