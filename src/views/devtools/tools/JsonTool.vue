<template>
  <div class="flex flex-col h-full">
    <h3 class="text-lg font-bold text-base-content mb-4 flex items-center gap-2">
      JSON 工具
      <span v-if="input" class="text-xs font-normal" :class="isValid ? 'text-success' : 'text-error'">
        <SvgIcon :name="isValid ? 'check' : 'alertTriangle'" :size="14" class="align-text-bottom" />
        {{ isValid ? '有效 JSON' : '解析失败' }}
      </span>
      <button class="btn btn-ghost btn-xs ml-auto" @click="loadExample">示例</button>
      <button class="btn btn-ghost btn-xs" @click="clearAll" v-if="input || output">清空</button>
    </h3>

    <!-- 主操作区：左右分屏 -->
    <div class="grid grid-cols-2 gap-3 flex-1 min-h-0">
      <!-- 左侧：输入 -->
      <div class="flex flex-col min-h-0">
        <div class="flex items-center justify-between mb-1.5">
          <h4 class="text-sm font-semibold text-base-content m-0">输入</h4>
          <div class="flex gap-1.5">
            <button class="btn btn-primary btn-xs" @click="formatJson" :disabled="!isValid">格式化</button>
            <button class="btn btn-ghost btn-xs" @click="compressJson" :disabled="!isValid">压缩</button>
            <button class="btn btn-ghost btn-xs" @click="escapeJson">转义</button>
            <button class="btn btn-ghost btn-xs" @click="unescapeJson">去转义</button>
            <button class="btn btn-ghost btn-xs" @click="pasteFromClipboard" title="从剪贴板粘贴">
              <SvgIcon name="copy" :size="12" />
            </button>
          </div>
        </div>
        <textarea
          v-model="input"
          class="textarea textarea-bordered w-full font-mono text-sm flex-1 min-h-[200px] resize-none"
          placeholder="在此输入 JSON..."
          spellcheck="false"
        ></textarea>
        <div v-if="parseError" class="mt-1.5 text-xs text-error font-mono break-all">{{ parseError }}</div>
      </div>

      <!-- 右侧：输出 -->
      <div class="flex flex-col min-h-0">
        <div class="flex items-center justify-between mb-1.5">
          <h4 class="text-sm font-semibold text-base-content m-0">输出</h4>
          <div class="flex gap-1.5">
            <button class="btn btn-ghost btn-xs" @click="copyOutput" :disabled="!output">
              <SvgIcon name="copy" :size="12" /> 复制
            </button>
            <button class="btn btn-ghost btn-xs" @click="outputToInput" :disabled="!output" title="将输出移到输入">
              <SvgIcon name="refresh" :size="12" />
            </button>
          </div>
        </div>
        <textarea
          v-model="output"
          class="textarea textarea-bordered w-full font-mono text-sm flex-1 min-h-[200px] resize-none"
          readonly
          placeholder="结果将显示在这里..."
        ></textarea>
      </div>
    </div>

    <hr class="border-base-content/10 my-4" />

    <!-- JsonPath 查询 -->
    <div class="mb-4">
      <h4 class="text-sm font-semibold text-base-content mb-2">JsonPath 查询</h4>
      <div class="flex flex-wrap gap-2 mb-2">
        <input
          v-model="jsonPath"
          class="input input-bordered flex-1 font-mono text-sm"
          placeholder="$.store.book[*].author  或  $..price  或  $.store..author"
          @keydown.enter="queryJsonPath"
        />
        <button class="btn btn-primary btn-sm" @click="queryJsonPath" :disabled="!isValid || !jsonPath">查询</button>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <span class="text-xs text-base-content/60">速查：</span>
        <button v-for="ex in jsonPathExamples" :key="ex" class="btn btn-ghost btn-xs font-mono" @click="jsonPath = ex; queryJsonPath()">{{ ex }}</button>
      </div>
    </div>

    <hr class="border-base-content/10 my-4" />

    <!-- 转换工具区 -->
    <div class="grid grid-cols-2 gap-4">
      <!-- Unicode 转换 -->
      <div>
        <h4 class="text-sm font-semibold text-base-content mb-2">Unicode 转换</h4>
        <div class="flex flex-wrap gap-1.5">
          <button class="btn btn-ghost btn-xs" @click="unicodeToChinese" :disabled="!input">\uXXXX → 中文</button>
          <button class="btn btn-ghost btn-xs" @click="chineseToUnicode" :disabled="!input">中文 → \uXXXX</button>
        </div>
      </div>

      <!-- JSON 转换 -->
      <div>
        <h4 class="text-sm font-semibold text-base-content mb-2">格式转换</h4>
        <div class="flex flex-wrap gap-1.5">
          <button class="btn btn-ghost btn-xs" @click="toJsonGetParams" :disabled="!isValid">GET 参数</button>
          <button class="btn btn-ghost btn-xs" @click="toJsonCsv" :disabled="!isValid">CSV</button>
          <button class="btn btn-ghost btn-xs" @click="toJsonTable" :disabled="!isValid">HTML 表格</button>
        </div>
      </div>
    </div>

    <!-- 转语言类定义 -->
    <div class="mt-4">
      <h4 class="text-sm font-semibold text-base-content mb-2">转语言类定义</h4>
      <div class="flex flex-wrap gap-1.5">
        <button class="btn btn-ghost btn-xs" @click="toJsonJava" :disabled="!isValid">Java</button>
        <button class="btn btn-ghost btn-xs" @click="toJsonCSharp" :disabled="!isValid">C#</button>
        <button class="btn btn-ghost btn-xs" @click="toJsonGo" :disabled="!isValid">Go</button>
        <button class="btn btn-ghost btn-xs" @click="toJsonDart" :disabled="!isValid">Dart</button>
        <button class="btn btn-ghost btn-xs" @click="toJsonTypeScript" :disabled="!isValid">TypeScript</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const input = ref('')
const output = ref('')
const jsonPath = ref('')

const jsonPathExamples = ['$', '$..author', '$.store.book[*].author', '$.store.book[0].title', '$..price']

const parsed = computed<{ value: any; error: string }>(() => {
  if (!input.value.trim()) {
    return { value: null, error: '' }
  }
  try {
    return { value: JSON.parse(input.value), error: '' }
  } catch (e: any) {
    const msg = e.message
    const posMatch = msg.match(/position\s+(\d+)/i)
    if (posMatch) {
      const pos = parseInt(posMatch[1])
      const textBefore = input.value.substring(0, pos)
      const line = (textBefore.match(/\n/g) || []).length + 1
      const col = pos - textBefore.lastIndexOf('\n')
      return { value: null, error: `${msg} (第 ${line} 行, 第 ${col} 列)` }
    }
    return { value: null, error: msg }
  }
})

const isValid = computed(() => input.value.trim() !== '' && parsed.value.value !== null)
const parseError = computed(() => parsed.value.error)

function formatJson() {
  if (parsed.value.value !== null) {
    output.value = JSON.stringify(parsed.value.value, null, 2)
    toast.success('格式化成功')
  }
}

function compressJson() {
  if (parsed.value.value !== null) {
    output.value = JSON.stringify(parsed.value.value)
    toast.success('压缩成功')
  }
}

function escapeJson() {
  if (!input.value.trim()) { toast.warning('请输入内容'); return }
  output.value = JSON.stringify(input.value)
  toast.success('转义成功')
}

function unescapeJson() {
  if (!input.value.trim()) { toast.warning('请输入内容'); return }
  try {
    output.value = JSON.parse(input.value)
    toast.success('去转义成功')
  } catch (e: any) { toast.error(`去转义失败: ${e.message}`) }
}

function unicodeToChinese() {
  if (!input.value.trim()) { toast.warning('请输入内容'); return }
  output.value = input.value.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => String.fromCharCode(parseInt(hex, 16)))
  toast.success('转换成功')
}

function chineseToUnicode() {
  if (!input.value.trim()) { toast.warning('请输入内容'); return }
  output.value = input.value.replace(/[^\u0000-\u007f]/g, (c) => '\\u' + c.charCodeAt(0).toString(16).padStart(4, '0'))
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
  const val = parsed.value.value
  if (val !== null && typeof val === 'object') {
    output.value = flattenToParams(val).join('&')
    toast.success('转换成功')
  }
}

function toJsonCsv() {
  const val = parsed.value.value
  if (val !== null && Array.isArray(val) && val.length > 0) {
    const keys = Object.keys(val[0])
    const header = keys.join(',')
    const rows = val.map(row =>
      keys.map(k => {
        const v = row[k]
        const str = v !== null && v !== undefined ? String(v) : ''
        return str.includes(',') || str.includes('"') || str.includes('\n') ? `"${str.replace(/"/g, '""')}"` : str
      }).join(',')
    )
    output.value = [header, ...rows].join('\n')
    toast.success('转换成功')
  } else { toast.warning('JSON 必须是一个数组') }
}

function toJsonTable() {
  const val = parsed.value.value
  if (val !== null && Array.isArray(val) && val.length > 0) {
    const keys = Object.keys(val[0])
    let html = '<table border="1" cellpadding="4" cellspacing="0">\n  <tr>\n'
    keys.forEach(k => { html += `    <th>${k}</th>\n` })
    html += '  </tr>\n'
    val.forEach(row => {
      html += '  <tr>\n'
      keys.forEach(k => { html += `    <td>${row[k] !== null && row[k] !== undefined ? row[k] : ''}</td>\n` })
      html += '  </tr>\n'
    })
    html += '</table>'
    output.value = html
    toast.success('转换成功')
  } else { toast.warning('JSON 必须是一个数组') }
}

// ─── 转语言类定义（支持嵌套对象生成嵌套类）───
function collectNestedObjects(obj: any, prefix: string, nested: Map<string, any>) {
  if (obj === null || typeof obj !== 'object' || Array.isArray(obj)) return
  for (const [key, value] of Object.entries(obj)) {
    if (value !== null && typeof value === 'object' && !Array.isArray(value)) {
      const className = `${prefix}${capitalize(key)}`
      nested.set(className, value)
      collectNestedObjects(value, className, nested)
    } else if (Array.isArray(value) && value.length > 0 && value[0] !== null && typeof value[0] === 'object') {
      const className = `${prefix}${capitalize(key)}Item`
      nested.set(className, value[0])
      collectNestedObjects(value[0], className, nested)
    }
  }
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1)
}

function generateJava(obj: any, className: string): string {
  const nested = new Map<string, any>()
  collectNestedObjects(obj, className, nested)

  const lines: string[] = [`public class ${className} {`]
  for (const [key, value] of Object.entries(obj)) {
    let type: string
    if (Array.isArray(value)) {
      if (value.length > 0 && value[0] !== null && typeof value[0] === 'object') {
        type = `List<${className}${capitalize(key)}Item>`
      } else if (value.length > 0) {
        type = `List<${getJavaPrimitiveType(value[0])}>`
      } else {
        type = 'List<Object>'
      }
    } else if (value !== null && typeof value === 'object') {
      type = `${className}${capitalize(key)}`
    } else {
      type = getJavaPrimitiveType(value)
    }
    lines.push(`    private ${type} ${key};`)
  }
  lines.push('}')

  // Append nested classes
  for (const [nestedName, nestedObj] of nested) {
    lines.push('')
    lines.push(generateJava(nestedObj, nestedName))
  }
  return lines.join('\n')
}

function getJavaPrimitiveType(value: any): string {
  if (value === null || value === undefined) return 'Object'
  switch (typeof value) {
    case 'string': return 'String'
    case 'number': return Number.isInteger(value) ? 'int' : 'double'
    case 'boolean': return 'boolean'
    default: return 'Object'
  }
}

function generateCSharp(obj: any, className: string): string {
  const nested = new Map<string, any>()
  collectNestedObjects(obj, className, nested)

  const lines: string[] = [`public class ${className}`, '{']
  for (const [key, value] of Object.entries(obj)) {
    let type: string
    if (Array.isArray(value)) {
      if (value.length > 0 && value[0] !== null && typeof value[0] === 'object') {
        type = `List<${className}${capitalize(key)}Item>`
      } else if (value.length > 0) {
        type = `List<${getCSharpType(value[0])}>`
      } else {
        type = 'List<object>'
      }
    } else if (value !== null && typeof value === 'object') {
      type = `${className}${capitalize(key)}`
    } else {
      type = getCSharpType(value)
    }
    lines.push(`    public ${type} ${capitalize(key)} { get; set; }`)
  }
  lines.push('}')

  for (const [nestedName, nestedObj] of nested) {
    lines.push('')
    lines.push(generateCSharp(nestedObj, nestedName))
  }
  return lines.join('\n')
}

function getCSharpType(value: any): string {
  if (value === null || value === undefined) return 'object'
  switch (typeof value) {
    case 'string': return 'string'
    case 'number': return Number.isInteger(value) ? 'int' : 'double'
    case 'boolean': return 'bool'
    default: return 'object'
  }
}

function generateGo(obj: any, className: string): string {
  const nested = new Map<string, any>()
  collectNestedObjects(obj, className, nested)

  const lines: string[] = [`type ${className} struct {`]
  for (const [key, value] of Object.entries(obj)) {
    let goType: string
    if (Array.isArray(value)) {
      if (value.length > 0 && value[0] !== null && typeof value[0] === 'object') {
        goType = `[]${className}${capitalize(key)}Item`
      } else if (value.length > 0) {
        goType = `[]${getGoPrimitiveType(value[0])}`
      } else {
        goType = '[]interface{}'
      }
    } else if (value !== null && typeof value === 'object') {
      goType = `${className}${capitalize(key)}`
    } else {
      goType = getGoPrimitiveType(value)
    }
    lines.push(`\t${capitalize(key)} ${goType} \`json:"${key}"\``)
  }
  lines.push('}')

  for (const [nestedName, nestedObj] of nested) {
    lines.push('')
    lines.push(generateGo(nestedObj, nestedName))
  }
  return lines.join('\n')
}

function getGoPrimitiveType(value: any): string {
  if (value === null || value === undefined) return 'interface{}'
  switch (typeof value) {
    case 'string': return 'string'
    case 'number': return Number.isInteger(value) ? 'int' : 'float64'
    case 'boolean': return 'bool'
    default: return 'interface{}'
  }
}

function generateDart(obj: any, className: string): string {
  const nested = new Map<string, any>()
  collectNestedObjects(obj, className, nested)

  const lines: string[] = [`class ${className} {`]
  for (const [key, value] of Object.entries(obj)) {
    let type: string
    if (Array.isArray(value)) {
      if (value.length > 0 && value[0] !== null && typeof value[0] === 'object') {
        type = `List<${className}${capitalize(key)}Item>`
      } else if (value.length > 0) {
        type = `List<${getDartType(value[0])}>`
      } else {
        type = 'List<dynamic>'
      }
    } else if (value !== null && typeof value === 'object') {
      type = `${className}${capitalize(key)}`
    } else {
      type = getDartType(value)
    }
    lines.push(`  ${type} ${key};`)
  }
  lines.push('}')

  for (const [nestedName, nestedObj] of nested) {
    lines.push('')
    lines.push(generateDart(nestedObj, nestedName))
  }
  return lines.join('\n')
}

function getDartType(value: any): string {
  if (value === null || value === undefined) return 'dynamic'
  switch (typeof value) {
    case 'string': return 'String'
    case 'number': return 'int'
    case 'boolean': return 'bool'
    default: return 'dynamic'
  }
}

function generateTypeScript(obj: any, className: string): string {
  const nested = new Map<string, any>()
  collectNestedObjects(obj, className, nested)

  const lines: string[] = [`export interface ${className} {`]
  for (const [key, value] of Object.entries(obj)) {
    let type: string
    if (Array.isArray(value)) {
      if (value.length > 0 && value[0] !== null && typeof value[0] === 'object') {
        type = `${className}${capitalize(key)}Item[]`
      } else if (value.length > 0) {
        type = `${getTsType(value[0])}[]`
      } else {
        type = 'any[]'
      }
    } else if (value !== null && typeof value === 'object') {
      type = `${className}${capitalize(key)}`
    } else {
      type = getTsType(value)
    }
    lines.push(`  ${key}: ${type};`)
  }
  lines.push('}')

  for (const [nestedName, nestedObj] of nested) {
    lines.push('')
    lines.push(generateTypeScript(nestedObj, nestedName))
  }
  return lines.join('\n')
}

function getTsType(value: any): string {
  if (value === null || value === undefined) return 'any'
  switch (typeof value) {
    case 'string': return 'string'
    case 'number': return 'number'
    case 'boolean': return 'boolean'
    default: return 'any'
  }
}

function toJsonJava() {
  const val = parsed.value.value
  if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
    output.value = generateJava(val, 'Root')
    toast.success('Java 类生成成功')
  } else { toast.warning('JSON 必须是一个对象') }
}

function toJsonCSharp() {
  const val = parsed.value.value
  if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
    output.value = generateCSharp(val, 'Root')
    toast.success('C# 类生成成功')
  } else { toast.warning('JSON 必须是一个对象') }
}

function toJsonGo() {
  const val = parsed.value.value
  if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
    output.value = generateGo(val, 'Root')
    toast.success('Go 结构体生成成功')
  } else { toast.warning('JSON 必须是一个对象') }
}

function toJsonDart() {
  const val = parsed.value.value
  if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
    output.value = generateDart(val, 'Root')
    toast.success('Dart 类生成成功')
  } else { toast.warning('JSON 必须是一个对象') }
}

function toJsonTypeScript() {
  const val = parsed.value.value
  if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
    output.value = generateTypeScript(val, 'Root')
    toast.success('TypeScript 接口生成成功')
  } else { toast.warning('JSON 必须是一个对象') }
}

// ─── JsonPath（支持 $.. 递归下降、$..key、数组索引/通配）───
function queryJsonPath() {
  if (!isValid.value || !jsonPath.value.trim()) return

  const path = jsonPath.value.trim()
  try {
    const result = evaluateJsonPath(parsed.value.value, path)
    output.value = typeof result === 'string' ? result : JSON.stringify(result, null, 2)
    toast.success('查询成功')
  } catch (e: any) {
    toast.error(`JsonPath 查询失败: ${e.message}`)
  }
}

function evaluateJsonPath(obj: any, path: string): any {
  if (path === '$') return obj

  let current: any = obj
  // Split by . but handle $.. (recursive descent) specially
  const parts = path.replace(/^\$/, '').split(/(?<!\.)\./).filter(p => p !== '')

  for (let i = 0; i < parts.length; i++) {
    const part = parts[i]

    // Recursive descent: ..key
    if (part.startsWith('..')) {
      const key = part.slice(2)
      const remaining = parts.slice(i + 1).join('.')
      if (key === '') {
        // $..  → all values recursively
        return collectAllValues(current)
      }
      const found = findKeyRecursive(current, key)
      if (remaining) {
        return found.map((item: any) => evaluateJsonPath(item, '$.' + remaining)).flat()
      }
      return found
    }

    if (current === null || current === undefined) return undefined

    // Handle array index: key[0] or key[*]
    const match = part.match(/^([^[]+)\[(\*|\d+)\]$/)
    if (match) {
      const key = match[1]
      const index = match[2]
      current = current[key]
      if (!Array.isArray(current)) return undefined
      if (index === '*') {
        const remaining = parts.slice(i + 1).join('.')
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
      if (idx === '*') {
        const remaining = parts.slice(i + 1).join('.')
        if (remaining) {
          return current.map((item: any) => evaluateJsonPath(item, '$.' + remaining)).flat()
        }
        return current
      }
      current = current[parseInt(idx)]
      continue
    }

    // Plain key access
    if (part && current !== null && typeof current === 'object') {
      current = current[part]
    }
  }

  return current
}

function findKeyRecursive(obj: any, key: string): any[] {
  const results: any[] = []
  if (obj === null || obj === undefined) return results

  if (Array.isArray(obj)) {
    for (const item of obj) {
      results.push(...findKeyRecursive(item, key))
    }
  } else if (typeof obj === 'object') {
    if (key in obj) {
      results.push(obj[key])
    }
    for (const value of Object.values(obj)) {
      results.push(...findKeyRecursive(value, key))
    }
  }
  return results
}

function collectAllValues(obj: any): any[] {
  const results: any[] = []
  if (obj === null || obj === undefined) return results
  if (Array.isArray(obj)) {
    for (const item of obj) {
      results.push(...collectAllValues(item))
    }
  } else if (typeof obj === 'object') {
    for (const value of Object.values(obj)) {
      results.push(...collectAllValues(value))
    }
  } else {
    results.push(obj)
  }
  return results
}

async function copyOutput() {
  if (!output.value) return
  await copyText(output.value, toast)
}

function outputToInput() {
  if (!output.value) return
  input.value = output.value
  output.value = ''
}

async function pasteFromClipboard() {
  try {
    const text = await navigator.clipboard.readText()
    if (text) {
      input.value = text
      toast.success('已粘贴')
    }
  } catch {
    toast.error('无法读取剪贴板')
  }
}

function loadExample() {
  input.value = JSON.stringify({
    store: {
      book: [
        { category: 'reference', author: 'Nigel Rees', title: 'Sayings of the Century', price: 8.95 },
        { category: 'fiction', author: 'Evelyn Waugh', title: 'Sword of Honour', price: 12.99 },
        { category: 'fiction', author: 'Herman Melville', title: 'Moby Dick', isbn: '0-553-21311-3', price: 8.99 },
      ],
      bicycle: { color: 'red', price: 19.95 },
    },
  }, null, 2)
  output.value = ''
}

function clearAll() {
  input.value = ''
  output.value = ''
  jsonPath.value = ''
}
</script>
