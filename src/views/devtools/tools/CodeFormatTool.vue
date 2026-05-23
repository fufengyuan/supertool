<template>
  <div class="code-format-tool">
    <h3 class="text-lg font-bold text-base-content mb-5">代码格式化</h3>

    <div class="flex flex-wrap gap-2.5 mb-3">
      <label class="label-text text-xs text-base-content/60 mb-1 block" style="align-self: center;">语言</label>
      <select v-model="language" class="select select-bordered text-xs bg-base-200">
        <option v-for="lang in languages" :key="lang.value" :value="lang.value">{{ lang.label }}</option>
      </select>

      <template v-if="language === 'sql'">
        <label class="label-text text-xs text-base-content/60 mb-1 block" style="align-self: center;">缩进宽度</label>
        <select v-model="tabWidth" class="select select-bordered text-xs bg-base-200" style="width: 80px;">
          <option :value="2">2</option>
          <option :value="4">4</option>
          <option :value="8">8</option>
        </select>
      </template>
    </div>

    <div class="flex flex-wrap gap-2.5 mb-3">
      <button class="btn btn-primary btn-sm" @click="formatCode" :disabled="!input">格式化</button>
      <button class="btn btn-ghost btn-sm" @click="compressCode" :disabled="!input">压缩</button>
      <button class="btn btn-ghost btn-sm" @click="copyOutput" :disabled="!output">复制结果</button>
      <button class="btn btn-ghost btn-sm" @click="clearAll">清空</button>
    </div>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content flex items-center gap-1.5 mb-2.5">输入</h4>
      <textarea v-model="input" class="textarea textarea-bordered w-full text-xs bg-base-200 font-mono min-h-[120px]" placeholder="在此输入代码..." rows="10"></textarea>
    </div>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content flex items-center gap-1.5 mb-2.5">输出</h4>
      <textarea v-model="output" class="textarea textarea-bordered w-full text-xs bg-base-200 font-mono min-h-[120px]" readonly rows="10" placeholder="结果将显示在这里..."></textarea>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
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
    if (!trimmed) {continue}

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
  if (!output.value) {return}
  await copyText(output.value, toast)
}

function clearAll() {
  input.value = ''
  output.value = ''
}
</script>
