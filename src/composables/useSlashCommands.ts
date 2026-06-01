/**
 * useSlashCommands — 斜杠命令自动补全菜单管理
 *
 * 对标 hermes-desktop，提供 30+ 命令，分四类：
 *   chat  — 对话控制（新建/清空）
 *   agent — Agent 操作（状态/重试/快速模式等）
 *   tools — 工具调用（Web/Code/Shell 等）
 *   info  — 信息查询（Help/Usage/Version 等）
 *
 * 使用方法：
 *   1. 在 ChatInput 中 import { useSlashCommands } from '@/composables/useSlashCommands'
 *   2. 创建实例，传入 onNewChat / onClear 回调
 *   3. watch(inputText, text => slash.updateInputText(text))
 *   4. 在 @keydown 中先调用 slash.handleSlashKeydown(e, sendText)
 *   5. 在模板中用 filteredSlashCommands / isSlashMenuVisible / slashMenuIndex 渲染弹窗
 */

import { ref, computed } from 'vue'
import type { Ref, ComputedRef } from 'vue'

// ---- 类型 ----

export interface SlashCommand {
  name: string
  description: string
  category: 'chat' | 'agent' | 'tools' | 'info'
  /** 为 true 时由前端本地处理；false 时当作普通文本发送给后端 Agent */
  local: boolean
}

export interface SlashCommandsOptions {
  onNewChat: () => void
  onClear: () => void
  /** 添加一条助手消息到对话（用于 help/usage/version 等本地命令的反馈） */
  addAgentMessage: (content: string) => void
  /** Token 用量信息（可选） */
  usageStats?: { inputTokens: number; outputTokens: number; totalTokens: number } | null
}

// ---- 命令列表（共 30+ 条，与 hermes-desktop 对齐） ----

export const SLASH_COMMANDS: SlashCommand[] = [
  // ===== Chat =====
  { name: '/new',     description: 'Start a new chat',                  category: 'chat',  local: true },
  { name: '/clear',   description: 'Clear conversation history',        category: 'chat',  local: true },

  // ===== Agent =====
  { name: '/btw',     description: 'Ask a side question without affecting context', category: 'agent', local: false },
  { name: '/approve', description: 'Approve a pending action',           category: 'agent', local: false },
  { name: '/deny',    description: 'Deny a pending action',              category: 'agent', local: false },
  { name: '/status',  description: 'Show current agent status',          category: 'agent', local: false },
  { name: '/compact', description: 'Compact and summarize the conversation',           category: 'agent', local: false },
  { name: '/reset',   description: 'Reset conversation context',         category: 'agent', local: false },
  { name: '/undo',    description: 'Undo the last action',               category: 'agent', local: false },
  { name: '/retry',   description: 'Retry the last failed action',       category: 'agent', local: false },
  { name: '/compress',description: 'Compress conversation with optional focus topic',  category: 'agent', local: false },
  { name: '/steer',   description: 'Steer the in-flight agent without interrupting',   category: 'agent', local: false },
  { name: '/queue',   description: 'Queue a follow-up to run after the current turn',  category: 'agent', local: false },
  { name: '/fast',    description: 'Toggle priority processing (lower latency)',        category: 'agent', local: true },
  { name: '/debug',   description: 'Show diagnostics and debug info',    category: 'agent', local: false },
  { name: '/goal',    description: 'Lock agent onto a persistent goal (Ralph loop)',    category: 'agent', local: false },

  // ===== Tools =====
  { name: '/web',     description: 'Search the web',                    category: 'tools', local: false },
  { name: '/image',   description: 'Generate an image',                  category: 'tools', local: false },
  { name: '/browse',  description: 'Browse a URL',                      category: 'tools', local: false },
  { name: '/code',    description: 'Write or execute code',              category: 'tools', local: false },
  { name: '/file',    description: 'Read or write files',                category: 'tools', local: false },
  { name: '/shell',   description: 'Run a shell command',                category: 'tools', local: false },

  // ===== Info =====
  { name: '/help',    description: 'Show available commands and help',   category: 'info',  local: true },
  { name: '/usage',   description: 'Show token usage and cost',          category: 'info',  local: true },
  { name: '/version', description: 'Show version information',           category: 'info',  local: true },
  { name: '/model',   description: 'Show or switch the current model',   category: 'info',  local: true },
  { name: '/tools',   description: 'List available tools',               category: 'info',  local: false },
  { name: '/skills',  description: 'List installed skills',              category: 'info',  local: false },
  { name: '/memory',  description: 'Show agent memory',                  category: 'info',  local: false },
  { name: '/persona', description: 'Show current persona',               category: 'info',  local: false },
  { name: '/kanban',  description: 'List or operate on kanban tasks',    category: 'info',  local: false },
  { name: '/curator', description: 'Show curator status (usage-ranked skills)',          category: 'info',  local: false },
]

// ---- 类别图标映射 ----

export const CATEGORY_ICONS: Record<string, string> = {
  chat:  'messageSquare',
  agent: 'zap',
  tools: 'tool',
  info:  'info',
}

// ---- 类别中文标签 ----

export const CATEGORY_LABELS: Record<string, string> = {
  chat:  'Chat',
  agent: 'Agent',
  tools: 'Tools',
  info:  'Info',
}

// ---- composable ----

export function useSlashCommands(options: SlashCommandsOptions) {
  const { onNewChat, onClear, addAgentMessage, usageStats } = options

  /** 是否显示斜杠菜单 */
  const isSlashMenuVisible: Ref<boolean> = ref(false)
  /** 当前选中项索引 */
  const slashMenuIndex: Ref<number> = ref(0)
  /** 当前输入框文本（由调用方通过 updateInputText 更新） */
  const currentInputText: Ref<string> = ref('')

  /** 上次输入的 '/'(不含空格) —— 用于在输入参数时也保持菜单打开 */
  function getSlashPrefix(text: string): string {
    const trimmed = text.trim()
    // 只有第一个 token 是斜杠命令
    const firstToken = trimmed.split(/\s+/)[0] ?? ''
    if (!firstToken.startsWith('/')) { return '' }
    return firstToken.toLowerCase()
  }

  /** 根据输入实时过滤匹配的命令 */
  const filteredSlashCommands: ComputedRef<SlashCommand[]> = computed(() => {
    const text = currentInputText.value.trim()
    if (!text.startsWith('/')) { return [] }
    const prefix = getSlashPrefix(text)
    if (!prefix) { return [] }
    // 精确匹配模式：输入 /help → 只返回 /help 本身
    const exactMatch = SLASH_COMMANDS.find(cmd => cmd.name === prefix)
    if (exactMatch) {
      return [exactMatch]
    }
    // 模糊匹配：输入 /h → 返回所有以 /h 开头的命令
    return SLASH_COMMANDS.filter(cmd => cmd.name.startsWith(prefix))
  })

  /** 隐藏菜单 */
  function hideSlashMenu(): void {
    isSlashMenuVisible.value = false
    slashMenuIndex.value = 0
  }

  /**
   * 更新输入文本并同步菜单状态。
   * 调用方应在 watch(inputText) 中调用。
   */
  function updateInputText(text: string): void {
    currentInputText.value = text
    if (text.startsWith('/')) {
      const prefix = getSlashPrefix(text)
      if (prefix) {
        // 精确匹配 + 已有参数（包括空格）→ 菜单应隐藏（用户已在输入参数）
        const exactCmd = SLASH_COMMANDS.find(cmd => cmd.name === prefix)
        if (exactCmd) {
          const afterCmd = text.trim().slice(exactCmd.name.length)
          if (afterCmd.length > 0) {
            hideSlashMenu()
            return
          }
        }
        const matches = SLASH_COMMANDS.filter(cmd => cmd.name.startsWith(prefix))
        if (matches.length > 0) {
          isSlashMenuVisible.value = true
          slashMenuIndex.value = 0
          return
        }
      }
    }
    // 不匹配任何命令时隐藏
    if (isSlashMenuVisible.value) {
      hideSlashMenu()
    }
  }

  /**
   * 处理斜杠菜单键盘事件。
   * @param e        键盘事件
   * @param emitSend 发送文本到输入框的回调（用于选中非本地命令时填入输入框）
   * @returns true 表示事件已被消费
   */
  function handleSlashKeydown(e: KeyboardEvent, emitSend?: (text: string) => void): boolean {
    if (!isSlashMenuVisible.value) { return false }

    const commands = filteredSlashCommands.value
    if (commands.length === 0) {
      hideSlashMenu()
      return false
    }

    switch (e.key) {
      case 'ArrowDown': {
        e.preventDefault()
        e.stopPropagation()
        slashMenuIndex.value = (slashMenuIndex.value + 1) % commands.length
        return true
      }
      case 'ArrowUp': {
        e.preventDefault()
        e.stopPropagation()
        slashMenuIndex.value = (slashMenuIndex.value - 1 + commands.length) % commands.length
        return true
      }
      case 'Tab':
      case 'Enter': {
        e.preventDefault()
        e.stopPropagation()
        const selected = commands[slashMenuIndex.value] ?? commands[0]
        if (selected.local) {
          // 本地命令：执行后清空输入
          executeLocal(selected.name)
        } else if (emitSend) {
          // 非本地命令：填入输入框末尾（含空格，方便继续说参数）
          emitSend(selected.name + ' ')
        }
        hideSlashMenu()
        return true
      }
      case 'Escape': {
        e.preventDefault()
        e.stopPropagation()
        hideSlashMenu()
        return true
      }
    }

    return false
  }

  /**
   * 执行本地命令。
   * 返回 true 表示已处理。
   */
  async function executeLocal(text: string): Promise<boolean> {
    const trimmed = text.trim()
    const commandName = trimmed.split(/\s+/)[0]?.toLowerCase()
    if (!commandName) { return false }

    switch (commandName) {
      case '/new':
        onNewChat()
        return true

      case '/clear':
        onClear()
        return true

      case '/help': {
        const grouped: Record<string, SlashCommand[]> = {}
        for (const cmd of SLASH_COMMANDS) {
          if (!grouped[cmd.category]) { grouped[cmd.category] = [] }
          grouped[cmd.category].push(cmd)
        }
        const lines: string[] = ['**Available Commands**\n']
        const catOrder = ['chat', 'agent', 'tools', 'info']
        for (const cat of catOrder) {
          const cmds = grouped[cat]
          if (!cmds) { continue }
          lines.push(`**${CATEGORY_LABELS[cat] ?? cat}**`)
          for (const c of cmds) {
            const badge = c.local ? 'local' : 'agent'
            lines.push(`- \`${c.name}\` — ${c.description} \`[${badge}]\``)
          }
          lines.push('')
        }
        addAgentMessage(lines.join('\n'))
        return true
      }

      case '/usage': {
        const stats = usageStats
        if (stats) {
          const msg = [
            '**Token Usage**',
            '',
            `- Prompt tokens: **${stats.inputTokens.toLocaleString()}**`,
            `- Completion tokens: **${stats.outputTokens.toLocaleString()}**`,
            `- Total: **${stats.totalTokens.toLocaleString()}**`,
          ].join('\n')
          addAgentMessage(msg)
        } else {
          addAgentMessage('No token usage data available for this session.')
        }
        return true
      }

      case '/model': {
        // 本地优先显示，实际切换由 parent 处理
        addAgentMessage('Use `/model <name>` to switch the active model.')
        return true
      }

      case '/version': {
        addAgentMessage('Version info: use the settings page to check version details.')
        return true
      }

      default:
        return false
    }
  }

  return {
    isSlashMenuVisible,
    filteredSlashCommands,
    slashMenuIndex,
    getSlashPrefix,
    updateInputText,
    handleSlashKeydown,
    hideSlashMenu,
    executeLocal,
  }
}
