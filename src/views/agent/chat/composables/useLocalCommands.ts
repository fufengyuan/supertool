import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Ref } from 'vue';
import { SLASH_COMMANDS } from '../slashCommands';
import type { UsageState } from '../types';

interface UseLocalCommandsArgs {
  usage: Ref<UsageState | null>;
  fastMode: Ref<boolean>;
  setFastMode: (next: boolean) => Promise<void>;
  onNewChat?: () => void;
  onClear: () => void;
  addAgentMessage: (content: string) => void;
}

interface UseLocalCommandsResult {
  executeLocal: (text: string) => Promise<boolean>;
  isLocal: (text: string) => boolean;
}

function isLocallyHandled(text: string): boolean {
  if (!text.startsWith('/')) return false;
  const cmd = text.split(/\s+/)[0].toLowerCase();
  return SLASH_COMMANDS.some(
    (c) => c.name === cmd && (c.local || c.category === 'info'),
  );
}

/**
 * Encapsulates slash commands handled locally without talking to the backend.
 */
export function useLocalCommands({
  usage,
  fastMode,
  setFastMode,
  onNewChat,
  onClear,
  addAgentMessage,
}: UseLocalCommandsArgs): UseLocalCommandsResult {
  const usageRef = usage;

  const executeLocal = async (cmdText: string): Promise<boolean> => {
    const cmd = cmdText.trim().split(/\s+/)[0].toLowerCase();

    switch (cmd) {
      case '/new':
        onNewChat?.();
        return true;

      case '/clear':
        onClear();
        return true;

      case '/model': {
        try {
          const mc = await invoke<{ model: string; provider: string; baseUrl: string }>(
            'hermes_config_get_model',
          );
          const display = mc.model || 'Not set';
          const prov = mc.provider || 'auto';
          addAgentMessage(
            `**Current model:** \`${display}\`\n**Provider:** ${prov}` +
              (mc.baseUrl ? `\n**Base URL:** ${mc.baseUrl}` : ''),
          );
        } catch {
          addAgentMessage('无法获取模型配置');
        }
        return true;
      }

      case '/memory': {
        try {
          const mem = await invoke<{ content: string; stats: { totalSessions: number; totalMessages: number } }>(
            'hermes_memory_read',
          );
          const lines: string[] = ['**Agent Memory**\n'];
          if (mem.content?.trim()) {
            lines.push(mem.content.trim());
          } else {
            lines.push('暂无记忆条目');
          }
          lines.push(
            `\n**Stats:** ${mem.stats.totalSessions} sessions, ${mem.stats.totalMessages} messages`,
          );
          addAgentMessage(lines.join('\n'));
        } catch {
          addAgentMessage('无法读取 Agent 记忆');
        }
        return true;
      }

      case '/tools': {
        try {
          const tools = await invoke<Array<{ label: string; description: string; enabled: boolean }>>(
            'hermes_skills_list',
          );
          if (!tools.length) {
            addAgentMessage('未找到可用工具');
          } else {
            const rows = tools
              .map(
                (tool) =>
                  `- **${tool.label}** — ${tool.description} ${tool.enabled ? '*(enabled)*' : '*(disabled)*'}`,
              )
              .join('\n');
            addAgentMessage(`**Available Toolsets**\n\n${rows}`);
          }
        } catch {
          addAgentMessage('无法获取工具列表');
        }
        return true;
      }

      case '/skills': {
        try {
          const skills = await invoke<Array<{ name: string; category: string; description: string }>>(
            'hermes_skills_list',
          );
          if (!skills.length) {
            addAgentMessage('No skills installed.');
          } else {
            const rows = skills
              .map((s) => `- **${s.name}** (${s.category}) — ${s.description}`)
              .join('\n');
            addAgentMessage(`**Installed Skills**\n\n${rows}`);
          }
        } catch {
          addAgentMessage('无法获取技能列表');
        }
        return true;
      }

      case '/fast': {
        const isOn = fastMode.value;
        const next = !isOn;
        await setFastMode(next);
        addAgentMessage(
          next
            ? '**Fast Mode: ON** — Priority processing enabled for lower latency.'
            : '**Fast Mode: OFF** — Standard processing restored.',
        );
        return true;
      }

      case '/usage': {
        const u = usageRef.value;
        if (u) {
          const lines = [
            `**Token Usage**\n`,
            `- **Prompt:** ${u.promptTokens.toLocaleString()} tokens`,
            `- **Completion:** ${u.completionTokens.toLocaleString()} tokens`,
            `- **Total:** ${u.totalTokens.toLocaleString()} tokens`,
          ];
          if (u.cost != null) lines.push(`- **Cost:** $${u.cost.toFixed(4)}`);
          addAgentMessage(lines.join('\n'));
        } else {
          addAgentMessage('暂无用量数据');
        }
        return true;
      }

      case '/help': {
        const categoryLabels: Record<string, string> = {
          chat: 'Chat 控制',
          agent: 'Agent 命令',
          tools: '工具命令',
          info: '信息命令',
        };
        const grouped = new Map<string, typeof SLASH_COMMANDS>();
        for (const c of SLASH_COMMANDS) {
          const arr = grouped.get(c.category) ?? [];
          arr.push(c);
          grouped.set(c.category, arr);
        }
        let md = `**Available Commands**\n`;
        for (const cat of ['chat', 'agent', 'tools', 'info'] as const) {
          const cmds = grouped.get(cat);
          if (!cmds) continue;
          md += `\n**${categoryLabels[cat]}**\n`;
          for (const c of cmds) md += `\`${c.name}\` — ${c.description}\n`;
        }
        addAgentMessage(md);
        return true;
      }

      default:
        return false;
    }
  };

  return { executeLocal, isLocal: isLocallyHandled };
}
