<template>
  <Teleport to="body">
    <Transition name="global-search">
      <div v-if="isOpen" class="global-search-overlay" @mousedown.self="close">
        <div class="global-search-container">
          <!-- 搜索输入区 -->
          <div class="search-input-wrapper">
            <svg class="search-icon" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
            </svg>
            <input
              ref="inputRef"
              v-model="query"
              type="text"
              class="search-input"
              placeholder="搜索任务、项目、服务器…"
              @keydown.down.prevent="navigateResults(1)"
              @keydown.up.prevent="navigateResults(-1)"
              @keydown.enter.prevent="selectResult"
              @keydown.esc.prevent="close"
            />
            <kbd class="shortcut-hint">ESC</kbd>
          </div>

          <!-- 搜索结果 -->
          <div v-if="query.trim()" class="search-results">
            <template v-if="results.length > 0">
              <!-- 任务结果 -->
              <template v-if="todoResults.length > 0">
                <div class="result-group-label">任务</div>
                <div
                  v-for="(item, idx) in todoResults"
                  :key="'todo-' + item.id"
                  class="search-result-item"
                  :class="{ active: activeIndex === getGlobalIndex('todo', idx) }"
                  @click="selectItem(item, 'todo')"
                  @mouseenter="activeIndex = getGlobalIndex('todo', idx)"
                >
                  <span class="result-icon todo-icon">✓</span>
                  <div class="result-info">
                    <span class="result-title" v-html="highlightMatch(item.text, query)"></span>
                    <span class="result-subtitle">{{ item.tag || '未分类' }} · {{ formatDate(item.createdAt) }}</span>
                  </div>
                  <span class="result-badge" :class="'priority-' + (item.priority || '')" v-if="item.priority">{{ priorityLabel(item.priority) }}</span>
                </div>
              </template>

              <!-- 项目结果 -->
              <template v-if="projectResults.length > 0">
                <div class="result-group-label">项目</div>
                <div
                  v-for="(item, idx) in projectResults"
                  :key="'project-' + item.id"
                  class="search-result-item"
                  :class="{ active: activeIndex === getGlobalIndex('project', idx) }"
                  @click="selectItem(item, 'project')"
                  @mouseenter="activeIndex = getGlobalIndex('project', idx)"
                >
                  <span class="result-icon project-icon" :style="{ color: item.color || '#6c63ff' }">📁</span>
                  <div class="result-info">
                    <span class="result-title" v-html="highlightMatch(item.name, query)"></span>
                    <span class="result-subtitle">{{ item.description || '暂无描述' }}</span>
                  </div>
                </div>
              </template>

              <!-- 服务器结果 -->
              <template v-if="serverResults.length > 0">
                <div class="result-group-label">服务器</div>
                <div
                  v-for="(item, idx) in serverResults"
                  :key="'server-' + item.id"
                  class="search-result-item"
                  :class="{ active: activeIndex === getGlobalIndex('server', idx) }"
                  @click="selectItem(item, 'server')"
                  @mouseenter="activeIndex = getGlobalIndex('server', idx)"
                >
                  <span class="result-icon server-icon">🖥️</span>
                  <div class="result-info">
                    <span class="result-title" v-html="highlightMatch(item.name, query)"></span>
                    <span class="result-subtitle">{{ item.host }}:{{ item.port }}</span>
                  </div>
                </div>
              </template>
            </template>

            <!-- 无结果 -->
            <div v-else class="no-results">
              <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="11" cy="11" r="8" /><path d="m21 21-4.35-4.35" />
              </svg>
              <p>未找到匹配的结果</p>
              <p class="hint">尝试其他关键词</p>
            </div>
          </div>

          <!-- 默认快捷操作 (无搜索词时) -->
          <div v-else class="quick-actions">
            <div class="result-group-label">快捷导航</div>
            <div
              class="search-result-item"
              :class="{ active: activeIndex === 0 }"
              @click="quickNavigate('todo')"
              @mouseenter="activeIndex = 0"
            >
              <span class="result-icon todo-icon">✓</span>
              <div class="result-info">
                <span class="result-title">全部任务</span>
                <span class="result-subtitle">切换到任务视图</span>
              </div>
              <kbd class="result-kbd">1</kbd>
            </div>
            <div
              class="search-result-item"
              :class="{ active: activeIndex === 1 }"
              @click="quickNavigate('projects')"
              @mouseenter="activeIndex = 1"
            >
              <span class="result-icon project-icon">📁</span>
              <div class="result-info">
                <span class="result-title">项目管理</span>
                <span class="result-subtitle">查看所有项目</span>
              </div>
              <kbd class="result-kbd">2</kbd>
            </div>
            <div
              class="search-result-item"
              :class="{ active: activeIndex === 2 }"
              @click="quickNavigate('servers')"
              @mouseenter="activeIndex = 2"
            >
              <span class="result-icon server-icon">🖥️</span>
              <div class="result-info">
                <span class="result-title">服务器管理</span>
                <span class="result-subtitle">查看服务器列表</span>
              </div>
              <kbd class="result-kbd">3</kbd>
            </div>
          </div>

          <!-- 底部提示 -->
          <div class="search-footer">
            <span class="footer-item"><kbd>↑↓</kbd> 导航</span>
            <span class="footer-item"><kbd>↵</kbd> 打开</span>
            <span class="footer-item"><kbd>esc</kbd> 关闭</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">// @ts-nocheck
import { getTauriAPI } from '../utils/tauri-api'
import { ref, computed, watch, nextTick } from 'vue'
import { useAppStore } from '../stores/appStore'
import { useTodoStore } from '../stores/todoStore'

interface SearchResult {
  id: string
  type: 'todo' | 'project' | 'server'
  [key: string]: any
}

const appStore = useAppStore()
const todoStore = useTodoStore()

const isOpen = ref(false)
const query = ref('')
const activeIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

// 数据源
const todos = computed(() => todoStore.todos)
const projects = ref<any[]>([])
const servers = ref<any[]>([])

// 加载项目和服务器数据
const loadData = async () => {
  try {
    console.log("[loadData] called")
    projects.value = await getTauriAPI().getProjects(true) || []
  } catch {}
  try {
    servers.value = await getTauriAPI().getAllServers() || []
  } catch {}
}

// 模糊匹配
const fuzzyMatch = (text: string, pattern: string): boolean => {
  if (!text || !pattern) return false
  const lowerText = text.toLowerCase()
  const lowerPattern = pattern.toLowerCase()
  return lowerText.includes(lowerPattern)
}

// 搜索结果
const todoResults = computed(() => {
  if (!query.value.trim()) return []
  return todos.value.filter(t =>
    fuzzyMatch(t.text, query.value) ||
    fuzzyMatch(t.tag, query.value) ||
    fuzzyMatch(t.description, query.value)
  ).slice(0, 8)
})

const projectResults = computed(() => {
  if (!query.value.trim()) return []
  return projects.value.filter(p =>
    fuzzyMatch(p.name, query.value) ||
    fuzzyMatch(p.description, query.value)
  ).slice(0, 5)
})

const serverResults = computed(() => {
  if (!query.value.trim()) return []
  return servers.value.filter(s =>
    fuzzyMatch(s.name, query.value) ||
    fuzzyMatch(s.host, query.value)
  ).slice(0, 5)
})

const results = computed(() => [
  ...todoResults.value,
  ...projectResults.value,
  ...serverResults.value
])

// 全局索引计算
const getGlobalIndex = (type: 'todo' | 'project' | 'server', localIdx: number): number => {
  let offset = 0
  if (type === 'project') offset = todoResults.value.length
  if (type === 'server') offset = todoResults.value.length + projectResults.value.length
  return offset + localIdx
}

// 键盘导航
const navigateResults = (direction: number) => {
  const totalResults = results.value.length
  const totalQuickActions = 3
  const total = query.value.trim() ? totalResults : totalQuickActions
  if (total === 0) return
  activeIndex.value = (activeIndex.value + direction + total) % total
}

// 选择结果
const selectResult = () => {
  if (query.value.trim()) {
    if (activeIndex.value < todoResults.value.length) {
      const item = todoResults.value[activeIndex.value]
      selectItem(item, 'todo')
    } else if (activeIndex.value < todoResults.value.length + projectResults.value.length) {
      const item = projectResults.value[activeIndex.value - todoResults.value.length]
      selectItem(item, 'project')
    } else {
      const item = serverResults.value[activeIndex.value - todoResults.value.length - projectResults.value.length]
      selectItem(item, 'server')
    }
  } else {
    quickNavigateByIndex(activeIndex.value)
  }
}

const selectItem = (item: any, type: string) => {
  close()
  if (type === 'todo') {
    appStore.setViewMode('todo')
    setTimeout(() => {
      window.dispatchEvent(new CustomEvent('navigate-to-todo', { detail: { todoId: item.id } }))
    }, 200)
  } else if (type === 'project') {
    appStore.setViewMode('projects')
  } else if (type === 'server') {
    appStore.setViewMode('servers')
  }
}

const quickNavigate = (mode: string) => {
  close()
  appStore.setViewMode(mode as any)
}

const quickNavigateByIndex = (index: number) => {
  const modes = ['todo', 'projects', 'servers']
  if (modes[index]) quickNavigate(modes[index])
}

// 打开/关闭
const open = async () => {
  isOpen.value = true
  query.value = ''
  activeIndex.value = 0
  await loadData()
  await nextTick()
  inputRef.value?.focus()
}

const close = () => {
  isOpen.value = false
  query.value = ''
  activeIndex.value = 0
}

// 高亮匹配文本
const highlightMatch = (text: string, pattern: string): string => {
  if (!text || !pattern) return text || ''
  const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${escaped})`, 'gi')
  return (text || '').replace(regex, '<mark>$1</mark>')
}

// 格式化日期
const formatDate = (dateStr: string): string => {
  if (!dateStr) return ''
  const d = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - d.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  if (days === 0) return '今天'
  if (days === 1) return '昨天'
  if (days < 7) return `${days}天前`
  return d.toLocaleDateString('zh-CN')
}

const priorityLabel = (p: string): string => {
  const map: Record<string, string> = { high: '高', medium: '中', low: '低' }
  return map[p] || ''
}

// 暴露给外部
defineExpose({ open, close })

// 快捷键监听
const handleKeydown = (e: KeyboardEvent) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    if (isOpen.value) close()
    else open()
  }
  if (e.key === 'Escape' && isOpen.value) {
    close()
  }
}

let cleanup: (() => void) | null = null

import { onMounted, onUnmounted } from 'vue'
onMounted(() => {
    console.log("[components/GlobalSearch.vue] mounted")
  window.addEventListener('keydown', handleKeydown)
  cleanup = () => window.removeEventListener('keydown', handleKeydown)
})
onUnmounted(() => {
  cleanup?.()
})
</script>

<style scoped>
.global-search-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
  background: rgba(0, 0, 0, 0.4);
}

.global-search-container {
  width: 560px;
  max-width: 90vw;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.search-icon {
  position: absolute;
  left: 24px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  pointer-events: none;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 16px;
  background: transparent;
  color: var(--color-base-content);
  padding-left: 32px;
}

.search-input::placeholder {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.shortcut-hint {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.search-results,
.quick-actions {
  max-height: 360px;
  overflow-y: auto;
  padding: 8px 0;
}

.result-group-label {
  padding: 6px 20px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  letter-spacing: 0.5px;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  cursor: pointer;
  transition: background 0.1s ease;
}

.search-result-item:hover,
.search-result-item.active {
  background: var(--bg-hover, #f0f0f0);
}

.result-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  flex-shrink: 0;
}

.todo-icon {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.project-icon {
  background: rgba(99, 102, 241, 0.1);
}

.server-icon {
  background: rgba(251, 146, 60, 0.1);
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-title {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-base-content);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-title :deep(mark) {
  background: rgba(99, 102, 241, 0.2);
  color: inherit;
  border-radius: 2px;
  padding: 0 1px;
}

.result-subtitle {
  display: block;
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-top: 2px;
}

.result-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
  flex-shrink: 0;
}

.priority-high {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.priority-medium {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.priority-low {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.result-kbd {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--color-base-200);
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  flex-shrink: 0;
}

.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px 20px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.no-results svg {
  opacity: 0.3;
  margin-bottom: 12px;
}

.no-results p {
  margin: 0;
  font-size: 14px;
}

.no-results .hint {
  font-size: 12px;
  opacity: 0.7;
  margin-top: 4px;
}

.search-footer {
  display: flex;
  gap: 16px;
  padding: 10px 20px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-200);
}

.footer-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.footer-item kbd {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

/* 过渡动画 */
.global-search-enter-active,
.global-search-leave-active {
  transition: opacity 0.15s ease;
}

.global-search-enter-from,
.global-search-leave-to {
  opacity: 0;
}

.global-search-enter-active .global-search-container,
.global-search-leave-active .global-search-container {
  transition: transform 0.15s ease, opacity 0.15s ease;
}

.global-search-enter-from .global-search-container {
  transform: scale(0.95) translateY(-10px);
  opacity: 0;
}

.global-search-leave-to .global-search-container {
  transform: scale(0.98);
  opacity: 0;
}
</style>
