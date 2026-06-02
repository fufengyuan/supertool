/**
 * useInputHistory — terminal-style command history navigation.
 *
 * Adapted from hermes-desktop's useInputHistory for Vue 3.
 * Arrow Up/Down recalls previously sent messages.
 */
import { ref } from 'vue'

export function useInputHistory() {
  const history = ref<string[]>([])
  let index = -1
  let draft = ''

  function push(text: string) {
    history.value = [...history.value, text]
    index = -1
    draft = ''
  }

  /** Recall previous entry. Returns the text to set, or null if nothing to recall. */
  function recallPrev(currentInput: string): string | null {
    if (history.value.length === 0) return null
    const cur = index
    const next = cur === -1 ? history.value.length - 1 : Math.max(0, cur - 1)
    if (cur === -1) draft = currentInput
    index = next
    return history.value[next]
  }

  /** Recall next entry. Returns the text to set, or null if at the end. */
  function recallNext(): string | null {
    if (index === -1) return null
    if (index < history.value.length - 1) {
      index++
      return history.value[index]
    }
    // Past the end — restore draft
    index = -1
    return draft
  }

  function isNavigating(): boolean {
    return index !== -1
  }

  return { push, recallPrev, recallNext, isNavigating }
}
