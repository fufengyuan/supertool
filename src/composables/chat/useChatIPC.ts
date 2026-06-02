/**
 * useChatIPC — Tauri IPC event listeners for chat streaming.
 *
 * Adapted from hermes-desktop's useChatIPC for Vue 3 + Tauri.
 * Listens for SSE events forwarded by the Rust backend and updates
 * messages/reactive state accordingly.
 *
 * Event names (from hermes_chat.rs):
 *   agent-delta          → text content delta
 *   agent-reasoning-delta → reasoning/thinking token delta
 *   agent-done           → stream finished
 *   agent-error          → error occurred
 *   agent-tool-start     → tool call started
 *   agent-tool-complete  → tool call completed
 *   agent-usage          → token usage stats
 */
import { ref, onUnmounted, type Ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import type { ChatMessage, UsageState } from '@/views/agent/chat/types'
import { dbItemsToChatMessages, reconcileStreamedWithDb, type DbHistoryItem } from '@/views/agent/chat/sessionHistory'

export interface UseChatIPCOptions {
  messages: Ref<ChatMessage[]>
  hermesSessionId: Ref<string | null>
  isLoading: Ref<boolean>
  usage: Ref<UsageState | null>
  toolProgress: Ref<string | null>
  scrollToBottom?: () => void
}

export function useChatIPC(options: UseChatIPCOptions) {
  const { messages, hermesSessionId, isLoading, usage, toolProgress, scrollToBottom } = options

  const listeners: UnlistenFn[] = []

  async function setup() {
    // agent-delta: text content streaming
    listeners.push(
      await listen<{ text: string | null; session_id: string | null }>('agent-delta', (event) => {
        const text = event.payload.text
        if (!text) return
        toolProgress.value = null

        const prev = messages.value
        const last = prev[prev.length - 1]

        if (last && last.role === 'agent' && 'content' in last && typeof last.content === 'string') {
          messages.value = [
            ...prev.slice(0, -1),
            { ...last, content: last.content + text },
          ]
        } else {
          if (!text.trim()) return
          messages.value = [
            ...prev,
            { id: `agent-${Date.now()}`, role: 'agent', content: text },
          ]
        }
        scrollToBottom?.()
      }),
    )

    // agent-reasoning-delta: thinking/reasoning tokens
    listeners.push(
      await listen<{ text: string | null; session_id: string | null }>('agent-reasoning-delta', (event) => {
        const text = event.payload.text
        if (!text) return

        messages.value = (() => {
          const prev = messages.value
          let insertAt = prev.length
          for (let i = prev.length - 1; i >= 0; i--) {
            const m = prev[i]
            if (m.role === 'user') break
            if ('kind' in m && m.kind === 'reasoning') {
              return [
                ...prev.slice(0, i),
                { ...m, text: m.text + text },
                ...prev.slice(i + 1),
              ]
            }
            insertAt = i
          }
          return [
            ...prev.slice(0, insertAt),
            { id: `reasoning-${Date.now()}`, kind: 'reasoning' as const, role: 'agent' as const, text },
            ...prev.slice(insertAt),
          ]
        })()
        scrollToBottom?.()
      }),
    )

    // agent-tool-start: tool call began
    listeners.push(
      await listen<{ id?: string; name: string; args: unknown; session_id: string | null; label?: string; emoji?: string }>('agent-tool-start', (event) => {
        const { id: toolId, name: toolName, label, emoji } = event.payload
        toolProgress.value = label || toolName

        // Find or create the current assistant message and append tool_call
        const prev = messages.value
        const lastIdx = prev.length - 1
        const last = lastIdx >= 0 ? prev[lastIdx] : null

        if (last && last.role === 'agent' && 'content' in last) {
          // Find the tool_call in this message or create the tool_call entry
          messages.value = [
            ...prev.slice(0, lastIdx),
            last,
            {
              id: `tc-${Date.now()}`,
              kind: 'tool_call' as const,
              role: 'agent' as const,
              callId: toolId || `tc-${Date.now()}`,
              name: toolName,
              args: typeof event.payload.args === 'string' ? event.payload.args : JSON.stringify(event.payload.args || {}),
            },
          ]
        } else {
          messages.value = [
            ...prev,
            {
              id: `agent-tc-${Date.now()}`,
              role: 'agent',
              content: '',
            },
            {
              id: `tc-${Date.now()}`,
              kind: 'tool_call' as const,
              role: 'agent' as const,
              callId: toolId || `tc-${Date.now()}`,
              name: toolName,
              args: typeof event.payload.args === 'string' ? event.payload.args : JSON.stringify(event.payload.args || {}),
            },
          ]
        }
        scrollToBottom?.()
      }),
    )

    // agent-tool-complete: tool call finished
    listeners.push(
      await listen<{ id?: string; name: string; result: string | null; duration_ms: number; session_id: string | null }>('agent-tool-complete', (event) => {
        toolProgress.value = null
        const { id: toolId, name: toolName, result, duration_ms } = event.payload

        // Find the matching tool_call and append tool_result
        const prev = messages.value
        const toolResult = {
          id: `tr-${Date.now()}`,
          kind: 'tool_result' as const,
          role: 'agent' as const,
          callId: toolId || '',
          name: toolName,
          content: result || '',
        }

        // Insert tool_result after the corresponding tool_call
        let inserted = false
        const newMsgs = [...prev]
        for (let i = newMsgs.length - 1; i >= 0; i--) {
          const m = newMsgs[i]
          if ('kind' in m && m.kind === 'tool_call' && m.callId === (toolId || m.callId) && m.name === toolName) {
            newMsgs.splice(i + 1, 0, toolResult)
            inserted = true
            break
          }
        }
        if (!inserted) {
          newMsgs.push(toolResult)
        }
        messages.value = newMsgs
        scrollToBottom?.()
      }),
    )

    // agent-done: stream finished
    listeners.push(
      await listen<{ response: string; session_id: string | null; message_count: number }>('agent-done', async (event) => {
        isLoading.value = false
        toolProgress.value = null
        if (event.payload.session_id) {
          hermesSessionId.value = event.payload.session_id
        }

        // End-of-stream merge from state.db (reasoning, tool_call, tool_result)
        if (event.payload.session_id) {
          try {
            const result = await invoke<{ success: boolean; messages: DbHistoryItem[] }>(
              'agent_list_messages',
              { sessionId: event.payload.session_id },
            )
            if (result.success && result.messages) {
              const dbMessages = dbItemsToChatMessages(result.messages)
              if (dbMessages.length > 0) {
                messages.value = reconcileStreamedWithDb(messages.value, dbMessages)
              }
            }
          } catch {
            // Merge is a UX nicety — don't break the chat flow if it fails.
          }
        }
      }),
    )

    // agent-error: error occurred
    listeners.push(
      await listen<{ message: string; session_id: string | null }>('agent-error', (event) => {
        messages.value = [
          ...messages.value,
          {
            id: `error-${Date.now()}`,
            role: 'agent',
            content: `Error: ${event.payload.message}`,
          },
        ]
        toolProgress.value = null
        isLoading.value = false
      }),
    )

    // agent-usage: token usage stats
    listeners.push(
      await listen<{ prompt_tokens: number; completion_tokens: number; total_tokens: number; cost?: number; session_id: string | null }>('agent-usage', (event) => {
        const u = event.payload
        usage.value = {
          promptTokens: (usage.value?.promptTokens || 0) + (u.prompt_tokens || 0),
          completionTokens: (usage.value?.completionTokens || 0) + (u.completion_tokens || 0),
          totalTokens: (usage.value?.totalTokens || 0) + (u.total_tokens || 0),
          cost: u.cost != null ? (usage.value?.cost || 0) + u.cost : usage.value?.cost,
        }
      }),
    )
  }

  function cleanup() {
    for (const unlisten of listeners) {
      unlisten()
    }
    listeners.length = 0
  }

  // Setup listeners on creation
  setup()

  onUnmounted(cleanup)

  return { cleanup }
}
