import { describe, it, expect, vi, beforeEach } from 'vitest'
import { nextTick } from 'vue'

const mockedInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockedInvoke(...args),
}))

import { useModelConfig } from '../composables/useModelConfig'

function mockModelConfig(model = 'claude-sonnet-4', provider = 'anthropic', baseUrl = '') {
  return { model, provider, baseUrl }
}

function mockModelList(
  models: { provider: string; model: string; name: string; baseUrl?: string }[] = [],
) {
  return models
}

describe('useModelConfig', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should start with empty model config', () => {
    const config = useModelConfig()
    expect(config.currentModel.value).toBe('')
    expect(config.currentProvider.value).toBe('auto')
    expect(config.currentBaseUrl.value).toBe('')
    expect(config.modelGroups.value).toEqual([])
  })

  it('should load model config via reload()', async () => {
    mockedInvoke
      .mockResolvedValueOnce(mockModelConfig('gpt-4', 'openai', ''))
      .mockResolvedValueOnce(mockModelList([
        { provider: 'openai', model: 'gpt-4', name: 'GPT-4' },
      ]))

    const config = useModelConfig()
    await config.reload()

    expect(config.currentModel.value).toBe('gpt-4')
    expect(config.currentProvider.value).toBe('openai')
    expect(config.modelGroups.value.length).toBeGreaterThanOrEqual(1)
  })

  it('should handle config load failure gracefully', async () => {
    mockedInvoke
      .mockRejectedValueOnce(new Error('config error'))
      .mockRejectedValueOnce(new Error('config error'))

    const config = useModelConfig()
    await config.reload()

    // Should keep default values
    expect(config.currentModel.value).toBe('')
    expect(config.currentProvider.value).toBe('auto')
  })

  it('should group models by provider', async () => {
    mockedInvoke
      .mockResolvedValueOnce(mockModelConfig('gpt-4', 'openai', ''))
      .mockResolvedValueOnce(mockModelList([
        { provider: 'openai', model: 'gpt-4', name: 'GPT-4' },
        { provider: 'openai', model: 'gpt-4-turbo', name: 'GPT-4 Turbo' },
        { provider: 'anthropic', model: 'claude-3', name: 'Claude 3' },
      ]))

    const config = useModelConfig()
    await config.reload()

    expect(config.modelGroups.value).toHaveLength(2)
    const openaiGroup = config.modelGroups.value.find((g) => g.provider === 'openai')
    expect(openaiGroup).toBeDefined()
    expect(openaiGroup!.models).toHaveLength(2)
  })

  it('should select a model via selectModel', async () => {
    const config = useModelConfig()

    mockedInvoke.mockResolvedValue(undefined)
    await config.selectModel('anthropic', 'claude-sonnet-4', '')

    expect(config.currentModel.value).toBe('claude-sonnet-4')
    expect(config.currentProvider.value).toBe('anthropic')
    expect(mockedInvoke).toHaveBeenCalledWith('hermes_config_set_model', {
      provider: 'anthropic',
      model: 'claude-sonnet-4',
      baseUrl: '',
      profile: null,
    })
  })

  it('should pass baseUrl for custom provider', async () => {
    const config = useModelConfig()

    mockedInvoke.mockResolvedValue(undefined)
    await config.selectModel('custom', 'my-model', 'http://localhost:8080')

    expect(mockedInvoke).toHaveBeenCalledWith('hermes_config_set_model', {
      provider: 'custom',
      model: 'my-model',
      baseUrl: 'http://localhost:8080',
      profile: null,
    })
  })

  describe('displayModel computed', () => {
    it('should show model name after last slash', async () => {
      mockedInvoke
        .mockResolvedValueOnce(mockModelConfig('anthropic/claude-sonnet-4', 'anthropic', ''))
        .mockResolvedValueOnce(mockModelList([]))

      const config = useModelConfig()
      await config.reload()

      expect(config.displayModel.value).toBe('claude-sonnet-4')
    })

    it('should show full model name when no slash', async () => {
      mockedInvoke
        .mockResolvedValueOnce(mockModelConfig('gpt-4o', 'openai', ''))
        .mockResolvedValueOnce(mockModelList([]))

      const config = useModelConfig()
      await config.reload()

      expect(config.displayModel.value).toBe('gpt-4o')
    })

    it('should show "Auto" when provider is auto and no model', () => {
      const config = useModelConfig()

      expect(config.displayModel.value).toBe('Auto')
    })

    it('should show "No model" when model is empty and provider is not auto', async () => {
      mockedInvoke
        .mockResolvedValueOnce(mockModelConfig('', 'openai', ''))
        .mockResolvedValueOnce(mockModelList([]))

      const config = useModelConfig()
      await config.reload()

      expect(config.displayModel.value).toBe('No model')
    })
  })

  it('should reload model config', async () => {
    mockedInvoke
      .mockResolvedValueOnce(mockModelConfig('old-model', 'openai', ''))
      .mockResolvedValueOnce(mockModelList([]))

    const config = useModelConfig()
    await config.reload()

    expect(config.currentModel.value).toBe('old-model')

    vi.clearAllMocks()
    mockedInvoke
      .mockResolvedValueOnce(mockModelConfig('new-model', 'anthropic', ''))
      .mockResolvedValueOnce(mockModelList([]))

    await config.reload()

    expect(config.currentModel.value).toBe('new-model')
    expect(config.currentProvider.value).toBe('anthropic')
  })

  it('should handle profile parameter', async () => {
    const config = useModelConfig('my-profile')

    mockedInvoke.mockResolvedValue(undefined)
    await config.selectModel('openai', 'gpt-4', '')

    // Profile is passed to set_model call
    const callArgs = mockedInvoke.mock.calls[0]
    expect(callArgs[1].profile).toBe('my-profile')
  })
})
