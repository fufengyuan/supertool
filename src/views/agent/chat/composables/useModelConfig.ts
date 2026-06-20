import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Ref } from 'vue';
import type { ModelGroup } from '../types';
import { getTauriAPI } from '@/utils/tauri-api';

function groupModelsByProvider(
  models: { provider: string; model: string; name: string; baseUrl?: string }[],
): ModelGroup[] {
  const groupMap = new Map<string, ModelGroup>();
  for (const m of models) {
    if (!groupMap.has(m.provider)) {
      groupMap.set(m.provider, {
        provider: m.provider,
        providerLabel: m.provider,
        models: [],
      });
    }
    groupMap.get(m.provider)!.models.push({
      provider: m.provider,
      model: m.model,
      label: m.name,
      baseUrl: m.baseUrl || '',
    });
  }
  return Array.from(groupMap.values());
}

interface UseModelConfigResult {
  currentModel: Ref<string>;
  currentProvider: Ref<string>;
  currentBaseUrl: Ref<string>;
  modelGroups: Ref<ModelGroup[]>;
  displayModel: Ref<string>;
  subAgentModel: Ref<string>;
  subAgentDisplayModel: Ref<string>;
  reload: () => Promise<void>;
  selectModel: (
    provider: string,
    model: string,
    baseUrl: string,
  ) => Promise<void>;
  selectSubAgentModel: (model: string) => Promise<void>;
}

export function useModelConfig(profile?: string): UseModelConfigResult {
  const currentModel = ref('');
  const currentProvider = ref('auto');
  const currentBaseUrl = ref('');
  const modelGroups = ref<ModelGroup[]>([]);
  const subAgentModel = ref('');
  const isClawMode = ref(false);

  /** 检查当前是否为 Claw 模式 */
  function checkClawMode(): boolean {
    try {
      const stored = localStorage.getItem('supertool:agentMode');
      return stored === 'claw';
    } catch {
      return false;
    }
  }

  const reload = async (): Promise<void> => {
    isClawMode.value = checkClawMode();

    if (isClawMode.value) {
      // Claw 模式：从 claw_config_get 加载模型列表和当前活跃模型
      try {
        const api = getTauriAPI();
        const config = await api.clawConfigGet();

        currentModel.value = config.activeModel || config.model || 'claude-sonnet-4-6';
        currentProvider.value = config.provider || 'Hermes Config';
        currentBaseUrl.value = config.baseUrl || '';
        subAgentModel.value = config.subAgentModel || '';

        // Convert ModelConfig[] from backend to ModelGroup[]
        if (config.models && config.models.length > 0) {
          modelGroups.value = groupModelsByProvider(
            config.models.map((m) => ({
              provider: m.provider || 'default',
              model: m.model,
              name: m.name || m.model,
              baseUrl: m.baseUrl || '',
            })),
          );
        } else {
          // Fallback: show the current model as a single entry
          modelGroups.value = [{
            provider: 'Hermes Config',
            providerLabel: 'Hermes Config',
            models: [{
              provider: 'Hermes Config',
              model: config.activeModel || config.model || 'claude-sonnet-4-6',
              label: config.activeModel || config.model || 'claude-sonnet-4-6',
              baseUrl: config.baseUrl || '',
            }],
          }];
        }
      } catch {
        // Fallback defaults
        currentModel.value = 'claude-sonnet-4-6';
        currentProvider.value = 'Hermes Config';
        currentBaseUrl.value = '';
        modelGroups.value = [];
      }
      return;
    }

    // Hermes 模式：从 hermes_config 加载
    try {
      const [mc, savedModels] = await Promise.all([
        invoke<{ model: string; provider: string; baseUrl: string }>(
          'hermes_config_get_model',
        ),
        invoke<{ provider: string; model: string; name: string; baseUrl?: string }[]>(
          'hermes_config_list_models',
        ),
      ]);
      currentModel.value = mc.model;
      currentProvider.value = mc.provider;
      currentBaseUrl.value = mc.baseUrl;
      modelGroups.value = groupModelsByProvider(savedModels);
    } catch {
      // Config not yet available
    }
  };

  onMounted(() => {
    void reload();
  });

  const selectModel = async (
    provider: string,
    model: string,
    baseUrl: string,
  ): Promise<void> => {
    if (isClawMode.value) {
      // Claw 模式：持久化到 claw config
      currentModel.value = model;
      currentProvider.value = provider;
      currentBaseUrl.value = baseUrl;
      try {
        const api = getTauriAPI();
        await api.clawConfigSet({ activeModel: model });
      } catch (e) {
        console.warn('[useModelConfig] Failed to persist Claw model:', e);
      }
      return;
    }
    const effectiveBaseUrl = provider === 'custom' ? baseUrl : '';
    await invoke('hermes_config_set_model', {
      provider,
      model,
      baseUrl: effectiveBaseUrl,
      profile: profile || null,
    });
    currentModel.value = model;
    currentProvider.value = provider;
    currentBaseUrl.value = effectiveBaseUrl;
  };

  const displayModel = computed(() =>
    currentModel.value
      ? currentModel.value.split('/').pop() || currentModel.value
      : currentProvider.value === 'auto'
        ? 'Auto'
        : 'No model',
  );

  const subAgentDisplayModel = computed(() =>
    subAgentModel.value
      ? subAgentModel.value.split('/').pop() || subAgentModel.value
      : '',
  );

  const selectSubAgentModel = async (model: string): Promise<void> => {
    subAgentModel.value = model;
    if (isClawMode.value) {
      try {
        const api = getTauriAPI();
        await api.clawConfigSet({ subAgentModel: model });
      } catch (e) {
        console.warn('[useModelConfig] Failed to persist sub-agent model:', e);
      }
    }
  };

  return {
    currentModel,
    currentProvider,
    currentBaseUrl,
    modelGroups,
    displayModel,
    subAgentModel,
    subAgentDisplayModel,
    reload,
    selectModel,
    selectSubAgentModel,
  };
}
