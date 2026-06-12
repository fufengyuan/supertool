import { invoke } from '@tauri-apps/api/core';
import type { Ref } from 'vue';
import { SLASH_COMMANDS } from '../slashCommands';
import type { UsageState } from '../types';

/** Parse /loop args: number = iterations, number + time unit = duration. Returns iterations count (0 = unlimited) or error string */
function parseLoopLimit(args: string): number | string {
  const trimmed = args.trim();
  if (!trimmed) return 0;
  const parts = trimmed.split(/\s+/);
  if (parts.length > 2) return 'Usage: /loop [count|duration]. Examples: /loop 10, /loop 5m, /loop 30s.';

  const token = parts.length === 1 ? parts[0] : `${parts[0]}${parts[1]}`;
  // Pure number = iterations
  const iterMatch = /^(\d+)$/.exec(token);
  if (iterMatch) {
    const n = Number(iterMatch[1]);
    if (!Number.isSafeInteger(n) || n <= 0) return 'Loop count must be a positive integer.';
    return n;
  }
  // Number + unit = duration (convert to approximate iterations: 1m ≈ 1 iteration)
  const durMatch = /^(\d+)([a-z]+)$/i.exec(token);
  if (durMatch) {
    const amount = Number(durMatch[1]);
    const unit = durMatch[2].toLowerCase();
    const unitMap: Record<string, number> = { s: 1, sec: 1, secs: 1, second: 1, seconds: 1, m: 60, min: 60, mins: 60, minute: 60, minutes: 60, h: 3600, hr: 3600, hrs: 3600, hour: 3600, hours: 3600 };
    const multiplier = unitMap[unit];
    if (!multiplier) return 'Duration unit must be seconds(s), minutes(m), or hours(h).';
    const totalSecs = amount * multiplier;
    // Convert to approximate iterations (1 iteration ≈ 30s). Min 1.
    const iterations = Math.max(1, Math.round(totalSecs / 30));
    return iterations;
  }
  return 'Usage: /loop [count|duration]. Examples: /loop 10, /loop 5m, /loop 30s.';
}

interface UseLocalCommandsArgs {
  usage: Ref<UsageState | null>;
  onNewChat?: () => void;
  onClear: () => void;
  addAgentMessage: (content: string) => void;
  /** Callback for when goal mode changes (called with active, text) */
  onGoalModeChange?: (active: boolean, goalText?: string) => void;
  /** Callback for when loop mode changes (active, maxIterations?) */
  onLoopModeChange?: (active: boolean, maxIterations?: number) => void;
}

interface UseLocalCommandsResult {
  executeLocal: (text: string) => Promise<boolean>;
  isLocal: (text: string) => boolean;
}

export function useLocalCommands({
  usage: usageRef,
  onNewChat,
  onClear,
  addAgentMessage,
  onGoalModeChange,
  onLoopModeChange,
}: UseLocalCommandsArgs): UseLocalCommandsResult {
  const localOnlyCommands = new Set<string>(['/help', '/usage', '/debug', '/new', '/clear', '/model', '/memory', '/tools', '/skills', '/goal', '/loop']);

  function isLocallyHandled(text: string): boolean {
    const cmd = text.split(/\s+/)[0].toLowerCase();
    return localOnlyCommands.has(cmd);
  }

  async function executeLocal(cmdText: string): Promise<boolean> {
    const cmd = cmdText.split(/\s+/)[0].toLowerCase();

    switch (cmd) {
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

      case '/goal': {
        const goalArg = cmdText.trim().slice('/goal'.length).trim();
        if (!goalArg) {
          // Show current goal status
          try {
            const res = await invoke<{
              active: boolean;
              goalText: string;
              status: string;
              turnsUsed: number;
              maxTurns: number;
              tokensUsed?: number;
              tokenBudget?: number;
              mode?: string;
            }>('claw_chat_get_goal_mode');
            if (res.active && res.goalText) {
              const lines: string[] = [];
              lines.push(`**Goal status:** ${res.status}`);
              lines.push(`**Goal:** ${res.goalText}`);
              if (res.tokensUsed && res.tokensUsed > 0) {
                lines.push(`**Tokens used:** ${res.tokensUsed}${res.tokenBudget ? ` / ${res.tokenBudget}` : ''}`);
              }
              addAgentMessage(lines.join('\n'));
            } else {
              addAgentMessage('**Goal mode: OFF** — Use /goal <text> to set a persistent target.');
            }
          } catch {
            addAgentMessage('无法读取 goal 模式状态');
          }
        } else {
          // Set goal
          try {
            await invoke('claw_chat_set_goal_mode', { active: true, goalText: goalArg });
            onGoalModeChange?.(true, goalArg);
            addAgentMessage(`**Goal set**\n> ${goalArg}\n\nAgent will work toward this goal persistently. Use /goal to check status or the toggle button to reset.`);
          } catch {
            addAgentMessage('设置 goal 失败');
          }
        }
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

      case '/loop': {
        try {
          const res = await invoke<{ active: boolean }>('claw_chat_get_loop_mode');
          if (res.active) {
            // Toggle OFF
            await invoke('claw_chat_set_loop_mode', { active: false });
            onLoopModeChange?.(false);
            addAgentMessage('**Loop mode: OFF** — Auto-resubmit disabled.');
          } else {
            // Parse limit: /loop 10 (iterations), /loop 5m (5 minutes)
            const loopArg = cmdText.trim().slice('/loop'.length).trim();
            if (loopArg) {
              const parsed = parseLoopLimit(loopArg);
              if (typeof parsed === 'string') {
                addAgentMessage(`❌ ${parsed}`);
                return true;
              }
              await invoke('claw_chat_set_loop_mode', { active: true });
              onLoopModeChange?.(true, parsed);
              const limitText = parsed > 0 ? ` Limited to ${parsed} iterations.` : '';
              addAgentMessage(`**Loop mode: ON** — Prompt will auto-resubmit after each turn.${limitText} Esc cancels current iteration; /loop again to disable.`);
            } else {
              // Toggle ON (no limit)
              await invoke('claw_chat_set_loop_mode', { active: true });
              onLoopModeChange?.(true, 0);
              addAgentMessage('**Loop mode: ON** — Prompt will auto-resubmit after each turn. Esc cancels current iteration; /loop again to disable.');
            }
          }
        } catch {
          addAgentMessage('切换 loop 模式失败');
        }
        return true;
      }

      case '/debug': {
        const lines = ['**Debug Info**'];
        const u = usageRef.value;
        if (u) {
          lines.push(`- **Tokens:** ${u.totalTokens.toLocaleString()} total`);
        }
        addAgentMessage(lines.join('\n'));
        return true;
      }

      case '/new': {
        onNewChat?.();
        return true;
      }

      case '/clear': {
        onClear();
        return true;
      }

      case '/model': {
        try {
          const res = await invoke<{ model: string; provider: string; baseUrl: string }>('claw_chat_get_model_config');
          addAgentMessage(`**Current Model**\n- **Model:** ${res.model}\n- **Provider:** ${res.provider}\n- **Base URL:** ${res.baseUrl || '(default)'}`);
        } catch {
          addAgentMessage('无法获取模型配置');
        }
        return true;
      }

      case '/memory': {
        try {
          const res = await invoke<{ content: string; stats: { totalSessions: number; totalMessages: number } }>('claw_read_memory');
          if (res.content) {
            addAgentMessage(`**Memory**\n${res.content}\n\n**Stats:** ${res.stats.totalSessions} sessions, ${res.stats.totalMessages} messages`);
          } else {
            addAgentMessage('暂无记忆条目');
          }
        } catch {
          addAgentMessage('无法读取 Agent 记忆');
        }
        return true;
      }

      case '/tools': {
        try {
          const tools = await invoke<{ label: string; description: string; enabled: boolean }[]>('claw_chat_get_tools');
          if (tools.length === 0) {
            addAgentMessage('未找到可用工具');
          } else {
            const lines = ['**Available Tools**'];
            for (const t of tools) {
              lines.push(`- **${t.label}** ${t.description} ${t.enabled ? '' : '(disabled)'}`);
            }
            addAgentMessage(lines.join('\n'));
          }
        } catch {
          addAgentMessage('无法获取工具列表');
        }
        return true;
      }

      case '/skills': {
        try {
          const skills = await invoke<{ name: string; category: string; description: string }[]>('claw_chat_get_skills');
          if (skills.length === 0) {
            addAgentMessage('No skills installed.');
          } else {
            const lines = ['**Installed Skills**'];
            for (const s of skills) {
              lines.push(`- **${s.name}** ${s.description} (${s.category})`);
            }
            addAgentMessage(lines.join('\n'));
          }
        } catch {
          addAgentMessage('无法获取技能列表');
        }
        return true;
      }

      default:
        return false;
    }
  }

  return { executeLocal, isLocal: isLocallyHandled };
}
