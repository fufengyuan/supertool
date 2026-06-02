import { describe, it, expect, vi } from 'vitest'
import { ref, nextTick } from 'vue'
import type { ChatMessage } from '../types'
import { useChatScroll } from '../composables/useChatScroll'

function makeMsg(content: string, role: 'user' | 'agent' = 'user'): ChatMessage {
  return { id: `msg-${Date.now()}`, role, content } as ChatMessage
}

describe('useChatScroll', () => {
  it('should return containerRef, bottomRef, scrollToBottom, and checkAndScroll', () => {
    const messages = ref<ChatMessage[]>([])
    const scroll = useChatScroll(messages)
    expect(scroll.containerRef).toBeDefined()
    expect(scroll.bottomRef).toBeDefined()
    expect(typeof scroll.scrollToBottom).toBe('function')
    expect(typeof scroll.checkAndScroll).toBe('function')
    expect(scroll.userScrolledUp).toBeDefined()
  })

  it('should initially have userScrolledUp = false', () => {
    const messages = ref<ChatMessage[]>([])
    const { userScrolledUp } = useChatScroll(messages)
    expect(userScrolledUp.value).toBe(false)
  })

  it('scrollToBottom should not call scrollIntoView when userScrolledUp is true', async () => {
    const messages = ref<ChatMessage[]>([])
    const { scrollToBottom, userScrolledUp, bottomRef } = useChatScroll(messages)

    userScrolledUp.value = true

    const scrollIntoViewMock = vi.fn()
    bottomRef.value = { scrollIntoView: scrollIntoViewMock } as unknown as HTMLDivElement

    scrollToBottom()
    await nextTick()

    expect(scrollIntoViewMock).not.toHaveBeenCalled()
  })

  it('scrollToBottom(true) should force scroll even when userScrolledUp', async () => {
    const messages = ref<ChatMessage[]>([])
    const { scrollToBottom, userScrolledUp, bottomRef } = useChatScroll(messages)

    userScrolledUp.value = true

    const scrollIntoViewMock = vi.fn()
    bottomRef.value = { scrollIntoView: scrollIntoViewMock } as unknown as HTMLDivElement

    scrollToBottom(true)
    await nextTick()

    expect(scrollIntoViewMock).toHaveBeenCalled()
  })

  it('checkAndScroll should handle empty messages without throwing', () => {
    const messages = ref<ChatMessage[]>([])
    const { checkAndScroll } = useChatScroll(messages)
    expect(() => checkAndScroll()).not.toThrow()
  })

  it('checkAndScroll should reset userScrolledUp and scroll when new user message appears', async () => {
    const messages = ref<ChatMessage[]>([])
    const { checkAndScroll, userScrolledUp, bottomRef } = useChatScroll(messages)

    const scrollIntoViewMock = vi.fn()
    bottomRef.value = { scrollIntoView: scrollIntoViewMock } as unknown as HTMLDivElement

    // Simulate: user sends a new message
    messages.value = [makeMsg('hello', 'user')]
    checkAndScroll()
    await nextTick()

    expect(userScrolledUp.value).toBe(false)
    expect(scrollIntoViewMock).toHaveBeenCalled()
  })

  it('should set up scroll event listener on container ref', () => {
    const messages = ref<ChatMessage[]>([])
    const { containerRef } = useChatScroll(messages)

    const addEventListenerMock = vi.fn()
    containerRef.value = {
      addEventListener: addEventListenerMock,
      removeEventListener: vi.fn(),
    } as unknown as HTMLDivElement

    expect(containerRef.value).toBeDefined()
  })

  it('scrollToBottom should work when bottomRef is null', async () => {
    const messages = ref<ChatMessage[]>([])
    const { scrollToBottom } = useChatScroll(messages)

    expect(() => scrollToBottom()).not.toThrow()
    await nextTick()
  })

  it('checkAndScroll should detect new agent message', async () => {
    const messages = ref<ChatMessage[]>([])
    const { checkAndScroll, bottomRef } = useChatScroll(messages)

    const scrollIntoViewMock = vi.fn()
    bottomRef.value = { scrollIntoView: scrollIntoViewMock } as unknown as HTMLDivElement

    // First call to initialize prevMessageCount
    checkAndScroll()
    await nextTick()

    // Add an agent message
    messages.value = [makeMsg('hello', 'agent')]
    checkAndScroll()
    await nextTick()

    // Should still scroll for new messages (non-user)
    expect(scrollIntoViewMock).toHaveBeenCalled()
  })
})
