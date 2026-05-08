<template>
  <Teleport to="body">
    <Transition name="quick-switch">
      <div v-if="isOpen" class="quick-switch-overlay" @mousedown.self="close">
        <div class="quick-switch-container">
          <!-- 功能列表 -->
          <div class="quick-switch-list">
            <div
              v-for="(item, index) in items"
              :key="item.id"
              class="quick-switch-item"
              :class="{ active: index === selectedIndex }"
              @click="selectItem(item)"
            >
              <div class="item-icon">
                <component :is="item.iconComponent" />
              </div>
              <div class="item-info">
                <span class="item-label">{{ item.label }}</span>
                <span class="item-group">{{ item.group }}</span>
              </div>
              <kbd v-if="item.accelerator" class="item-shortcut">{{ item.accelerator }}</kbd>
            </div>
          </div>

          <!-- 快捷键提示 -->
          <div class="quick-switch-hints">
            <span>↑↓ 选择</span>
            <span>Enter 确认</span>
            <span>ESC 取消</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import {
  IconClipboardText,
  IconCalendarEvent,
  IconFolder,
  IconWallet,
  IconServer,
  IconRocket,
  IconFileText,
  IconDatabase,
  IconTools,
  IconBrandGit,
  IconNotes,
  IconLock,
  IconShieldLock,
  IconChartBar,
  IconCloudDownload,
  IconBell,
} from '@tabler/icons-vue'

interface QuickSwitchItem {
  id: string
  label: string
  group: string
  iconComponent: any
  accelerator?: string
}

const isOpen = ref(false)
const selectedIndex = ref(0)

const emit = defineEmits<{
  close: []
  select: [id: string]
}>()

const items = computed<QuickSwitchItem[]>(() => [
  { id: 'todo', label: '任务列表', group: '业务', iconComponent: IconClipboardText, accelerator: '⌘1' },
  { id: 'weekly-report', label: '周报', group: '业务', iconComponent: IconCalendarEvent, accelerator: '⌘2' },
  { id: 'projects', label: '项目', group: '业务', iconComponent: IconFolder, accelerator: '⌘3' },
  { id: 'accounting', label: '记账本', group: '业务', iconComponent: IconWallet },
  { id: 'servers', label: '服务器', group: '运维', iconComponent: IconServer, accelerator: '⌘4' },
  { id: 'cicd', label: 'CI/CD', group: '运维', iconComponent: IconRocket, accelerator: '⌘5' },
  { id: 'log-aggregator', label: '日志聚合', group: '运维', iconComponent: IconFileText },
  { id: 'database', label: '数据库', group: '开发', iconComponent: IconDatabase, accelerator: '⌘6' },
  { id: 'devtools', label: '开发工具', group: '开发', iconComponent: IconTools, accelerator: '⌘8' },
  { id: 'git', label: 'Git 管理', group: '开发', iconComponent: IconBrandGit },
  { id: 'notes', label: '笔记', group: '开发', iconComponent: IconNotes, accelerator: '⌘7' },
  { id: 'mfa', label: 'MFA 验证码', group: '安全', iconComponent: IconLock },
  { id: 'vpn', label: 'VPN', group: '安全', iconComponent: IconShieldLock },
  { id: 'data-backup', label: '数据备份', group: '系统', iconComponent: IconCloudDownload },
  { id: 'notifications', label: '通知设置', group: '系统', iconComponent: IconBell },
])

const open = (): void => {
  isOpen.value = true
  selectedIndex.value = 0
  nextTick(() => {
    document.addEventListener('keydown', handleKeydown)
  })
}

const close = (): void => {
  isOpen.value = false
  document.removeEventListener('keydown', handleKeydown)
}

const handleKeydown = (e: KeyboardEvent): void => {
  if (!isOpen.value) return

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, items.value.length - 1)
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
      break
    case 'Enter':
      e.preventDefault()
      selectItem(items.value[selectedIndex.value])
      break
    case 'Escape':
      e.preventDefault()
      close()
      break
  }
}

const selectItem = (item: QuickSwitchItem): void => {
  close()
  if (item) {
    // 通过 emit 通知父组件切换视图
    emit('select', item.id)
  }
}

// 全局快捷键监听 (Cmd/Ctrl + Shift + K)
const handleGlobalKeydown = (e: KeyboardEvent): void => {
  if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'K') {
    e.preventDefault()
    open()
  }
}

// 监听全局快捷键
onMounted(() => {
    console.log("[components/QuickSwitch.vue] mounted")
  window.addEventListener('keydown', handleGlobalKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('keydown', handleGlobalKeydown)
})

defineExpose({ open, close })
</script>

<style scoped>
/* 动画 */
.quick-switch-enter-active,
.quick-switch-leave-active {
  transition: opacity 0.15s ease;
}
.quick-switch-enter-from,
.quick-switch-leave-to {
  opacity: 0;
}

/* 遮罩层 */
.quick-switch-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  -webkit-app-region: no-drag;
}

/* 容器 */
.quick-switch-container {
  width: 520px;
  max-height: 420px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 列表 */
.quick-switch-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.quick-switch-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.12s ease;
}

.quick-switch-item:hover {
  background: oklch(var(--b2)));
}

.quick-switch-item.active {
  background: oklch(var(--p) / 0.1));
}

.item-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: oklch(var(--p) / 0.2));
  color: oklch(var(--bc));
  flex-shrink: 0;
}

.item-icon svg {
  width: 20px;
  height: 20px;
}

.item-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.item-label {
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
}

.item-group {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
}

.item-shortcut {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 5px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  color: oklch(var(--bc) / 0.6);
  font-family: inherit;
  flex-shrink: 0;
}

/* 底部提示 */
.quick-switch-hints {
  display: flex;
  gap: 16px;
  padding: 12px 16px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
  background: oklch(var(--b2));
}

.quick-switch-hints span {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* 滚动条 */
.quick-switch-list::-webkit-scrollbar {
  width: 6px;
}
.quick-switch-list::-webkit-scrollbar-track {
  background: transparent;
}
.quick-switch-list::-webkit-scrollbar-thumb {
  background: oklch(var(--bc) / 0.1);
  border-radius: 3px;
}
</style>
