/**
 * useSlashCommands — 斜杠命令菜单管理
 *
 * 支持的命令:
 *   /new      — 新建对话 (本地)
 *   /clear    — 清空对话 (本地)
 *   /help     — 显示帮助 (本地)
 *   /usage    — 显示 Token 用量 (本地)
 *   /model    — 切换模型 (发送到后端)
 *   /compact  — 压缩对话 (发送到后端)
 *   /btw      — 旁问（不影响上下文）(发送到后端)
 *   /approve  — 批准操作 (发送到后端)
 *   /deny     — 拒绝操作 (发送到后端)
 *   /status   — 查看状态 (发送到后端)
 *   /version  — 版本信息 (本地)
 *   /tools    — 工具列表 (发送到后端)
 *   /skills   — 技能列表 (发送到后端)
 *   /memory   — 查看记忆 (发送到后端)
 */

import { ref, computed } from 'vue';
import type { Ref, ComputedRef } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// ---- 类型 ----

export interface SlashCommand {
  name: string;
  description: string;
  local: boolean;
  category: '会话' | '模型' | '操作' | '信息';
}

export interface SlashCommandsOptions {
  onNewChat: () => void;
  onClear: () => void;
  addAgentMessage: (content: string) => void;
  /** 当前会话的 token 统计（可选，由父组件传入） */
  tokenStats?: { inputTokens: number; outputTokens: number; totalTokens: number };
}

// ---- 命令列表 ----

export const SLASH_COMMANDS: SlashCommand[] = [
  { name: '/new',     description: '新建对话',             local: true,  category: '会话' },
  { name: '/clear',   description: '清空对话',             local: true,  category: '会话' },
  { name: '/compact', description: '压缩对话',             local: false, category: '会话' },

  { name: '/model',   description: '切换模型',             local: false, category: '模型' },
  { name: '/btw',     description: '旁问（不影响上下文）',  local: false, category: '模型' },

  { name: '/approve', description: '批准操作',             local: false, category: '操作' },
  { name: '/deny',    description: '拒绝操作',             local: false, category: '操作' },

  { name: '/usage',   description: '显示 Token 用量',      local: true,  category: '信息' },
  { name: '/help',    description: '显示帮助',             local: true,  category: '信息' },
  { name: '/status',  description: '查看状态',             local: false, category: '信息' },
  { name: '/version', description: '版本信息',             local: true,  category: '信息' },
  { name: '/tools',   description: '工具列表',             local: false, category: '信息' },
  { name: '/skills',  description: '技能列表',             local: false, category: '信息' },
  { name: '/memory',  description: '查看记忆',             local: false, category: '信息' },
];

// ---- composable ----

export function useSlashCommands(options: SlashCommandsOptions) {
  const { onNewChat, onClear, addAgentMessage, tokenStats } = options;

  const isSlashMenuVisible = ref(false);
  const slashMenuIndex = ref(0);
  /** 当前输入框文本（由调用方通过 updateInputText 更新） */
  const currentInputText = ref('');

  /**
   * 判断输入是否是斜杠命令开头。
   * 调用方应在 inputText 变化时调用此方法以更新菜单可见性。
   */
  function isSlashInput(text: string): boolean {
    if (!text.startsWith('/')) return false;
    const prefix = text.trim().toLowerCase();
    if (!prefix.startsWith('/')) return false;
    return SLASH_COMMANDS.some(cmd => cmd.name.startsWith(prefix));
  }

  /** 根据当前输入文本实时过滤匹配的命令 */
  const filteredSlashCommands: ComputedRef<SlashCommand[]> = computed(() => {
    const text = currentInputText.value.trim();
    if (!text.startsWith('/')) return [];
    return SLASH_COMMANDS.filter(cmd => cmd.name.startsWith(text));
  });

  /** 隐藏斜杠菜单 */
  function hideSlashMenu(): void {
    isSlashMenuVisible.value = false;
    slashMenuIndex.value = 0;
  }

  /**
   * 更新输入框文本并同步菜单状态。
   * 调用方应在 watch(inputText) 中调用此方法。
   */
  function updateInputText(text: string): void {
    currentInputText.value = text;
    if (isSlashInput(text)) {
      isSlashMenuVisible.value = true;
      slashMenuIndex.value = 0;
    } else {
      hideSlashMenu();
    }
  }

  /**
   * 处理斜杠菜单的键盘事件。
   * 返回 true 表示事件已被消费（阻止默认行为/冒泡）。
   */
  function handleSlashKeydown(
    e: KeyboardEvent,
    emitSend?: (text: string) => void,
  ): boolean {
    const commands = filteredSlashCommands.value;
    if (commands.length === 0) {
      hideSlashMenu();
      return false;
    }

    switch (e.key) {
      case 'ArrowDown': {
        e.preventDefault();
        slashMenuIndex.value = (slashMenuIndex.value + 1) % commands.length;
        return true;
      }
      case 'ArrowUp': {
        e.preventDefault();
        slashMenuIndex.value = (slashMenuIndex.value - 1 + commands.length) % commands.length;
        return true;
      }
      case 'Tab': {
        e.preventDefault();
        if (commands.length > 0) {
          const selected = commands[slashMenuIndex.value] || commands[0];
          if (emitSend) {
            emitSend(selected.name + ' ');
          }
        }
        hideSlashMenu();
        return true;
      }
      case 'Enter': {
        if (isSlashMenuVisible.value && commands.length > 0) {
          e.preventDefault();
          const selected = commands[slashMenuIndex.value] || commands[0];
          if (selected.local) {
            executeLocal(selected.name);
          } else if (emitSend) {
            emitSend(currentInputText.value);
          }
          hideSlashMenu();
          return true;
        }
        return false;
      }
      case 'Escape': {
        if (isSlashMenuVisible.value) {
          e.preventDefault();
          hideSlashMenu();
          return true;
        }
        return false;
      }
    }
    return false;
  }

  /**
   * 执行本地命令，返回 true 表示已处理。
   */
  async function executeLocal(text: string): Promise<boolean> {
    const trimmed = text.trim();
    const commandName = trimmed.split(/\s+/)[0]?.toLowerCase();
    if (!commandName || !SLASH_COMMANDS.some(c => c.name === commandName)) {
      return false;
    }

    switch (commandName) {
      case '/new': {
        onNewChat();
        return true;
      }
      case '/clear': {
        onClear();
        return true;
      }
      case '/usage': {
        const stats = tokenStats;
        if (stats && stats.totalTokens > 0) {
          const msg = [
            '📊 **Token 用量统计**',
            '',
            `- 输入 Token: **${stats.inputTokens.toLocaleString()}**`,
            `- 输出 Token: **${stats.outputTokens.toLocaleString()}**`,
            `- 总计: **${stats.totalTokens.toLocaleString()}**`,
          ].join('\n');
          addAgentMessage(msg);
        } else {
          addAgentMessage('📊 当前会话暂无 Token 用量数据。');
        }
        return true;
      }
      case '/help': {
        const groups: Record<string, SlashCommand[]> = {};
        for (const cmd of SLASH_COMMANDS) {
          if (!groups[cmd.category]) groups[cmd.category] = [];
          groups[cmd.category].push(cmd);
        }
        const lines: string[] = ['**📖 斜杠命令帮助**', ''];
        for (const [category, cmds] of Object.entries(groups)) {
          lines.push(`**${category}**`);
          for (const cmd of cmds) {
            const badge = cmd.local ? '本地' : '后端';
            lines.push(`- \`${cmd.name}\` — ${cmd.description} \`[${badge}]\``);
          }
          lines.push('');
        }
        addAgentMessage(lines.join('\n'));
        return true;
      }
      case '/version': {
        try {
          const result = await invoke<{
            version?: string;
            hermes_version?: string;
            version_info?: string;
          }>('agent_get_stats');
          const versionInfo = result.version_info || result.version || result.hermes_version || '未知';
          addAgentMessage(`📦 **版本信息**\n\n${versionInfo}`);
        } catch {
          addAgentMessage('📦 无法获取版本信息（Hermes 可能未安装或不可用）。');
        }
        return true;
      }
      default:
        return false;
    }
  }

  return {
    isSlashMenuVisible: isSlashMenuVisible as Ref<boolean>,
    filteredSlashCommands,
    slashMenuIndex: slashMenuIndex as Ref<number>,
    isSlashInput,
    executeLocal,
    handleSlashKeydown,
    hideSlashMenu,
    updateInputText,
  };
}
