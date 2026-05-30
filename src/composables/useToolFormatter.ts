import { ref, computed } from 'vue';

// 工具图标映射
const toolIconMap: Record<string, { icon: string; color: string }> = {
  // 搜索类
  'search_files': { icon: 'search', color: 'text-info' },
  'web_search': { icon: 'search', color: 'text-info' },
  'browser_*': { icon: 'browser', color: 'text-info' },
  
  // 文件操作
  'read_file': { icon: 'file', color: 'text-success' },
  'write_file': { icon: 'fileEdit', color: 'text-warning' },
  'patch': { icon: 'tool', color: 'text-warning' },
  
  // 终端/代码
  'terminal': { icon: 'terminal', color: 'text-error' },
  'execute_code': { icon: 'code', color: 'text-primary' },
  
  // Agent/技能
  'delegate_task': { icon: 'bot', color: 'text-info' },
  'skill_view': { icon: 'skill', color: 'text-secondary' },
  'skill_manage': { icon: 'skill', color: 'text-secondary' },
  'skills_list': { icon: 'list', color: 'text-secondary' },
  
  // 会话/记忆
  'session_search': { icon: 'history', color: 'text-accent' },
  'memory': { icon: 'brain', color: 'text-accent' },
  
  // 浏览器操作
  'browser_navigate': { icon: 'browser', color: 'text-info' },
  'browser_click': { icon: 'mouse', color: 'text-info' },
  'browser_snapshot': { icon: 'camera', color: 'text-info' },
  'browser_vision': { icon: 'eye', color: 'text-info' },
  
  // Cron
  'cronjob': { icon: 'clock', color: 'text-warning' },
  
  // 其他
  'clarify': { icon: 'question', color: 'text-warning' },
  'todo': { icon: 'checklist', color: 'text-success' },
  'image_generate': { icon: 'image', color: 'text-secondary' },
  'text_to_speech': { icon: 'audio', color: 'text-secondary' },
  'vision_analyze': { icon: 'eye', color: 'text-info' },
  'send_message': { icon: 'send', color: 'text-success' },
};

/**
 * 获取工具图标信息
 */
export function getToolIcon(toolName: string): { icon: string; color: string } {
  // 精确匹配
  if (toolIconMap[toolName]) {
    return toolIconMap[toolName];
  }
  
  // 通配符匹配 (browser_*)
  for (const [pattern, info] of Object.entries(toolIconMap)) {
    if (pattern.endsWith('*') && toolName.startsWith(pattern.slice(0, -1))) {
      return info;
    }
  }
  
  // 默认
  return { icon: 'tool', color: 'text-warning' };
}

/**
 * 格式化工具参数摘要（显示关键参数的一行）
 */
export function formatArgsSummary(args: Record<string, unknown>): string {
  if (!args || typeof args !== 'object') {return '';}
  
  // 优先显示的关键参数名
  const priorityKeys = ['path', 'url', 'message', 'query', 'command', 'file', 'text', 'pattern', 'name', 'target'];
  
  for (const key of priorityKeys) {
    if (args[key]) {
      const value = String(args[key]);
      return `${key}: ${value.length > 200 ? value.slice(0, 200) + '...' : value}`;
    }
  }
  
  // 没有优先参数，显示第一个参数
  const firstKey = Object.keys(args)[0];
  if (firstKey) {
    const value = String(args[firstKey]);
    return `${firstKey}: ${value.length > 200 ? value.slice(0, 200) + '...' : value}`;
  }
  
  return '';
}

/**
 * HTML 转义，防止 XSS
 */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

/**
 * 格式化 todo 工具参数摘要 - 显示任务数量和首个任务内容
 */
export function formatTodoArgsSummary(args: Record<string, unknown>): string {
  const todos = args?.todos as Array<{ id?: string; content?: string; status?: string }> | undefined;
  if (!todos || !Array.isArray(todos) || todos.length === 0) {return '待办任务';}

  const total = todos.length;
  const pending = todos.filter(t => t.status === 'pending').length;
  const inProgress = todos.filter(t => t.status === 'in_progress').length;
  const completed = todos.filter(t => t.status === 'completed').length;

  const parts: string[] = [];
  if (pending > 0) {parts.push(`${pending} 待办`);}
  if (inProgress > 0) {parts.push(`${inProgress} 进行中`);}
  if (completed > 0) {parts.push(`${completed} 已完成`);}

  const summary = parts.join(', ');
  const first = todos.find(t => t.content);
  const preview = first ? ` — ${first.content}` : '';

  return `${total} 项任务 (${summary})${preview}`;
}

/**
 * 格式化 todo 工具返回结果 - HTML 格式用于 v-html 渲染
 */
export function formatTodoResult(result: string): string {
  try {
    const parsed = JSON.parse(result);
    if (parsed.todos && Array.isArray(parsed.todos)) {
      const lines = parsed.todos.map((t: { id: string; content: string; status: string }) => {
        const icon = t.status === 'completed' ? '✅' : t.status === 'in_progress' ? '🔄' : '⏳';
        const textClass = t.status === 'completed' ? 'text-base-content/50 line-through' : 
                      t.status === 'in_progress' ? 'text-primary font-medium' : 'text-base-content/70';
        return `<div class="flex items-start gap-1.5 py-0.5">
          <span class="shrink-0">${icon}</span>
          <span class="${textClass}">${escapeHtml(t.content)}</span>
        </div>`;
      });
      return `<div class="space-y-1">${lines.join('<br>')}</div>`;
    }
    return `<div>${escapeHtml(result.replace(/\n/g, '<br>'))}</div>`;
  } catch {
    return `<div>${escapeHtml(result.replace(/\n/g, '<br>'))}</div>`;
  }
}

/**
 * 格式化 delegate_task 工具返回结果 - HTML 格式用于 v-html 渲染
 */
export function formatDelegateResult(result: string): string {
  try {
    const parsed = JSON.parse(result);
    if (parsed.results && Array.isArray(parsed.results)) {
      const lines = parsed.results.map((r: { task_index: number; status: string; summary: string }, i: number) => {
        const statusClass = r.status === 'completed' ? 'text-success' : r.status === 'error' ? 'text-error' : 'text-warning';
        const icon = r.status === 'completed' ? '✓' : r.status === 'error' ? '✕' : '○';
        return `<div class="py-1">
          <div class="flex items-center gap-1.5">
            <span class="${statusClass}">${icon}</span>
            <span class="font-semibold">Task ${i + 1}</span>
            <span class="text-base-content/50 text-xs">(${r.status})</span>
          </div>
          <div class="text-base-content/80 text-xs pl-4">${escapeHtml(r.summary || 'No summary')}</div>
        </div>`;
      });
      return `<div class="space-y-1">${lines.join('')}</div>`;
    }
    return escapeHtml(result);
  } catch {
    return escapeHtml(result);
  }
}

/**
 * 根据工具名选择格式化方法
 */
export function formatToolResult(toolName: string, result: string): string {
  switch (toolName) {
    case 'todo':
      return formatTodoResult(result);
    case 'delegate_task':
      return formatDelegateResult(result);
    default:
      return result;
  }
}

/**
 * 工具调用展开状态管理
 */
export function useToolExpandState() {
  const expandedToolCalls = ref<Set<string>>(new Set());
  const expandedThinking = ref<Set<number>>(new Set());
  
  const toggleToolCallExpand = (key: string) => {
    if (expandedToolCalls.value.has(key)) {
      expandedToolCalls.value.delete(key);
    } else {
      expandedToolCalls.value.add(key);
    }
  };
  
  const isToolCallExpanded = (key: string): boolean => {
    return expandedToolCalls.value.has(key);
  };
  
  const toggleThinkingExpand = (msgIdx: number) => {
    if (expandedThinking.value.has(msgIdx)) {
      expandedThinking.value.delete(msgIdx);
    } else {
      expandedThinking.value.add(msgIdx);
    }
  };
  
  const isThinkingExpanded = (msgIdx: number): boolean => {
    return expandedThinking.value.has(msgIdx);
  };
  
  return {
    expandedToolCalls,
    expandedThinking,
    toggleToolCallExpand,
    isToolCallExpanded,
    toggleThinkingExpand,
    isThinkingExpanded,
  };
}