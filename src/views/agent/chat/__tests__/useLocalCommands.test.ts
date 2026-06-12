import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'
import type { UsageState } from '../types'

const mockedInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockedInvoke(...args),
}))

import { useLocalCommands } from '../composables/useLocalCommands'

function setup() {
  const usage = ref<UsageState | null>(null)
  const onNewChat = vi.fn()
  const onClear = vi.fn()
  const addAgentMessage = vi.fn()

  const commands = useLocalCommands({
    usage,
    onNewChat,
    onClear,
    addAgentMessage,
  })

  return { usage, onNewChat, onClear, addAgentMessage, commands }
}

describe('useLocalCommands', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('isLocal', () => {
    it('should return false for non-slash text', () => {
      const { commands } = setup()
      expect(commands.isLocal('hello')).toBe(false)
    })

    it('should return true for /new', () => {
      const { commands } = setup()
      expect(commands.isLocal('/new')).toBe(true)
    })

    it('should return true for /clear', () => {
      const { commands } = setup()
      expect(commands.isLocal('/clear')).toBe(true)
    })

    it('should return true for /help', () => {
      const { commands } = setup()
      expect(commands.isLocal('/help')).toBe(true)
    })

    it('should return true for /model', () => {
      const { commands } = setup()
      expect(commands.isLocal('/model')).toBe(true)
    })

    it('should return true for /tools', () => {
      const { commands } = setup()
      expect(commands.isLocal('/tools')).toBe(true)
    })

    it('should return true for /skills', () => {
      const { commands } = setup()
      expect(commands.isLocal('/skills')).toBe(true)
    })

    it('should return true for /memory', () => {
      const { commands } = setup()
      expect(commands.isLocal('/memory')).toBe(true)
    })

    it('should return false for /approve (non-local)', () => {
      const { commands } = setup()
      expect(commands.isLocal('/approve')).toBe(false)
    })

    it('should return false for /btw (non-local)', () => {
      const { commands } = setup()
      expect(commands.isLocal('/btw')).toBe(false)
    })

    it('should handle leading whitespace (not trimmed — false)', () => {
      const { commands } = setup()
      // The composable uses text.startsWith('/') without trimming,
      // so leading whitespace results in false
      expect(commands.isLocal('  /new')).toBe(false)
    })
  })

  describe('executeLocal', () => {
    it('/new should call onNewChat', async () => {
      const { commands, onNewChat } = setup()
      const result = await commands.executeLocal('/new')
      expect(result).toBe(true)
      expect(onNewChat).toHaveBeenCalled()
    })

    it('/clear should call onClear', async () => {
      const { commands, onClear } = setup()
      const result = await commands.executeLocal('/clear')
      expect(result).toBe(true)
      expect(onClear).toHaveBeenCalled()
    })

    it('/help should show help text', async () => {
      const { commands, addAgentMessage } = setup()
      const result = await commands.executeLocal('/help')
      expect(result).toBe(true)
      expect(addAgentMessage).toHaveBeenCalled()
      const msg = addAgentMessage.mock.calls[0][0] as string
      expect(msg).toContain('Available Commands')
    })

    it('/model should show current model', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockResolvedValue({ model: 'claude-sonnet-4', provider: 'anthropic', baseUrl: '' })
      const result = await commands.executeLocal('/model')
      expect(result).toBe(true)
      expect(addAgentMessage).toHaveBeenCalled()
      const msg = addAgentMessage.mock.calls[0][0] as string
      expect(msg).toContain('claude-sonnet-4')
    })

    it('/model should show error when invoke fails', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockRejectedValue(new Error('not available'))
      const result = await commands.executeLocal('/model')
      expect(result).toBe(true)
      expect(addAgentMessage).toHaveBeenCalledWith('无法获取模型配置')
    })

    it('/memory should show memory content', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockResolvedValue({
        content: 'User prefers Python',
        stats: { totalSessions: 10, totalMessages: 50 },
      })
      const result = await commands.executeLocal('/memory')
      expect(result).toBe(true)
      const msg = addAgentMessage.mock.calls[0][0] as string
      expect(msg).toContain('User prefers Python')
      expect(msg).toContain('10 sessions')
      expect(msg).toContain('50 messages')
    })

    it('/memory should handle empty memory', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockResolvedValue({
        content: '',
        stats: { totalSessions: 0, totalMessages: 0 },
      })
      const result = await commands.executeLocal('/memory')
      expect(result).toBe(true)
      const msg = addAgentMessage.mock.calls[0][0] as string
      expect(msg).toContain('暂无记忆条目')
    })

    it('/memory should handle invoke failure', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockRejectedValue(new Error('fail'))
      const result = await commands.executeLocal('/memory')
      expect(result).toBe(true)
      expect(addAgentMessage).toHaveBeenCalledWith('无法读取 Agent 记忆')
    })

    it('/usage should show token usage when available', async () => {
      const { commands, addAgentMessage, usage } = setup()
      usage.value = { promptTokens: 100, completionTokens: 50, totalTokens: 150, cost: 0.002 }
      const result = await commands.executeLocal('/usage')
      expect(result).toBe(true)
      const msg = addAgentMessage.mock.calls[0][0] as string
      expect(msg).toContain('100')
      expect(msg).toContain('50')
      expect(msg).toContain('150')
      expect(msg).toContain('$0.0020')
    })

    it('/usage should show empty state when no usage', async () => {
      const { commands, addAgentMessage } = setup()
      const result = await commands.executeLocal('/usage')
      expect(result).toBe(true)
      expect(addAgentMessage).toHaveBeenCalledWith('暂无用量数据')
    })

    it('/tools should show tools list', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockResolvedValue([
        { label: 'web_search', description: 'Search the web', enabled: true },
      ])
      const result = await commands.executeLocal('/tools')
      expect(result).toBe(true)
      const msg = addAgentMessage.mock.calls[0][0] as string
      expect(msg).toContain('web_search')
    })

    it('/tools should handle empty list', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockResolvedValue([])
      const result = await commands.executeLocal('/tools')
      expect(result).toBe(true)
      expect(addAgentMessage).toHaveBeenCalledWith('未找到可用工具')
    })

    it('/tools should handle invoke failure', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockRejectedValue(new Error('fail'))
      const result = await commands.executeLocal('/tools')
      expect(result).toBe(true)
      expect(addAgentMessage).toHaveBeenCalledWith('无法获取工具列表')
    })

    it('/skills should show skills list', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockResolvedValue([
        { name: 'test-skill', category: 'testing', description: 'A test skill' },
      ])
      const result = await commands.executeLocal('/skills')
      expect(result).toBe(true)
      const msg = addAgentMessage.mock.calls[0][0] as string
      expect(msg).toContain('test-skill')
    })

    it('/skills should handle empty list', async () => {
      const { commands, addAgentMessage } = setup()
      mockedInvoke.mockResolvedValue([])
      const result = await commands.executeLocal('/skills')
      expect(result).toBe(true)
      expect(addAgentMessage).toHaveBeenCalledWith('No skills installed.')
    })

    it('should return false for unknown command', async () => {
      const { commands } = setup()
      const result = await commands.executeLocal('/nonexistent')
      expect(result).toBe(false)
    })
  })
})
