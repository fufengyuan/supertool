/**
 * agentModeStore — Agent 模式切换（hermes / claw）
 *
 * 所有 Agent 页面共享同一个模式状态，保存在 localStorage 中。
 */
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type AgentMode = 'hermes' | 'claw'

const STORAGE_KEY = 'supertool:agentMode'

function loadMode(): AgentMode {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored === 'hermes' || stored === 'claw') return stored
  } catch { /* ignore */ }
  return 'hermes'
}

export const useAgentModeStore = defineStore('agentMode', () => {
  const mode = ref<AgentMode>(loadMode())

  function setMode(m: AgentMode) {
    mode.value = m
  }

  function toggle() {
    mode.value = mode.value === 'hermes' ? 'claw' : 'hermes'
  }

  // 持久化
  watch(mode, (val) => {
    try { localStorage.setItem(STORAGE_KEY, val) } catch { /* ignore */ }
  })

  return { mode, setMode, toggle }
})
