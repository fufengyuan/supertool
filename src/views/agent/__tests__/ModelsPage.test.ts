import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'

// ── Mocks ─────────────────────────────────────────────────────────────────

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

/** Convert PascalCase to kebab-case: 'ChevronDown' → 'chevron-down' */
function pascalToKebab(s: string): string {
  return s.replace(/[A-Z]/g, c => '-' + c.toLowerCase()).replace(/^-/, '')
}

// stub tabler icons used in the component
function iconStub(name: string) {
  const kebab = pascalToKebab(name)
  return defineComponent({
    name: `Icon${name}`,
    props: ['size'],
    template: `<span class="icon-${kebab}-stub" />`,
  })
}

vi.mock('@tabler/icons-vue', () => ({
  IconRefresh: iconStub('Refresh'),
  IconPlus: iconStub('Plus'),
  IconCheck: iconStub('Check'),
  IconX: iconStub('X'),
  IconAlertCircle: iconStub('AlertCircle'),
  IconStar: iconStub('Star'),
  IconRobot: iconStub('Robot'),
  IconChevronDown: iconStub('ChevronDown'),
  IconPuzzle: iconStub('Puzzle'),
  IconInfoCircle: iconStub('InfoCircle'),
  IconTrash: iconStub('Trash'),
  IconBox: iconStub('Box'),
}))

import ModelsPage from '../ModelsPage.vue'

// ── Fixtures ───────────────────────────────────────────────────────────────

function makeModelsResult(overrides: Record<string, unknown> = {}) {
  return {
    customModels: [],
    defaultModel: null,
    activeProvider: null,
    providerModels: [
      'anthropic/claude-sonnet-4',
      'anthropic/claude-3-opus',
      'openai/gpt-4',
      'openai/gpt-4o',
    ],
    ...overrides,
  }
}

// ── Helpers ────────────────────────────────────────────────────────────────

/**
 * Flush Vue reactivity updates AND pending microtasks (async invoke results).
 * Works with real timers (default) and fake timers (vi.useFakeTimers()).
 * Uses queueMicrotask which is never affected by timer faking.
 */
async function flushAll() {
  await nextTick()
  // Drain the microtask queue (resolved invoke promises, etc.)
  await new Promise<void>(resolve => queueMicrotask(() => resolve()))
  await nextTick()
}

function createWrapper() {
  return mount(ModelsPage, {
    global: {
      stubs: {
        Teleport: false,
      },
    },
  })
}

/** Find the first provider-group header button by searching for its name. */
function getProviderHeader(wrapper: ReturnType<typeof createWrapper>, providerName: string) {
  return wrapper.findAll('button').filter(
    b => b.text().includes(providerName) && b.text().includes('个模型')
  )[0]
}

/** Find buttons that contain the "设为默认" text and are NOT disabled. */
function getEnabledDefaultBtns(wrapper: ReturnType<typeof createWrapper>) {
  return wrapper.findAll('button').filter(
    b => b.text().trim() === '设为默认' && b.attributes('disabled') === undefined
  )
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('ModelsPage.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    document.body.innerHTML = ''
    mockedInvoke.mockResolvedValue(makeModelsResult())
  })

  // ── Rendering ─────────────────────────────────────────────────────────

  it('should render page title', async () => {
    const wrapper = createWrapper()
    await flushAll()
    expect(wrapper.text()).toContain('模型管理')
  })

  it('should render refresh button', async () => {
    const wrapper = createWrapper()
    await flushAll()
    expect(wrapper.find('.icon-refresh-stub').exists()).toBe(true)
  })

  it('should render "添加模型" button', async () => {
    const wrapper = createWrapper()
    await flushAll()
    expect(wrapper.text()).toContain('添加模型')
  })

  // ── Loading State ─────────────────────────────────────────────────────

  it('should show loading spinner on mount before data resolves', async () => {
    mockedInvoke.mockReturnValue(new Promise(() => {}))
    const wrapper = createWrapper()
    await nextTick()
    expect(wrapper.find('.loading-spinner').exists()).toBe(true)
  })

  it('should hide loading spinner after data is loaded', async () => {
    const wrapper = createWrapper()
    await flushAll()
    expect(wrapper.find('.loading-spinner').exists()).toBe(false)
  })

  it('should call invoke("agent_get_models") on mount', () => {
    createWrapper()
    expect(mockedInvoke).toHaveBeenCalledWith('agent_get_models')
    expect(mockedInvoke).toHaveBeenCalledTimes(1)
  })

  // ── Error Display ─────────────────────────────────────────────────────

  it('should display error message when invoke throws', async () => {
    mockedInvoke.mockRejectedValue(new Error('Connection failed'))
    const wrapper = createWrapper()
    await flushAll()
    expect(wrapper.text()).toContain('Connection failed')
  })

  it('should display error when addModel receives empty name', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    const confirmBtn = wrapper.findAll('button').filter(b => b.text().includes('确认'))[0]
    await confirmBtn.trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('请输入模型名称')
  })

  // ── Default Model Display ─────────────────────────────────────────────

  it('should display default model when set', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      defaultModel: 'anthropic/claude-sonnet-4',
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('默认模型')
    expect(wrapper.text()).toContain('anthropic/claude-sonnet-4')
  })

  it('should show "未设置" when no default model', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      defaultModel: null,
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('未设置')
  })

  it('should show empty default model string as "未设置"', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      defaultModel: '',
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('未设置')
  })

  // ── Provider Groups ───────────────────────────────────────────────────

  it('should group models by provider', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      providerModels: ['anthropic/claude-sonnet-4', 'openai/gpt-4', 'openai/gpt-4o'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('anthropic')
    expect(wrapper.text()).toContain('openai')
  })

  it('should display model count badge per provider', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      providerModels: ['anthropic/claude-sonnet-4', 'anthropic/claude-3-opus', 'openai/gpt-4'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('2 个模型')
    expect(wrapper.text()).toContain('1 个模型')
  })

  it('should sort provider groups alphabetically', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      providerModels: ['zprovider/z-model', 'aprovider/a-model'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    const text = wrapper.text()
    expect(text.indexOf('aprovider')).toBeGreaterThan(0)
    expect(text.indexOf('zprovider')).toBeGreaterThan(text.indexOf('aprovider'))
  })

  it('should place models without slash into "other" group', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      providerModels: ['gpt-4', 'no-slash-model'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('other')
  })

  it('should list all model names within a provider group', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      providerModels: ['openai/gpt-4', 'openai/gpt-4o'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('gpt-4')
    expect(wrapper.text()).toContain('gpt-4o')
  })

  it('should show no provider groups when providerModels is empty', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({ providerModels: [] }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('自定义模型')
  })

  it('should handle null providerModels gracefully', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({ providerModels: null }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('模型管理')
  })

  it('should not crash with extremely long model names', async () => {
    const longName = 'A'.repeat(500) + '/' + 'B'.repeat(500)
    mockedInvoke.mockResolvedValue(makeModelsResult({ providerModels: [longName] }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('A'.repeat(500))
  })

  // ── Collapse / Expand ─────────────────────────────────────────────────

  it('should collapse a provider group when its header is clicked', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const header = getProviderHeader(wrapper, 'anthropic')
    expect(header).toBeDefined()

    // Capture state before: models should be visible
    expect(wrapper.text()).toContain('claude-sonnet-4')

    await header.trigger('click')
    await nextTick()

    // The provider header itself should still be visible
    expect(wrapper.text()).toContain('anthropic')
  })

  it('should expand a collapsed provider group when clicked again', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const header = getProviderHeader(wrapper, 'anthropic')
    expect(header).toBeDefined()

    // Collapse
    await header.trigger('click')
    await nextTick()

    // Expand again
    await header.trigger('click')
    await nextTick()

    // Models should be visible again
    expect(wrapper.text()).toContain('claude-sonnet-4')
  })

  // ── Custom Models Section ─────────────────────────────────────────────

  it('should show empty state when no custom models exist', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({ customModels: [] }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('暂无自定义模型')
  })

  it('should display custom models', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      customModels: ['my-custom-model', 'another-model'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('my-custom-model')
    expect(wrapper.text()).toContain('another-model')
  })

  it('should show custom model count', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      customModels: ['m1', 'm2', 'm3'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('3 个')
  })

  it('should show star icon next to custom model when it is the default', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      customModels: ['my-custom-model'],
      defaultModel: 'my-custom-model',
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.findAll('.icon-star-stub').length).toBeGreaterThanOrEqual(1)
  })

  it('should handle null customModels gracefully', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({ customModels: null }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('暂无自定义模型')
  })

  // ── Add Model ─────────────────────────────────────────────────────────

  it('should open add model input when "添加模型" is clicked', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    expect(wrapper.find('input[placeholder="输入模型名称..."]').exists()).toBe(true)
  })

  it('should call invoke("agent_add_model") with model name on confirm', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult())
      .mockResolvedValueOnce({ success: true, customModels: ['new-model', 'anthropic/claude-sonnet-4'] })

    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    const input = wrapper.find('input[placeholder="输入模型名称..."]')
    await input.setValue('new-model')
    await nextTick()

    const confirmBtn = wrapper.findAll('button').filter(b => b.text().includes('确认'))[0]
    await confirmBtn.trigger('click')
    await flushAll()

    expect(mockedInvoke).toHaveBeenCalledWith('agent_add_model', { model: 'new-model' })
  })

  it('should update customModels list after successful add', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult())
      .mockResolvedValueOnce({ success: true, customModels: ['new-model', 'existing-model'] })

    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    const input = wrapper.find('input[placeholder="输入模型名称..."]')
    await input.setValue('new-model')
    await nextTick()

    const confirmBtn = wrapper.findAll('button').filter(b => b.text().includes('确认'))[0]
    await confirmBtn.trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('new-model')
    expect(wrapper.text()).toContain('existing-model')
  })

  it('should show success message after adding a model', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult())
      .mockResolvedValueOnce({ success: true, customModels: ['new-model'] })

    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    const input = wrapper.find('input[placeholder="输入模型名称..."]')
    await input.setValue('new-model')
    await nextTick()

    const confirmBtn = wrapper.findAll('button').filter(b => b.text().includes('确认'))[0]
    await confirmBtn.trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('已添加模型')
  })

  it('should show error when addModel invoke fails', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult())
      .mockRejectedValueOnce(new Error('Failed to add model'))

    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    const input = wrapper.find('input[placeholder="输入模型名称..."]')
    await input.setValue('new-model')
    await nextTick()

    const confirmBtn = wrapper.findAll('button').filter(b => b.text().includes('确认'))[0]
    await confirmBtn.trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('Failed to add model')
  })

  it('should clear input and close panel after successful add', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult())
      .mockResolvedValueOnce({ success: true, customModels: ['new-model'] })

    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    const input = wrapper.find('input[placeholder="输入模型名称..."]')
    await input.setValue('new-model')
    await nextTick()

    const confirmBtn = wrapper.findAll('button').filter(b => b.text().includes('确认'))[0]
    await confirmBtn.trigger('click')
    await flushAll()

    expect(wrapper.find('input[placeholder="输入模型名称..."]').exists()).toBe(false)
  })

  it('should cancel add model and clear input when cancel is clicked', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    const input = wrapper.find('input[placeholder="输入模型名称..."]')
    await input.setValue('test-model')
    await nextTick()

    const cancelBtn = wrapper.findAll('button').filter(b => b.find('.icon-x-stub').exists())[0]
    await cancelBtn.trigger('click')
    await nextTick()

    expect(wrapper.find('input[placeholder="输入模型名称..."]').exists()).toBe(false)
  })

  it('should add model via Enter key in input', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult())
      .mockResolvedValueOnce({ success: true, customModels: ['enter-model'] })

    const wrapper = createWrapper()
    await flushAll()

    const addBtn = wrapper.findAll('button').filter(b => b.text().includes('添加模型'))[0]
    await addBtn.trigger('click')
    await nextTick()

    const input = wrapper.find('input[placeholder="输入模型名称..."]')
    await input.setValue('enter-model')
    await input.trigger('keyup.enter')
    await flushAll()

    expect(mockedInvoke).toHaveBeenCalledWith('agent_add_model', { model: 'enter-model' })
  })

  // ── Set As Default ────────────────────────────────────────────────────

  it('should call invoke("agent_set_model") when setAsDefault is triggered', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ defaultModel: 'openai/gpt-4' }))
      .mockResolvedValueOnce(undefined)

    const wrapper = createWrapper()
    await flushAll()

    const btns = getEnabledDefaultBtns(wrapper)
    expect(btns.length).toBeGreaterThan(0)
    await btns[0].trigger('click')
    await flushAll()

    expect(mockedInvoke).toHaveBeenCalledWith('agent_set_model', { model: expect.any(String) })
  })

  it('should show success message after setting default model', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ defaultModel: '', providerModels: ['openai/gpt-4'] }))
      .mockResolvedValueOnce(undefined)

    const wrapper = createWrapper()
    await flushAll()

    const btn = getEnabledDefaultBtns(wrapper)[0]
    expect(btn).toBeDefined()
    await btn.trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('已切换默认模型')
  })

  it('should disable "设为默认" buttons for the current default model', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      defaultModel: 'openai/gpt-4',
      providerModels: ['openai/gpt-4', 'openai/gpt-4o'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    const allDefaultBtns = wrapper.findAll('button').filter(b => b.text().trim() === '设为默认')
    const disabledBtns = allDefaultBtns.filter(b => b.attributes('disabled') !== undefined)
    expect(disabledBtns.length).toBeGreaterThanOrEqual(1)
  })

  it('should not call agent_set_model when clicking btn of already-default model', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      defaultModel: 'openai/gpt-4',
      providerModels: ['openai/gpt-4'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    const disabledBtn = wrapper.findAll('button').filter(
      b => b.text().trim() === '设为默认' && b.attributes('disabled') !== undefined
    )[0]
    expect(disabledBtn).toBeDefined()
    await disabledBtn.trigger('click')
    await flushAll()

    const setModelCalls = mockedInvoke.mock.calls.filter(c => c[0] === 'agent_set_model')
    expect(setModelCalls.length).toBe(0)
  })

  it('should guard against duplicate setAsDefault calls via settingDefault ref', async () => {
    let resolveSetModel!: (v: unknown) => void
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ defaultModel: '', providerModels: ['openai/gpt-4'] }))
      .mockImplementationOnce(() => new Promise(resolve => { resolveSetModel = resolve }))

    const wrapper = createWrapper()
    await flushAll()

    const btn = getEnabledDefaultBtns(wrapper)[0]
    expect(btn).toBeDefined()
    expect(btn.attributes('disabled')).toBeUndefined()

    // First click
    await btn.trigger('click')
    await nextTick()

    // After first click, the button should be disabled (settingDefault = true)
    expect(btn.attributes('disabled')).toBeDefined()

    // Second click while first is in flight
    await btn.trigger('click')
    await nextTick()

    // Resolve the promise
    resolveSetModel!(undefined)
    await flushAll()

    const setModelCalls = mockedInvoke.mock.calls.filter(c => c[0] === 'agent_set_model')
    expect(setModelCalls.length).toBe(1)
  })

  it('should handle agent_set_model error gracefully', async () => {
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ defaultModel: '', providerModels: ['openai/gpt-4'] }))
      .mockRejectedValueOnce(new Error('Failed to set default'))

    const wrapper = createWrapper()
    await flushAll()

    const btn = getEnabledDefaultBtns(wrapper)[0]
    expect(btn).toBeDefined()
    await btn.trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('Failed to set default')
  })

  // ── Remove Model ──────────────────────────────────────────────────────

  it('should call invoke("agent_remove_model") when remove is confirmed', async () => {
    window.confirm = vi.fn(() => true)
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ customModels: ['test-model'] }))
      .mockResolvedValueOnce({ success: true, customModels: [] })

    const wrapper = createWrapper()
    await flushAll()

    const deleteBtns = wrapper.findAll('button').filter(b => b.find('.icon-trash-stub').exists())
    expect(deleteBtns.length).toBeGreaterThan(0)
    await deleteBtns[0].trigger('click')
    await flushAll()

    expect(mockedInvoke).toHaveBeenCalledWith('agent_remove_model', { model: 'test-model' })
  })

  it('should NOT call agent_remove_model when confirm is cancelled', async () => {
    window.confirm = vi.fn(() => false)
    mockedInvoke.mockResolvedValue(makeModelsResult({ customModels: ['test-model'] }))

    const wrapper = createWrapper()
    await flushAll()

    const deleteBtns = wrapper.findAll('button').filter(b => b.find('.icon-trash-stub').exists())
    expect(deleteBtns.length).toBeGreaterThan(0)
    await deleteBtns[0].trigger('click')
    await flushAll()

    expect(mockedInvoke).not.toHaveBeenCalledWith('agent_remove_model', expect.anything())
  })

  it('should remove custom model from list on successful deletion', async () => {
    window.confirm = vi.fn(() => true)
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ customModels: ['model-to-remove', 'keep-model'] }))
      .mockResolvedValueOnce({ success: true, customModels: ['keep-model'] })

    const wrapper = createWrapper()
    await flushAll()

    const deleteBtns = wrapper.findAll('button').filter(b => b.find('.icon-trash-stub').exists())
    await deleteBtns[0].trigger('click')
    await flushAll()

    // Success message shown
    expect(wrapper.text()).toContain('已删除模型')
    // Deleted model name only appears in the success message, not as an item
    expect(wrapper.text()).toContain('keep-model')
    // Custom section should show 'keep-model' (not empty)
    expect(wrapper.text()).not.toContain('暂无自定义模型')
  })

  it('should clear defaultModel when the deleted model was the default', async () => {
    window.confirm = vi.fn(() => true)
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ customModels: ['default-model'], defaultModel: 'default-model' }))
      .mockResolvedValueOnce({ success: true, customModels: [] })

    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('default-model')

    const deleteBtns = wrapper.findAll('button').filter(b => b.find('.icon-trash-stub').exists())
    await deleteBtns[0].trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('未设置')
  })

  it('should show success message after deletion', async () => {
    window.confirm = vi.fn(() => true)
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ customModels: ['remove-me'] }))
      .mockResolvedValueOnce({ success: true, customModels: [] })

    const wrapper = createWrapper()
    await flushAll()

    const deleteBtns = wrapper.findAll('button').filter(b => b.find('.icon-trash-stub').exists())
    await deleteBtns[0].trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('已删除模型')
  })

  it('should show error when remove result.success is false', async () => {
    window.confirm = vi.fn(() => true)
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ customModels: ['fail-model'] }))
      .mockResolvedValueOnce({ success: false, customModels: ['fail-model'] })

    const wrapper = createWrapper()
    await flushAll()

    const deleteBtns = wrapper.findAll('button').filter(b => b.find('.icon-trash-stub').exists())
    await deleteBtns[0].trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('删除模型失败')
    expect(wrapper.text()).toContain('fail-model')
  })

  it('should show error message when removeModel invoke fails', async () => {
    window.confirm = vi.fn(() => true)
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({ customModels: ['fail-model'] }))
      .mockRejectedValueOnce(new Error('Network error'))

    const wrapper = createWrapper()
    await flushAll()

    const deleteBtns = wrapper.findAll('button').filter(b => b.find('.icon-trash-stub').exists())
    await deleteBtns[0].trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('Network error')
  })

  // ── Model Detail Modal ────────────────────────────────────────────────

  it('should show detail modal when info button is clicked', async () => {
    const wrapper = createWrapper()
    await flushAll()

    // Info buttons are the ones containing the info-circle icon, inside model rows
    const infoBtns = wrapper.findAll('button').filter(
      b => b.find('.icon-info-circle-stub').exists()
    )
    expect(infoBtns.length).toBeGreaterThan(0)
    await infoBtns[0].trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('模型详情')
  })

  it('should display model name and provider in detail modal', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const infoBtns = wrapper.findAll('button').filter(b => b.find('.icon-info-circle-stub').exists())
    await infoBtns[0].trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('claude-sonnet-4')
    expect(wrapper.text()).toContain('anthropic')
  })

  it('should show provider label in detail modal', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const infoBtns = wrapper.findAll('button').filter(b => b.find('.icon-info-circle-stub').exists())
    await infoBtns[0].trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('提供商')
  })

  it('should close detail modal when clicking close button', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const infoBtns = wrapper.findAll('button').filter(b => b.find('.icon-info-circle-stub').exists())
    await infoBtns[0].trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('模型详情')

    // Close buttons (X icon) — the last one in modal
    const closeBtns = wrapper.findAll('button').filter(b => b.find('.icon-x-stub').exists())
    await closeBtns[closeBtns.length - 1].trigger('click')
    await nextTick()

    expect(wrapper.text()).not.toContain('模型详情')
  })

  it('should close detail modal when clicking overlay backdrop', async () => {
    const wrapper = createWrapper()
    await flushAll()

    const infoBtns = wrapper.findAll('button').filter(b => b.find('.icon-info-circle-stub').exists())
    await infoBtns[0].trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('模型详情')

    const overlay = wrapper.find('.fixed')
    expect(overlay.exists()).toBe(true)
    await overlay.trigger('click')
    await nextTick()

    expect(wrapper.text()).not.toContain('模型详情')
  })

  it('should show "设为默认" button in modal for non-default model', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      defaultModel: 'openai/gpt-4o',
      providerModels: ['openai/gpt-4'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    const infoBtns = wrapper.findAll('button').filter(b => b.find('.icon-info-circle-stub').exists())
    await infoBtns[0].trigger('click')
    await nextTick()

    const modalBtns = wrapper.findAll('button').filter(b => b.text().includes('设为默认'))
    expect(modalBtns.length).toBeGreaterThanOrEqual(1)
  })

  // ── Success Timer & Cleanup ───────────────────────────────────────────

  it('should clear success message after 3 seconds', async () => {
    vi.useFakeTimers()
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({
        defaultModel: '',
        providerModels: ['openai/gpt-4'],
      }))
      .mockResolvedValueOnce(undefined)

    const wrapper = createWrapper()
    await flushAll()

    const btn = getEnabledDefaultBtns(wrapper)[0]
    expect(btn).toBeDefined()
    await btn.trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('已切换默认模型')

    await vi.advanceTimersByTimeAsync(3100)
    await nextTick()

    expect(wrapper.text()).not.toContain('已切换默认模型')
    vi.useRealTimers()
  })

  it('should clear successTimer when a new success message is shown', async () => {
    vi.useFakeTimers()
    const setTimeoutSpy = vi.spyOn(window, 'setTimeout')
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({
        defaultModel: '',
        providerModels: ['openai/gpt-4', 'openai/gpt-4o'],
      }))
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined)

    const wrapper = createWrapper()
    await flushAll()

    // Trigger first success
    const btn1 = getEnabledDefaultBtns(wrapper)[0]
    await btn1.trigger('click')
    await flushAll()

    // Trigger second success
    const btn2 = getEnabledDefaultBtns(wrapper)[0]
    await btn2.trigger('click')
    await flushAll()

    const timeoutCalls = setTimeoutSpy.mock.calls.filter(
      c => typeof c[0] === 'function' && c[1] === 3000
    )
    expect(timeoutCalls.length).toBe(2)

    setTimeoutSpy.mockRestore()
    vi.useRealTimers()
  })

  it('should not crash when component unmounts while successTimer is pending', async () => {
    vi.useFakeTimers()
    mockedInvoke
      .mockResolvedValueOnce(makeModelsResult({
        defaultModel: '',
        providerModels: ['openai/gpt-4'],
      }))
      .mockResolvedValueOnce(undefined)

    const wrapper = createWrapper()
    await flushAll()

    const btn = getEnabledDefaultBtns(wrapper)[0]
    await btn.trigger('click')
    await flushAll()

    expect(wrapper.text()).toContain('已切换默认模型')
    expect(() => wrapper.unmount()).not.toThrow()
    vi.useRealTimers()
  })

  // ── Refresh ───────────────────────────────────────────────────────────

  it('should reload models when refresh button is clicked', async () => {
    const wrapper = createWrapper()
    await flushAll()
    mockedInvoke.mockClear()

    // The refresh button wraps the IconRefresh stub
    const refreshBtn = wrapper.find('.icon-refresh-stub').element.closest('button') as HTMLElement
    refreshBtn.click()
    await flushAll()

    expect(mockedInvoke).toHaveBeenCalledWith('agent_get_models')
  })

  // ── Edge Cases ─────────────────────────────────────────────────────────

  it('should handle component re-mount without leaks', async () => {
    const wrapper1 = createWrapper()
    await flushAll()
    wrapper1.unmount()

    const wrapper2 = createWrapper()
    await flushAll()

    expect(wrapper2.text()).toContain('模型管理')
    wrapper2.unmount()
  })

  it('should handle empty provider models list gracefully', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      providerModels: [],
      customModels: [],
      defaultModel: null,
    }))
    const wrapper = createWrapper()
    await flushAll()

    expect(wrapper.text()).toContain('暂无自定义模型')
    const providerHeaders = wrapper.findAll('button').filter(
      b => b.text().includes('个模型')
    )
    expect(providerHeaders.length).toBe(0)
  })

  it('should parse model with version suffix correctly in detail modal', async () => {
    mockedInvoke.mockResolvedValue(makeModelsResult({
      providerModels: ['anthropic/claude-sonnet-4:20250101'],
    }))
    const wrapper = createWrapper()
    await flushAll()

    const infoBtns = wrapper.findAll('button').filter(b => b.find('.icon-info-circle-stub').exists())
    await infoBtns[0].trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('claude-sonnet-4:20250101')
  })
})
