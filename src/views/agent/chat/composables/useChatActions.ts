import { ref, isRef } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Ref } from 'vue';
import type { ChatMessage, Attachment } from '../types';

interface LocalCommands {
  isLocal: (text: string) => boolean;
  executeLocal: (text: string) => Promise<boolean>;
}

interface UseChatActionsArgs {
  hermesSessionId: Ref<string | null>;
  messages: Ref<ChatMessage[]>;
  setMessages: (msgs: ChatMessage[]) => void;
  isLoading: Ref<boolean>;
  onSessionStarted?: () => void;
  localCommands: LocalCommands;
  contextFolder: Ref<string | null>;
  scrollToBottom: () => void;
  inputRef: Ref<{ clear: () => void; focus: () => void } | null>;
  isClawMode?: Ref<boolean> | boolean;
}

interface UseChatActionsResult {
  handleSend: (
    text: string,
    attachments?: Attachment[],
    skipLoadingCheck?: boolean,
  ) => Promise<void>;
  handleQuickAsk: (text: string, attachments?: Attachment[]) => Promise<void>;
  handleAbort: () => void;
  handleApprove: () => void;
  handleDeny: () => void;
}

function hasContent(msg: ChatMessage): msg is ChatMessage & { content: string } {
  return (
    msg.kind === 'user' ||
    msg.kind === 'assistant' ||
    (!msg.kind && (msg.role === 'user' || msg.role === 'agent'))
  );
}

/**
 * Encapsulates the chat's user-facing actions (send, quick-ask, abort,
 * approve, deny).
 */
export function useChatActions({
  hermesSessionId,
  messages,
  setMessages,
  isLoading,
  onSessionStarted,
  localCommands,
  contextFolder,
  scrollToBottom,
  inputRef,
  isClawMode: isClawModeArg = false,
}: UseChatActionsArgs): UseChatActionsResult {
  const getIsClawMode = () => isRef(isClawModeArg) ? isClawModeArg.value : isClawModeArg;
  const pushUser = (
    content: string,
    idPrefix = 'user',
    attachments?: Attachment[],
  ) => {
    setMessages([
      ...messages.value,
      {
        id: `${idPrefix}-${Date.now()}`,
        role: 'user',
        content,
        ...(attachments && attachments.length > 0 ? { attachments } : {}),
      },
    ]);
  };

  const sendToAgent = async (
    text: string,
    attachments?: Attachment[],
  ): Promise<void> => {
    try {
      await invoke('agent_chat', {
        message: text,
        sessionId: hermesSessionId.value || undefined,
        history: messages.value.filter(hasContent).map((m) => ({
          role: m.role,
          content: m.content,
        })),
        attachments: attachments || [],
        contextFolder: contextFolder.value || undefined,
      });
    } catch {
      // IPC error already surfaces via agent-error event
    }
  };

  const handleSend = async (
    text: string,
    attachments?: Attachment[],
    skipLoadingCheck = false,
  ): Promise<void> => {
    const hasPayload = text.length > 0 || (attachments?.length ?? 0) > 0;
    if (!hasPayload) return;
    if (!skipLoadingCheck && isLoading.value) return;

    if (text && localCommands.isLocal(text)) {
      const cmd = text.split(/\s+/)[0].toLowerCase();
      if (cmd !== '/new' && cmd !== '/clear') pushUser(text);
      await localCommands.executeLocal(text);
      return;
    }

    isLoading.value = true;
    pushUser(text, 'user', attachments);
    onSessionStarted?.();
    await sendToAgent(text, attachments);
  };

  const handleQuickAsk = async (
    text: string,
    attachments?: Attachment[],
  ): Promise<void> => {
    if (!text || isLoading.value) return;
    isLoading.value = true;
    pushUser(`💭 ${text}`, 'user-btw', attachments);
    await sendToAgent(`/btw ${text}`, attachments);
  };

  const handleAbort = async () => {
    try {
      if (getIsClawMode()) {
        await invoke('claw_chat_abort');
      } else {
        await invoke('agent_abort_chat');
      }
    } catch {
      // Ignore abort errors
    }
    isLoading.value = false;
    setTimeout(() => inputRef.value?.focus(), 50);
  };

  const handleApprove = () => {
    inputRef.value?.clear();
    isLoading.value = true;
    pushUser('/approve', 'user-approve');
    sendToAgent('/approve').catch(() => (isLoading.value = false));
  };

  const handleDeny = () => {
    inputRef.value?.clear();
    isLoading.value = true;
    pushUser('/deny', 'user-deny');
    sendToAgent('/deny').catch(() => (isLoading.value = false));
  };

  return { handleSend, handleQuickAsk, handleAbort, handleApprove, handleDeny };
}
