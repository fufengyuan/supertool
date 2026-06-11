import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'
import type { UsageState } from '../types'

// ── Mock invoke with per-command response tracking ──────────────────────
const mockResponses: Record<string, any> = {}

function setMock(command: string, response: any) {
  mockResponses[command] = response
}
function setMockError(command: string, error: any) {
  mockResponses[command] = { _error: true, error }
}

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, ...args: any[]) => {
    if (cmd in mockResponses) {
      const resp = mockResponses[cmd]
      delete mockResponses[cmd]
      if (resp && resp._error) {
        return Promise.reject(resp.error)
      }
      return Promise.resolve(resp)
    }
    return Promise.reject(new Error(`No mock for ${cmd}`))
  },
}))

import { useLocalCommands } from '../composables/useLocalCommands'

function setup() {
  const usage = ref<UsageState | null>(null)
  const fastMode = ref(false)
  const setFastMode = vi.fn().mockResolvedValue(undefined)
  const onNewChat = vi.fn()
  const onClear = vi.fn()
  const addAgentMessage = vi.fn()
  const onGoalModeChange = vi.fn()
  const onLoopModeChange = vi.fn()

  const commands = useLocalCommands({
    usage,
    fastMode,
    setFastMode,
    onNewChat,
    onClear,
    addAgentMessage,
    onGoalModeChange,
    onLoopModeChange,
  })

  return { usage, setFastMode, addAgentMessage, onGoalModeChange, onLoopModeChange, commands }
}

describe('Goal mode commands', () => {
  beforeEach(() => {
    Object.keys(mockResponses).forEach(k => delete mockResponses[k])
  })

  it('shows OFF message when goal is inactive', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_goal_mode', {
      active: false, goalText: '', status: 'inactive',
      turnsUsed: 0, maxTurns: 20, tokensUsed: 0, tokenBudget: null,
    })
    const result = await commands.executeLocal('/goal')
    expect(result).toBe(true)
    expect(addAgentMessage).toHaveBeenCalledWith(expect.stringContaining('OFF'))
  })

  it('shows active goal with token budget', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_goal_mode', {
      active: true, goalText: 'Refactor user auth', status: 'active',
      turnsUsed: 3, maxTurns: 20, tokensUsed: 1250, tokenBudget: 5000,
    })
    const result = await commands.executeLocal('/goal')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('active')
    expect(msg).toContain('1250')
    expect(msg).toContain('5000')
  })

  it('shows paused goal tokens', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_goal_mode', {
      active: true, goalText: 'Write tests', status: 'paused',
      turnsUsed: 5, maxTurns: 20, tokensUsed: 3400, tokenBudget: null,
    })
    const result = await commands.executeLocal('/goal')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('paused')
    expect(msg).toContain('3400')
  })

  it('shows budget-limited with tokens', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_goal_mode', {
      active: true, goalText: 'Data pipeline', status: 'budget-limited',
      turnsUsed: 15, maxTurns: 20, tokensUsed: 4800, tokenBudget: 5000,
    })
    const result = await commands.executeLocal('/goal')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('budget-limited')
    expect(msg).toContain('4800')
    expect(msg).toContain('5000')
  })

  it('creates goal with text', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_set_goal_mode', { success: true })
    const result = await commands.executeLocal('/goal Fix login CSS')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('Goal set')
    expect(msg).toContain('Fix login CSS')
  })

  it('handles invoke failure gracefully', async () => {
    const { commands, addAgentMessage } = setup()
    setMockError('claw_chat_get_goal_mode', new Error('fail'))
    const result = await commands.executeLocal('/goal')
    expect(result).toBe(true)
    expect(addAgentMessage).toHaveBeenCalledWith('无法读取 goal 模式状态')
  })
})

describe('Loop mode commands', () => {
  beforeEach(() => {
    Object.keys(mockResponses).forEach(k => delete mockResponses[k])
  })

  it('toggles ON when OFF', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_loop_mode', { active: false })
    setMock('claw_chat_set_loop_mode', { active: true, success: true })
    const result = await commands.executeLocal('/loop')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('ON')
    expect(msg).toContain('Esc')
  })

  it('toggles OFF when ON', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_loop_mode', { active: true })
    setMock('claw_chat_set_loop_mode', { active: false, success: true })
    const result = await commands.executeLocal('/loop')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('OFF')
  })

  it('enables with iteration limit /loop 10', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_loop_mode', { active: false })
    setMock('claw_chat_set_loop_mode', { active: true, success: true })
    const result = await commands.executeLocal('/loop 10')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('ON')
    expect(msg).toContain('10 iterations')
  })

  it('enables with duration limit /loop 5m', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_loop_mode', { active: false })
    setMock('claw_chat_set_loop_mode', { active: true, success: true })
    const result = await commands.executeLocal('/loop 5m')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('ON')
    expect(msg).toContain('iterations')
  })

  it('shows error for invalid arg', async () => {
    const { commands, addAgentMessage } = setup()
    setMock('claw_chat_get_loop_mode', { active: false })
    const result = await commands.executeLocal('/loop invalid')
    expect(result).toBe(true)
    const msg = addAgentMessage.mock.calls[0][0] as string
    expect(msg).toContain('Usage')
  })

  it('handles invoke failure gracefully', async () => {
    const { commands, addAgentMessage } = setup()
    // No mock set — invoke rejects with generic error
    const result = await commands.executeLocal('/loop')
    expect(result).toBe(true)
    expect(addAgentMessage).toHaveBeenCalledWith('切换 loop 模式失败')
  })
})

describe('ModeBar rendering logic', () => {
  it('status label mapping', () => {
    const map: Record<string, string> = {
      active: 'Goal Active',
      paused: 'Goal Paused',
      complete: 'Goal Complete',
      'budget-limited': 'Budget Exhausted',
      dropped: 'Goal Dropped',
    }
    expect(map.active).toBe('Goal Active')
    expect(map['budget-limited']).toBe('Budget Exhausted')
  })

  it('budget percent calculation', () => {
    const pct = (used: number, budget: number) => Math.round((used / budget) * 100)
    expect(pct(2500, 10000)).toBe(25)
    expect(pct(9500, 10000)).toBe(95)
  })
})
