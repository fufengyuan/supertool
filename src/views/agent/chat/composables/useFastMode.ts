import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Ref } from 'vue';

interface UseFastModeResult {
  fastMode: Ref<boolean>;
  toggle: () => Promise<void>;
  set: (next: boolean) => Promise<void>;
}

function isFastTier(val: unknown): boolean {
  return val === 'fast' || val === 'priority';
}

/**
 * Fast mode toggle — reads/writes agent.service_tier via Tauri IPC.
 */
export function useFastMode(): UseFastModeResult {
  const fastMode = ref(false);

  onMounted(async () => {
    try {
      const val = await invoke<string>('hermes_config_get', {
        key: 'agent.service_tier',
      });
      fastMode.value = isFastTier(val);
    } catch {
      // Config not available
    }
  });

  const set = async (next: boolean) => {
    fastMode.value = next;
    try {
      await invoke('hermes_config_set', {
        key: 'agent.service_tier',
        value: next ? 'fast' : 'normal',
      });
    } catch {
      // Config write failed silently
    }
  };

  const toggle = async () => {
    await set(!fastMode.value);
  };

  return { fastMode, toggle, set };
}
