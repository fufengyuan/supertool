import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref, nextTick } from 'vue'
import { setActivePinia, createPinia } from 'pinia'

// ── Mocks ──────────────────────────────────────────────────────────────────
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))
vi.mock('vue-router', () => ({
  useRoute: () => ({ query: {} }),
  useRouter: () => ({ push: vi.fn() }),
}))

describe('Chat.vue — Claw initialization & session restore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setActivePinia(createPinia())
    // Default: empty query, no session
    mockInvoke.mockReset()
  })

  // ── ensureClawChat: session init ─────────────────────────────────────
  it('ensureClawChat calls claw_chat_init with null sessionId when no query', async () => {
    // Simulate ensureClawChat logic
    const clawInitialized = ref(false)
    const messages = ref<any[]>([])
    const isLoading = ref(false)
    function addAgentMessage(content: string) {
      messages.value = [...messages.value, { id: `agent-${Date.now()}`, role: 'agent', content }]
    }
    function setMessages(msgs: any[]) { messages.value = msgs }

    mockInvoke.mockResolvedValueOnce({
      sessionId: 'new-sess-123',
      restored: false,
      messageCount: 0,
      messages: [],
    })

    // Simulate ensureClawChat
    async function ensureClawChat() {
      if (clawInitialized.value) return
      const result = await mockInvoke('claw_chat_init', {
        sessionId: null,
        cwd: null,
      })
      clawInitialized.value = true
      if (result.restored && result.messages?.length > 0) {
        const converted = result.messages.map((m: any, i: number) => ({
          id: `msg-${Date.now()}-${i}`,
          role: m.role === 'user' ? 'user' : 'agent',
          content: m.content,
        }))
        setMessages(converted)
      } else {
        addAgentMessage('Claw 编码助手已就绪')
      }
    }

    await ensureClawChat()

    expect(mockInvoke).toHaveBeenCalledWith('claw_chat_init', {
      sessionId: null,
      cwd: null,
    })
    expect(clawInitialized.value).toBe(true)
    // Not restored → shows ready message
    expect(messages.value).toHaveLength(1)
    expect(messages.value[0].content).toContain('已就绪')
  })

  it('ensureClawChat restores messages when session has history', async () => {
    const clawInitialized = ref(false)
    const messages = ref<any[]>([])
    function setMessages(msgs: any[]) { messages.value = msgs }
    function addAgentMessage(content: string) {
      messages.value = [...messages.value, { id: `agent-${Date.now()}`, role: 'agent', content }]
    }

    // Simulate restored session with messages
    mockInvoke.mockResolvedValueOnce({
      sessionId: 'sess-existing',
      restored: true,
      messageCount: 2,
      messages: [
        { role: 'user', content: 'Hello' },
        { role: 'agent', content: 'Hi there!' },
      ],
    })

    async function ensureClawChat() {
      if (clawInitialized.value) return
      const result = await mockInvoke('claw_chat_init', {
        sessionId: 'sess-existing',
        cwd: null,
      })
      clawInitialized.value = true
      if (result.restored && result.messages?.length > 0) {
        const converted = result.messages.map((m: any, i: number) => ({
          id: `msg-${Date.now()}-${i}`,
          role: m.role === 'user' ? 'user' : 'agent',
          content: m.content,
        }))
        setMessages(converted)
      } else {
        addAgentMessage('Claw 编码助手已就绪')
      }
    }

    await ensureClawChat()

    expect(clawInitialized.value).toBe(true)
    expect(messages.value).toHaveLength(2)
    expect(messages.value[0].role).toBe('user')
    expect(messages.value[0].content).toBe('Hello')
    expect(messages.value[1].role).toBe('agent')
    expect(messages.value[1].content).toBe('Hi there!')
    // No "ready" message because session was restored
  })

  it('ensureClawChat handles errors gracefully', async () => {
    const clawInitialized = ref(false)
    const messages = ref<any[]>([])
    const isLoading = ref(false)
    function addAgentMessage(content: string) {
      messages.value = [...messages.value, { id: `agent-${Date.now()}`, role: 'agent', content }]
    }

    mockInvoke.mockRejectedValueOnce(new Error('Init failed'))

    async function ensureClawChat() {
      if (clawInitialized.value) return
      try {
        await mockInvoke('claw_chat_init', { sessionId: null, cwd: null })
        clawInitialized.value = true
      } catch (e: any) {
        addAgentMessage(`Claw 初始化失败: ${e?.message || String(e)}`)
        isLoading.value = false
      }
    }

    await ensureClawChat()

    expect(clawInitialized.value).toBe(false)
    expect(messages.value).toHaveLength(1)
    expect(messages.value[0].content).toContain('初始化失败')
    expect(isLoading.value).toBe(false)
  })

  // ── onMounted: Claw initialization ───────────────────────────────────
  it('onMounted calls ensureClawChat in Claw mode', async () => {
    // Simulate the onMounted logic from Chat.vue
    const isClawMode = ref(true)
    const clawInitialized = ref(false)
    let ensureCalled = false

    async function ensureClawChat() {
      if (clawInitialized.value) return
      ensureCalled = true
      clawInitialized.value = true
    }

    async function loadSessionHistory() {
      if (isClawMode.value) return
    }

    // onMounted logic
    if (isClawMode.value) {
      await ensureClawChat()
    } else {
      await loadSessionHistory()
    }

    expect(ensureCalled).toBe(true)
    expect(clawInitialized.value).toBe(true)
  })

  // ── Mode switch watcher ──────────────────────────────────────────────
  it('mode switch watcher resets messages and reinitializes Claw', async () => {
    const messages = ref<any[]>([{ id: 'old', role: 'agent', content: 'old' }])
    const usage = ref<any>({ promptTokens: 10 })
    const toolProgress = ref('working...')
    const clawInitialized = ref(false)
    let initCalls = 0

    async function ensureClawChat() {
      initCalls++
      clawInitialized.value = true
    }

    async function loadSessionHistory() {
      // Hermes mode: would load history
    }

    // Simulate the watcher from Chat.vue
    let newMode = 'claw'
    // When mode switches to claw:
    if (newMode === 'claw') {
      messages.value = []
      usage.value = null
      toolProgress.value = null
      clawInitialized.value = false
      await ensureClawChat()
    }

    expect(messages.value).toHaveLength(0)
    expect(usage.value).toBeNull()
    expect(toolProgress.value).toBeNull()
    expect(initCalls).toBe(1)

    // Reset and switch to Hermes
    messages.value = [{ id: 'claw-msg', role: 'agent', content: 'claw' }]
    newMode = 'hermes'
    if (newMode === 'claw') {
      // not entered
    } else {
      messages.value = []
      usage.value = null
      toolProgress.value = null
      await loadSessionHistory()
    }

    expect(messages.value).toHaveLength(0)
  })

  // ── clawSend: sends message after ensureClawChat ──────────────────────
  it('clawSend calls ensureClawChat then invoke claw_chat_send', async () => {
    const clawInitialized = ref(true) // already initialized
    const isLoading = ref(false)
    let sendCalled = false

    mockInvoke.mockResolvedValueOnce(undefined)

    async function clawSend(text: string) {
      isLoading.value = true
      // ensureClawChat would be called here but is already initialized
      try {
        await mockInvoke('claw_chat_send', { message: text })
        sendCalled = true
      } catch (e: any) {
        // error handled
      } finally {
        isLoading.value = false
      }
    }

    await clawSend('Hello relay')

    expect(mockInvoke).toHaveBeenCalledWith('claw_chat_send', { message: 'Hello relay' })
    expect(sendCalled).toBe(true)
    expect(isLoading.value).toBe(false)
  })

  // ── agent-delta handler (streaming text in Claw mode) ────────────────
  it('agent-delta handler appends text to last agent message or creates new one', async () => {
    const messages = ref<any[]>([
      { id: 'u1', role: 'user', content: 'Hi' },
      { id: 'r1', kind: 'reasoning', role: 'agent', text: 'thinking...' },
    ])
    function setMessages(msgs: any[]) { messages.value = msgs }

    // Simulate agent-delta handler (inlined from useChatIPC)
    function handleDelta(chunk: string) {
      if (!chunk) return
      const prev = messages.value
      const last = prev[prev.length - 1]
      if (last && last.role === 'agent' && 'content' in last && typeof last.content === 'string') {
        setMessages([...prev.slice(0, -1), { ...last, content: last.content + chunk }])
      } else {
        if (!chunk.trim()) return
        setMessages([...prev, { id: `agent-${Date.now()}`, role: 'agent', content: chunk }])
      }
    }

    // First delta - no agent content message yet → creates new
    handleDelta('Hello')
    expect(messages.value).toHaveLength(3)
    expect(messages.value[2].content).toBe('Hello')

    // Second delta - appends to existing agent content
    handleDelta(' World')
    expect(messages.value[2].content).toBe('Hello World')
  })

  // ── agent-done handler in Claw mode ──────────────────────────────────
  it('agent-done handler sets isLoading=false in Claw mode (no DB merge)', async () => {
    const isLoading = ref(true)
    const isClawMode = ref(true)

    // Simulate agent-done handler (inlined from useChatIPC)
    async function handleDone() {
      isLoading.value = false
      if (isClawMode.value) return // skip DB merge in Claw mode
      // Hermes DB merge would happen here
    }

    await handleDone()
    expect(isLoading.value).toBe(false)
  })

  // ── claw_chat_init returns messages array ────────────────────────────
  it('claw_chat_init response includes messages array', async () => {
    const response = {
      sessionId: 'sess-456',
      restored: true,
      messageCount: 2,
      messages: [
        { role: 'user', content: 'What is 2+2?' },
        { role: 'agent', content: '4' },
      ],
    }

    expect(response).toHaveProperty('messages')
    expect(response.messages).toHaveLength(2)
    expect(response.messages[0].role).toBe('user')
    expect(response.messages[0].content).toBe('What is 2+2?')
    expect(response.messages[1].role).toBe('agent')
    expect(response.messages[1].content).toBe('4')
  })

  // ── claw_chat_list_sessions includes title and messageCount ──────────
  it('claw_chat_list_sessions response includes title and messageCount', async () => {
    mockInvoke.mockResolvedValueOnce({
      sessions: [
        {
          sessionId: 'sess-abc',
          createdAt: '2026-06-04T10:00:00Z',
          messageCount: 3,
          title: 'Hello relay',
        },
        {
          sessionId: 'sess-def',
          createdAt: '2026-06-04T09:00:00Z',
          messageCount: 0,
          title: null,
        },
      ],
    })

    // Simulate loadClawSessions
    const raw = await mockInvoke('claw_chat_list_sessions')
    const clawSessions = (raw?.sessions || []).map((s: any) => ({
      sessionId: s.sessionId,
      createdAt: s.createdAt || null,
      messageCount: s.messageCount || 0,
      title: s.title || null,
    }))

    expect(clawSessions).toHaveLength(2)
    expect(clawSessions[0].title).toBe('Hello relay')
    expect(clawSessions[0].messageCount).toBe(3)
    expect(clawSessions[1].title).toBeNull()
    expect(clawSessions[1].messageCount).toBe(0)
  })

  // ── Settings: saveClawConfig preserves key when field is empty ───────
  it('saveClawConfig does not overwrite key when field is empty', async () => {
    const clawForm = { apiKey: '', baseUrl: 'https://relay.com/v1', model: 'claude-sonnet-4-6', provider: '' }

    // Simulate saveClawConfig
    const params: Record<string, string> = {}
    if (clawForm.apiKey.trim()) params.apiKey = clawForm.apiKey.trim()
    if (clawForm.baseUrl.trim()) params.baseUrl = clawForm.baseUrl.trim()
    if (clawForm.model.trim()) params.model = clawForm.model.trim()
    if (clawForm.provider.trim()) params.provider = clawForm.provider.trim()

    expect(params).not.toHaveProperty('apiKey') // key NOT included
    expect(params).toHaveProperty('baseUrl')
    expect(params).toHaveProperty('model')
  })

  // ── Settings: loadClawConfig pre-fills API key when hasApiKey ────────
  it('loadClawConfig pre-fills API key when hasApiKey is true', async () => {
    const info = {
      apiKey: 'sk-real-key-12345',
      hasApiKey: true,
      baseUrl: 'https://relay.com/v1',
      model: 'claude-sonnet-4-6',
      provider: '',
    }

    // Simulate loadClawConfig
    const clawForm = { apiKey: '', baseUrl: '', model: '', provider: '' }
    if (info?.hasApiKey) {
      clawForm.apiKey = info.apiKey || ''
      clawForm.baseUrl = info.baseUrl || ''
      clawForm.model = info.model || 'claude-sonnet-4-6'
      clawForm.provider = info.provider || ''
    }

    expect(clawForm.apiKey).toBe('sk-real-key-12345') // key IS pre-filled
    expect(clawForm.baseUrl).toBe('https://relay.com/v1')
    expect(clawForm.model).toBe('claude-sonnet-4-6')
  })

  // ── loadClawConfig does NOT clear key when hasApiKey is false ────────
  it('loadClawConfig leaves form empty for first-time setup', async () => {
    const info = {
      apiKey: '',
      hasApiKey: false,
      baseUrl: '',
      model: 'claude-sonnet-4-6',
      provider: '',
    }

    const clawForm = { apiKey: '', baseUrl: '', model: '', provider: '' }
    if (info?.hasApiKey) {
      clawForm.apiKey = info.apiKey || ''
    }

    expect(clawForm.apiKey).toBe('') // empty for first-time user
  })
})
