import { useToast } from '@/composables/useToast'

/* ─── Clipboard ─── */
export async function copyText(text: string, toast: ReturnType<typeof useToast>, msg = '已复制到剪贴板') {
  try {
    await navigator.clipboard.writeText(text)
    toast.success(msg)
  } catch {
    const ta = document.createElement('textarea')
    ta.value = text
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    toast.success(msg)
  }
}

/* ─── Download File ─── */
export function downloadFile(content: string, filename: string, mime = 'text/plain') {
  const blob = new Blob([content], { type: mime })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}

/* ─── File Read ─── */
export function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsText(file)
  })
}

export function readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as ArrayBuffer)
    reader.onerror = reject
    reader.readAsArrayBuffer(file)
  })
}

/* ─── Base Conversion (2-64) ─── */
// i < 10 用数字 '0'-'9'；i >= 10 从此表取值：A-Z(10-35) → a-z(36-61) → +(62) → /(63)
const BASE64_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/'

export function baseConvert(numStr: string, fromBase: number, toBase: number): string {
  if (fromBase < 2 || fromBase > 64 || toBase < 2 || toBase > 64) {return '错误: 支持 2-64 进制'}
  
  // Normalize input
  const normalized = numStr.trim()
  if (!normalized) {return ''}
  
  // Convert to BigInt
  let bigInt: bigint
  try {
    if (fromBase <= 36) {
      bigInt = BigInt(parseInt(normalized, fromBase))
    } else {
      // For bases > 36, we need custom parsing
      bigInt = customParseBigInt(normalized, fromBase)
    }
  } catch {
    return '错误: 无效的输入'
  }
  
  // Convert to target base
  if (toBase <= 36) {
    return bigInt.toString(toBase).toUpperCase()
  } else {
    return customToString(bigInt, toBase)
  }
}

function customParseBigInt(str: string, base: number): bigint {
  let result = 0n
  const charMap = new Map<string, number>()
  for (let i = 0; i < base; i++) {
    const char = i < 10 ? String(i) : BASE64_CHARS[i - 10]
    charMap.set(char, i)
    charMap.set(char.toLowerCase(), i)
  }
  
  for (const char of str) {
    const val = charMap.get(char)
    if (val === undefined) {throw new Error('Invalid character')}
    result = result * BigInt(base) + BigInt(val)
  }
  return result
}

function customToString(num: bigint, base: number): string {
  if (num === 0n) {return '0'}
  
  const charMap = []
  for (let i = 0; i < base; i++) {
    charMap.push(i < 10 ? String(i) : BASE64_CHARS[i - 10])
  }
  
  let result = ''
  let n = num
  while (n > 0n) {
    result = charMap[Number(n % BigInt(base))] + result
    n = n / BigInt(base)
  }
  return result
}

/* ─── Note: toolStyles removed ───
 * Styles are now defined directly in each component's <style scoped> block.
 * Previously, v-bind: toolStyles caused postcss parsing errors (ENOENT on data:text/css).
 */

/* ─── 报文格式化 ─── */
/** 工具页支持的报文格式；auto 表示按内容自动识别 */
export type TextFormat = 'auto' | 'json' | 'xml' | 'query' | 'text'

export const FORMAT_OPTIONS: { value: TextFormat; label: string }[] = [
  { value: 'auto', label: '自动识别' },
  { value: 'json', label: 'JSON' },
  { value: 'xml', label: 'XML' },
  { value: 'query', label: 'Query/表单' },
  { value: 'text', label: '纯文本' },
]

/** 自动识别报文格式：JSON / XML / Query(URL 查询串或 k=v& 表单) / 纯文本 */
export function detectFormat(text: string): Exclude<TextFormat, 'auto'> {
  const s = text.trim()
  if (!s) {return 'text'}
  if (s.startsWith('{') || s.startsWith('[')) {
    try {
      JSON.parse(s)
      return 'json'
    } catch {/* 长得像 JSON 但解析失败，继续按其他格式判断 */}
  }
  if (s.startsWith('<')) {return 'xml'}
  // a=1&b=2 表单串 / URL 查询串：每组都是 k=v，且至少一组的值非空
  // （避免把 base64 结尾的 "abc=" 这类纯文本误判成表单串）
  if (s.includes('=') && !/\s/.test(s) && !s.includes('{') && !s.includes('<')) {
    const pairs = s.split('&')
    if (pairs.every(p => /^[^=]+=/.test(p)) && pairs.some(p => p.slice(p.indexOf('=') + 1) !== '')) {
      return 'query'
    }
  }
  return 'text'
}

function decodeSafe(v: string): string {
  try {
    return decodeURIComponent(v.replace(/\+/g, ' '))
  } catch {
    return v
  }
}

/** 简易 XML 缩进（不引入额外依赖）：按标签换行 + 两空格缩进 */
function formatXml(raw: string): string {
  const oneLine = raw.replace(/>\s*</g, '><').trim()
  const reg = /(>)(<)(\/*)/g
  const xmlStr = oneLine.replace(reg, '$1\n$2$3')
  let pad = 0
  return xmlStr
    .split('\n')
    .map((line) => {
      let indent = 0
      if (/^<\/\w/.test(line)) {
        if (pad !== 0) {pad -= 1}
      } else if (/^<\w[^>]*[^/]>.*$/.test(line) && !/<\/\w[^>]*>$/.test(line)) {
        indent = 1
      }
      const out = '  '.repeat(pad) + line
      pad += indent
      return out
    })
    .join('\n')
}

/** 查询串/表单串：每项一行，值做 URL 解码；完整 URL 只取 ? 之后的部分 */
function formatQuery(raw: string): string {
  const s = raw.trim()
  const q = s.includes('?') ? s.slice(s.indexOf('?') + 1) : s
  return q
    .split('&')
    .filter(kv => kv.length > 0)
    .map((kv) => {
      const i = kv.indexOf('=')
      if (i < 0) {return decodeSafe(kv)}
      return `${decodeSafe(kv.slice(0, i))} = ${decodeSafe(kv.slice(i + 1))}`
    })
    .join('\n')
}

/**
 * 格式化报文。pretty=true 美化（缩进/换行），false 压缩成单行。
 * 解析失败时抛错，由调用方决定提示或原样展示。
 */
export function formatText(text: string, format: TextFormat = 'auto', pretty = true): string {
  const raw = text ?? ''
  const fmt = format === 'auto' ? detectFormat(raw) : format
  switch (fmt) {
    case 'json': {
      const obj = JSON.parse(raw.trim())
      return pretty ? JSON.stringify(obj, null, 2) : JSON.stringify(obj)
    }
    case 'xml':
      return pretty ? formatXml(raw) : raw.replace(/>\s*</g, '><').trim()
    case 'query':
      // 压缩时要去掉美化时加的 " = " 两侧空格，否则拼回去不是合法查询串
      return pretty
        ? formatQuery(raw)
        : raw.trim().replace(/\s*\n\s*/g, '&').replace(/\s*=\s*/g, '=')
    default:
      return pretty ? raw : raw.replace(/\s*\n\s*/g, '')
  }
}
