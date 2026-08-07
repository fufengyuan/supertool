<template>
  <ToolPage
    icon="code"
    name="代码格式化"
    description="JS / TS / HTML / CSS / JSON / YAML / SQL 等 10 种语言格式化与压缩"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl px-4 py-3 flex flex-wrap items-center gap-3">
      <div class="flex items-center gap-2">
        <span class="text-xs text-base-content/60">语言</span>
        <select v-model="language" class="select select-bordered select-sm bg-base-200/60">
          <option v-for="lang in languages" :key="lang.value" :value="lang.value">{{ lang.label }}</option>
        </select>
      </div>
      <template v-if="language === 'sql'">
        <div class="flex items-center gap-2">
          <span class="text-xs text-base-content/60">缩进</span>
          <select v-model="tabWidth" class="select select-bordered select-sm bg-base-200/60" style="width: 76px;">
            <option :value="2">2</option>
            <option :value="4">4</option>
            <option :value="8">8</option>
          </select>
        </div>
      </template>
      <div class="ml-auto flex gap-2">
        <button class="btn btn-primary btn-sm" @click="formatCode" :disabled="!input">格式化</button>
        <button class="btn btn-outline btn-sm" @click="compressCode" :disabled="!input">压缩</button>
        <button class="btn btn-ghost btn-sm" @click="clearAll" :disabled="!input && !output">清空</button>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[260px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入</h4>
        </div>
        <textarea v-model="input" class="textarea textarea-bordered w-full text-xs bg-base-200/60 font-mono flex-1 resize-none min-h-[140px]" placeholder="在此输入代码..."></textarea>
      </div>
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[260px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 输出</h4>
          <button class="btn btn-primary btn-xs" @click="copyOutput" :disabled="!output"><SvgIcon name="copy" size="11" /> 复制</button>
        </div>
        <textarea v-model="output" readonly class="textarea textarea-bordered w-full text-xs bg-base-200/60 font-mono flex-1 resize-none min-h-[140px]" placeholder="结果将显示在这里..."></textarea>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import { format as formatSql } from 'sql-formatter'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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

function formatYaml(code: string): string {
  const lines = code.split('\n')
  const result: string[] = []
  for (const line of lines) {
    // 去除行尾空格，保留行首缩进
    const trimmedEnd = line.replace(/\s+$/, '')
    if (trimmedEnd.trim() === '') {
      result.push('')
      continue
    }
    // 规范化 key: value 间距（冒号后单空格）
    const match = trimmedEnd.match(/^(\s*-?\s*)([^:#]+?):\s*(.*)$/)
    if (match) {
      const [, indent, key, val] = match
      result.push(`${indent}${key}: ${val.trim()}`)
    } else {
      result.push(trimmedEnd)
    }
  }
  return result.join('\n').replace(/\n{3,}/g, '\n\n').trim()
}

function formatMarkdown(code: string): string {
  return code
    // 标题前后各空一行
    .replace(/\n*(^#{1,6}\s.*)\n*/gm, '\n\n$1\n')
    // 标题 # 后加空格
    .replace(/^(#{1,6})([^\s#])/gm, '$1 $2')
    // 列表项前后空行归一化（连续列表项不空行）
    .replace(/\n{2,}(\s*[-*+]\s)/g, '\n$1')
    // 代码块前后空一行
    .replace(/\n*(^```.*$)\n*/gm, '\n\n$1\n')
    // 多余空行压缩为最多2个换行
    .replace(/\n{3,}/g, '\n\n')
    .trim() + '\n'
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
      output.value = formatYaml(input.value)
      toast.success('YAML 格式化成功')
      return
    }

    if (language.value === 'markdown') {
      output.value = formatMarkdown(input.value)
      toast.success('Markdown 格式化成功')
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
