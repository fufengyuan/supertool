import { ref } from 'vue';
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
  fastMode: Ref<boolean>;
  setFastMode: (next: boolean) => Promise<void>;
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
  onGoalModeChange,
  onLoopModeChange,
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
              lastVerdict: string | null;
              lastReason: string | null;
            }>('claw_chat_get_goal_mode');
            if (res.active) {
              const lines = [`**Goal mode: ON** — Status: **${res.status}**`];
              lines.push(`> ${res.goalText || '(no text)'}`);
              lines.push(`**Turns:** ${res.turnsUsed}/${res.maxTurns}`);
              if (res.lastVerdict) {
                lines.push(`**Last verdict:** ${res.lastVerdict} — ${res.lastReason || ''}`);
              }
              addAgentMessage(lines.join('\n'));
            } else {
              addAgentMessage('**Goal mode: OFF** — Use `/goal <text>` to set a persistent target.');
            }
          } catch {
            addAgentMessage('无法读取 goal 模式状态');
          }
        } else {
          // Set goal
          try {
            await invoke('claw_chat_set_goal_mode', { active: true, goalText: goalArg });
            onGoalModeChange?.(true, goalArg);
            addAgentMessage(`**Goal set**\n> ${goalArg}\n\nAgent will work toward this goal persistently. Use \`/goal\` to check status or the toggle button to reset.`);
          } catch {
            addAgentMessage('设置 goal 失败');
          }
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
