<template>
  <ToolPage
    icon="refresh"
    name="序列化转换"
    description="JSON / XML / YAML / PHP Array / PHP Serialize / Properties 双向互转"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl px-4 py-3 flex flex-wrap items-center gap-3">
      <div class="flex items-center gap-2">
        <span class="text-xs text-base-content/60">转换类型</span>
        <select v-model="conversionType" class="select select-bordered select-sm bg-base-200/60">
          <option v-for="opt in conversionOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>
      </div>
      <div class="ml-auto flex gap-2">
        <button class="btn btn-primary btn-sm" @click="convert" :disabled="!input">转换</button>
        <button class="btn btn-outline btn-sm" @click="swapInputOutput" :disabled="!output">⇄ 互换</button>
        <button class="btn btn-ghost btn-sm" @click="clearAll" :disabled="!input && !output">清空</button>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[240px]">
        <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入</h4>
        <textarea v-model="input" class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 min-h-[140px]" placeholder="在此输入..."></textarea>
      </div>
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[240px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 输出</h4>
          <button class="btn btn-primary btn-xs" @click="copyOutput" :disabled="!output"><SvgIcon name="copy" size="11" /> 复制</button>
        </div>
        <textarea v-model="output" readonly class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 min-h-[140px]" placeholder="结果将显示在这里..."></textarea>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'// @ts-nocheck
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import yaml from 'js-yaml'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

const conversionOptions = [
  { value: 'json2xml', label: 'JSON → XML' },
  { value: 'xml2json', label: 'XML → JSON' },
  { value: 'json2yaml', label: 'JSON → YAML' },
  { value: 'yaml2json', label: 'YAML → JSON' },
  { value: 'json2php', label: 'JSON → PHP Array' },
  { value: 'php2json', label: 'PHP Array → JSON' },
  { value: 'json2phpser', label: 'JSON → PHP Serialize' },
  { value: 'phpser2json', label: 'PHP Serialize → JSON' },
  { value: 'json2props', label: 'JSON → Properties' },
  { value: 'props2json', label: 'Properties → JSON' },
]

const conversionType = ref('json2xml')
const input = ref('')
const output = ref('')

function clearAll() {
  input.value = ''
  output.value = ''
}

function swapInputOutput() {
  const tmp = input.value
  input.value = output.value
  output.value = tmp
}

function convert() {
  try {
    switch (conversionType.value) {
      case 'json2xml':
        output.value = jsonToXml(input.value)
        toast.success('JSON → XML 转换成功')
        break
      case 'xml2json':
        output.value = xmlToJson(input.value)
        toast.success('XML → JSON 转换成功')
        break
      case 'json2yaml':
        output.value = jsonToYaml(input.value)
        toast.success('JSON → YAML 转换成功')
        break
      case 'yaml2json':
        output.value = yamlToJson(input.value)
        toast.success('YAML → JSON 转换成功')
        break
      case 'json2php':
        output.value = jsonToPhpArray(input.value)
        toast.success('JSON → PHP Array 转换成功')
        break
      case 'php2json':
        output.value = phpArrayToJson(input.value)
        toast.success('PHP Array → JSON 转换成功')
        break
      case 'json2phpser':
        output.value = jsonToPhpSerialize(input.value)
        toast.success('JSON → PHP Serialize 转换成功')
        break
      case 'phpser2json':
        output.value = phpSerializeToJson(input.value)
        toast.success('PHP Serialize → JSON 转换成功')
        break
      case 'json2props':
        output.value = jsonToProperties(input.value)
        toast.success('JSON → Properties 转换成功')
        break
      case 'props2json':
        output.value = propertiesToJson(input.value)
        toast.success('Properties → JSON 转换成功')
        break
    }
  } catch (e: any) {
    output.value = ''
    toast.error(`转换失败: ${e.message}`)
  }
}

/* ─── JSON ↔ XML ─── */
function jsonToXml(jsonStr: string): string {
  const obj = JSON.parse(jsonStr)
  return objToXml(obj, 'root')
}

function objToXml(obj: any, tagName: string, indent = 0): string {
  const pad = '  '.repeat(indent)

  if (obj === null || obj === undefined) {
    return `${pad}<${tagName} />\n`
  }

  if (typeof obj !== 'object') {
    return `${pad}<${tagName}>${escapeXml(String(obj))}</${tagName}>\n`
  }

  if (Array.isArray(obj)) {
    let result = ''
    for (const item of obj) {
      result += objToXml(item, tagName.replace(/s$/, ''), indent)
    }
    return result
  }

  let result = `${pad}<${tagName}>\n`
  for (const [key, value] of Object.entries(obj)) {
    result += objToXml(value, key, indent + 1)
  }
  result += `${pad}</${tagName}>\n`
  return result
}

function escapeXml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')
}

function xmlToJson(xmlStr: string): string {
  const obj = simpleXmlParse(xmlStr.trim())
  return JSON.stringify(obj, null, 2)
}

function simpleXmlParse(xml: string): any {
  const result: any = {}

  function parseNode(str: string): any {
    // Remove XML declaration and comments
    str = str.replace(/<\?xml[^?]*\?>/g, '').replace(/<!--[\s\S]*?-->/g, '')

    // Find root tag
    const rootMatch = str.match(/<(\w+)(\s[^>]*)?>([\s\S]*)<\/\1>/)
    if (!rootMatch) {
      // Try self-closing
      const selfMatch = str.match(/<(\w+)\s*\/>/)
      if (selfMatch) {return { [selfMatch[1]]: null }}
      return str.trim()
    }

    const tagName = rootMatch[1]
    let content = rootMatch[3].trim()

    // Check if content has nested tags
    if (content.includes('<')) {
      const children: any = {}
      let remaining = content
      while (remaining.trim()) {
        // Match a tag
        const tagMatch = remaining.match(/<(\w+)(\s[^>]*)?>([\s\S]*?)<\/\1>/) || remaining.match(/<(\w+)(\s[^>]*)?\/>/)
        if (!tagMatch) {break}

        const childName = tagMatch[1]
        let childValue: any

        if (tagMatch[3] !== undefined) {
          childValue = parseNode(`<${childName}>${tagMatch[3]}</${childName}>`)
        } else {
          childValue = null
        }

        if (children[childName] !== undefined) {
          if (!Array.isArray(children[childName])) {
            children[childName] = [children[childName]]
          }
          children[childName].push(childValue)
        } else {
          children[childName] = childValue
        }

        remaining = remaining.slice(tagMatch[0].length).trim()
      }
      return { [tagName]: children }
    } else {
      return { [tagName]: content }
    }
  }

  return parseNode(xml)
}

/* ─── JSON ↔ YAML ─── */
function jsonToYaml(jsonStr: string): string {
  const obj = JSON.parse(jsonStr)
  return yaml.dump(obj, { indent: 2, lineWidth: -1 })
}

function yamlToJson(yamlStr: string): string {
  const obj = yaml.load(yamlStr)
  return JSON.stringify(obj, null, 2)
}

/* ─── JSON ↔ PHP Array ─── */
function jsonToPhpArray(jsonStr: string): string {
  const obj = JSON.parse(jsonStr)
  return valToPhpArray(obj, 0)
}

function valToPhpArray(val: any, indent: number): string {
  const pad = '  '.repeat(indent)

  if (val === null || val === undefined) {return 'null'}
  if (typeof val === 'boolean') {return val ? 'true' : 'false'}
  if (typeof val === 'number') {return String(val)}
  if (typeof val === 'string') {return `'${val.replace(/'/g, "\\'")}'`}

  if (Array.isArray(val)) {
    if (val.length === 0) {return '[]'}
    let result = '[\n'
    for (const item of val) {
      result += `${pad}  ${valToPhpArray(item, indent + 1)},\n`
    }
    result += `${pad}]`
    return result
  }

  if (typeof val === 'object') {
    const keys = Object.keys(val)
    if (keys.length === 0) {return '[]'}
    let result = '[\n'
    for (const key of keys) {
      result += `${pad}  '${key}' => ${valToPhpArray(val[key], indent + 1)},\n`
    }
    result += `${pad}]`
    return result
  }

  return String(val)
}

function phpArrayToJson(phpStr: string): string {
  // Simple PHP array parser
  const cleaned = phpStr.trim()
  if (!cleaned.startsWith('[')) {
    throw new Error('无效的 PHP 数组格式')
  }
  // Very simplified: remove PHP syntax and convert to JSON-like
  let json = cleaned
    .replace(/=>/g, ':')
    .replace(/'/g, '"')
    .replace(/,\s*\]/g, ']')
    .replace(/,\s*\}/g, '}')
    .replace(/\btrue\b/g, 'true')
    .replace(/\bfalse\b/g, 'false')
    .replace(/\bnull\b/g, 'null')

  const obj = JSON.parse(json)
  return JSON.stringify(obj, null, 2)
}

/* ─── JSON ↔ PHP Serialize ─── */
function jsonToPhpSerialize(jsonStr: string): string {
  const obj = JSON.parse(jsonStr)
  return phpSerialize(obj)
}

function phpSerialize(val: any): string {
  if (val === null || val === undefined) {return 'N;'}
  if (typeof val === 'boolean') {return val ? 'b:1;' : 'b:0;'}
  if (typeof val === 'number') {
    if (Number.isInteger(val)) {return `i:${val};`}
    return `d:${val};`
  }
  if (typeof val === 'string') {return `s:${val.length}:"${val}";`}

  if (Array.isArray(val)) {
    const items = val.map((v, i) => `i:${i};${phpSerialize(v)}`).join('')
    return `a:${val.length}:{${items}}`
  }

  if (typeof val === 'object') {
    const keys = Object.keys(val)
    const items = keys.map(k => `s:${k.length}:"${k}";${phpSerialize(val[k])}`).join('')
    return `a:${keys.length}:{${items}}`
  }

  return `s:${String(val).length}:"${String(val)}";`
}

function phpSerializeToJson(phpSer: string): string {
  const obj = phpUnserialize(phpSer.trim())
  return JSON.stringify(obj, null, 2)
}

function phpUnserialize(str: string): any {
  let pos = 0

  function parse(): any {
    if (pos >= str.length) {return null}

    const type = str[pos]
    pos += 2 // skip type and colon

    switch (type) {
      case 'N':
        pos++ // skip semicolon
        return null
      case 'b': {
        const val = str[pos] === '1'
        pos += 2
        return val
      }
      case 'i': {
        const end = str.indexOf(';', pos)
        const val = parseInt(str.substring(pos, end))
        pos = end + 1
        return val
      }
      case 'd': {
        const end = str.indexOf(';', pos)
        const val = parseFloat(str.substring(pos, end))
        pos = end + 1
        return val
      }
      case 's': {
        const lenEnd = str.indexOf(':', pos)
        const len = parseInt(str.substring(pos, lenEnd))
        pos = lenEnd + 2 // skip : and "
        const val = str.substring(pos, pos + len)
        pos += len + 2 // skip string and closing " and ;
        return val
      }
      case 'a': {
        const lenEnd = str.indexOf(':', pos)
        const len = parseInt(str.substring(pos, lenEnd))
        pos = lenEnd + 2 // skip : and {
        const result: any = {}
        for (let i = 0; i < len; i++) {
          const key = parse()
          const value = parse()
          if (typeof key === 'number' && key === i) {
            if (!Array.isArray(result)) {Object.assign(result, [])}
          }
          if (Array.isArray(result)) {
            result.push(value)
          } else {
            result[String(key)] = value
          }
        }
        pos++ // skip }
        return Array.isArray(result) ? result : Object.keys(result).length > 0 ? result : []
      }
      default:
        throw new Error(`Unknown serialize type: ${type}`)
    }
  }

  return parse()
}

/* ─── JSON ↔ Properties ─── */
function jsonToProperties(jsonStr: string): string {
  const obj = JSON.parse(jsonStr)
  return flatObjToProps(obj)
}

function flatObjToProps(obj: any, prefix = ''): string {
  const lines: string[] = []
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key
    if (value !== null && typeof value === 'object' && !Array.isArray(value)) {
      lines.push(flatObjToProps(value, fullKey))
    } else if (Array.isArray(value)) {
      value.forEach((item, i) => {
        lines.push(`${fullKey}[${i}]=${String(item)}`)
      })
    } else {
      lines.push(`${fullKey}=${String(value)}`)
    }
  }
  return lines.join('\n')
}

function propertiesToJson(propsStr: string): string {
  const result: any = {}
  const lines = propsStr.split('\n').filter(l => l.trim() && !l.trim().startsWith('#'))

  for (const line of lines) {
    const eqIndex = line.indexOf('=')
    if (eqIndex === -1) {continue}

    const key = line.substring(0, eqIndex).trim()
    const value = line.substring(eqIndex + 1).trim()

    // Parse dotted keys into nested objects
    const parts = key.split('.')
    let current = result
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      const arrayMatch = part.match(/^(.+)\[(\d+)\]$/)
      if (arrayMatch && i === parts.length - 1) {
        const arrKey = arrayMatch[1]
        const idx = parseInt(arrayMatch[2])
        if (!current[arrKey]) {current[arrKey] = []}
        current[arrKey][idx] = tryParseValue(value)
      } else if (i === parts.length - 1) {
        current[part] = tryParseValue(value)
      } else {
        if (!current[part] || typeof current[part] !== 'object') {
          current[part] = {}
        }
        current = current[part]
      }
    }
  }

  return JSON.stringify(result, null, 2)
}

function tryParseValue(val: string): any {
  if (val === 'true') {return true}
  if (val === 'false') {return false}
  if (val === 'null') {return null}
  const num = Number(val)
  if (!isNaN(num) && val !== '') {return num}
  return val
}

async function copyOutput() {
  if (!output.value) {return}
  await copyText(output.value, toast)
}
</script>
