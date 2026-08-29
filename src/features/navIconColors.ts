/**
 * 侧栏/标签页图标的配色表（viewId → 主色）
 *
 * 目的：折叠侧栏后只剩图标，靠颜色区分功能比纯线条图标快得多。
 * 约定：
 * - 每个 viewId 一个固定色，跨主题不变（浅色 cupcake / 深色 sunset 都能看清的中亮度色值）
 * - 同一分组内避免相邻同色
 * - 新增功能页时在此登记，否则回退到 currentColor（继承文本色，不会坏样式）
 */
export const NAV_ICON_COLORS: Record<string, string> = {
  // 业务
  'dashboard': '#3b82f6', // 蓝
  'todo': '#10b981', // 翠绿
  'projects': '#f59e0b', // 琥珀
  'weekly-report': '#7c3aed',
  'accounting': '#16a34a',
  'kanban': '#65a30d',
  // 运维
  'servers': '#06b6d4', // 青
  'cicd': '#8b5cf6', // 紫罗兰
  'log-aggregator': '#64748b', // 石板灰（日志=文本）
  'nginx': '#14b8a6', // 蓝绿
  'assistant': '#d946ef', // 品红（AI 更显眼）
  'alert': '#e11d48',
  'data-backup': '#0284c7',
  'disk-cleaner': '#dc2626',
  'report': '#0d9488',
  // 开发
  'database': '#0ea5e9', // 天蓝
  'devtools': '#f43f5e', // 玫红
  'notes': '#84cc16', // 黄绿
  'git': '#fb923c', // Git 橙
  'image-processor': '#db2777',
  'tools': '#eab308',
  'cron': '#0891b2',
  'providers': '#ea580c',
  'models': '#c026d3',
  'skills': '#a855f7',
  'memory': '#9333ea',
  // 安全
  'mfa': '#ef4444', // 红
  'vpn': '#a855f7', // 紫
  // 底部操作区
  'more': '#6366f1', // 更多功能
  'lan': '#10b981', // 局域网
  'settings': '#94a3b8',
}

/** 图标主色；未登记的 viewId 返回 undefined（保持继承 currentColor） */
export function navIconColor(viewId: string): string | undefined {
  return NAV_ICON_COLORS[viewId]
}

/** 图标着色样式 */
export function navIconStyle(viewId: string): Record<string, string> | undefined {
  const color = navIconColor(viewId)
  return color ? { color } : undefined
}

/** 图标底色块：8 位 hex 追加透明度，未选中 12%、选中 26% */
export function navChipStyle(viewId: string, active: boolean): Record<string, string> | undefined {
  const color = navIconColor(viewId)
  if (!color) {return undefined}
  return { backgroundColor: color + (active ? '44' : '1f') }
}
