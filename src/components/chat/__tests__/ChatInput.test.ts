import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

// ── Mocks ─────────────────────────────────────────────────────────────────
// IMPORTANT: vi.mock calls are hoisted to top of file, before imports.
// Factory functions must not reference outer variables.

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

vi.mock('@/utils/tauri-api', () => ({
  getTauriAPI: vi.fn(() => ({
    getGitRepos: vi.fn(),
  })),
}))

vi.mock('@/components/ui/SvgIcon.vue', () => ({
  default: {
    name: 'SvgIcon',
    props: ['name', 'size', 'strokeWidth'],
    template: '<span :class="`icon-${name}-stub`" />',
  },
}))

vi.mock('@/composables/useAttachmentProcessor', () => ({
  filesFromClipboard: vi.fn(() => []),
  processFiles: vi.fn(async () => ({ attachments: [], errors: [] })),
}))

import ChatInput from '../ChatInput.vue'
import { invoke } from '@tauri-apps/api/core'

const mockedInvoke = vi.mocked(invoke)

// ── Fixtures ───────────────────────────────────────────────────────────────

function createWrapper(props: Record<string, any> = {}) {
  return mount(ChatInput, {
    props: {
      isStreaming: false,
      currentSession: null,
      favoriteFolders: [],
      gitRepos: [],
      hermesAvailable: true,
      onNewChat: undefined,
      onClear: undefined,
      usageStats: undefined,
      ...props,
    },
    global: {
      stubs: {
        Teleport: false,
      },
    },
    attachTo: document.body,
  })
}

describe('ChatInput.vue — slash commands integration', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockedInvoke.mockResolvedValue({
      customModels: [],
      defaultModel: null,
      activeProvider: null,
      providerModels: [],
    })
  })

  // ── Slash menu rendering ─────────────────────────────────────────────

  describe('slash menu rendering', () => {
    it('should not show slash menu when typing normal text', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('hello world')
      await nextTick()

      const menuHeader = wrapper.findAll('span').filter(s => s.text() === 'Slash Commands')
      expect(menuHeader.length).toBe(0)
    })

    it('should show slash menu when typing /', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/')
      await nextTick()

      const menuHeader = wrapper.findAll('span').filter(s => s.text() === 'Slash Commands')
      expect(menuHeader.length).toBeGreaterThanOrEqual(1)
    })

    it('should show filtered commands when typing partial slash', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/h')
      await nextTick()

      const codeEls = wrapper.findAll('code')
      const helpCode = codeEls.filter(c => c.text() === '/help')
      expect(helpCode.length).toBeGreaterThanOrEqual(1)
    })

    it('should show category labels for grouped commands', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/')
      await nextTick()

      // Category labels are rendered in <div> elements with uppercase class
      // The label text comes from CATEGORY_LABELS mapping
      const allEls = wrapper.findAll('*')  // search all elements
      const chatLabel = allEls.filter(el => el.text() === 'Chat')
      expect(chatLabel.length).toBeGreaterThanOrEqual(1)
      const agentLabel = allEls.filter(el => el.text() === 'Agent')
      expect(agentLabel.length).toBeGreaterThanOrEqual(1)
    })

    it('should show local badge for local commands', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/new')
      await nextTick()

      const badge = wrapper.find('.badge')
      expect(badge.exists()).toBe(true)
      expect(badge.text()).toBe('local')
    })
  })

  // ── Keyboard interactions ─────────────────────────────────────────────

  describe('keyboard interactions', () => {
    it('should close slash menu when typing full command with args', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/new')
      await nextTick()

      let menuHeader = wrapper.findAll('span').filter(s => s.text() === 'Slash Commands')
      expect(menuHeader.length).toBeGreaterThanOrEqual(1)

      await textarea.setValue('/new test-session')
      await nextTick()

      menuHeader = wrapper.findAll('span').filter(s => s.text() === 'Slash Commands')
      expect(menuHeader.length).toBe(0)
    })

    it('should close slash menu on Escape keydown', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/h')
      await nextTick()

      let menuHeader = wrapper.findAll('span').filter(s => s.text() === 'Slash Commands')
      expect(menuHeader.length).toBeGreaterThanOrEqual(1)

      const evt = new KeyboardEvent('keydown', { key: 'Escape' })
      document.dispatchEvent(evt)
      await nextTick()

      menuHeader = wrapper.findAll('span').filter(s => s.text() === 'Slash Commands')
      expect(menuHeader.length).toBe(0)
    })

    it('should execute local command on click', async () => {
      const onNewChat = vi.fn()
      const wrapper = createWrapper({ onNewChat })
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/new')
      await nextTick()

      const buttons = wrapper.findAll('button')
      const newBtn = buttons.filter(b => b.text().includes('/new'))[0]
      expect(newBtn).toBeTruthy()
      await newBtn.trigger('click')
      await nextTick()

      expect(onNewChat).toHaveBeenCalledTimes(1)
    })

    it('should emit commandMessage for local /help command', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/help')
      await nextTick()

      const buttons = wrapper.findAll('button')
      const helpBtn = buttons.filter(b => b.text().includes('/help'))[0]
      expect(helpBtn).toBeTruthy()
      await helpBtn.trigger('click')
      await nextTick()

      const emitted = wrapper.emitted('commandMessage')
      expect(emitted).toBeTruthy()
      expect((emitted![0][0] as string)).toContain('Available Commands')
    })

    it('should fill input with non-local command text on click', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/web')
      await nextTick()

      const buttons = wrapper.findAll('button')
      const webBtn = buttons.filter(b => b.text().includes('/web'))[0]
      expect(webBtn).toBeTruthy()
      await webBtn.trigger('click')
      await nextTick()

      expect((wrapper.vm as any).inputText).toBe('/web ')
    })
  })

  // ── Edge cases ─────────────────────────────────────────────────────────

  describe('edge cases', () => {
    it('should show all 30+ commands when typing just /', async () => {
      const wrapper = createWrapper()
      const textarea = wrapper.find('textarea')
      await textarea.setValue('/')
      await nextTick()

      const codeEls = wrapper.findAll('code')
      expect(codeEls.length).toBeGreaterThanOrEqual(30)
    })

    it('should not show textarea when hermes is unavailable', async () => {
      const wrapper = createWrapper({ hermesAvailable: false })
      expect(wrapper.find('textarea').exists()).toBe(false)
    })
  })

  // ── Input history navigation ─────────────────────────────────────────

  describe('input history navigation', () => {
    it('should not conflict ArrowUp with slash menu navigation', async () => {
      const wrapper = createWrapper()
      const vm = wrapper.vm as any

      // send a message to populate history
      vm.inputText = 'previous message'
      vm.handleSend()
      await nextTick()

      // start typing slash command
      vm.inputText = '/'
      await nextTick()

      expect(vm.slash.isSlashMenuVisible.value).toBe(true)

      // ArrowUp should be consumed by slash menu handler, not history navigation
      const textarea = wrapper.find('textarea')
      await textarea.trigger('keydown', { key: 'ArrowUp' })
      await nextTick()

      expect(vm.slash.isSlashMenuVisible.value).toBe(true)
    })
  })
})
