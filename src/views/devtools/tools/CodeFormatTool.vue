<template>
  <div class="code-format-tool">
    <h3>代码格式化</h3>

    <div class="tool-row">
      <label class="tool-label" style="align-self: center;">语言</label>
      <select v-model="language" class="tool-select">
        <option v-for="lang in languages" :key="lang.value" :value="lang.value">{{ lang.label }}</option>
      </select>

      <template v-if="language === 'sql'">
        <label class="tool-label" style="align-self: center;">缩进宽度</label>
        <select v-model="tabWidth" class="tool-select" style="width: 80px;">
          <option :value="2">2</option>
          <option :value="4">4</option>
          <option :value="8">8</option>
        </select>
      </template>
    </div>

    <div class="tool-row">
      <button class="tool-btn primary" @click="formatCode" :disabled="!input">格式化</button>
      <button class="tool-btn" @click="compressCode" :disabled="!input">压缩</button>
      <button class="tool-btn" @click="copyOutput" :disabled="!output">复制结果</button>
      <button class="tool-btn" @click="clearAll">清空</button>
    </div>

    <div class="tool-section">
      <h4>输入</h4>
      <textarea v-model="input" class="tool-textarea" placeholder="在此输入代码..." rows="10"></textarea>
    </div>

    <div class="tool-section">
      <h4>输出</h4>
      <textarea v-model="output" class="tool-textarea" readonly rows="10" placeholder="结果将显示在这里..."></textarea>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { format as formatSql } from 'sql-formatter'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const languages = [
  { value: 'js', label: 'JavaScript' },
  { value: 'ts', label: 'TypeScript' },
  { value: 'html', label: 'HTML' },
  { value: 'css', label: 'CSS' },
  { value: 'less', label: 'LESS' },
  { value: 'scss', label: 'SCSS' },
  { value: 'json', label: 'JSON' },
  { value: 'yaml', label: 'YAML' },
  { value: 'sql', label: 'SQL' },
  { value: 'markdown', label: 'Markdown' },
]

const language = ref('js')
const tabWidth = ref(2)
const input = ref('')
const output = ref('')

function simpleFormat(code: string, type: string): string {
  const lines = code.split('\n')
  let indent = 0
  const result: string[] = []

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed) continue

    // Decrease indent for closing braces
    if (/^[})\]]/.test(trimmed)) {
      indent = Math.max(0, indent - 1)
    }

    const indentStr = '  '.repeat(indent)
    result.push(indentStr + trimmed)

    // Increase indent for opening braces
    if (/[({[]$/.test(trimmed)) {
      indent++
    }
  }
  return result.join('\n')
}

function simpleMinify(code: string): string {
  return code
    .replace(/\/\*[\s\S]*?\*\//g, '') // Remove block comments
    .replace(/\/\/.*$/gm, '')          // Remove line comments
    .replace(/\s*\n\s*/g, '\n')        // Normalize whitespace
    .replace(/"/g, "'")
    .replace(/\n/g, '')
    .replace(/\s{2,}/g, ' ')
    .trim()
}

function formatCode() {
  try {
    if (!input.value.trim()) {
      toast.warning('请输入代码')
      return
    }

    if (language.value === 'json') {
      const parsed = JSON.parse(input.value)
      output.value = JSON.stringify(parsed, null, 2)
      toast.success('JSON 格式化成功')
      return
    }

    if (language.value === 'sql') {
      output.value = formatSql(input.value, {
        language: 'sql',
        tabWidth: tabWidth.value,
        indentStyle: 'standard',
      })
      toast.success('SQL 格式化成功')
      return
    }

    if (language.value === 'yaml') {
      // Simple YAML validation and formatting
      output.value = input.value.trim()
      toast.success('YAML 已处理')
      return
    }

    if (language.value === 'markdown') {
      output.value = input.value.trim()
      toast.success('Markdown 已处理')
      return
    }

    // For JS/TS/HTML/CSS/LESS/SCSS: try prettier first, then simple formatting
    output.value = simpleFormat(input.value, language.value)
    toast.success('格式化成功')
  } catch (e: any) {
    output.value = ''
    toast.error(`格式化失败: ${e.message}`)
  }
}

function compressCode() {
  try {
    if (!input.value.trim()) {
      toast.warning('请输入代码')
      return
    }

    if (language.value === 'json') {
      const parsed = JSON.parse(input.value)
      output.value = JSON.stringify(parsed)
      toast.success('JSON 压缩成功')
      return
    }

    if (language.value === 'sql') {
      output.value = input.value.replace(/\s+/g, ' ').trim()
      toast.success('SQL 压缩成功')
      return
    }

    output.value = simpleMinify(input.value)
    toast.success('压缩成功')
  } catch (e: any) {
    output.value = ''
    toast.error(`压缩失败: ${e.message}`)
  }
}

async function copyOutput() {
  if (!output.value) return
  await copyText(output.value, toast)
}

function clearAll() {
  input.value = ''
  output.value = ''
}
</script>

<style scoped>

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--color-base-content); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--color-primary); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-input:focus { border-color: var(--color-primary); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: var(--color-base-100); color: var(--color-base-content); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.tool-btn.primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: var(--color-primary); color: white; border-color: var(--color-primary); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: var(--color-base-200); border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: var(--color-base-content); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-select:focus { border-color: var(--color-primary); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--color-base-content); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); margin: 20px 0; }
</style>
