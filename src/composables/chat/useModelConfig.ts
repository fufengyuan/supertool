/**
 * useModelConfig — model configuration management.
 *
 * Adapted from hermes-desktop's useModelConfig for Vue 3.
 * Reads available models from the Tauri backend, provides grouping
 * by provider, and handles model selection.
 */
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ModelGroup } from '@/views/agent/chat/types'

interface ModelInfo {
  provider: string
  model: string
  name: string
  baseUrl?: string
}

interface GetModelsResult {
  customModels?: string[]
  defaultModel?: string
  activeProvider?: string
  providerModels?: string[]
}

const PROVIDER_LABELS: Record<string, string> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
  deepseek: 'DeepSeek',
  google: 'Google',
  groq: 'Groq',
  mistral: 'Mistral',
  openrouter: 'OpenRouter',
  custom: 'Custom',
}

function groupModelsByProvider(models: ModelInfo[]): ModelGroup[] {
  const groupMap = new Map<string, ModelGroup>()
  for (const m of models) {
    if (!groupMap.has(m.provider)) {
      groupMap.set(m.provider, {
        provider: m.provider,
        providerLabel: PROVIDER_LABELS[m.provider] || m.provider,
        models: [],
      })
    }
    groupMap.get(m.provider)!.models.push({
      provider: m.provider,
      model: m.model,
      label: m.name,
      baseUrl: m.baseUrl || '',
    })
  }
  return Array.from(groupMap.values())
}

function parseModelInfo(fullName: string): { provider: string; model: string; name: string } {
  const slashIdx = fullName.indexOf('/')
  if (slashIdx > 0) {
    const provider = fullName.substring(0, slashIdx)
    const model = fullName.substring(slashIdx + 1)
    const name = model.split('.').pop() || model
    return { provider, model: fullName, name }
  }
  return { provider: 'custom', model: fullName, name: fullName }
}

export function useModelConfig() {
  const currentModel = ref('')
  const currentProvider = ref('auto')
  const modelGroups = ref<ModelGroup[]>([])

  const displayModel = computed(() => {
    if (currentModel.value) {
      const parts = currentModel.value.split('/')
      return parts.length > 1 ? parts[parts.length - 1] : currentModel.value
    }
    return currentProvider.value === 'auto' ? 'Auto' : 'Not set'
  })

  async function reload() {
    try {
      const result = await invoke<GetModelsResult>('get_models')
      currentModel.value = result.defaultModel || ''
      currentProvider.value = result.activeProvider || 'auto'

      const allModels: ModelInfo[] = []

      // Parse provider models
      if (result.providerModels) {
        for (const full of result.providerModels) {
          const info = parseModelInfo(full)
          allModels.push(info)
        }
      }

      // Parse custom models
      if (result.customModels) {
        for (const full of result.customModels) {
          const info = parseModelInfo(full)
          allModels.push(info)
        }
      }

      modelGroups.value = groupModelsByProvider(allModels)
    } catch {
      // Config not available
    }
  }

  async function selectModel(model: string) {
    try {
      await invoke('agent_set_model', { model })
      currentModel.value = model
    } catch {
      // Best effort
    }
  }

  onMounted(reload)

  return {
    currentModel,
    currentProvider,
    modelGroups,
    displayModel,
    reload,
    selectModel,
  }
}
