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

    <!-- CodeMirror 编辑器 -->
    <div class="editor-content">
      <codemirror
        v-if="!loading"
        v-model="content"
        :style="editorStyle"
        :extensions="extensions"
        :autofocus="true"
        @change="handleChange"
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
import { ref, computed, watch, shallowRef, onMounted } from 'vue'
import { Codemirror } from 'vue-codemirror'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { rust } from '@codemirror/lang-rust'
import { java } from '@codemirror/lang-java'
import { json } from '@codemirror/lang-json'
import { yaml } from '@codemirror/lang-yaml'
import { sql } from '@codemirror/lang-sql'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { markdown } from '@codemirror/lang-markdown'
import { vue } from '@codemirror/lang-vue'
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view'
import { EditorState, Prec } from '@codemirror/state'
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, indentOnInput } from '@codemirror/language'
import { oneDark } from '@codemirror/theme-one-dark'
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

const isModified = computed(() => content.value !== originalContent.value)

const lineCount = computed(() => {
  return content.value.split('\n').length
})

// 文件扩展名到语言映射
const extToLang: Record<string, string> = {
  'ts': 'typescript',
  'tsx': 'typescript',
  'js': 'javascript',
  'jsx': 'javascript',
  'mjs': 'javascript',
  'vue': 'vue',
  'html': 'html',
  'htm': 'html',
  'xml': 'xml',
  'svg': 'xml',
  'css': 'css',
  'scss': 'css',
  'sass': 'css',
  'less': 'css',
  'json': 'json',
  'json5': 'json',
  'md': 'markdown',
  'markdown': 'markdown',
  'rs': 'rust',
  'py': 'python',
  'pyw': 'python',
  'java': 'java',
  'go': 'go',
  'yaml': 'yaml',
  'yml': 'yaml',
  'toml': 'yaml',
  'sh': 'bash',
  'bash': 'bash',
  'zsh': 'bash',
  'sql': 'sql',
  'txt': 'plaintext',
  'cfg': 'plaintext',
  'ini': 'plaintext',
}

// 显示语言名映射
const langDisplayName: Record<string, string> = {
  'typescript': 'TypeScript',
  'javascript': 'JavaScript',
  'vue': 'Vue',
  'html': 'HTML',
  'xml': 'XML',
  'css': 'CSS',
  'json': 'JSON',
  'markdown': 'Markdown',
  'rust': 'Rust',
  'python': 'Python',
  'java': 'Java',
  'yaml': 'YAML',
  'sql': 'SQL',
  'plaintext': 'Plain Text',
}

const detectedLanguage = computed(() => {
  const ext = props.filePath.split('.').pop()?.toLowerCase() || ''
  const lang = extToLang[ext] || 'plaintext'
  return langDisplayName[lang] || lang
})

// 编辑器样式
const editorStyle = {
  height: '100%',
  fontSize: '14px',
  fontFamily: "'JetBrains Mono', 'Fira Code', 'SF Mono', 'Consolas', monospace",
}

// 动态获取语言扩展
function getLanguageExtension(lang: string) {
  switch (lang) {
    case 'typescript':
      return javascript({ typescript: true })
    case 'javascript':
      return javascript()
    case 'vue':
      return vue()
    case 'html':
      return html()
    case 'css':
      return css()
    case 'json':
      return json()
    case 'markdown':
      return markdown()
    case 'rust':
      return rust()
    case 'python':
      return python()
    case 'java':
      return java()
    case 'yaml':
      return yaml()
    case 'sql':
      return sql()
    default:
      return []
  }
}

// CodeMirror 扩展配置
const extensions = computed(() => {
  const ext = props.filePath.split('.').pop()?.toLowerCase() || ''
  const lang = extToLang[ext] || 'plaintext'
  
  const baseExtensions = [
    lineNumbers(),
    highlightActiveLine(),
    highlightActiveLineGutter(),
    history(),
    bracketMatching(),
    indentOnInput(),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
    oneDark, // 使用 oneDark 主题，类似 IDEA 深色主题
    EditorView.lineWrapping, // 支持长行换行
  ]
  
  const langExtension = getLanguageExtension(lang)
  if (langExtension) {
    baseExtensions.push(langExtension)
  }
  
  return baseExtensions
})

// 内容变化处理
function handleChange(value: string) {
  content.value = value
}

// 加载文件内容
async function loadFile() {
  if (!props.repoPath || !props.filePath) return
  loading.value = true
  try {
    const fileContent = await tauriCall<string>('read_file_content', { repoPath: props.repoPath, filePath: props.filePath })
    content.value = fileContent
    originalContent.value = fileContent
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

// 监听 filePath 变化
watch(() => props.filePath, loadFile, { immediate: true })

onMounted(loadFile)
</script>

<style scoped>
/* IDEA 风格代码编辑器 */
.git-code-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
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

.editor-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
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

/* CodeMirror 编辑器容器样式 */
.cm-editor {
  height: 100% !important;
  width: 100% !important;
}

.cm-scroller {
  overflow: auto !important;
}
</style>