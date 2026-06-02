/**
 * useChatScroll — smart auto-scroll for the chat messages container.
 *
 * Adapted from hermes-desktop's useChatScroll for Vue 3:
 * - Tracks whether the user has manually scrolled up.
 * - Re-engages auto-scroll when a new user message is sent.
 * - Exposes containerRef and scrollToBottom for the template.
 */
import { ref, watch, onMounted, onUnmounted, type Ref } from 'vue'
import type { ChatMessage } from '@/views/agent/chat/types'

export function useChatScroll(messages: Ref<ChatMessage[]>) {
  const containerRef = ref<HTMLDivElement | null>(null)
  const userScrolledUp = ref(false)
  const prevMessageCount = ref(0)

  function scrollToBottom(force = false) {
    if (!force && userScrolledUp.value) return
    const el = containerRef.value
    if (el) {
      el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' })
    }
  }

  function handleScroll() {
    const el = containerRef.value
    if (!el) return
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 60
    userScrolledUp.value = !atBottom
  }

  onMounted(() => {
    containerRef.value?.addEventListener('scroll', handleScroll, { passive: true })
  })

  onUnmounted(() => {
    containerRef.value?.removeEventListener('scroll', handleScroll)
  })

  // Auto-scroll on incoming messages; force-scroll when user sends a new one.
  watch(messages, (newMsgs) => {
    const prev = prevMessageCount.value
    prevMessageCount.value = newMsgs.length
    const userJustSent =
      newMsgs.length > prev &&
      newMsgs[newMsgs.length - 1]?.role === 'user'
    if (userJustSent) {
      userScrolledUp.value = false
      scrollToBottom(true)
    } else {
      scrollToBottom()
    }
  })

  return {
    containerRef,
    scrollToBottom,
    userScrolledUp,
  }
}
