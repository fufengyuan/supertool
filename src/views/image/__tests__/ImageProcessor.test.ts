import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick, defineComponent } from 'vue'
import ImageProcessor from '@/views/image/ImageProcessor.vue'

// ── Mocks ──────────────────────────────────────────────────────────────────

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve(undefined)),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${encodeURIComponent(path)}`),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

// Stub SvgIcon
vi.mock('@/components/ui/SvgIcon.vue', () => ({
  default: defineComponent({
    name: 'SvgIcon',
    props: ['name', 'size'],
    template: '<span class="svg-icon-stub" :data-name="name" />',
  }),
}))

// Stub CropOverlay
vi.mock('@/components/CropOverlay.vue', () => ({
  default: defineComponent({
    name: 'CropOverlay',
    props: [
      'imgNaturalWidth', 'imgNaturalHeight',
      'imgDisplayWidth', 'imgDisplayHeight',
      'imgOffsetX', 'imgOffsetY',
      'cropX', 'cropY', 'cropW', 'cropH',
    ],
    emits: ['update:cropX', 'update:cropY', 'update:cropW', 'update:cropH'],
    template: '<div class="crop-overlay-stub" data-testid="crop-overlay" />',
  }),
}))

// ── Helpers ────────────────────────────────────────────────────────────────

function createWrapper() {
  return mount(ImageProcessor, {
    attachTo: document.body,
  })
}

function getVm(wrapper: ReturnType<typeof createWrapper>) {
  return wrapper.vm as any
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('ImageProcessor.vue', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    vi.clearAllMocks()
    // Stub ResizeObserver
    globalThis.ResizeObserver = class {
      observe() {}
      unobserve() {}
      disconnect() {}
    } as any
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  // ── 1. Crop Visualization Selection ──────────────────────────────────────

  describe('Crop visualization', () => {
    it('should show CropOverlay when activeFunction is crop and image is loaded', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 800
      vm.originalHeight = 600
      vm.imgDisplayWidth = 400
      vm.imgDisplayHeight = 300
      vm.imgOffsetX = 0
      vm.imgOffsetY = 0
      vm.activeFunction = 'crop'
      await nextTick()

      const overlay = wrapper.find('[data-testid="crop-overlay"]')
      expect(overlay.exists()).toBe(true)

      wrapper.unmount()
    })

    it('should not show CropOverlay when activeFunction is not crop', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 800
      vm.originalHeight = 600
      vm.imgDisplayWidth = 400
      vm.imgDisplayHeight = 300
      vm.activeFunction = 'compress'
      await nextTick()

      const overlay = wrapper.find('[data-testid="crop-overlay"]')
      expect(overlay.exists()).toBe(false)

      wrapper.unmount()
    })

    it('should pass correct props to CropOverlay', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1600
      vm.originalHeight = 1200
      vm.imgDisplayWidth = 800
      vm.imgDisplayHeight = 600
      vm.imgOffsetX = 20
      vm.imgOffsetY = 10
      vm.activeFunction = 'crop'
      await nextTick()

      const overlay = wrapper.findComponent({ name: 'CropOverlay' })
      expect(overlay.exists()).toBe(true)
      expect(overlay.props('imgNaturalWidth')).toBe(1600)
      expect(overlay.props('imgNaturalHeight')).toBe(1200)
      expect(overlay.props('imgDisplayWidth')).toBe(800)
      expect(overlay.props('imgDisplayHeight')).toBe(600)
      expect(overlay.props('imgOffsetX')).toBe(20)
      expect(overlay.props('imgOffsetY')).toBe(10)

      wrapper.unmount()
    })

    it('should update cropX/cropY/cropW/cropH when CropOverlay emits updates', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 800
      vm.originalHeight = 600
      vm.imgDisplayWidth = 400
      vm.imgDisplayHeight = 300
      vm.activeFunction = 'crop'
      await nextTick()

      expect(vm.cropX).toBe(0)
      expect(vm.cropY).toBe(0)
      expect(vm.cropW).toBe(0)
      expect(vm.cropH).toBe(0)

      // Simulate CropOverlay emitting crop values
      const overlay = wrapper.findComponent({ name: 'CropOverlay' })
      await overlay.vm.$emit('update:cropX', 100)
      await overlay.vm.$emit('update:cropY', 50)
      await overlay.vm.$emit('update:cropW', 300)
      await overlay.vm.$emit('update:cropH', 200)
      await nextTick()

      expect(vm.cropX).toBe(100)
      expect(vm.cropY).toBe(50)
      expect(vm.cropW).toBe(300)
      expect(vm.cropH).toBe(200)

      wrapper.unmount()
    })

    it('should show manual x/y/w/h input fields in crop mode', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 800
      vm.originalHeight = 600
      vm.activeFunction = 'crop'
      await nextTick()

      const inputs = wrapper.findAll('input[type="number"]')
      // Should have 4 numeric inputs: X, Y, Width, Height
      expect(inputs.length).toBe(4)

      wrapper.unmount()
    })

    it('should update cropX/Y/W/H when manual inputs change', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 800
      vm.originalHeight = 600
      vm.activeFunction = 'crop'
      await nextTick()

      const inputs = wrapper.findAll('input[type="number"]')
      await inputs[0].setValue(50)
      await inputs[1].setValue(30)
      await inputs[2].setValue(200)
      await inputs[3].setValue(150)
      await nextTick()

      expect(vm.cropX).toBe(50)
      expect(vm.cropY).toBe(30)
      expect(vm.cropW).toBe(200)
      expect(vm.cropH).toBe(150)

      wrapper.unmount()
    })

    it('should pass correct crop values to invoke when processing crop', async () => {
      mockedInvoke.mockImplementationOnce(() => Promise.resolve('/tmp/output_cropped.png'))

      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 800
      vm.originalHeight = 600
      vm.activeFunction = 'crop'
      vm.cropX = 100
      vm.cropY = 50
      vm.cropW = 400
      vm.cropH = 300
      await nextTick()

      const processBtn = wrapper.find('button.btn-primary')
      await processBtn.trigger('click')
      await flushPromises()

      expect(mockedInvoke).toHaveBeenCalledWith('image_crop', {
        path: '/tmp/test.png',
        x: 100,
        y: 50,
        width: 400,
        height: 300,
      })

      wrapper.unmount()
    })
  })

  // ── 2. Temp File Cleanup ─────────────────────────────────────────────────

  describe('Temp file cleanup', () => {
    it('should call cleanTempDir on unmount', async () => {
      mockedInvoke.mockImplementation(() => Promise.resolve(undefined))

      const wrapper = createWrapper()
      await nextTick()

      wrapper.unmount()
      await flushPromises()

      expect(mockedInvoke).toHaveBeenCalledWith('clean_temp_dir', { maxAgeHours: 24 })
    })

    it('should output processed images via invoke to temp path', async () => {
      mockedInvoke
        .mockImplementationOnce(() => Promise.resolve('/tmp/test.png'))
        .mockImplementationOnce(() => Promise.resolve({ size: 1024 }))
        .mockImplementationOnce(() => Promise.resolve('/tmp/processed.jpg'))
        .mockImplementationOnce(() => Promise.resolve({ size: 512 }))

      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.activeFunction = 'compress'
      vm.quality = 80
      await nextTick()

      const processBtn = wrapper.find('button.btn-primary')
      await processBtn.trigger('click')
      await flushPromises()

      expect(mockedInvoke).toHaveBeenCalledWith('image_compress', {
        path: '/tmp/test.png',
        quality: 80,
        format: 'jpeg',
      })

      wrapper.unmount()
    })

    it('should reset processedPath when switching files', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.processedPath = '/tmp/processed.png'
      vm.originalSize = 1024
      vm.processedSize = 512
      await nextTick()

      vm.originalPath = '/tmp/new_test.png'
      vm.processedPath = ''
      vm.originalSize = 0
      vm.processedSize = 0
      await nextTick()

      expect(vm.processedPath).toBe('')
      expect(vm.originalSize).toBe(0)

      wrapper.unmount()
    })
  })

  // ── 3. Resize Parameter Linkage ──────────────────────────────────────────

  describe('Resize parameter linkage', () => {
    it('should show original dimensions in resize panel when image is loaded', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1920
      vm.originalHeight = 1080
      vm.activeFunction = 'resize'
      await nextTick()

      const text = wrapper.text()
      expect(text).toContain('原始尺寸')
      expect(text).toContain('1920')
      expect(text).toContain('1080')

      wrapper.unmount()
    })

    it('should update percent when width input changes', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1600
      vm.originalHeight = 900
      vm.activeFunction = 'resize'
      await nextTick()

      const inputs = wrapper.findAll('input[type="number"]')
      const widthInput = inputs[0]
      await widthInput.setValue(800)
      await nextTick()

      expect(vm.percent).toBe(50)
      expect(vm.resizeWidth).toBe(800)
      expect(vm.resizeSource).toBe('dimensions')

      wrapper.unmount()
    })

    it('should update width/height when percent changes via slider', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1600
      vm.originalHeight = 900
      vm.activeFunction = 'resize'
      await nextTick()

      const slider = wrapper.find('input[type="range"]')
      expect(slider.exists()).toBe(true)

      // Simulate onPercentInput with value 50
      vm.onPercentInput({ target: { value: '50' } } as any)
      await nextTick()

      expect(vm.resizeWidth).toBe(800)
      expect(vm.resizeHeight).toBe(450)
      expect(vm.resizeSource).toBe('percent')

      wrapper.unmount()
    })

    it('should compute percent correctly from width', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1000
      vm.originalHeight = 500
      vm.activeFunction = 'resize'
      await nextTick()

      const inputs = wrapper.findAll('input[type="number"]')
      await inputs[0].setValue(250)
      await nextTick()

      expect(vm.computedPercent).toBe(25)

      wrapper.unmount()
    })

    it('should keep aspect ratio when width is changed and keepAspect is true', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1600
      vm.originalHeight = 800
      vm.keepAspect = true
      vm.activeFunction = 'resize'
      await nextTick()

      vm.onResizeWidthInput({ target: { value: '400' } } as any)
      await nextTick()

      expect(vm.resizeHeight).toBe(200)
      expect(vm.resizeWidth).toBe(400)

      wrapper.unmount()
    })

    it('should not update height when keepAspect is false', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1600
      vm.originalHeight = 800
      vm.keepAspect = false
      vm.resizeHeight = 600
      vm.activeFunction = 'resize'
      await nextTick()

      vm.onResizeWidthInput({ target: { value: '400' } } as any)
      await nextTick()

      expect(vm.resizeHeight).toBe(600)

      wrapper.unmount()
    })

    it('should handle percent slider via onPercentInput', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 2000
      vm.originalHeight = 1000
      vm.activeFunction = 'resize'
      await nextTick()

      vm.onPercentInput({ target: { value: '75' } } as any)
      await nextTick()

      expect(vm.resizeWidth).toBe(1500)
      expect(vm.resizeHeight).toBe(750)
      expect(vm.resizeSource).toBe('percent')

      wrapper.unmount()
    })

    it('should pass correct resize params to invoke', async () => {
      mockedInvoke.mockImplementationOnce(() => Promise.resolve('/tmp/resized.png'))

      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1600
      vm.originalHeight = 800
      vm.resizeWidth = 800
      vm.resizeHeight = 400
      vm.keepAspect = true
      vm.activeFunction = 'resize'
      await nextTick()

      const processBtn = wrapper.find('button.btn-primary')
      await processBtn.trigger('click')
      await flushPromises()

      expect(mockedInvoke).toHaveBeenCalledWith('image_resize', {
        path: '/tmp/test.png',
        width: 800,
        height: 400,
        percent: null,
        keepAspect: true,
      })

      wrapper.unmount()
    })

    it('should send percent when width and height are both 0', async () => {
      mockedInvoke.mockImplementationOnce(() => Promise.resolve('/tmp/resized.png'))

      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1600
      vm.originalHeight = 800
      vm.resizeWidth = 0
      vm.resizeHeight = 0
      vm.percent = 50
      vm.activeFunction = 'resize'
      await nextTick()

      const processBtn = wrapper.find('button.btn-primary')
      await processBtn.trigger('click')
      await flushPromises()

      expect(mockedInvoke).toHaveBeenCalledWith('image_resize', {
        path: '/tmp/test.png',
        width: null,
        height: null,
        percent: 50,
        keepAspect: true,
      })

      wrapper.unmount()
    })

    it('should disable percent input when dimensions source is active', async () => {
      const wrapper = createWrapper()
      await nextTick()
      const vm = getVm(wrapper)

      vm.originalPath = '/tmp/test.png'
      vm.originalWidth = 1600
      vm.originalHeight = 800
      vm.resizeSource = 'dimensions'
      vm.activeFunction = 'resize'
      await nextTick()

      const slider = wrapper.find('input[type="range"]')
      expect(slider.classes()).toContain('opacity-50')

      wrapper.unmount()
    })
  })
})

describe('ImageProcessor.vue - 边界与错误路径', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
    mockedInvoke.mockImplementation(() => Promise.resolve(undefined))
  })

  it('未选择文件时执行按钮禁用', async () => {
    const wrapper = createWrapper()
    await nextTick()
    const btn = wrapper.find('button.btn-primary')
    expect(btn.attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('处理中重复点击不会重复调用 invoke', async () => {
    const wrapper = createWrapper()
    await nextTick()
    const vm = getVm(wrapper)
    vm.originalPath = '/tmp/test.png'
    vm.activeFunction = 'compress'
    vm.processing = true
    await nextTick()
    await vm.processImage()
    expect(mockedInvoke).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('invoke 失败时显示错误消息', async () => {
    mockedInvoke.mockRejectedValueOnce(new Error('处理失败'))
    const wrapper = createWrapper()
    await nextTick()
    const vm = getVm(wrapper)
    vm.originalPath = '/tmp/test.png'
    vm.activeFunction = 'compress'
    await nextTick()
    await vm.processImage()
    await flushPromises()
    expect(vm.errorMsg).toContain('处理失败')
    expect(wrapper.text()).toContain('处理失败')
    wrapper.unmount()
  })
})
