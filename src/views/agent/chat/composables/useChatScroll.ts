import { ref, onMounted, onUnmounted, nextTick, type Ref } from 'vue';
import type { ChatMessage } from '../types';

/**
 * Auto-scroll behavior for the chat messages container.
 *
 * - Tracks whether the user has manually scrolled up; pauses auto-scroll.
 * - Re-engages auto-scroll when a new user message is sent.
 * - Exposes the container ref and a bottom sentinel ref.
 */
export function useChatScroll(messages: Ref<ChatMessage[]>) {
  const containerRef = ref<HTMLDivElement | null>(null);
  const bottomRef = ref<HTMLDivElement | null>(null);
  const userScrolledUp = ref(false);
  const prevMessageCount = ref(0);

  const scrollToBottom = (force = false) => {
    if (!force && userScrolledUp.value) return;
    nextTick(() => {
      bottomRef.value?.scrollIntoView({ behavior: 'smooth' });
    });
  };

  const handleScroll = () => {
    const el = containerRef.value;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 60;
    userScrolledUp.value = !atBottom;
  };

  onMounted(() => {
    containerRef.value?.addEventListener('scroll', handleScroll, { passive: true });
  });

  onUnmounted(() => {
    containerRef.value?.removeEventListener('scroll', handleScroll);
  });

  // Watch messages for auto-scroll
  const stopWatch = ref<(() => void) | null>(null);

  const setupWatch = () => {
    // We'll call this from the component after messages ref is stable
    // Using a simple polling approach via a watch-like mechanism
  };

  // Track message count changes for auto-scroll
  const checkAndScroll = () => {
    const currentCount = messages.value.length;
    const prevCount = prevMessageCount.value;
    prevMessageCount.value = currentCount;

    const userJustSent =
      currentCount > prevCount &&
      messages.value[currentCount - 1]?.role === 'user';

    if (userJustSent) {
      userScrolledUp.value = false;
      scrollToBottom(true);
    } else {
      scrollToBottom();
    }
  };

  return {
    containerRef,
    bottomRef,
    scrollToBottom,
    checkAndScroll,
    userScrolledUp,
  };
}
