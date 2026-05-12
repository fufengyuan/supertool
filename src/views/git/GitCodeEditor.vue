<template>
  <div class="git-code-editor">
    <!-- 头部 -->
    <div class="editor-header">
      <div class="file-info">
        <span class="file-path">{{ filePath }}</span>
        <span v-if="isModified" class="modified-badge">已修改</span>
      </div>
      <div class="editor-actions">
        <button
          v-if="isModified"
          class="save-btn"
          @click="saveFile"
          :disabled="saving"
        >
          {{ saving ? '保存中...' : '保存' }}
        </button>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>
    </div>

    <!-- 编辑器 -->
    <div class="editor-content">
      <textarea
        v-if="!loading"
        ref="editorRef"
        v-model="content"
        class="code-textarea"
        :class="{ modified: isModified }"
        spellcheck="false"
        @input="handleInput"
      />
      <div v-else class="editor-loading">
        <span>加载文件...</span>
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="editor-statusbar">
      <span class="language">{{ detectedLanguage }}</span>
      <span class="line-count">{{ lineCount }} 行</span>
      <span class="encoding">UTF-8</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { tauriCall } from '@/utils/tauri-api'

const props = defineProps<{
  repoPath: string
  filePath: string
}>()

const emit = defineEmits<{
  'close': []
  'saved': [path: string]
}>()

const content = ref('')
const originalContent = ref('')
const loading = ref(false)
const saving = ref(false)
const editorRef = ref<HTMLTextAreaElement | null>(null)

const isModified = computed(() => content.value !== originalContent.value)

const lineCount = computed(() => {
  return content.value.split('\n').length
})

const detectedLanguage = computed(() => {
  const ext = props.filePath.split('.').pop()?.toLowerCase() || ''
  const langMap: Record<string, string> = {
    'ts': 'TypeScript',
    'tsx': 'TypeScript React',
    'js': 'JavaScript',
    'jsx': 'JavaScript React',
    'vue': 'Vue',
    'html': 'HTML',
    'css': 'CSS',
    'scss': 'SCSS',
    'json': 'JSON',
    'md': 'Markdown',
    'rs': 'Rust',
    'py': 'Python',
    'java': 'Java',
    'go': 'Go',
    'yaml': 'YAML',
    'yml': 'YAML',
    'toml': 'TOML',
    'sh': 'Shell',
    'txt': 'Text',
  }
  return langMap[ext] || 'Plain Text'
})

// 加载文件内容
async function loadFile() {
  if (!props.repoPath || !props.filePath) return
  loading.value = true
  try {
    const fileContent = await tauriCall<string>('read_file_content', { repoPath: props.repoPath, filePath: props.filePath })
    content.value = fileContent
    originalContent.value = fileContent
    // 设置滚动到顶部
    await nextTick()
    if (editorRef.value) {
      editorRef.value.scrollTop = 0
    }
  } catch (err) {
    console.error('加载文件失败:', err)
    content.value = `// 加载文件失败: ${err}`
    originalContent.value = ''
  } finally {
    loading.value = false
  }
}

// 保存文件
async function saveFile() {
  if (!isModified.value || saving.value) return
  saving.value = true
  try {
    await tauriCall<void>('save_file_content', { repoPath: props.repoPath, filePath: props.filePath, content: content.value })
    originalContent.value = content.value
    emit('saved', props.filePath)
  } catch (err) {
    console.error('保存文件失败:', err)
    alert('保存失败: ' + err)
  } finally {
    saving.value = false
  }
}

// 输入处理
function handleInput() {
  // 可以在这里添加自动缩进等处理
}

// 监听 filePath 变化
watch(() => props.filePath, loadFile, { immediate: true })

onMounted(loadFile)
</script>

<style scoped>
.git-code-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-base-100);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-path {
  font-size: 13px;
  font-weight: 500;
}

.modified-badge {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: color-mix(in oklab, var(--color-warning) 20%, transparent);
  color: var(--color-warning);
}

.editor-actions {
  display: flex;
  gap: 8px;
}

.save-btn {
  padding: 4px 12px;
  border-radius: 4px;
  background: var(--color-primary);
  color: white;
  font-size: 12px;
  cursor: pointer;
}

.save-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.close-btn {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  cursor: pointer;
}

.close-btn:hover {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.editor-content {
  flex: 1;
  overflow: hidden;
}

.code-textarea {
  width: 100%;
  height: 100%;
  padding: 12px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  background: var(--color-base-200);
  border: none;
  resize: none;
  outline: none;
  tab-size: 2;
}

.code-textarea.modified {
  border-left: 3px solid var(--color-warning);
}

.editor-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.editor-statusbar {
  display: flex;
  gap: 16px;
  padding: 4px 12px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
}
</style>