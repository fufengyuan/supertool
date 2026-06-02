/**
 * useChatActions — send, abort, approve, deny operations.
 *
 * Adapted from hermes-desktop's useChatActions for Vue 3 + Tauri.
 * All callbacks have stable references (no reactive deps in closures)
 * so that child components don't re-render on streaming chunks.
 */
import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ChatMessage, Attachment } from '@/views/agent/chat/types'

export interface UseChatActionsOptions {
  messages: Ref<ChatMessage[]>
  hermesSessionId: Ref<string | null>
  isLoading: Ref<boolean>
  contextFolder: Ref<string | null>
  scrollToBottom?: () => void
}

export function useChatActions(options: UseChatActionsOptions) {
  const { messages, hermesSessionId, isLoading, contextFolder, scrollToBottom } = options

  function pushUser(content: string, idPrefix = 'user', attachments?: Attachment[]) {
    messages.value = [
      ...messages.value,
      {
        id: `${idPrefix}-${Date.now()}`,
        role: 'user',
        content,
        ...(attachments && attachments.length > 0 ? { attachments } : {}),
      },
    ]
  }

  async function sendToAgent(text: string, attachments?: Attachment[]) {
    try {
      await invoke('agent_chat', {
        message: text,
        sessionId: hermesSessionId.value || undefined,
        model: undefined,
        toolsets: undefined,
        contextFolder: contextFolder.value || undefined,
      })
    } catch {
      // IPC error — the backend emits agent-error which the IPC listener handles
    }
  }

  async function handleSend(text: string, attachments?: Attachment[]) {
    const hasPayload = text.length > 0 || (attachments?.length ?? 0) > 0
    if (!hasPayload) return
    if (isLoading.value) return

    isLoading.value = true
    pushUser(text, 'user', attachments)
    scrollToBottom?.()
    await sendToAgent(text, attachments)
  }

  async function handleQuickAsk(text: string, attachments?: Attachment[]) {
    if (!text || isLoading.value) return
    isLoading.value = true
    pushUser(`💭 ${text}`, 'user-btw', attachments)
    scrollToBottom?.()
    await sendToAgent(`/btw ${text}`, attachments)
  }

  function handleAbort() {
    invoke('agent_abort_chat').catch(() => {})
    isLoading.value = false
  }

  function handleApprove() {
    isLoading.value = true
    pushUser('/approve', 'user-approve')
    sendToAgent('/approve').catch(() => { isLoading.value = false })
  }

  function handleDeny() {
    isLoading.value = true
    pushUser('/deny', 'user-deny')
    sendToAgent('/deny').catch(() => { isLoading.value = false })
  }

  return {
    handleSend,
    handleQuickAsk,
    handleAbort,
    handleApprove,
    handleDeny,
  }
}
