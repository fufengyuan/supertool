import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref, nextTick } from 'vue'
import type { ChatMessage } from '../types'

const mockedInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockedInvoke(...args),
}))

import { useChatActions } from '../composables/useChatActions'

function setup() {
  const hermesSessionId = ref<string | null>(null)
  const messages = ref<ChatMessage[]>([])
  const isLoading = ref(false)
  const contextFolder = ref<string | null>(null)
  const setMessages = vi.fn((msgs: ChatMessage[]) => {
    messages.value = msgs
  })
  const onSessionStarted = vi.fn()
  const scrollToBottom = vi.fn()
  const inputRef = ref<{ clear: () => void; focus: () => void } | null>({
    clear: vi.fn(),
    focus: vi.fn(),
  })

  const localCommands = {
    isLocal: vi.fn().mockReturnValue(false),
    executeLocal: vi.fn().mockResolvedValue(true),
  }

  const actions = useChatActions({
    hermesSessionId,
    messages,
    setMessages,
    isLoading,
    onSessionStarted,
    localCommands,
    contextFolder,
    scrollToBottom,
    inputRef,
  })

  return {
    hermesSessionId,
    messages,
    isLoading,
    contextFolder,
    setMessages,
    onSessionStarted,
    scrollToBottom,
    inputRef,
    localCommands,
    actions,
  }
}

describe('useChatActions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('handleSend', () => {
    it('should not send empty text without attachments', async () => {
      const { actions, isLoading, setMessages } = setup()
      await actions.handleSend('')
      expect(isLoading.value).toBe(false)
      expect(setMessages).not.toHaveBeenCalled()
    })

    it('should not send when already loading', async () => {
      const { actions, isLoading, setMessages } = setup()
      isLoading.value = true
      await actions.handleSend('hello')
      expect(setMessages).not.toHaveBeenCalled()
    })

    it('should send when skipLoadingCheck is true even if loading', async () => {
      const { actions, isLoading, setMessages } = setup()
      isLoading.value = true
      mockedInvoke.mockResolvedValue({})

      await actions.handleSend('hello', undefined, true)

      expect(setMessages).toHaveBeenCalled()
      expect(mockedInvoke).toHaveBeenCalled()
    })

    it('should send user message to agent', async () => {
      const { actions, isLoading, setMessages } = setup()
      mockedInvoke.mockResolvedValue({})

      await actions.handleSend('hello')

      expect(isLoading.value).toBe(true)
      expect(setMessages).toHaveBeenCalled()
      // History contains the pushed user message
      expect(mockedInvoke).toHaveBeenCalledWith('agent_chat', {
        message: 'hello',
        sessionId: undefined,
        history: [{ role: 'user', content: 'hello' }],
        attachments: [],
        contextFolder: undefined,
      })
    })

    it('should include attachments when provided', async () => {
      const { actions, setMessages } = setup()
      mockedInvoke.mockResolvedValue({})

      const attachment = {
        id: 'att-1',
        kind: 'image' as const,
        name: 'photo.png',
        mime: 'image/png',
        size: 1024,
        dataUrl: 'data:image/png;base64,...',
      }

      await actions.handleSend('check this', [attachment])

      expect(mockedInvoke).toHaveBeenCalledWith('agent_chat', {
        message: 'check this',
        sessionId: undefined,
        history: [{ role: 'user', content: 'check this' }],
        attachments: [attachment],
        contextFolder: undefined,
      })
    })

    it('should handle local commands', async () => {
      const { actions, localCommands } = setup()
      localCommands.isLocal.mockReturnValue(true)
      localCommands.executeLocal.mockResolvedValue(true)

      await actions.handleSend('/help')

      expect(localCommands.executeLocal).toHaveBeenCalledWith('/help')
      expect(mockedInvoke).not.toHaveBeenCalled()
    })

    it('should push user message for local commands except /new and /clear', async () => {
      const { actions, localCommands, setMessages } = setup()
      localCommands.isLocal.mockReturnValue(true)

      await actions.handleSend('/help')
      expect(setMessages).toHaveBeenCalled()
    })

    it('should pass session id when available', async () => {
      const { actions, hermesSessionId } = setup()
      hermesSessionId.value = 'sess_existing'
      mockedInvoke.mockResolvedValue({})

      await actions.handleSend('continue')

      expect(mockedInvoke).toHaveBeenCalledWith('agent_chat', {
        message: 'continue',
        sessionId: 'sess_existing',
        history: [{ role: 'user', content: 'continue' }],
        attachments: [],
        contextFolder: undefined,
      })
    })

    it('should include context folder when set', async () => {
      const { actions, contextFolder } = setup()
      contextFolder.value = '/path/to/project'
      mockedInvoke.mockResolvedValue({})

      await actions.handleSend('read project')

      const invokeCall = mockedInvoke.mock.calls[0]
      expect(invokeCall[1].contextFolder).toBe('/path/to/project')
    })
  })

  describe('handleQuickAsk', () => {
    it('should send quick ask with /btw prefix', async () => {
      const { actions, isLoading, setMessages } = setup()
      mockedInvoke.mockResolvedValue({})

      await actions.handleQuickAsk('what is the weather?')

      expect(isLoading.value).toBe(true)
      expect(setMessages).toHaveBeenCalled()
      expect(mockedInvoke).toHaveBeenCalledWith('agent_chat', {
        message: '/btw what is the weather?',
        sessionId: undefined,
        history: [{ role: 'user', content: '💭 what is the weather?' }],
        attachments: [],
        contextFolder: undefined,
      })
    })

    it('should not send empty quick ask', async () => {
      const { actions, isLoading, setMessages } = setup()
      await actions.handleQuickAsk('')
      expect(isLoading.value).toBe(false)
      expect(setMessages).not.toHaveBeenCalled()
    })

    it('should not send when loading', async () => {
      const { actions, isLoading } = setup()
      isLoading.value = true
      await actions.handleQuickAsk('question')
      expect(mockedInvoke).not.toHaveBeenCalled()
    })
  })

  describe('handleAbort', () => {
    it('should call agent_abort_chat and reset loading', async () => {
      const { actions, isLoading, inputRef } = setup()
      isLoading.value = true
      mockedInvoke.mockResolvedValue({})

      await actions.handleAbort()

      expect(mockedInvoke).toHaveBeenCalledWith('agent_abort_chat')
      expect(isLoading.value).toBe(false)
    })

    it('should handle abort errors gracefully', async () => {
      const { actions, isLoading } = setup()
      mockedInvoke.mockRejectedValue(new Error('abort error'))

      await actions.handleAbort()

      expect(isLoading.value).toBe(false)
    })
  })

  describe('handleApprove', () => {
    it('should send /approve to agent', async () => {
      const { actions, isLoading, setMessages, inputRef } = setup()
      mockedInvoke.mockResolvedValue({})

      await actions.handleApprove()

      expect(isLoading.value).toBe(true)
      expect(setMessages).toHaveBeenCalled()
      expect(mockedInvoke).toHaveBeenCalledWith('agent_chat', {
        message: '/approve',
        sessionId: undefined,
        history: [{ role: 'user', content: '/approve' }],
        attachments: [],
        contextFolder: undefined,
      })
    })

    it('should clear input on approve', async () => {
      const { actions, inputRef } = setup()
      mockedInvoke.mockResolvedValue({})

      await actions.handleApprove()
      expect(inputRef.value?.clear).toHaveBeenCalled()
    })
  })

  describe('handleDeny', () => {
    it('should send /deny to agent', async () => {
      const { actions, isLoading, setMessages } = setup()
      mockedInvoke.mockResolvedValue({})

      await actions.handleDeny()

      expect(isLoading.value).toBe(true)
      expect(setMessages).toHaveBeenCalled()
      expect(mockedInvoke).toHaveBeenCalledWith('agent_chat', {
        message: '/deny',
        sessionId: undefined,
        history: [{ role: 'user', content: '/deny' }],
        attachments: [],
        contextFolder: undefined,
      })
    })
  })
})
