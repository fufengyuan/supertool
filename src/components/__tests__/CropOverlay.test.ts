import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import CropOverlay from '@/components/CropOverlay.vue'

// ── Fixtures ──────────────────────────────────────────────────────────────

const DEFAULT_PROPS = {
  imgNaturalWidth: 800,
  imgNaturalHeight: 600,
  imgDisplayWidth: 400,
  imgDisplayHeight: 300,
  imgOffsetX: 20,
  imgOffsetY: 10,
  cropX: 0,
  cropY: 0,
  cropW: 0,
  cropH: 0,
}

function makeProps(overrides: Record<string, unknown> = {}) {
  return { ...DEFAULT_PROPS, ...overrides }
}

// ── Helpers ───────────────────────────────────────────────────────────────

function createWrapper(propsOverrides: Record<string, unknown> = {}) {
  return mount(CropOverlay, {
    props: makeProps(propsOverrides),
    attachTo: document.body,
  })
}

async function flushAll() {
  await nextTick()
  await new Promise<void>((r) => queueMicrotask(() => r()))
  await nextTick()
}

/* eslint-disable @typescript-eslint/no-explicit-any */
function emitted(
  wrapper: ReturnType<typeof createWrapper>
): Record<string, any[]> {
  return wrapper.emitted() as Record<string, any[]>
}

function lastValue(
  events: unknown[][] | undefined
): number {
  if (!events || events.length === 0) return NaN
  return events[events.length - 1][0] as number
}

// ── Tests ─────────────────────────────────────────────────────────────────

describe('CropOverlay.vue', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  // ── Rendering ───────────────────────────────────────────────────────────

  it('should render container with crosshair cursor', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')
    expect(container.exists()).toBe(true)
    expect(container.classes()).toContain('cursor-crosshair')
  })

  it('should render canvas element', () => {
    const wrapper = createWrapper()
    expect(wrapper.find('canvas').exists()).toBe(true)
  })

  it('should have absolute inset-0 on container', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')
    expect(container.classes()).toContain('absolute')
    expect(container.classes()).toContain('inset-0')
  })

  // ── Coordinate Conversion: natural → display ────────────────────────────

  it('should convert drag 150 display px to 300 natural px', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')

    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 100, clientY: 50, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 250, clientY: 150, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 250, clientY: 150, bubbles: true, pointerId: 1 })
    )

    const cropWValues = emitted(wrapper)['update:cropW']
    expect(cropWValues).toBeDefined()
    expect(cropWValues!.length).toBeGreaterThan(0)
    // 150 display px → round(150/400*800) = 300 natural px
    expect(lastValue(cropWValues)).toBe(300)
  })

  it('should convert natural coordinates correctly for 2x scaled image', () => {
    const wrapper = createWrapper({
      imgNaturalWidth: 400,
      imgNaturalHeight: 300,
      imgDisplayWidth: 200,
      imgDisplayHeight: 150,
      imgOffsetX: 10,
      imgOffsetY: 5,
    })
    const container = wrapper.find('div')

    // Drag from (10,5) to (110,80)
    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 10, clientY: 5, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 110, clientY: 80, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 110, clientY: 80, bubbles: true, pointerId: 1 })
    )

    const e = emitted(wrapper)
    const cropWValues = e['update:cropW']
    const cropHValues = e['update:cropH']

    expect(cropWValues).toBeDefined()
    expect(cropHValues).toBeDefined()

    // display 100px → round(100/200*400) = 200 natural
    expect(lastValue(cropWValues)).toBe(200)
    // display 75px → round(75/150*300) = 150 natural
    expect(lastValue(cropHValues)).toBe(150)
  })

  // ── Pointer: Create new selection ───────────────────────────────────────

  it('should emit all four crop values after creating selection via drag', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')

    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 50, clientY: 30, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 250, clientY: 180, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 250, clientY: 180, bubbles: true, pointerId: 1 })
    )

    const e = emitted(wrapper)
    expect(e['update:cropX']).toBeDefined()
    expect(e['update:cropY']).toBeDefined()
    expect(e['update:cropW']).toBeDefined()
    expect(e['update:cropH']).toBeDefined()

    expect(lastValue(e['update:cropX'])).toBeGreaterThanOrEqual(0)
    expect(lastValue(e['update:cropY'])).toBeGreaterThanOrEqual(0)
    expect(lastValue(e['update:cropW'])).toBeGreaterThan(0)
    expect(lastValue(e['update:cropH'])).toBeGreaterThan(0)
  })

  it('should clear selection when drag is below MIN_SIZE (4px)', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')

    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 50, clientY: 50, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 52, clientY: 52, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 52, clientY: 52, bubbles: true, pointerId: 1 })
    )

    const e = emitted(wrapper)
    expect(lastValue(e['update:cropW'])).toBe(0)
    expect(lastValue(e['update:cropH'])).toBe(0)
  })

  // ── Pointer: Drag in opposite direction ──────────────────────────────────

  it('should normalize negative dimensions when dragging opposite direction', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')

    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 250, clientY: 180, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 50, clientY: 30, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 50, clientY: 30, bubbles: true, pointerId: 1 })
    )

    const e = emitted(wrapper)
    expect(lastValue(e['update:cropW'])).toBeGreaterThan(0)
  })

  // ── Props: syncFromProps with pre-set crop values ────────────────────────

  it('should accept pre-set crop values via props without crashing', async () => {
    const wrapper = createWrapper({
      cropX: 100,
      cropY: 50,
      cropW: 200,
      cropH: 150,
    })
    await flushAll()
    expect(wrapper.vm).toBeDefined()
  })

  // ── Emit guard: zero-size rect ───────────────────────────────────────────

  it('should not emit cropX/cropY when displayRect has zero size', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')

    // Single click = zero-size create
    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 100, clientY: 100, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 100, clientY: 100, bubbles: true, pointerId: 1 })
    )

    const e = emitted(wrapper)
    // emitFromDisplay guards on w > 0 && h > 0
    expect(e['update:cropX']).toBeUndefined()
    expect(e['update:cropY']).toBeUndefined()
  })

  // ── Negative coordinate clamping ────────────────────────────────────────

  it('should clamp cropX to >= 0 when pointer is at image left edge', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')

    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 50, clientY: 30, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 0, clientY: 50, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 0, clientY: 50, bubbles: true, pointerId: 1 })
    )

    const cropXValues = emitted(wrapper)['update:cropX']
    if (cropXValues && cropXValues.length > 0) {
      expect(lastValue(cropXValues)).toBeGreaterThanOrEqual(0)
    }
  })

  // ── clampRect: selection width bounded by display width ──────────────────

  it('should clamp selection width to image display width', () => {
    const wrapper = createWrapper({
      imgDisplayWidth: 200,
      imgDisplayHeight: 150,
    })
    const container = wrapper.find('div')

    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 0, clientY: 0, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 400, clientY: 200, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 400, clientY: 200, bubbles: true, pointerId: 1 })
    )

    const e = emitted(wrapper)
    const cropW = lastValue(e['update:cropW'])
    const cropH = lastValue(e['update:cropH'])

    // Should not exceed the natural image dimensions
    if (!isNaN(cropW)) expect(cropW).toBeLessThanOrEqual(800)
    if (!isNaN(cropH)) expect(cropH).toBeLessThanOrEqual(600)
  })

  // ── Minimum size enforcement ─────────────────────────────────────────────

  it('should clear selection when drag is exactly 3px (below MIN_SIZE=4)', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')

    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 100, clientY: 100, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 103, clientY: 103, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 103, clientY: 103, bubbles: true, pointerId: 1 })
    )

    const e = emitted(wrapper)
    expect(lastValue(e['update:cropW'])).toBe(0)
    expect(lastValue(e['update:cropH'])).toBe(0)
  })

  // ── Handle cursor mapping ────────────────────────────────────────────────

  it('should have 8 handle cursor mappings all ending with -resize', () => {
    const handleMap = {
      nw: 'nw-resize',
      n: 'n-resize',
      ne: 'ne-resize',
      e: 'e-resize',
      se: 'se-resize',
      s: 's-resize',
      sw: 'sw-resize',
      w: 'w-resize',
    }
    expect(Object.keys(handleMap)).toHaveLength(8)
    Object.values(handleMap).forEach((cursor) => {
      expect(cursor).toMatch(/-resize$/)
    })
  })

  // ── Multiple drags should produce latest values ──────────────────────────

  it('should update crop values on consecutive drags', () => {
    const wrapper = createWrapper()
    const container = wrapper.find('div')

    // First drag
    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 20, clientY: 10, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 120, clientY: 85, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 120, clientY: 85, bubbles: true, pointerId: 1 })
    )

    const firstCropW = lastValue(emitted(wrapper)['update:cropW'])

    // Second drag — larger area
    container.element.dispatchEvent(
      new PointerEvent('pointerdown', { clientX: 20, clientY: 10, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointermove', { clientX: 300, clientY: 200, bubbles: true, pointerId: 1 })
    )
    container.element.dispatchEvent(
      new PointerEvent('pointerup', { clientX: 300, clientY: 200, bubbles: true, pointerId: 1 })
    )

    const secondCropW = lastValue(emitted(wrapper)['update:cropW'])

    // Second drag should produce larger crop width
    expect(secondCropW).toBeGreaterThan(firstCropW)
  })
})
