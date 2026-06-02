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
      content: 'Helo', // appended
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
})
