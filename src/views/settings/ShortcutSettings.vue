<template>
  <div class="p-6 max-w-2xl mx-auto text-base-content">
    <h2 class="m-0 mb-2 text-xl font-bold">⌨️ 快捷键设置</h2>
    <p class="text-sm text-base-content/60 mb-5">自定义功能快捷键，点击输入框后按下新快捷键即可修改。</p>
    <div class="flex flex-col gap-0.5">
      <div v-for="item in shortcutItems" :key="item.key" class="flex items-center justify-between p-3 px-4 bg-base-100 rounded-lg transition-colors duration-200 hover:bg-base-200" :class="{ 'bg-primary/10 border border-primary': editingKey === item.key }">
        <div class="flex flex-col gap-0.5">
          <span class="font-semibold text-sm">{{ item.label }}</span>
          <span class="text-xs text-base-content/60">{{ item.description }}</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="min-w-[160px] px-3 py-2 bg-base-200 border border-base-content/10 rounded-md font-mono text-sm cursor-pointer text-center transition-all duration-200 flex items-center justify-center gap-2 hover:border-primary" :class="{ 'border-primary bg-base-100': editingKey === item.key }" @click="startEdit(item.key)">
            <template v-if="editingKey === item.key">
              <span class="text-base-content/60 text-xs">按下新快捷键...</span>
              <span v-if="tempKeys" class="text-primary font-semibold">{{ tempKeys }}</span>
            </template>
            <template v-else>
              <span v-if="item.shortcut" class="text-primary font-semibold">{{ item.shortcut }}</span>
              <span v-else class="text-base-content/60 italic">未设置</span>
            </template>
          </div>
          <button v-if="item.shortcut && item.defaultShortcut && item.shortcut !== item.defaultShortcut" @click="resetShortcut(item.key)" class="bg-transparent border-none text-lg cursor-pointer opacity-50 p-1 transition-opacity duration-200 hover:opacity-100" title="恢复默认">↩️</button>
        </div>
      </div>
    </div>
    <div class="mt-6 flex justify-end">
      <button @click="resetAll" class="px-4 py-2 bg-base-200 border border-base-content/10 rounded-md cursor-pointer text-sm text-base-content transition-all duration-200 hover:bg-base-100 hover:border-primary">恢复所有默认</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useToast } from '../../composables/useToast'
import { getTauriAPI } from '../../utils/tauri-api'

const toast = useToast()
const tauri = getTauriAPI()

interface ShortcutItem {
  key: string
  label: string
  description: string
  defaultShortcut: string
  shortcut: string
  category: string
}

const shortcutItems = ref<ShortcutItem[]>([])
const editingKey = ref<string | null>(null)
const tempKeys = ref('')
const savedShortcuts = ref<Record<string, string>>({})

const defaultShortcuts: ShortcutItem[] = [
  { key: 'new_task', label: '新建任务', description: '快速创建新任务', defaultShortcut: 'CmdOrCtrl+N', shortcut: 'CmdOrCtrl+N', category: '任务' },
  { key: 'search', label: '搜索任务', description: '在任务列表中搜索', defaultShortcut: 'CmdOrCtrl+F', shortcut: 'CmdOrCtrl+F', category: '任务' },
  { key: 'global_search', label: '全局搜索', description: '搜索所有模块内容', defaultShortcut: 'CmdOrCtrl+K', shortcut: 'CmdOrCtrl+K', category: '搜索' },
  { key: 'quick_switch', label: '快速切换', description: '在功能模块间快速切换', defaultShortcut: 'CmdOrCtrl+Shift+Tab', shortcut: 'CmdOrCtrl+Shift+Tab', category: '导航' },
  { key: 'settings', label: '设置', description: '打开设置页面', defaultShortcut: 'CmdOrCtrl+,', shortcut: 'CmdOrCtrl+,', category: '系统' },
  { key: 'toggle_theme', label: '切换主题', description: '在亮色/暗色主题间切换', defaultShortcut: 'CmdOrCtrl+Shift+D', shortcut: 'CmdOrCtrl+Shift+D', category: '系统' },
  { key: 'toggle_locale', label: '切换语言', description: '在中英文之间切换', defaultShortcut: 'CmdOrCtrl+Shift+L', shortcut: 'CmdOrCtrl+Shift+L', category: '系统' },
  { key: 'nav:todo', label: '任务列表', description: '切换到任务列表', defaultShortcut: 'CmdOrCtrl+1', shortcut: 'CmdOrCtrl+1', category: '导航' },
  { key: 'nav:weekly-report', label: '周报', description: '切换到周报', defaultShortcut: 'CmdOrCtrl+2', shortcut: 'CmdOrCtrl+2', category: '导航' },
  { key: 'nav:projects', label: '项目', description: '切换到项目管理', defaultShortcut: 'CmdOrCtrl+3', shortcut: 'CmdOrCtrl+3', category: '导航' },
  { key: 'nav:servers', label: '服务器', description: '切换到服务器管理', defaultShortcut: 'CmdOrCtrl+4', shortcut: 'CmdOrCtrl+4', category: '导航' },
  { key: 'nav:cicd', label: 'CI/CD', description: '切换到 CI/CD 部署', defaultShortcut: 'CmdOrCtrl+5', shortcut: 'CmdOrCtrl+5', category: '导航' },
  { key: 'nav:database', label: '数据库', description: '切换到数据库管理', defaultShortcut: 'CmdOrCtrl+6', shortcut: 'CmdOrCtrl+6', category: '导航' },
  { key: 'nav:notes', label: '笔记', description: '切换到笔记', defaultShortcut: 'CmdOrCtrl+7', shortcut: 'CmdOrCtrl+7', category: '导航' },
  { key: 'nav:devtools', label: '开发工具', description: '切换到开发工具', defaultShortcut: 'CmdOrCtrl+8', shortcut: 'CmdOrCtrl+8', category: '导航' },
]

function formatKeyCombo(e: KeyboardEvent): string {
  const parts: string[] = []
  if (e.ctrlKey || e.metaKey) parts.push('CmdOrCtrl')
  if (e.shiftKey) parts.push('Shift')
  if (e.altKey) parts.push('Alt')
  const key = e.key
  if (key === 'Control' || key === 'Meta' || key === 'Shift' || key === 'Alt') return ''
  if (key === ' ') return 'Space'
  if (key === 'Escape') return 'Esc'
  if (key === 'Backspace') return 'Backspace'
  if (key === 'Delete') return 'Delete'
  if (key === 'Tab') return 'Tab'
  if (key === 'ArrowUp') return '↑'
  if (key === 'ArrowDown') return '↓'
  if (key === 'ArrowLeft') return '←'
  if (key === 'ArrowRight') return '→'
  if (key.length === 1) parts.push(key.toUpperCase())
  else parts.push(key)
  return parts.join('+')
}

function handleKeyDown(e: KeyboardEvent): void {
  if (editingKey.value === null) return
  e.preventDefault()
  const combo = formatKeyCombo(e)
  if (!combo) return
  tempKeys.value = combo
  if (e.key === 'Enter') {
    saveShortcut(editingKey.value, combo)
    editingKey.value = null
    tempKeys.value = ''
  } else if (e.key === 'Escape') {
    editingKey.value = null
    tempKeys.value = ''
  }
}

function startEdit(key: string): void {
  editingKey.value = key
  tempKeys.value = shortcutItems.value.find(i => i.key === key)?.shortcut || ''
  setTimeout(() => { window.addEventListener('keydown', handleKeyDown) }, 100)
}

async function saveShortcut(key: string, shortcut: string): Promise<void> {
  const conflict = shortcutItems.value.find(i => i.shortcut === shortcut && i.key !== key)
  if (conflict) {
    if (!confirm(`"${conflict.label}" 正在使用此快捷键，是否覆盖？`)) return
    conflict.shortcut = ''
    await tauri.setSetting(`shortcut_${conflict.key}`, '')
  }
  const item = shortcutItems.value.find(i => i.key === key)
  if (item) item.shortcut = shortcut
  savedShortcuts.value[key] = shortcut
  try {
    await tauri.setSetting(`shortcut_${key}`, shortcut)
    await tauri.setSetting('shortcuts', JSON.stringify(savedShortcuts.value))
    toast.success('快捷键已保存')
  } catch (e: any) {
    toast.error('保存失败: ' + e.message)
  }
}

function resetShortcut(key: string): void {
  const item = shortcutItems.value.find(i => i.key === key)
  if (!item) return
  item.shortcut = item.defaultShortcut
  savedShortcuts.value[key] = item.defaultShortcut
  tauri.setSetting(`shortcut_${key}`, item.defaultShortcut)
  toast.success('已恢复默认')
}

async function resetAll(): Promise<void> {
  if (!confirm('确定恢复所有快捷键为默认设置？')) return
  for (const item of shortcutItems.value) {
    item.shortcut = item.defaultShortcut
    savedShortcuts.value[item.key] = item.defaultShortcut
    await tauri.setSetting(`shortcut_${item.key}`, item.defaultShortcut)
  }
  toast.success('已恢复所有默认快捷键')
}

onMounted(async () => {
  shortcutItems.value = [...defaultShortcuts]
  try {
    for (const item of shortcutItems.value) {
      const saved = await tauri.getSetting(`shortcut_${item.key}`)
      if (saved) {
        item.shortcut = saved
        savedShortcuts.value[item.key] = saved
      }
    }
  } catch (e) {
    console.error('Failed to load shortcuts:', e)
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>
