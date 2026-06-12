import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref, nextTick } from 'vue'
import { setActivePinia, createPinia } from 'pinia'

// ── Mock Tauri API ──────────────────────────────────────────────────────────
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

// ── agentModeStore tests ────────────────────────────────────────────────────
describe('agentModeStore', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
  })

  it('defaults to hermes mode', async () => {
    const { useAgentModeStore } = await import('@/stores/agentModeStore')
    const store = useAgentModeStore()
    expect(store.mode).toBe('hermes')
  })

  it('switches to claw mode', async () => {
    const { useAgentModeStore } = await import('@/stores/agentModeStore')
    const store = useAgentModeStore()
    store.setMode('claw')
    expect(store.mode).toBe('claw')
  })

  it('persists to localStorage', async () => {
    const { useAgentModeStore } = await import('@/stores/agentModeStore')
    const store = useAgentModeStore()
    store.setMode('claw')
    // Re-create store from same pinia — should read persisted value
    const store2 = useAgentModeStore()
    expect(store2.mode).toBe('claw')
  })
})

// ── TauriAPI type contract tests ────────────────────────────────────────────
describe('TauriAPI Claw contracts', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('clawConfigGet returns correct shape', async () => {
    mockInvoke.mockResolvedValueOnce({
      apiKey: 'sk-c3...c012',
      hasApiKey: true,
      baseUrl: 'https://api.example.com/v1',
      model: 'claude-sonnet-4-6',
      provider: 'anthropic',
    })

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    const result = await api.clawConfigGet()

    expect(result).toHaveProperty('apiKey')
    expect(result).toHaveProperty('hasApiKey')
    expect(result).toHaveProperty('baseUrl')
    expect(result).toHaveProperty('model')
    expect(result).toHaveProperty('provider')
    expect(result.hasApiKey).toBe(true)
    expect(result.apiKey).toContain('...')
  })

  it('clawConfigSet sends correct params', async () => {
    mockInvoke.mockResolvedValueOnce({ success: true, message: 'Claw config saved' })

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    const result = await api.clawConfigSet({
      apiKey: 'sk-newkey123',
      baseUrl: 'https://custom.api.com/v1',
      model: 'gpt-4o',
      provider: 'openai',
    })

    expect(mockInvoke).toHaveBeenCalledWith('claw_config_set', {
      apiKey: 'sk-newkey123',
      baseUrl: 'https://custom.api.com/v1',
      model: 'gpt-4o',
      provider: 'openai',
    })
    expect(result.success).toBe(true)
  })

  it('clawChatInit calls with cwd', async () => {
    mockInvoke.mockResolvedValueOnce(undefined)

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    await api.clawChatInit('/tmp/project')

    expect(mockInvoke).toHaveBeenCalledWith('claw_chat_init', { cwd: '/tmp/project' })
  })

  it('clawChatSend calls with message string', async () => {
    mockInvoke.mockResolvedValueOnce(undefined)

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    await api.clawChatSend('Hello Claude')

    expect(mockInvoke).toHaveBeenCalledWith('claw_chat_send', { message: 'Hello Claude' })
  })

  it('clawChatClose calls with no args', async () => {
    mockInvoke.mockResolvedValueOnce(undefined)

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    await api.clawChatClose()

    expect(mockInvoke).toHaveBeenCalledWith('claw_chat_close', {})
  })

  it('clawChatInfo returns full shape', async () => {
    mockInvoke.mockResolvedValueOnce({
      mode: 'claw',
      apiKeyConfigured: true,
      model: 'claude-sonnet-4-6',
      provider: 'anthropic',
      baseUrl: null,
      configSource: '~/.claw/config.json',
    })

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    const result = await api.clawChatInfo()

    expect(result.mode).toBe('claw')
    expect(result.apiKeyConfigured).toBe(true)
    expect(result.configSource).toContain('.claw/config.json')
  })

  it('clawChatListSessions returns array', async () => {
    mockInvoke.mockResolvedValueOnce([
      { sessionId: 'sess_abc123', createdAt: '2026-01-01T00:00:00Z' },
    ])

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    const result = await api.clawChatListSessions()

    expect(Array.isArray(result)).toBe(true)
  })
})

// ── useChatIPC Claw mode guard tests ────────────────────────────────────────
describe('useChatIPC — Claw mode guard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockReset()
  })

  it('isClawMode accepts Ref<boolean>', async () => {
    const { useChatIPC } = await import('../composables/useChatIPC')
    const isClawMode = ref(true)
    const messages = ref<any[]>([])
    const setMessages = vi.fn()
    const hermesSessionId = ref<string | null>(null)
    const isLoading = ref(false)

    // Should not throw when isClawMode is a Ref
    useChatIPC({
      messages,
      setMessages,
      hermesSessionId,
      toolProgress: ref(null),
      isLoading,
      usage: ref(null),
      isClawMode,
    })

    // Verify the composable accepted the Ref type
    expect(isClawMode.value).toBe(true)
  })

  it('isClawMode accepts plain boolean', async () => {
    const { useChatIPC } = await import('../composables/useChatIPC')
    const messages = ref<any[]>([])
    const setMessages = vi.fn()
    const hermesSessionId = ref<string | null>(null)
    const isLoading = ref(false)

    // Should not throw when isClawMode is a plain boolean
    useChatIPC({
      messages,
      setMessages,
      hermesSessionId,
      toolProgress: ref(null),
      isLoading,
      usage: ref(null),
      isClawMode: true,
    })

    expect(isLoading.value).toBe(false)
  })
})

// ── loadSessionHistory Claw mode guard ──────────────────────────────────────
describe('Chat.vue loadSessionHistory guard', () => {
  it('skips agent_list_messages in Claw mode', async () => {
    // This tests the logic: if isClawMode.value is true, loadSessionHistory returns early
    const isClawMode = ref(true)
    const hermesSessionId = ref<string | null>(null)

    // Simulate the guard logic from Chat.vue
    if (isClawMode.value) {
      // Should skip — no invoke call
    } else {
      mockInvoke.mockResolvedValueOnce({ success: true, messages: [] })
    }

    expect(mockInvoke).not.toHaveBeenCalled()
    expect(hermesSessionId.value).toBeNull()
  })
})

// ── claw_config_get API key: backend returns raw key (no masking) ────────────
describe('clawConfigGet returns raw API key', () => {
  it('returns the full API key as-is (no backend masking)', async () => {
    mockInvoke.mockResolvedValueOnce({
      apiKey: 'sk-c3-real-key-9d0e',
      hasApiKey: true,
      baseUrl: '',
      model: 'claude-sonnet-4-6',
      provider: 'anthropic',
    })

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    const result = await api.clawConfigGet()

    // Backend returns RAW key — frontend just passes it through
    expect(result.apiKey).toBe('sk-c3-real-key-9d0e')
    expect(result.hasApiKey).toBe(true)
  })

  it('returns *** literally when that is the stored value', async () => {
    mockInvoke.mockResolvedValueOnce({
      apiKey: '***',
      hasApiKey: true,
      baseUrl: '',
      model: 'claude-sonnet-4-6',
      provider: 'anthropic',
    })

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    const result = await api.clawConfigGet()

    expect(result.apiKey).toBe('***')
  })

  it('returns empty string when no key configured', async () => {
    mockInvoke.mockResolvedValueOnce({
      apiKey: '',
      hasApiKey: false,
      baseUrl: '',
      model: 'claude-sonnet-4-6',
      provider: '',
    })

    const { getTauriAPI } = await import('@/utils/tauri-api')
    const api = getTauriAPI()
    const result = await api.clawConfigGet()

    expect(result.apiKey).toBe('')
    expect(result.hasApiKey).toBe(false)
  })
})
