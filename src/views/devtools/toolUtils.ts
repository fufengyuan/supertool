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
const BASE64_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'

export function baseConvert(numStr: string, fromBase: number, toBase: number): string {
  if (fromBase < 2 || fromBase > 64 || toBase < 2 || toBase > 64) return '错误: 支持 2-64 进制'
  
  // Normalize input
  const normalized = numStr.trim()
  if (!normalized) return ''
  
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
    if (val === undefined) throw new Error('Invalid character')
    result = result * BigInt(base) + BigInt(val)
  }
  return result
}

function customToString(num: bigint, base: number): string {
  if (num === 0n) return '0'
  
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
