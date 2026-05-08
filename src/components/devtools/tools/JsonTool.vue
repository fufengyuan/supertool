<template>
  <div class="json-tool">
    <h3>JSON 工具</h3>

    <div class="tool-section">
      <h4>输入 JSON</h4>
      <textarea v-model="input" class="tool-textarea" placeholder="在此输入 JSON..." rows="6"></textarea>
    </div>

    <div class="tool-row">
      <button class="tool-btn primary" @click="formatJson" :disabled="!input">格式化</button>
      <button class="tool-btn" @click="validateJson" :disabled="!input">校验</button>
      <button class="tool-btn" @click="compressJson" :disabled="!input">压缩</button>
      <button class="tool-btn" @click="escapeJson">转义</button>
      <button class="tool-btn" @click="unescapeJson">去转义</button>
      <button class="tool-btn" @click="copyOutput" :disabled="!output">复制结果</button>
    </div>

    <div class="tool-section" v-if="validateResult">
      <h4>校验结果</h4>
      <div :class="['tool-result', validateResult.ok ? 'result-ok' : 'result-error']">
        {{ validateResult.msg }}
      </div>
    </div>

    <div class="tool-section">
      <h4>输出</h4>
      <textarea v-model="output" class="tool-textarea" readonly rows="6" placeholder="结果将显示在这里..."></textarea>
    </div>

    <hr class="tool-divider" />

    <!-- Unicode 转换 -->
    <div class="tool-section">
      <h4>Unicode 转换</h4>
      <div class="tool-row">
        <button class="tool-btn" @click="unicodeToChinese" :disabled="!input">\uXXXX → 中文</button>
        <button class="tool-btn" @click="chineseToUnicode" :disabled="!input">中文 → \uXXXX</button>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- JSON 转换 -->
    <div class="tool-section">
      <h4>JSON 转换</h4>
      <div class="tool-row">
        <button class="tool-btn" @click="toJsonGetParams" :disabled="!input">转 GET 参数</button>
        <button class="tool-btn" @click="toJsonCsv" :disabled="!input">转 CSV</button>
        <button class="tool-btn" @click="toJsonTable" :disabled="!input">转 HTML 表格</button>
      </div>
    </div>

    <div class="tool-section">
      <h4>转语言类定义</h4>
      <div class="tool-row">
        <button class="tool-btn" @click="toJsonJava" :disabled="!input">Java</button>
        <button class="tool-btn" @click="toJsonCSharp" :disabled="!input">C#</button>
        <button class="tool-btn" @click="toJsonGo" :disabled="!input">Go</button>
        <button class="tool-btn" @click="toJsonDart" :disabled="!input">Dart</button>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- JsonPath -->
    <div class="tool-section">
      <h4>JsonPath 查询</h4>
      <div class="tool-row">
        <input v-model="jsonPath" class="tool-input" style="flex:1;" placeholder="输入 JsonPath 表达式，如 $.store.book[*].author" />
        <button class="tool-btn primary" @click="queryJsonPath" :disabled="!input || !jsonPath">查询</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const input = ref('')
const output = ref('')
const jsonPath = ref('')
const validateResult = ref<{ ok: boolean; msg: string } | null>(null)

function parseInput(): any {
  try {
    return JSON.parse(input.value)
  } catch (e: any) {
    toast.error(`JSON 解析失败: ${e.message}`)
    return null
  }
}

function formatJson() {
  const parsed = parseInput()
  if (parsed !== null) {
    output.value = JSON.stringify(parsed, null, 2)
    toast.success('格式化成功')
  }
}

function validateJson() {
  try {
    JSON.parse(input.value)
    validateResult.value = { ok: true, msg: '✅ JSON 格式正确' }
    toast.success('JSON 格式正确')
  } catch (e: any) {
    const msg = e.message
    // Try to extract line/column from error message
    let detail = msg
    const posMatch = msg.match(/position\s+(\d+)/i)
    if (posMatch) {
      const pos = parseInt(posMatch[1])
      const textBefore = input.value.substring(0, pos)
      const line = (textBefore.match(/\n/g) || []).length + 1
      const col = pos - textBefore.lastIndexOf('\n')
      detail = `${msg} (第 ${line} 行, 第 ${col} 列)`
    }
    validateResult.value = { ok: false, msg: `❌ ${detail}` }
    toast.error(`JSON 校验失败: ${msg}`)
  }
}

function compressJson() {
  const parsed = parseInput()
  if (parsed !== null) {
    output.value = JSON.stringify(parsed)
    toast.success('压缩成功')
  }
}

function escapeJson() {
  if (!input.value.trim()) {
    toast.warning('请输入内容')
    return
  }
  output.value = JSON.stringify(input.value)
  toast.success('转义成功')
}

function unescapeJson() {
  if (!input.value.trim()) {
    toast.warning('请输入内容')
    return
  }
  try {
    output.value = JSON.parse(input.value)
    toast.success('去转义成功')
  } catch (e: any) {
    toast.error(`去转义失败: ${e.message}`)
  }
}

function unicodeToChinese() {
  if (!input.value.trim()) {
    toast.warning('请输入内容')
    return
  }
  try {
    output.value = input.value.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => {
      return String.fromCharCode(parseInt(hex, 16))
    })
    toast.success('转换成功')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
  }
}

function chineseToUnicode() {
  if (!input.value.trim()) {
    toast.warning('请输入内容')
    return
  }
  output.value = input.value.replace(/[^\u0000-\u007f]/g, (c) => {
    return '\\u' + c.charCodeAt(0).toString(16).padStart(4, '0')
  })
  toast.success('转换成功')
}

function flattenToParams(obj: any, prefix = ''): string[] {
  const pairs: string[] = []
  for (const key of Object.keys(obj)) {
    const fullKey = prefix ? `${prefix}[${key}]` : key
    if (obj[key] !== null && typeof obj[key] === 'object' && !Array.isArray(obj[key])) {
      pairs.push(...flattenToParams(obj[key], fullKey))
    } else if (Array.isArray(obj[key])) {
      obj[key].forEach((item: any, i: number) => {
        if (item !== null && typeof item === 'object') {
          pairs.push(...flattenToParams(item, `${fullKey}[${i}]`))
        } else {
          pairs.push(`${encodeURIComponent(fullKey)}=${encodeURIComponent(String(item))}`)
        }
      })
    } else {
      pairs.push(`${encodeURIComponent(fullKey)}=${encodeURIComponent(String(obj[key]))}`)
    }
  }
  return pairs
}

function toJsonGetParams() {
  const parsed = parseInput()
  if (parsed !== null && typeof parsed === 'object') {
    output.value = flattenToParams(parsed).join('&')
    toast.success('转换成功')
  }
}

function toJsonCsv() {
  const parsed = parseInput()
  if (parsed !== null && Array.isArray(parsed) && parsed.length > 0) {
    const keys = Object.keys(parsed[0])
    const header = keys.join(',')
    const rows = parsed.map(row =>
      keys.map(k => {
        const val = row[k]
        const str = val !== null && val !== undefined ? String(val) : ''
        return str.includes(',') || str.includes('"') || str.includes('\n')
          ? `"${str.replace(/"/g, '""')}"`
          : str
      }).join(',')
    )
    output.value = [header, ...rows].join('\n')
    toast.success('转换成功')
  } else {
    toast.warning('JSON 必须是一个数组')
  }
}

function toJsonTable() {
  const parsed = parseInput()
  if (parsed !== null && Array.isArray(parsed) && parsed.length > 0) {
    const keys = Object.keys(parsed[0])
    let html = '<table border="1" cellpadding="4" cellspacing="0">\n'
    html += '  <tr>\n'
    keys.forEach(k => { html += `    <th>${k}</th>\n` })
    html += '  </tr>\n'
    parsed.forEach(row => {
      html += '  <tr>\n'
      keys.forEach(k => { html += `    <td>${row[k] !== null && row[k] !== undefined ? row[k] : ''}</td>\n` })
      html += '  </tr>\n'
    })
    html += '</table>'
    output.value = html
    toast.success('转换成功')
  } else {
    toast.warning('JSON 必须是一个数组')
  }
}

function generateClassFromJson(obj: any, className: string, lang: 'java' | 'csharp' | 'go' | 'dart'): string {
  const lines: string[] = []
  const typeMap: Record<string, Record<string, string>> = {
    java: { string: 'String', number: 'int', boolean: 'boolean', object: 'Object', array: 'List' },
    csharp: { string: 'string', number: 'int', boolean: 'bool', object: 'object', array: 'List' },
    go: { string: 'string', number: 'int', boolean: 'bool', object: 'struct', array: '[]' },
    dart: { string: 'String', number: 'int', boolean: 'bool', object: 'dynamic', array: 'List' },
  }

  if (lang === 'go') {
    lines.push(`type ${className} struct {`)
    for (const [key, value] of Object.entries(obj)) {
      const goType = getGoType(value)
      const fieldName = key.charAt(0).toUpperCase() + key.slice(1)
      const jsonTag = `json:"${key}"`
      lines.push(`\t${fieldName} ${goType} \`${jsonTag}\``)
    }
    lines.push('}')
    return lines.join('\n')
  }

  const t = typeMap[lang]
  const access = lang === 'dart' ? '' : lang === 'java' ? '    private ' : '    public '
  const getter = lang === 'csharp' ? ' { get; set; }' : ''
  const semi = lang === 'dart' ? ';' : lang === 'java' || lang === 'csharp' ? ';' : ''

  if (lang === 'java') {
    lines.push(`public class ${className} {`)
  } else if (lang === 'csharp') {
    lines.push(`public class ${className}`)
    lines.push(`{`)
  } else if (lang === 'dart') {
    lines.push(`class ${className} {`)
  }

  for (const [key, value] of Object.entries(obj)) {
    const valType = Array.isArray(value) ? 'array' : value === null ? 'object' : typeof value
    const type = t[valType] || 'String'
    const fieldName = lang === 'dart' ? key : key.charAt(0).toLowerCase() + key.slice(1)

    if (lang === 'dart') {
      lines.push(`  ${type} ${fieldName};`)
    } else {
      lines.push(`${access}${type} ${fieldName}${getter}${semi}`)
    }
  }

  lines.push('}')
  return lines.join('\n')
}

function getGoType(value: any): string {
  if (Array.isArray(value)) {
    if (value.length > 0) return `[]${getGoType(value[0])}`
    return '[]interface{}'
  }
  if (value === null || value === undefined) return 'interface{}'
  switch (typeof value) {
    case 'string': return 'string'
    case 'number': return Number.isInteger(value) ? 'int' : 'float64'
    case 'boolean': return 'bool'
    case 'object': return 'struct'
    default: return 'interface{}'
  }
}

function toJsonJava() {
  const parsed = parseInput()
  if (parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed)) {
    output.value = generateClassFromJson(parsed, 'Root', 'java')
    toast.success('Java 类生成成功')
  } else {
    toast.warning('JSON 必须是一个对象')
  }
}

function toJsonCSharp() {
  const parsed = parseInput()
  if (parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed)) {
    output.value = generateClassFromJson(parsed, 'Root', 'csharp')
    toast.success('C# 类生成成功')
  } else {
    toast.warning('JSON 必须是一个对象')
  }
}

function toJsonGo() {
  const parsed = parseInput()
  if (parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed)) {
    output.value = generateClassFromJson(parsed, 'Root', 'go')
    toast.success('Go 结构体生成成功')
  } else {
    toast.warning('JSON 必须是一个对象')
  }
}

function toJsonDart() {
  const parsed = parseInput()
  if (parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed)) {
    output.value = generateClassFromJson(parsed, 'Root', 'dart')
    toast.success('Dart 类生成成功')
  } else {
    toast.warning('JSON 必须是一个对象')
  }
}

// Simple JsonPath implementation
function queryJsonPath() {
  if (!input.value.trim() || !jsonPath.value.trim()) return

  let obj: any
  try {
    obj = JSON.parse(input.value)
  } catch {
    toast.error('无效的 JSON')
    return
  }

  const path = jsonPath.value.trim()
  try {
    const result = evaluateJsonPath(obj, path)
    output.value = typeof result === 'string' ? result : JSON.stringify(result, null, 2)
    toast.success('查询成功')
  } catch (e: any) {
    toast.error(`JsonPath 查询失败: ${e.message}`)
  }
}

function evaluateJsonPath(obj: any, path: string): any {
  // Simple jsonpath evaluator supporting: $, $.key, $.arr[0], $.arr[*].key
  if (path === '$') return obj

  let current: any = obj
  const parts = path.replace(/^\$\./, '').split('.')

  for (const part of parts) {
    if (current === null || current === undefined) return undefined

    // Handle array index: key[0] or key[*]
    const match = part.match(/^([^[]+)\[(\*|\d+)\]$/)
    if (match) {
      const key = match[1]
      const index = match[2]
      current = current[key]
      if (!Array.isArray(current)) return undefined

      if (index === '*') {
        // Return all items (if more parts follow, map them)
        const remaining = parts.slice(parts.indexOf(part) + 1).join('.')
        if (remaining) {
          return current.map((item: any) => evaluateJsonPath(item, '$.' + remaining)).flat()
        }
        return current
      } else {
        current = current[parseInt(index)]
      }
      continue
    }

    // Handle bare array index: [0]
    const bareIndex = part.match(/^\[(\*|\d+)\]$/)
    if (bareIndex) {
      if (!Array.isArray(current)) return undefined
      const idx = bareIndex[1]
      if (idx === '*') return current
      current = current[parseInt(idx)]
      continue
    }

    // Handle key access
    if (part && current !== null && typeof current === 'object') {
      current = current[part]
    }
  }

  return current
}

async function copyOutput() {
  if (!output.value) return
  await copyText(output.value, toast)
}
</script>

<style scoped>

.result-ok {
  color: #22c55e;
}

.result-error {
  color: #ef4444;
}

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
