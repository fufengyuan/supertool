import { onUnmounted, isRef } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { Ref } from 'vue';
import type { ChatMessage, UsageState } from '../types';
import {
  dbItemsToChatMessages,
  reconcileStreamedWithDb,
  type DbHistoryItem,
} from '../sessionHistory';

interface UseChatIPCArgs {
  messages: Ref<ChatMessage[]>;
  setMessages: (msgs: ChatMessage[]) => void;
  hermesSessionId: Ref<string | null>;
  toolProgress: Ref<string | null>;
  isLoading: Ref<boolean>;
  usage: Ref<UsageState | null>;
  scrollToBottom?: () => void;
  isClawMode?: Ref<boolean> | boolean;
}

/**
 * Registers all chat-related IPC listeners once and tears them down on unmount.
 * Aligned with hermes-desktop's useChatIPC event model.
 */
export function useChatIPC({
  messages,
  setMessages,
  hermesSessionId,
  toolProgress,
  isLoading,
  usage,
  scrollToBottom,
  isClawMode: isClawModeArg = false,
}: UseChatIPCArgs): void {
  const cleanups: UnlistenFn[] = [];
  // Normalize: accept both Ref<boolean> and plain boolean
  const getIsClawMode = () => isRef(isClawModeArg) ? isClawModeArg.value : isClawModeArg;

  const setup = async () => {
    console.log('[ChatIPC] 🔧 Setting up IPC listeners...');
    // agent-delta: text content chunk
    const unlistenDelta = await listen<{
      text: string | null;
      session_id: string | null;
    }>('agent-delta', (event) => {
      console.log('[ChatIPC] 📨 agent-delta:', event.payload?.text?.slice(0, 80), 'session:', event.payload?.session_id);
      const chunk = event.payload?.text;
      if (!chunk) return;
      const prev = messages.value;
      const last = prev[prev.length - 1];
      if (
        last &&
        last.role === 'agent' &&
        'content' in last &&
        typeof last.content === 'string'
      ) {
        setMessages([
          ...prev.slice(0, -1),
          { ...last, content: last.content + chunk },
        ]);
      } else {
        if (!chunk || !chunk.trim()) return;
        setMessages([
          ...prev,
          {
            id: `agent-${Date.now()}`,
            role: 'agent',
            content: chunk,
          },
        ]);
      }
      scrollToBottom?.();
    });
    cleanups.push(unlistenDelta);

    // agent-reasoning-delta: streaming reasoning/thinking tokens
    const unlistenReasoning = await listen<{
      text: string | null;
      session_id: string | null;
    }>('agent-reasoning-delta', (event) => {
      console.log('[ChatIPC] 📨 agent-reasoning-delta:', event.payload?.text?.slice(0, 80));
      const chunk = event.payload?.text;
      if (!chunk) return;
      const prev = messages.value;
      let insertAt = prev.length;
      for (let i = prev.length - 1; i >= 0; i--) {
        const m = prev[i];
        if (m.role === 'user') break;
        if ('kind' in m && m.kind === 'reasoning') {
          setMessages([
            ...prev.slice(0, i),
            { ...m, text: m.text + chunk },
            ...prev.slice(i + 1),
          ]);
          scrollToBottom?.();
          return;
        }
        insertAt = i;
      }
      setMessages([
        ...prev.slice(0, insertAt),
        {
          id: `reasoning-${Date.now()}`,
          kind: 'reasoning',
          role: 'agent',
          text: chunk,
        },
        ...prev.slice(insertAt),
      ]);
      scrollToBottom?.();
    });
    cleanups.push(unlistenReasoning);

    // agent-tool-start: tool started
    const unlistenToolStart = await listen<{
      id?: string;
      name: string;
      args: unknown;
      session_id: string | null;
    }>('agent-tool-start', (event) => {
      console.log('[ChatIPC] 📨 agent-tool-start:', event.payload?.name, 'id:', event.payload?.id);
      const payload = event.payload;
      const prev = messages.value;
      const callId = payload.id || `tc-${Date.now()}`;
      // Find the last assistant bubble and append a tool_call after it
      let insertAt = prev.length;
      for (let i = prev.length - 1; i >= 0; i--) {
        if (prev[i].role === 'user') break;
        insertAt = i;
        break;
      }
      const toolCallMsg: ChatMessage = {
        id: `tc-${callId}`,
        kind: 'tool_call',
        role: 'agent',
        callId,
        name: payload.name,
        args: typeof payload.args === 'string'
          ? payload.args
          : JSON.stringify(payload.args || {}, null, 2),
      };
      setMessages([...prev.slice(0, insertAt), toolCallMsg, ...prev.slice(insertAt)]);
      toolProgress.value = `🔧 ${payload.name}...`;
      scrollToBottom?.();
    });
    cleanups.push(unlistenToolStart);

    // agent-tool-complete: tool completed
    const unlistenToolComplete = await listen<{
      id?: string;
      name: string;
      result: string | null;
      duration_ms: number;
      session_id: string | null;
    }>('agent-tool-complete', (event) => {
      console.log('[ChatIPC] 📨 agent-tool-complete:', event.payload?.name, 'result:', event.payload?.result);
      const payload = event.payload;
      const prev = messages.value;
      const callId = payload.id || '';
      // Append tool_result after the matching tool_call
      let insertAt = prev.length;
      for (let i = prev.length - 1; i >= 0; i--) {
        if (prev[i].role === 'user') break;
        if ('kind' in prev[i] && prev[i].kind === 'tool_call') {
          insertAt = i + 1;
          break;
        }
      }
      const toolResultMsg: ChatMessage = {
        id: `tr-${callId || Date.now()}`,
        kind: 'tool_result',
        role: 'agent',
        callId,
        name: payload.name,
        content: payload.result || '',
      };
      setMessages([...prev.slice(0, insertAt), toolResultMsg, ...prev.slice(insertAt)]);
      toolProgress.value = null;
      scrollToBottom?.();
    });
    cleanups.push(unlistenToolComplete);

    // agent-done: stream finished
    const unlistenDone = await listen<{
      response?: string | null;
      session_id?: string;
      message_count?: number;
    }>('agent-done', async (event) => {
      console.log('[ChatIPC] 📨 agent-done:', event.payload);
      const sessionId = event.payload?.session_id;
      if (sessionId) hermesSessionId.value = sessionId;
      toolProgress.value = null;
      isLoading.value = false;
      // In Claw mode, skip the Hermes DB merge — there's no Hermes backend
      if (getIsClawMode()) return;
      // End-of-stream DB merge
      if (!sessionId) return;
      try {
        const dbResult = await invoke<{
          success: boolean;
          messages: DbHistoryItem[];
        }>('agent_list_messages', { sessionId });
        if (dbResult.success && dbResult.messages.length > 0) {
          const dbMessages = dbItemsToChatMessages(dbResult.messages);
          if (dbMessages.length > 0) {
            setMessages(reconcileStreamedWithDb(messages.value, dbMessages));
          }
        }
      } catch {
        // Merge is a UX nicety — don't break the chat flow if it fails.
      }
    });
    cleanups.push(unlistenDone);

    // agent-error
    const unlistenError = await listen<{
      message: string;
      session_id: string | null;
    }>('agent-error', (event) => {
      console.log('[ChatIPC] 📨 agent-error:', event.payload?.message);
      setMessages([
        ...messages.value,
        {
          id: `error-${Date.now()}`,
          role: 'agent',
          content: `Error: ${event.payload?.message}`,
        },
      ]);
      toolProgress.value = null;
      isLoading.value = false;
    });
    cleanups.push(unlistenError);

    // agent-tool-progress: progress text
    const unlistenToolProgress = await listen<string | null>(
      'agent-tool-progress',
      (event) => {
        toolProgress.value = event.payload;
      },
    );
    cleanups.push(unlistenToolProgress);

    // agent-usage: token usage
    const unlistenUsage = await listen<{
      prompt_tokens: number;
      completion_tokens: number;
      total_tokens: number;
      cost?: number;
      session_id: string | null;
    }>('agent-usage', (event) => {
      const u = event.payload;
      usage.value = {
        promptTokens: (usage.value?.promptTokens || 0) + u.prompt_tokens,
        completionTokens: (usage.value?.completionTokens || 0) + u.completion_tokens,
        totalTokens: (usage.value?.totalTokens || 0) + u.total_tokens,
        cost: u.cost != null ? (usage.value?.cost || 0) + u.cost : usage.value?.cost,
      };
    });
    cleanups.push(unlistenUsage);
  };

  setup();

  onUnmounted(() => {
    for (const cleanup of cleanups) {
      cleanup();
    }
  });
}
