import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'
import type { ChatMessage, UsageState } from '../types'

// ── Tauri mocks ─────────────────────────────────────────────────────────────

type ListenCallback = (event: { payload: unknown }) => void
const listenHandlers = new Map<string, ListenCallback>()
const unlistenFns = new Map<string, ReturnType<typeof vi.fn>>()

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, callback: ListenCallback) => {
    listenHandlers.set(event, callback)
    const unlisten = vi.fn()
    unlistenFns.set(event, unlisten)
    return Promise.resolve(unlisten)
  }),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { useChatIPC } from '../composables/useChatIPC'

function setup() {
  const messages = ref<ChatMessage[]>([])
  const hermesSessionId = ref<string | null>(null)
  const toolProgress = ref<string | null>(null)
  const isLoading = ref(false)
  const usage = ref<UsageState | null>(null)

  const setMessages = vi.fn((msgs: ChatMessage[]) => {
    messages.value = msgs
  })

  return { messages, hermesSessionId, toolProgress, isLoading, usage, setMessages }
}

/** Flush all pending promises (async setup, etc.) */
async function flushPromises(times = 20) {
  for (let i = 0; i < times; i++) {
    await new Promise(resolve => setTimeout(resolve, 0))
  }
}

describe('useChatIPC', () => {
  beforeEach(() => {
    listenHandlers.clear()
    unlistenFns.clear()
    vi.clearAllMocks()
  })

  it('should register all IPC listeners', async () => {
    const args = setup()
    useChatIPC(args)
    await flushPromises()

    expect(listenHandlers.size).toBeGreaterThanOrEqual(7)
    expect(listenHandlers.has('agent-delta')).toBe(true)
    expect(listenHandlers.has('agent-reasoning-delta')).toBe(true)
    expect(listenHandlers.has('agent-tool-start')).toBe(true)
    expect(listenHandlers.has('agent-tool-complete')).toBe(true)
    expect(listenHandlers.has('agent-done')).toBe(true)
    expect(listenHandlers.has('agent-error')).toBe(true)
    expect(listenHandlers.has('agent-tool-progress')).toBe(true)
    expect(listenHandlers.has('agent-usage')).toBe(true)
  })

  it('agent-delta should append text to last agent message', async () => {
    const args = setup()
    const { messages, setMessages } = args
    messages.value = [
      { id: 'u-1', role: 'user', content: 'hi' } as ChatMessage,
      { id: 'a-1', role: 'agent', content: 'Hel' } as ChatMessage,
    ]
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-delta')
    expect(handler).toBeDefined()

    handler!({ payload: { text: 'lo', session_id: null } })
    await flushPromises()

    // setMessages should have been called with appended content
    const lastCall = setMessages.mock.calls[setMessages.mock.calls.length - 1]
    const updatedMessages = lastCall[0] as ChatMessage[]
    const lastMsg = updatedMessages[updatedMessages.length - 1]
    expect(lastMsg).toMatchObject({
      role: 'agent',
      content: 'Hello', // appended
    })
  })

  it('agent-delta should create first agent message when no agent message exists', async () => {
    const args = setup()
    const { messages } = args
    // Empty messages
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-delta')
    handler!({ payload: { text: 'Hello', session_id: null } })
    await flushPromises()

    const { setMessages } = args
    const lastCall = setMessages.mock.calls[setMessages.mock.calls.length - 1]
    const updatedMessages = lastCall[0] as ChatMessage[]
    expect(updatedMessages).toHaveLength(1)
    expect(updatedMessages[0]).toMatchObject({
      role: 'agent',
      content: 'Hello',
    })
  })

  it('agent-delta should skip null or empty text', async () => {
    const args = setup()
    const { setMessages } = args
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-delta')

    handler!({ payload: { text: null, session_id: null } })
    await flushPromises()
    expect(setMessages).not.toHaveBeenCalled()
  })

  it('agent-reasoning-delta should append to existing reasoning message', async () => {
    const args = setup()
    const { messages, setMessages } = args
    messages.value = [
      { id: 'u-1', role: 'user', content: 'think' } as ChatMessage,
      { id: 'r-1', kind: 'reasoning', role: 'agent', text: 'step 1' } as ChatMessage,
    ]
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-reasoning-delta')
    handler!({ payload: { text: ', step 2', session_id: null } })
    await flushPromises()

    const lastCall = setMessages.mock.calls[setMessages.mock.calls.length - 1]
    const updatedMessages = lastCall[0] as ChatMessage[]
    const reasoningMsg = updatedMessages.find((m) => 'kind' in m && m.kind === 'reasoning')
    expect(reasoningMsg).toBeDefined()
    expect((reasoningMsg as { text: string }).text).toBe('step 1, step 2')
  })

  it('agent-reasoning-delta should create new reasoning message when none exists', async () => {
    const args = setup()
    const { setMessages } = args
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-reasoning-delta')
    handler!({ payload: { text: 'thinking deeply', session_id: null } })
    await flushPromises()

    const lastCall = setMessages.mock.calls[setMessages.mock.calls.length - 1]
    const updatedMessages = lastCall[0] as ChatMessage[]
    const reasoningMsg = updatedMessages.find((m) => 'kind' in m && m.kind === 'reasoning')
    expect(reasoningMsg).toBeDefined()
    expect((reasoningMsg as { text: string }).text).toBe('thinking deeply')
  })

  it('agent-tool-start should add tool_call message', async () => {
    const args = setup()
    const { messages, toolProgress } = args
    messages.value = [
      { id: 'u-1', role: 'user', content: 'search' } as ChatMessage,
    ]
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-tool-start')
    handler!({ payload: { id: 'call_1', name: 'web_search', args: { q: 'test' }, session_id: null } })
    await flushPromises()

    const { setMessages } = args
    const lastCall = setMessages.mock.calls[setMessages.mock.calls.length - 1]
    const updatedMessages = lastCall[0] as ChatMessage[]
    const toolCall = updatedMessages.find((m) => 'kind' in m && m.kind === 'tool_call')
    expect(toolCall).toBeDefined()
    expect(toolProgress.value).toBe('🔧 web_search...')
  })

  it('agent-tool-complete should add tool_result message', async () => {
    const args = setup()
    const { messages, toolProgress } = args
    messages.value = [
      { id: 'u-1', role: 'user', content: 'search' } as ChatMessage,
      {
        id: 'tc-1',
        kind: 'tool_call',
        role: 'agent',
        callId: 'call_1',
        name: 'web_search',
        args: '{}',
      } as ChatMessage,
    ]
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-tool-complete')
    handler!({
      payload: {
        id: 'call_1',
        name: 'web_search',
        result: 'found results',
        duration_ms: 500,
        session_id: null,
      },
    })
    await flushPromises()

    const { setMessages } = args
    const lastCall = setMessages.mock.calls[setMessages.mock.calls.length - 1]
    const updatedMessages = lastCall[0] as ChatMessage[]
    const toolResult = updatedMessages.find((m) => 'kind' in m && m.kind === 'tool_result')
    expect(toolResult).toBeDefined()
    expect(toolProgress.value).toBeNull()
    expect((toolResult as { content: string }).content).toBe('found results')
  })

  it('agent-done should clear loading and toolProgress', async () => {
    const args = setup()
    const { isLoading, toolProgress } = args
    isLoading.value = true
    toolProgress.value = 'working...'

    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-done')
    handler!({ payload: { response: 'done', session_id: 'sess_1', message_count: 1 } })

    expect(isLoading.value).toBe(false)
    expect(toolProgress.value).toBeNull()
  })

  it('agent-done should set session id', async () => {
    const args = setup()
    const { hermesSessionId } = args
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-done')
    handler!({ payload: { response: 'done', session_id: 'sess_abc', message_count: 5 } })

    expect(hermesSessionId.value).toBe('sess_abc')
  })

  it('agent-error should add error message and clear loading', async () => {
    const args = setup()
    const { isLoading, toolProgress } = args
    isLoading.value = true
    toolProgress.value = 'running'

    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-error')
    handler!({ payload: { message: 'Rate limit exceeded', session_id: null } })

    expect(isLoading.value).toBe(false)
    expect(toolProgress.value).toBeNull()

    const { setMessages } = args
    const lastCall = setMessages.mock.calls[setMessages.mock.calls.length - 1]
    const updatedMessages = lastCall[0] as ChatMessage[]
    const errorMsg = updatedMessages.find(
      (m) => 'content' in m && (m.content as string).startsWith('Error:'),
    )
    expect(errorMsg).toBeDefined()
  })

  it('agent-tool-progress should update toolProgress', async () => {
    const args = setup()
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-tool-progress')
    handler!({ payload: '🔍 Searching...' })

    expect(args.toolProgress.value).toBe('🔍 Searching...')
  })

  it('agent-tool-progress should accept null payload', async () => {
    const args = setup()
    args.toolProgress.value = 'working'

    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-tool-progress')
    handler!({ payload: null })

    expect(args.toolProgress.value).toBeNull()
  })

  it('agent-usage should accumulate token counts', async () => {
    const args = setup()
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-usage')
    handler!({
      payload: {
        prompt_tokens: 10,
        completion_tokens: 20,
        total_tokens: 30,
        cost: 0.001,
        session_id: null,
      },
    })

    expect(args.usage.value).toMatchObject({
      promptTokens: 10,
      completionTokens: 20,
      totalTokens: 30,
      cost: 0.001,
    })
  })

  it('agent-usage should accumulate multiple events', async () => {
    const args = setup()
    useChatIPC(args)
    await flushPromises()

    const handler = listenHandlers.get('agent-usage')

    handler!({
      payload: {
        prompt_tokens: 10,
        completion_tokens: 20,
        total_tokens: 30,
        session_id: null,
      },
    })
    handler!({
      payload: {
        prompt_tokens: 5,
        completion_tokens: 10,
        total_tokens: 15,
        session_id: null,
      },
    })

    expect(args.usage.value).toMatchObject({
      promptTokens: 15,
      completionTokens: 30,
      totalTokens: 45,
    })
  })

  it('should teardown listeners on unmount', async () => {
    const args = setup()
    useChatIPC(args)
    await flushPromises()
  })

  // ── Claw mode agent-done ─────────────────────────────────────────────

  it('agent-done sets isLoading=false in Claw mode (real useChatIPC handler)', async () => {
    const args = setup()
    const { isLoading } = args
    isLoading.value = true

    // Pass isClawMode = true (as a ref, matching Chat.vue usage)
    useChatIPC({ ...args, isClawMode: ref(true) })
    await flushPromises()

    const handler = listenHandlers.get('agent-done')
    expect(handler).toBeDefined()

    // Simulate agent-done event with the exact payload shape the backend sends
    handler!({
      payload: {
        session_id: 'claw-sess-123',
      },
    })

    // CRITICAL: isLoading must become false in Claw mode
    expect(isLoading.value).toBe(false)
  })

  it('agent-done sets isLoading=false and does NOT call agent_list_messages in Claw mode', async () => {
    const args = setup()
    const { isLoading } = args
    isLoading.value = true

    const mockInvoke = vi.fn()
    vi.mocked(await import('@tauri-apps/api/core')).invoke = mockInvoke

    useChatIPC({ ...args, isClawMode: ref(true) })
    await flushPromises()

    const handler = listenHandlers.get('agent-done')
    handler!({
      payload: {
        session_id: 'claw-sess-456',
      },
    })

    // Must not call agent_list_messages (Claw has no Hermes DB)
    expect(mockInvoke).not.toHaveBeenCalledWith('agent_list_messages', expect.anything())
    expect(isLoading.value).toBe(false)
  })

  // ── FULL CLAW CONVERSATION FLOW ──────────────────────────────────────
  // Tests that simulate a complete Claw conversation from user sending
  // a message through reasoning, text chunks, completion, matching how
  // the real claw_chat_send emits events.

  it('full Claw conversation: reasoning + text + done correctly updates state', async () => {
    const args = setup()
    const { isLoading, messages, setMessages } = args
    isLoading.value = true

    useChatIPC({ ...args, isClawMode: ref(true) })
    await flushPromises()

    // Simulate pushUser(text) that Chat.vue does before clawSend
    setMessages([{
      id: 'user-1',
      role: 'user',
      content: '在吗',
    }])

    // Step 1: Reasoning chunks arrive (agent-reasoning-delta)
    const reasoningHandler = listenHandlers.get('agent-reasoning-delta')!
    reasoningHandler({ payload: { text: '我们需要理解用户的问题。', session_id: 'sess-1' } })
    reasoningHandler({ payload: { text: '用户说"在吗"是询问是否在线。', session_id: 'sess-1' } })

    // Check reasoning was added
    const reasoningMsgs = messages.value.filter(m => 'kind' in m && m.kind === 'reasoning')
    expect(reasoningMsgs.length).toBe(1)
    expect(reasoningMsgs[0].text).toContain('我们需要理解用户的问题。')

    // Step 2: Text delta arrives (agent-delta)
    const deltaHandler = listenHandlers.get('agent-delta')!
    deltaHandler({ payload: { text: '我在，', session_id: 'sess-1' } })
    deltaHandler({ payload: { text: '有什么可以帮你的？', session_id: 'sess-1' } })

    // Check text was added to agent message
    expect(messages.value.some((m) => {
      const mm = m as { content?: string }
      return mm.content === '我在，有什么可以帮你的？'
    })).toBe(true)

    // Step 3: agent-done arrives (the backend emits agent-done inside Usage)
    const doneHandler = listenHandlers.get('agent-done')!
    doneHandler({
      payload: { session_id: 'sess-1' },
    })

    // FINAL STATE CHECK: isLoading must be false
    expect(isLoading.value).toBe(false)

    // All messages in order: user, reasoning, agent content
    expect(messages.value.length).toBe(3)
    const msgs = messages.value as Array<Record<string, unknown>>
    expect(msgs[0].role).toBe('user')
    expect(msgs[0].content).toBe('在吗')
    expect(msgs[1].kind).toBe('reasoning')
    expect(msgs[2].content).toBe('我在，有什么可以帮你的？')
  })
})
