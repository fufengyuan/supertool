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

    <!-- 编辑器 - 语法高亮 overlay -->
    <div class="editor-content">
      <!-- 高亮背景层 -->
      <pre class="code-highlight" aria-hidden="true"><code ref="highlightCode" v-html="highlightedHtml"></code></pre>
      <!-- 编辑 textarea -->
      <textarea
        v-if="!loading"
        ref="editorRef"
        v-model="content"
        class="code-textarea"
        :class="{ modified: isModified }"
        spellcheck="false"
        @scroll="syncScroll"
        @input="updateHighlight"
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
import hljs from 'highlight.js/lib/core'

// 注册常用语言
import typescript from 'highlight.js/lib/languages/typescript'
import javascript from 'highlight.js/lib/languages/javascript'
import python from 'highlight.js/lib/languages/python'
import rust from 'highlight.js/lib/languages/rust'
import java from 'highlight.js/lib/languages/java'
import go from 'highlight.js/lib/languages/go'
import json from 'highlight.js/lib/languages/json'
import yaml from 'highlight.js/lib/languages/yaml'
import bash from 'highlight.js/lib/languages/bash'
import sql from 'highlight.js/lib/languages/sql'
import xml from 'highlight.js/lib/languages/xml'
import css from 'highlight.js/lib/languages/css'
import scss from 'highlight.js/lib/languages/scss'
import markdown from 'highlight.js/lib/languages/markdown'
import vueLang from 'highlight.js/lib/languages/vue'

hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('rust', rust)
hljs.registerLanguage('java', java)
hljs.registerLanguage('go', go)
hljs.registerLanguage('json', json)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('sql', sql)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('html', xml) // HTML 使用 xml 解析器
hljs.registerLanguage('css', css)
hljs.registerLanguage('scss', scss)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('vue', vueLang)

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
const highlightCode = ref<HTMLElement | null>(null)
const highlightedHtml = ref('')

const isModified = computed(() => content.value !== originalContent.value)

const lineCount = computed(() => {
  return content.value.split('\n').length
})

// 文件扩展名到 highlight.js 语言映射
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
  'scss': 'scss',
  'sass': 'scss',
  'less': 'scss',
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
  'toml': 'yaml', // TOML 类似 YAML 语法
  'sh': 'bash',
  'bash': 'bash',
  'zsh': 'bash',
  'sql': 'sql',
  'txt': 'plaintext',
  'cfg': 'plaintext',
  'ini': 'plaintext',
  'env': 'bash',
  'gitignore': 'bash',
  'dockerfile': 'bash',
  'makefile': 'bash',
  'gradle': 'bash',
  'properties': 'bash',
}

// 显示语言名映射
const langDisplayName: Record<string, string> = {
  'typescript': 'TypeScript',
  'javascript': 'JavaScript',
  'vue': 'Vue',
  'html': 'HTML',
  'xml': 'XML',
  'css': 'CSS',
  'scss': 'SCSS',
  'json': 'JSON',
  'markdown': 'Markdown',
  'rust': 'Rust',
  'python': 'Python',
  'java': 'Java',
  'go': 'Go',
  'yaml': 'YAML',
  'bash': 'Shell',
  'sql': 'SQL',
  'plaintext': 'Plain Text',
}

const detectedLanguage = computed(() => {
  const ext = props.filePath.split('.').pop()?.toLowerCase() || ''
  const lang = extToLang[ext] || 'plaintext'
  return langDisplayName[lang] || lang
})

// 获取 highlight.js 语言名
const hljsLanguage = computed(() => {
  const ext = props.filePath.split('.').pop()?.toLowerCase() || ''
  return extToLang[ext] || 'plaintext'
})

// 语法高亮更新
function updateHighlight() {
  if (!content.value) {
    highlightedHtml.value = ''
    return
  }
  try {
    const lang = hljsLanguage.value
    if (lang === 'plaintext') {
      // plaintext 使用 autoDetect
      const result = hljs.highlightAuto(content.value)
      highlightedHtml.value = result.value
    } else {
      const result = hljs.highlight(content.value, { language: lang, ignoreIllegals: true })
      highlightedHtml.value = result.value
    }
  } catch {
    highlightedHtml.value = escapeHtml(content.value)
  }
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

// 同步滚动
function syncScroll() {
  const pre = highlightCode.value?.parentElement
  const ta = editorRef.value
  if (pre && ta) {
    pre.scrollTop = ta.scrollTop
    pre.scrollLeft = ta.scrollLeft
  }
}

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
    updateHighlight()
  } catch (err) {
    console.error('加载文件失败:', err)
    content.value = `// 加载文件失败: ${err}`
    originalContent.value = ''
    updateHighlight()
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
/* IDEA 风格代码编辑器 - 支持主题切换 */
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
  position: relative;
  min-height: 200px;
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

<style>
/* 语法高亮 overlay 层 - 全局样式 */
.code-highlight {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  margin: 0;
  padding: 12px 16px;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Consolas', monospace;
  font-size: 14px;
  line-height: 1.6;
  white-space: pre;
  overflow: auto;
  pointer-events: none;
  color: transparent;
  background: var(--color-base-100);
}

.code-highlight code {
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
}

/* highlight.js token colors - Catppuccin Mocha 主题 */
.code-highlight :deep(.hljs-keyword) { color: #cba6f7; font-weight: 500; }
.code-highlight :deep(.hljs-built_in) { color: #89b4fa; }
.code-highlight :deep(.hljs-type) { color: #89dceb; }
.code-highlight :deep(.hljs-string) { color: #a6e3a1; }
.code-highlight :deep(.hljs-number) { color: #fab387; }
.code-highlight :deep(.hljs-comment) { color: #6c7086; font-style: italic; }
.code-highlight :deep(.hljs-operator) { color: #94e2d5; }
.code-highlight :deep(.hljs-variable) { color: #f38ba8; }
.code-highlight :deep(.hljs-title) { color: #f9e2af; }
.code-highlight :deep(.hljs-title.function_) { color: #89b4fa; }
.code-highlight :deep(.hljs-title.class_) { color: #f9e2af; }
.code-highlight :deep(.hljs-literal) { color: #fab387; }
.code-highlight :deep(.hljs-attr) { color: #f9e2af; }
.code-highlight :deep(.hljs-attribute) { color: #a6e3a1; }
.code-highlight :deep(.hljs-meta) { color: #6c7086; }
.code-highlight :deep(.hljs-tag) { color: #89b4fa; }
.code-highlight :deep(.hljs-name) { color: #f38ba8; }
.code-highlight :deep(.hljs-selector-tag) { color: #f38ba8; }
.code-highlight :deep(.hljs-selector-id) { color: #f9e2af; }
.code-highlight :deep(.hljs-selector-class) { color: #f9e2af; }
.code-highlight :deep(.hljs-selector-attr) { color: #f9e2af; }
.code-highlight :deep(.hljs-regexp) { color: #f5c2e7; }
.code-highlight :deep(.hljs-symbol) { color: #f5c2e7; }
.code-highlight :deep(.hljs-bullet) { color: #f5c2e7; }
.code-highlight :deep(.hljs-link) { color: #89b4fa; }
.code-highlight :deep(.hljs-quote) { color: #a6e3a1; }
.code-highlight :deep(.hljs-addition) { color: #a6e3a1; background: rgba(166, 227, 161, 0.15); }
.code-highlight :deep(.hljs-deletion) { color: #f38ba8; background: rgba(243, 139, 168, 0.15); }
.code-highlight :deep(.hljs-emphasis) { font-style: italic; }
.code-highlight :deep(.hljs-strong) { font-weight: bold; }

/* 编辑 textarea - 全局样式 */
.code-textarea {
  position: relative;
  width: 100%;
  height: 100%;
  padding: 12px 16px;
  border: none;
  background: transparent;
  color: var(--color-base-content);
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Consolas', monospace;
  font-size: 14px;
  line-height: 1.6;
  resize: none;
  outline: none;
  white-space: pre;
  overflow: auto;
  tab-size: 4;
  caret-color: var(--color-base-content);
}

.code-textarea::selection {
  background: color-mix(in oklab, var(--color-primary) 30%, transparent);
}

.code-textarea.modified {
  border-left: 2px solid var(--color-warning);
}
</style>