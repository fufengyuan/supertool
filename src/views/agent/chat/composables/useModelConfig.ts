import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Ref } from 'vue';
import type { ModelGroup } from '../types';

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
  reload: () => Promise<void>;
  selectModel: (
    provider: string,
    model: string,
    baseUrl: string,
  ) => Promise<void>;
}

export function useModelConfig(profile?: string): UseModelConfigResult {
  const currentModel = ref('');
  const currentProvider = ref('auto');
  const currentBaseUrl = ref('');
  const modelGroups = ref<ModelGroup[]>([]);
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
      // Claw 模式：从 claw_chat_info 读取配置
      try {
        const info = await invoke<{
          model: string;
          provider: string;
          baseUrl: string | null;
          apiKeyConfigured: boolean;
          configSource: string;
        }>('claw_chat_info');

        currentModel.value = info.model || 'claude-sonnet-4-6';
        currentProvider.value = info.provider || 'Hermes Config';
        currentBaseUrl.value = info.baseUrl || '';
        modelGroups.value = [{
          provider: 'Hermes Config',
          providerLabel: 'Hermes Config',
          models: [{
            provider: 'Hermes Config',
            model: info.model || 'claude-sonnet-4-6',
            label: info.model || 'claude-sonnet-4-6',
            baseUrl: info.baseUrl || '',
          }],
        }];
      } catch {
        // Fallback
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
      // Claw 模式：模型选择暂不持久化
      currentModel.value = model;
      currentProvider.value = provider;
      currentBaseUrl.value = baseUrl;
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

  return {
    currentModel,
    currentProvider,
    currentBaseUrl,
    modelGroups,
    displayModel,
    reload,
    selectModel,
  };
}
