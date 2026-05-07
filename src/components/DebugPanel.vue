<template>
  <Teleport to="body">
    <transition name="debug-panel">
      <div v-if="visible" class="debug-panel-overlay" @click.self="close">
        <div class="debug-panel">
          <div class="debug-header">
            <h3 class="debug-title">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="4 17 10 11 4 5"/>
                <line x1="12" y1="19" x2="20" y2="19"/>
              </svg>
              {{ $t('debug.title') }}
            </h3>
            <button class="debug-close" @click="close">×</button>
          </div>

          <div class="debug-tabs">
            <button
              v-for="tab in tabs"
              :key="tab.key"
              class="debug-tab"
              :class="{ active: activeTab === tab.key }"
              @click="activeTab = tab.key"
            >
              {{ $t(tab.label) }}
              <span v-if="tab.badge" class="tab-badge">{{ tab.badge }}</span>
            </button>
          </div>

          <div class="debug-content">
            <!-- 日志 Tab -->
            <div v-if="activeTab === 'logs'" class="debug-logs">
              <div class="log-filters">
                <button
                  v-for="level in levelOptions"
                  :key="level.value"
                  class="log-filter-btn"
                  :class="{ active: logFilter === level.value }"
                  @click="logFilter = level.value"
                >
                  {{ $t(level.label) }}
                </button>
                <button class="log-clear-btn" @click="clearLogs">{{ $t('debug.clear') }}</button>
              </div>
              <div class="log-list">
                <div v-if="filteredLogs.length === 0" class="log-empty">{{ $t('debug.noLogs') }}</div>
                <div
                  v-for="(entry, i) in filteredLogs"
                  :key="i"
                  class="log-entry"
                  :class="`log-${entry.levelLabel.toLowerCase()}`"
                >
                  <span class="log-time">{{ entry.timestamp.split('T')[1]?.split('.')[0] }}</span>
                  <span class="log-level">{{ entry.levelLabel }}</span>
                  <span v-if="entry.context" class="log-context">[{{ entry.context }}]</span>
                  <span class="log-message">{{ entry.message }}</span>
                </div>
              </div>
            </div>

            <!-- 性能 Tab -->
            <div v-if="activeTab === 'performance'" class="debug-performance">
              <div class="perf-grid">
                <div class="perf-card">
                  <span class="perf-label">{{ $t('debug.componentCount') }}</span>
                  <span class="perf-value">{{ perfStats.componentCount }}</span>
                </div>
                <div class="perf-card">
                  <span class="perf-label">{{ $t('debug.memoryLogs') }}</span>
                  <span class="perf-value">{{ perfStats.logCount }} / 100</span>
                </div>
                <div class="perf-card">
                  <span class="perf-label">{{ $t('debug.errorCount') }}</span>
                  <span class="perf-value perf-error">{{ perfStats.errorCount }}</span>
                </div>
                <div class="perf-card">
                  <span class="perf-label">{{ $t('debug.uptime') }}</span>
                  <span class="perf-value">{{ perfStats.uptime }}</span>
                </div>
              </div>

              <div class="perf-section">
                <h4 class="perf-section-title">{{ $t('debug.envVars') }}</h4>
                <div class="perf-env">
                  <div class="perf-env-row">
                    <span class="perf-env-key">NODE_ENV</span>
                    <span class="perf-env-val">{{ envInfo.nodeEnv }}</span>
                  </div>
                  <div class="perf-env-row">
                    <span class="perf-env-key">Vite Mode</span>
                    <span class="perf-env-val">{{ envInfo.mode }}</span>
                  </div>
                  <div class="perf-env-row">
                    <span class="perf-env-key">Tauri API</span>
                    <span class="perf-env-val">{{ envInfo.hasTauriApi ? $t('debug.available') : $t('debug.unavailable') }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<script setup lang="ts">// @ts-nocheck
console.log("[components/DebugPanel.vue] component loaded")
import { getTauriAPI } from '../utils/tauri-api'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { getLogs, clearLogs, LogLevel, getLogCount } from '../services/logger'

const props = defineProps({
  // 只在开发环境启用
  enabled: {
    type: Boolean,
    default: true,
  },
})

const visible = ref(false)
const activeTab = ref('logs')
const logFilter = ref<number>(LogLevel.DEBUG)

const tabs = [
  { key: 'logs', label: 'debug.logs', badge: null },
  { key: 'performance', label: 'debug.performance', badge: null },
]

const levelOptions = [
  { label: 'debug.all', value: LogLevel.DEBUG },
  { label: 'INFO', value: LogLevel.INFO },
  { label: 'WARN', value: LogLevel.WARN },
  { label: 'ERROR', value: LogLevel.ERROR },
]

const filteredLogs = computed(() => getLogs(logFilter.value as any))

const perfStats = computed(() => {
  const allLogs = getLogs()
  const errorCount = allLogs.filter((l) => l.level === LogLevel.ERROR).length

  return {
    componentCount: document?.querySelectorAll('*')?.length ?? 0,
    logCount: getLogCount(),
    errorCount,
    uptime: getUptime(),
  }
})

const envInfo = computed(() => ({
  nodeEnv: import.meta.env?.MODE ?? 'unknown',
  mode: import.meta.env?.DEV ? 'development' : 'production',
  hasTauriApi: typeof window !== 'undefined' && !!getTauriAPI(),
}))

const startTime = Date.now()
function getUptime() {
  const diff = Date.now() - startTime
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  if (hours > 0) return `${hours}h ${minutes % 60}m`
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`
  return `${seconds}s`
}

function open() {
  if (props.enabled) {
    visible.value = true
  }
}

function close() {
  visible.value = false
}

// 暴露 open/close 供外部调用
defineExpose({ open, close })

// Ctrl+Shift+D 快捷键
function handleKeydown(e) {
  if (e.ctrlKey && e.shiftKey && e.key === 'D') {
    e.preventDefault()
    if (visible.value) {
      close()
    } else {
      open()
    }
  }
}

onMounted(() => {
  if (props.enabled) {
    document.addEventListener('keydown', handleKeydown)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.debug-panel-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 9999;
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
}

.debug-panel {
  width: 600px;
  max-width: 100vw;
  height: 70vh;
  background: #fff;
  color: #1f2937;
  display: flex;
  flex-direction: column;
  box-shadow: -4px 0 24px rgba(0, 0, 0, 0.15);
  font-size: 13px;
}

.debug-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
}

.debug-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  margin: 0;
}

.debug-close {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #6b7280;
}

.debug-close:hover {
  background: #f3f4f6;
  color: #1f2937;
}

.debug-tabs {
  display: flex;
  border-bottom: 1px solid #e5e7eb;
  padding: 0 12px;
}

.debug-tab {
  padding: 10px 16px;
  border: none;
  background: transparent;
  font-size: 13px;
  font-weight: 500;
  color: #6b7280;
  cursor: pointer;
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
}

.debug-tab.active {
  color: #3b82f6;
}

.debug-tab.active::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 16px;
  right: 16px;
  height: 2px;
  background: #3b82f6;
  border-radius: 1px;
}

.tab-badge {
  font-size: 11px;
  background: #e5e7eb;
  color: #6b7280;
  padding: 1px 6px;
  border-radius: 10px;
}

.debug-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 日志 */
.debug-logs {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.log-filters {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  border-bottom: 1px solid #e5e7eb;
}

.log-filter-btn {
  padding: 4px 10px;
  border: 1px solid #e5e7eb;
  border-radius: 4px;
  background: #fff;
  font-size: 12px;
  cursor: pointer;
  color: #6b7280;
}

.log-filter-btn.active {
  background: #3b82f6;
  color: #fff;
  border-color: #3b82f6;
}

.log-clear-btn {
  margin-left: auto;
  padding: 4px 10px;
  border: 1px solid #f87171;
  border-radius: 4px;
  background: #fff;
  font-size: 12px;
  cursor: pointer;
  color: #f87171;
}

.log-clear-btn:hover {
  background: #fef2f2;
}

.log-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.log-empty {
  text-align: center;
  color: #9ca3af;
  padding: 40px;
  font-size: 13px;
}

.log-entry {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 4px 16px;
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 12px;
  line-height: 1.5;
}

.log-entry:hover {
  background: #f9fafb;
}

.log-time {
  color: #9ca3af;
  flex-shrink: 0;
  min-width: 65px;
}

.log-level {
  flex-shrink: 0;
  font-weight: 600;
  min-width: 42px;
}

.log-log .log-level { color: #6b7280; }
.log-info .log-level { color: #3b82f6; }
.log-warn .log-level { color: #f59e0b; }
.log-error .log-level { color: #ef4444; }

.log-context {
  color: #8b5cf6;
  flex-shrink: 0;
}

.log-message {
  color: #374151;
  word-break: break-all;
}

/* 性能 */
.debug-performance {
  padding: 16px;
  overflow-y: auto;
}

.perf-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 20px;
}

.perf-card {
  background: #f9fafb;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.perf-label {
  font-size: 12px;
  color: #6b7280;
}

.perf-value {
  font-size: 24px;
  font-weight: 700;
  color: #1f2937;
}

.perf-error {
  color: #ef4444;
}

.perf-section {
  margin-top: 8px;
}

.perf-section-title {
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 12px;
  color: #1f2937;
}

.perf-env {
  background: #f9fafb;
  border-radius: 8px;
  overflow: hidden;
}

.perf-env-row {
  display: flex;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid #e5e7eb;
}

.perf-env-row:last-child {
  border-bottom: none;
}

.perf-env-key {
  color: #6b7280;
  font-weight: 500;
}

.perf-env-val {
  color: #1f2937;
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 12px;
}

/* 过渡动画 */
.debug-panel-enter-active,
.debug-panel-leave-active {
  transition: opacity 0.2s ease;
}

.debug-panel-enter-from,
.debug-panel-leave-to {
  opacity: 0;
}
</style>
