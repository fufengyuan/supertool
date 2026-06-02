/**
 * useFastMode — fast/priority mode toggle.
 *
 * Reads `agent.service_tier` from Hermes config and toggles between
 * "fast" (priority) and "normal" tiers. Adapted from hermes-desktop.
 */
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useFastMode() {
  const fastMode = ref(false)

  onMounted(async () => {
    try {
      const val = await invoke<string | null>('hermes_get_config', { key: 'agent.service_tier' })
      fastMode.value = val === 'fast' || val === 'priority'
    } catch {
      // Config not available — default to off
    }
  })

  async function set(next: boolean) {
    fastMode.value = next
    try {
      await invoke('hermes_set_config', {
        key: 'agent.service_tier',
        value: next ? 'fast' : 'normal',
      })
    } catch {
      // Best effort
    }
  }

  async function toggle() {
    await set(!fastMode.value)
  }

  return { fastMode, toggle, set }
}
