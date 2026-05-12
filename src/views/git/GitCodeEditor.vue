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
/* IDEA 风格代码编辑器 - 支持主题切换 */
.git-code-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-base-100);
  color: var(--color-base-content);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  height: 28px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: var(--color-base-200);
}

.file-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.file-path {
  font-size: 12px;
  color: var(--color-base-content);
}

.modified-badge {
  font-size: 10px;
  padding: 1px 4px;
  border-radius: 2px;
  background: color-mix(in oklab, var(--color-warning) 20%, transparent);
  color: var(--color-warning);
}

.editor-actions {
  display: flex;
  gap: 6px;
}

.save-btn {
  padding: 3px 10px;
  border-radius: 3px;
  background: var(--color-primary);
  color: white;
  font-size: 11px;
  cursor: pointer;
  border: none;
}

.save-btn:hover {
  opacity: 0.9;
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.close-btn {
  width: 20px;
  height: 20px;
  border-radius: 3px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
}

.close-btn:hover {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  color: var(--color-base-content);
}

.editor-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  min-height: 200px;
}

.code-textarea {
  width: 100%;
  height: 100%;
  flex: 1;
  padding: 12px 16px;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Consolas', monospace;
  font-size: 14px;
  line-height: 1.6;
  background: var(--color-base-100);
  border: none;
  resize: none;
  outline: none;
  tab-size: 4;
  color: var(--color-base-content);
}

.code-textarea::selection {
  background: color-mix(in oklab, var(--color-primary) 30%, transparent);
}

.code-textarea.modified {
  border-left: 2px solid var(--color-warning);
}

.editor-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  font-size: 12px;
}

.editor-statusbar {
  display: flex;
  gap: 12px;
  padding: 0 12px;
  height: 22px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: var(--color-base-200);
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  align-items: center;
}

.editor-statusbar span {
  padding: 2px 4px;
}
</style>