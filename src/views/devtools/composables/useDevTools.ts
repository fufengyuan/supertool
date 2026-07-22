import { ref, computed, watch } from 'vue'
import { DEV_TOOL_REGISTRY, type DevTool } from '../DevToolRegistry'

const FAVORITES_KEY = 'devtools:favorites'
const RECENT_KEY = 'devtools:recent'
const COLLAPSED_KEY = 'devtools:collapsed-categories'
const MAX_RECENT = 10

// 全局共享状态（单例）
const favorites = ref<string[]>(loadFavorites())
const recent = ref<string[]>(loadRecent())
const collapsedCategories = ref<Set<string>>(loadCollapsed())

function loadFavorites(): string[] {
  try {
    const raw = localStorage.getItem(FAVORITES_KEY)
    return raw ? JSON.parse(raw) : []
  } catch {
    return []
  }
}

function loadRecent(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY)
    return raw ? JSON.parse(raw) : []
  } catch {
    return []
  }
}

function loadCollapsed(): Set<string> {
  try {
    const raw = localStorage.getItem(COLLAPSED_KEY)
    return raw ? new Set(JSON.parse(raw)) : new Set()
  } catch {
    return new Set()
  }
}

// 持久化
watch(favorites, (val) => {
  localStorage.setItem(FAVORITES_KEY, JSON.stringify(val))
}, { deep: true })

watch(recent, (val) => {
  localStorage.setItem(RECENT_KEY, JSON.stringify(val))
}, { deep: true })

watch(collapsedCategories, (val) => {
  localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...val]))
}, { deep: true })

export function useDevTools() {
  const favoriteTools = computed<DevTool[]>(() =>
    favorites.value
      .map(id => DEV_TOOL_REGISTRY.find(t => t.id === id))
      .filter((t): t is DevTool => !!t)
  )

  const recentTools = computed<DevTool[]>(() =>
    recent.value
      .map(id => DEV_TOOL_REGISTRY.find(t => t.id === id))
      .filter((t): t is DevTool => !!t)
  )

  function isFavorite(id: string): boolean {
    return favorites.value.includes(id)
  }

  function toggleFavorite(id: string): void {
    if (isFavorite(id)) {
      favorites.value = favorites.value.filter(f => f !== id)
    } else {
      favorites.value = [...favorites.value, id]
    }
  }

  function recordUsage(id: string): void {
    // 去重并放到最前面
    recent.value = [id, ...recent.value.filter(r => r !== id)].slice(0, MAX_RECENT)
  }

  function isCategoryCollapsed(key: string): boolean {
    return collapsedCategories.value.has(key)
  }

  function toggleCategoryCollapsed(key: string): void {
    const next = new Set(collapsedCategories.value)
    if (next.has(key)) {
      next.delete(key)
    } else {
      next.add(key)
    }
    collapsedCategories.value = next
  }

  /**
   * 智能搜索：匹配 name / description / keywords
   * 支持拼音首字母匹配（keywords 字段已包含拼音首字母）
   */
  function searchTools(query: string): DevTool[] {
    if (!query.trim()) return DEV_TOOL_REGISTRY
    const q = query.toLowerCase().trim()
    return DEV_TOOL_REGISTRY.filter(t => {
      const name = t.name.toLowerCase()
      const desc = t.description.toLowerCase()
      const kw = t.keywords.toLowerCase()
      // 模糊匹配：query 的每个字符按顺序出现即可
      if (name.includes(q) || desc.includes(q) || kw.includes(q)) return true
      // 拼音首字母模糊匹配（如 "zz" 匹配 keywords 中的 "zzbb"）
      if (kw.split(' ').some(k => k.startsWith(q))) return true
      return false
    })
  }

  return {
    favorites,
    recent,
    favoriteTools,
    recentTools,
    isFavorite,
    toggleFavorite,
    recordUsage,
    collapsedCategories,
    isCategoryCollapsed,
    toggleCategoryCollapsed,
    searchTools,
  }
}
