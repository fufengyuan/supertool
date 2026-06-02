import { describe, it, expect } from 'vitest'
import { isImeComposing } from '../keyboard'

describe('isImeComposing', () => {
  it('should detect active IME composition from nativeEvent.isComposing', () => {
    expect(
      isImeComposing({
        nativeEvent: { isComposing: true },
      }),
    ).toBe(true)
  })

  it('should detect IME process key events (keyCode 229)', () => {
    expect(
      isImeComposing({
        keyCode: 229,
        nativeEvent: { isComposing: false },
      }),
    ).toBe(true)
  })

  it('should not treat regular Enter key (13) as composing', () => {
    expect(
      isImeComposing({
        keyCode: 13,
        nativeEvent: { isComposing: false },
      }),
    ).toBe(false)
  })

  it('should not treat regular key events as composing', () => {
    expect(
      isImeComposing({
        keyCode: 65,
        nativeEvent: { isComposing: false },
      }),
    ).toBe(false)
  })

  it('should handle empty nativeEvent gracefully', () => {
    expect(
      isImeComposing({
        nativeEvent: {} as unknown as { isComposing?: boolean },
      }),
    ).toBe(false)
  })

  it('should handle keyCode 0 as not composing', () => {
    expect(
      isImeComposing({
        keyCode: 0,
        nativeEvent: { isComposing: false },
      }),
    ).toBe(false)
  })
})
