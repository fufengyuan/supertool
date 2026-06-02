import { describe, it, expect, vi } from 'vitest'
import { ref } from 'vue'
import { useInputHistory } from '../composables/useInputHistory'

describe('useInputHistory', () => {
  function setup(initialText = '') {
    const currentInput = ref(initialText)
    const applyText = vi.fn()
    const history = useInputHistory({ currentInput, applyText })
    return { currentInput, applyText, history }
  }

  describe('push', () => {
    it('should add text to history', () => {
      const { history } = setup()
      history.push('hello')
      expect(history.size()).toBe(1)
    })

    it('should add multiple texts in order', () => {
      const { history } = setup()
      history.push('first')
      history.push('second')
      history.push('third')
      expect(history.size()).toBe(3)
    })

    it('should reset navigation state after push', () => {
      const { history, applyText } = setup()
      history.push('first')
      history.push('second')
      // Navigate back
      history.recallPrev()
      expect(history.isNavigating()).toBe(true)
      // Push resets navigation
      history.push('third')
      expect(history.isNavigating()).toBe(false)
    })

    it('should not be navigating after push', () => {
      const { history } = setup()
      expect(history.isNavigating()).toBe(false)
      history.push('msg')
      expect(history.isNavigating()).toBe(false)
    })
  })

  describe('recallPrev / recallNext', () => {
    it('should recall previous entries in reverse order', () => {
      const { history, applyText } = setup()
      history.push('first')
      history.push('second')
      history.push('third')

      history.recallPrev()
      expect(applyText).toHaveBeenCalledWith('third')
      applyText.mockClear()

      history.recallPrev()
      expect(applyText).toHaveBeenCalledWith('second')
      applyText.mockClear()

      history.recallPrev()
      expect(applyText).toHaveBeenCalledWith('first')
    })

    it('should return false when no history to recall', () => {
      const { history } = setup()
      expect(history.recallPrev()).toBe(false)
      expect(history.recallNext()).toBe(false)
    })

    it('should cycle through history with recallNext', () => {
      const { history, applyText } = setup('current draft')
      history.push('a')
      history.push('b')
      history.push('c')

      // Go back to start
      history.recallPrev() // c
      history.recallPrev() // b
      history.recallPrev() // a
      applyText.mockClear()

      // Go forward
      history.recallNext() // b
      expect(applyText).toHaveBeenCalledWith('b')
      applyText.mockClear()

      history.recallNext() // c
      expect(applyText).toHaveBeenCalledWith('c')
      applyText.mockClear()

      // Past end should restore draft
      history.recallNext()
      expect(applyText).toHaveBeenCalledWith('current draft')
    })

    it('should save current input as draft before navigating', () => {
      const { history, applyText, currentInput } = setup('current draft')
      history.push('first')
      history.push('second')

      // First recallPrev saves draft and navigates to 'second'
      history.recallPrev()
      expect(applyText).toHaveBeenNthCalledWith(1, 'second')
      applyText.mockClear()

      // Second recallPrev navigates to 'first'
      history.recallPrev()
      expect(applyText).toHaveBeenNthCalledWith(1, 'first')
      applyText.mockClear()

      // Third recallPrev stays at 'first' (at beginning)
      history.recallPrev()
      expect(applyText).toHaveBeenNthCalledWith(1, 'first')
    })

    it('should return false for recallNext when not navigating', () => {
      const { history } = setup()
      expect(history.recallNext()).toBe(false)
    })

    it('should handle empty history gracefully', () => {
      const { history } = setup()
      expect(history.recallPrev()).toBe(false)
      expect(history.recallNext()).toBe(false)
      expect(history.isNavigating()).toBe(false)
      expect(history.size()).toBe(0)
    })
  })
})
