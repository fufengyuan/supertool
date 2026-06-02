/**
 * useLocalCommands — slash commands handled locally without backend.
 *
 * Adapted from hermes-desktop's useLocalCommands for Vue 3 + Tauri.
 * Commands like /new, /clear, /help, /usage, /fast are handled locally;
 * other commands are sent to the backend as regular messages.
 */
import { invoke } from '@tauri-apps/api/core'
import { SLASH_COMMANDS } from '@/composables/useSlashCommands'
import type { UsageState } from '@/views/agent/chat/types'

export interface UseLocalCommandsOptions {
  usage: UsageState | null
  addAgentMessage: (content: string) => void
  onNewChat?: () => void
  onClear: () => void
  toggleFastMode?: () => Promise<void>
}

export function useLocalCommands(options: UseLocalCommandsOptions) {
  const { usage, addAgentMessage, onNewChat, onClear, toggleFastMode } = options

  function isLocal(text: string): boolean {
    if (!text.startsWith('/')) return false
    const cmd = text.split(/\s+/)[0].toLowerCase()
    return SLASH_COMMANDS.some(
      (c) => c.name === cmd && (c.local || c.category === 'info'),
    )
  }

  async function executeLocal(text: string): Promise<boolean> {
    const cmd = text.trim().split(/\s+/)[0].toLowerCase()

    switch (cmd) {
      case '/new':
        onNewChat?.()
        return true

      case '/clear':
        onClear()
        return true

      case '/model': {
        try {
          const result = await invoke<{ defaultModel?: string; activeProvider?: string }>('get_models')
          const display = result.defaultModel || 'Not set'
          const prov = result.activeProvider || 'auto'
          addAgentMessage(
            `**Current model:** \`${display}\`\n**Provider:** ${prov}`,
          )
        } catch {
          addAgentMessage('Unable to read model configuration.')
        }
        return true
      }

      case '/memory': {
        try {
          const mem = await invoke<{ memory: { exists: boolean; content: string }; stats: { totalSessions: number; totalMessages: number } }>('hermes_memory_read')
          const lines = ['**Agent Memory**\n']
          if (mem.memory.exists && mem.memory.content.trim()) {
            lines.push(mem.memory.content.trim())
          } else {
            lines.push('No memory entries yet.')
          }
          lines.push(
            `\n**Stats:** ${mem.stats.totalSessions} sessions, ${mem.stats.totalMessages} messages`,
          )
          addAgentMessage(lines.join('\n'))
        } catch {
          addAgentMessage('Unable to read agent memory.')
        }
        return true
      }

      case '/tools': {
        try {
          const tools = await invoke<Array<{ label: string; description: string; enabled: boolean }>>('list_toolsets')
          if (!tools.length) {
            addAgentMessage('No toolsets found.')
          } else {
            const rows = tools
              .map(
                (tool) =>
                  `- **${tool.label}** — ${tool.description} ${tool.enabled ? '*(enabled)*' : '*(disabled)*'}`,
              )
              .join('\n')
            addAgentMessage(`**Available Toolsets**\n\n${rows}`)
          }
        } catch {
          addAgentMessage('Unable to list toolsets.')
        }
        return true
      }

      case '/fast': {
        if (toggleFastMode) {
          await toggleFastMode()
        }
        addAgentMessage(
          '**Fast Mode toggled.** Priority processing enabled for lower latency.',
        )
        return true
      }

      case '/usage': {
        if (usage) {
          const lines = [
            `**Token Usage**\n`,
            `- **Prompt:** ${usage.promptTokens.toLocaleString()} tokens`,
            `- **Completion:** ${usage.completionTokens.toLocaleString()} tokens`,
            `- **Total:** ${usage.totalTokens.toLocaleString()} tokens`,
          ]
          if (usage.cost != null) lines.push(`- **Cost:** $${usage.cost.toFixed(4)}`)
          addAgentMessage(lines.join('\n'))
        } else {
          addAgentMessage('No token usage data available for this session.')
        }
        return true
      }

      case '/help': {
        const grouped = new Map<string, typeof SLASH_COMMANDS>()
        for (const c of SLASH_COMMANDS) {
          const arr = grouped.get(c.category) ?? []
          arr.push(c)
          grouped.set(c.category, arr)
        }
        const categoryLabels: Record<string, string> = {
          chat: 'Chat',
          agent: 'Agent',
          tools: 'Tools',
          info: 'Info',
        }
        let md = '**Available Commands**\n'
        for (const cat of ['chat', 'agent', 'tools', 'info'] as const) {
          const cmds = grouped.get(cat)
          if (!cmds) continue
          md += `\n**${categoryLabels[cat]}**\n`
          for (const c of cmds) md += `\`${c.name}\` — ${c.description}\n`
        }
        addAgentMessage(md)
        return true
      }

      case '/version': {
        addAgentMessage('**SuperTool** v4.0.0')
        return true
      }

      default:
        return false
    }
  }

  return { isLocal, executeLocal }
}
