/**
 * useMenuUsage — 追踪侧边栏菜单点击次数，支持时间衰减
 * 数据存储在 localStorage，按 viewMode 统计
 */

import { ref, computed } from 'vue'

interface UsageEntry {
  count: number
  lastClick: number // timestamp
}

const STORAGE_KEY = 'sidebar_menu_usage'
const DECAY_HALF_LIFE = 7 * 24 * 60 * 60 * 1000 // 7天半衰期
const TOP_N = 5

const rawUsage = ref<Record<string, UsageEntry>>({})

function load(): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) rawUsage.value = JSON.parse(raw)
  } catch {}
}

function save(): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(rawUsage.value))
  } catch {}
}

// 衰减：距离上次点击越久，有效分数越低
function decayedScore(entry: UsageEntry): number {
  const now = Date.now()
  const age = now - entry.lastClick
  // 指数衰减: score = count * 2^(-age / half_life)
  return entry.count * Math.pow(0.5, age / DECAY_HALF_LIFE)
}

export function useMenuUsage() {
  // 初始化时加载
  load()

  const recordClick = (viewMode: string): void => {
    if (!viewMode || viewMode === 'todo') return // 不统计默认首页
    const entry = rawUsage.value[viewMode]
    if (entry) {
      entry.count++
      entry.lastClick = Date.now()
    } else {
      rawUsage.value[viewMode] = { count: 1, lastClick: Date.now() }
    }
    save()
  }

  /** Top N 常用功能（带时间衰减排序） */
  const topMenus = computed(() => {
    const entries = Object.entries(rawUsage.value)
      .map(([viewMode, entry]) => ({
        viewMode,
        score: decayedScore(entry),
        count: entry.count,
        lastClick: entry.lastClick,
      }))
      .filter(e => e.score > 0.1) // 过滤几乎衰减到0的
      .sort((a, b) => b.score - a.score)
      .slice(0, TOP_N)
      .map(e => e.viewMode)

    return entries
  })

  /** 清除统计 */
  const clearUsage = (): void => {
    rawUsage.value = {}
    save()
  }

  return { recordClick, topMenus, clearUsage }
}
