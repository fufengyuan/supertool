<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-150 ease"
      leave-active-class="transition-opacity duration-150 ease"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div v-if="isOpen" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000] [-webkit-app-region:no-drag]" @mousedown.self="close">
        <div class="w-[520px] max-h-[420px] bg-base-100 border border-base-content/10 rounded-2xl shadow-[0_20px_60px_rgba(0,0,0,0.3)] overflow-hidden flex flex-col">
          <!-- 功能列表 -->
          <div class="flex-1 overflow-y-auto p-2">
            <div
              v-for="(item, index) in items"
              :key="item.id"
              class="flex items-center gap-3.5 px-3.5 py-2.5 rounded-xl cursor-pointer transition-all duration-100 ease-in-out hover:bg-base-200"
              :class="{ 'bg-primary/10': index === selectedIndex }"
              @click="selectItem(item)"
            >
              <div class="w-9 h-9 flex items-center justify-center rounded-lg bg-primary/20 text-base-content shrink-0 [&_svg]:w-5 [&_svg]:h-5">
                <component :is="item.iconComponent" />
              </div>
              <div class="flex-1 flex flex-col gap-0.5 min-w-0">
                <span class="text-sm font-semibold text-base-content">{{ item.label }}</span>
                <span class="text-[11px] text-base-content/60">{{ item.group }}</span>
              </div>
              <kbd v-if="item.accelerator" class="text-[11px] px-2 py-[3px] rounded-md bg-base-200 border border-base-content/10 text-base-content/60 font-inherit shrink-0">{{ item.accelerator }}</kbd>
            </div>
          </div>

          <!-- 快捷键提示 -->
          <div class="flex gap-4 px-4 py-3 border-t border-base-content/10 text-xs text-base-content/60 bg-base-200">
            <span class="flex items-center gap-1">↑↓ 选择</span>
            <span class="flex items-center gap-1">Enter 确认</span>
            <span class="flex items-center gap-1">ESC 取消</span>
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
  if (!isOpen.value) {return}

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
  window.addEventListener('keydown', handleGlobalKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('keydown', handleGlobalKeydown)
})

defineExpose({ open, close })
</script>
