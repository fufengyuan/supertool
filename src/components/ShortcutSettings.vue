<template>
  <div class="shortcut-settings">
    <h2>⌨️ 快捷键设置</h2>
    <p class="desc">自定义功能快捷键，点击输入框后按下新快捷键即可修改。</p>
    <div class="shortcut-list">
      <div v-for="item in shortcutItems" :key="item.key" class="shortcut-row" :class="{ editing: editingKey === item.key }">
        <div class="shortcut-info">
          <span class="shortcut-label">{{ item.label }}</span>
          <span class="shortcut-desc">{{ item.description }}</span>
        </div>
        <div class="shortcut-actions">
          <div class="shortcut-key-display" :class="{ 'is-editing': editingKey === item.key }" @click="startEdit(item.key)">
            <template v-if="editingKey === item.key">
              <span class="recording">按下新快捷键...</span>
              <span v-if="tempKeys" class="temp-keys">{{ tempKeys }}</span>
            </template>
            <template v-else>
              <span v-if="item.shortcut" class="keys">{{ item.shortcut }}</span>
              <span v-else class="none">未设置</span>
            </template>
          </div>
          <button v-if="item.shortcut && item.defaultShortcut && item.shortcut !== item.defaultShortcut" @click="resetShortcut(item.key)" class="btn-reset" title="恢复默认">↩️</button>
        </div>
      </div>
    </div>
    <div class="footer-actions">
      <button @click="resetAll" class="btn-reset-all">恢复所有默认</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useToast } from '../composables/useToast'
import { getTauriAPI } from '../utils/tauri-api'

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

<style scoped>
.shortcut-settings { padding: 20px; max-width: 800px; margin: 0 auto; color: var(--text-primary); }
.shortcut-settings h2 { margin: 0 0 8px 0; font-size: 20px; }
.desc { color: var(--text-secondary); font-size: 13px; margin: 0 0 20px 0; }
.shortcut-list { display: flex; flex-direction: column; gap: 2px; }
.shortcut-row { display: flex; align-items: center; justify-content: space-between; padding: 12px 16px; background: var(--card-bg); border-radius: 8px; transition: background 0.2s; }
.shortcut-row:hover { background: var(--input-bg); }
.shortcut-row.editing { background: var(--primary-light); border: 1px solid var(--primary-color); }
.shortcut-info { display: flex; flex-direction: column; gap: 2px; }
.shortcut-label { font-weight: 600; font-size: 14px; }
.shortcut-desc { font-size: 12px; color: var(--text-secondary); }
.shortcut-actions { display: flex; align-items: center; gap: 8px; }
.shortcut-key-display { min-width: 160px; padding: 8px 12px; background: var(--input-bg); border: 1px solid var(--border-color); border-radius: 6px; font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 13px; cursor: pointer; text-align: center; transition: all 0.2s; display: flex; align-items: center; justify-content: center; gap: 8px; }
.shortcut-key-display:hover { border-color: var(--primary-color); }
.shortcut-key-display.is-editing { border-color: var(--primary-color); background: var(--card-bg); animation: pulse-border 1s infinite; }
@keyframes pulse-border { 0%, 100% { box-shadow: 0 0 0 0 rgba(var(--primary-rgb, 74, 222, 128), 0.4); } 50% { box-shadow: 0 0 0 4px rgba(var(--primary-rgb, 74, 222, 128), 0); } }
.keys { color: var(--primary-color); font-weight: 600; }
.none { color: var(--text-secondary); font-style: italic; }
.recording { color: var(--text-secondary); font-size: 12px; }
.temp-keys { color: var(--primary-color); font-weight: 600; }
.btn-reset { background: none; border: none; font-size: 16px; cursor: pointer; opacity: 0.5; padding: 4px; transition: opacity 0.2s; }
.btn-reset:hover { opacity: 1; }
.footer-actions { margin-top: 24px; display: flex; justify-content: flex-end; }
.btn-reset-all { padding: 8px 16px; background: var(--input-bg); border: 1px solid var(--border-color); border-radius: 6px; cursor: pointer; color: var(--text-primary); font-size: 13px; transition: all 0.2s; }
.btn-reset-all:hover { background: var(--card-bg); border-color: var(--primary-color); }
</style>
