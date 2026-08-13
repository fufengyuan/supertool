<template>
  <ToolPage
    icon="globe"
    name="Unicode 转换"
    description="字符串与 Unicode / HTML 实体 / CSS 实体互转，支持 Emoji，实时预览"
    @back="$emit('back')"
  >
    <!-- 输入与转换方向 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex items-center justify-between mb-2.5">
        <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入文本</h4>
        <button class="btn btn-ghost btn-xs" @click="clearAll" :disabled="!inputText && !outputText">清空</button>
      </div>
      <textarea
        v-model="inputText"
        class="textarea textarea-bordered w-full font-mono text-sm bg-base-200/60 min-h-[100px] resize-none"
        placeholder="输入文本，如：Hello 你好 👋"
      ></textarea>
      <div class="flex flex-wrap gap-2 mt-3">
        <button class="btn btn-primary btn-sm" @click="toUnicode">String → Unicode</button>
        <button class="btn btn-outline btn-sm" @click="fromUnicode">Unicode → String</button>
        <button class="btn btn-outline btn-sm" @click="toHtmlEntity">String → HTML 实体</button>
        <button class="btn btn-outline btn-sm" @click="fromHtmlEntity">HTML 实体 → String</button>
        <button class="btn btn-outline btn-sm" @click="toCssEntity">String → CSS 实体</button>
        <button class="btn btn-outline btn-sm" @click="fromCssEntity">CSS 实体 → String</button>
      </div>
    </div>

    <!-- 输出 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex items-center justify-between mb-2.5">
        <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 输出</h4>
        <button class="btn btn-primary btn-xs" @click="copyResult" :disabled="!outputText"><SvgIcon name="copy" size="11" /> 复制</button>
      </div>
      <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-sm whitespace-pre-wrap break-all max-h-60 overflow-y-auto">{{ outputText || '结果将显示在这里...' }}</div>
    </div>

    <!-- 实时预览 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="sparkles" size="12" /> 实时预览</h4>
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1.5 block">Unicode → 文本</span>
          <textarea
            v-model="unicodeInput"
            class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[90px] resize-none"
            placeholder="输入 Unicode，如：\u4f60\u597d"
            @input="previewUnicode"
          ></textarea>
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1.5 block">HTML 实体 → 文本</span>
          <textarea
            v-model="htmlInput"
            class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[90px] resize-none"
            placeholder="输入 HTML 实体，如：&#20320;&#22909;"
            @input="previewHtml"
          ></textarea>
        </div>
      </div>
      <div v-if="unicodePreview || htmlPreview" class="mt-3 flex flex-col gap-2 p-3 bg-primary/5 border border-primary/20 rounded-lg">
        <div v-if="unicodePreview" class="flex items-center gap-3">
          <span class="text-xs font-semibold text-primary shrink-0 w-24">Unicode 预览</span>
          <span class="text-base text-base-content break-all">{{ unicodePreview }}</span>
        </div>
        <div v-if="htmlPreview" class="flex items-center gap-3">
          <span class="text-xs font-semibold text-primary shrink-0 w-24">HTML 实体预览</span>
          <span class="text-base text-base-content break-all" v-html="htmlPreview"></span>
        </div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

const inputText = ref('')
const outputText = ref('')
const unicodeInput = ref('')
const htmlInput = ref('')
const unicodePreview = ref('')
const htmlPreview = ref('')

// String → Unicode (\uXXXX)
function toUnicode() {
  if (!inputText.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  try {
    outputText.value = inputText.value
      .split('')
      .map(char => {
        const code = char.codePointAt(0) || 0
        if (code <= 0xFFFF) {
          return '\\u' + code.toString(16).padStart(4, '0')
        } else {
          // Surrogate pair for characters outside BMP
          return '\\u' + char.charCodeAt(0).toString(16).padStart(4, '0') +
                 '\\u' + char.charCodeAt(1).toString(16).padStart(4, '0')
        }
      })
      .join('')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    outputText.value = ''
  }
}

// Unicode → String（支持 \uXXXX 代理对 与 \u{1F600} 大括号形式，1-6 位）
const UNICODE_RE = /\\u\{([0-9a-fA-F]{1,6})\}|\\u([0-9a-fA-F]{4})/g

function unicodeToText(str: string): string {
  return str.replace(UNICODE_RE, (_, brace, plain) => {
    const code = parseInt(brace ?? plain, 16)
    return fromCodePointSafe(code)
  })
}

// Unicode → String
function fromUnicode() {
  if (!inputText.value.trim()) {
    toast.warning('请输入 Unicode 字符串')
    return
  }
  try {
    outputText.value = unicodeToText(inputText.value)
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    outputText.value = ''
  }
}

// String → HTML Entity
function toHtmlEntity() {
  if (!inputText.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  try {
    outputText.value = inputText.value
      .split('')
      .map(char => {
        const code = char.codePointAt(0) || 0
        return '&#' + code + ';'
      })
      .join('')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    outputText.value = ''
  }
}

// HTML 实体 → String（支持十进制 &#123; 与十六进制 &#x1F600; / &#X1F600;，大小写不敏感）
const HTML_ENTITY_RE = /&#(x[0-9a-fA-F]{1,6}|X[0-9a-fA-F]{1,6}|[0-9]{1,7});/g

// 码点转字符：超过 Unicode 上限（0x10FFFF）时用替换字符 U+FFFD，避免 RangeError 中断
function fromCodePointSafe(code: number): string {
  return code >= 0 && code <= 0x10FFFF ? String.fromCodePoint(code) : '\uFFFD'
}

function htmlEntityToText(str: string): string {
  return str.replace(HTML_ENTITY_RE, (_, num) => {
    const code = (num[0] === 'x' || num[0] === 'X') ? parseInt(num.slice(1), 16) : parseInt(num, 10)
    return fromCodePointSafe(code)
  })
}

// HTML Entity → String
function fromHtmlEntity() {
  if (!inputText.value.trim()) {
    toast.warning('请输入 HTML 实体字符串')
    return
  }
  try {
    outputText.value = htmlEntityToText(inputText.value)
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    outputText.value = ''
  }
}

// String → CSS Entity
function toCssEntity() {
  if (!inputText.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  try {
    outputText.value = inputText.value
      .split('')
      .map(char => {
        const code = char.codePointAt(0) || 0
        return '\\' + code.toString(16).padStart(4, '0')
      })
      .join('')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    outputText.value = ''
  }
}

// CSS 实体 → String（CSS 转义支持 1-6 位十六进制，贪婪匹配）
const CSS_ENTITY_RE = /\\[0-9a-fA-F]{1,6}/g

function cssEntityToText(str: string): string {
  return str.replace(CSS_ENTITY_RE, m => fromCodePointSafe(parseInt(m.slice(1), 16)))
}

// CSS Entity → String
function fromCssEntity() {
  if (!inputText.value.trim()) {
    toast.warning('请输入 CSS 实体字符串')
    return
  }
  try {
    outputText.value = cssEntityToText(inputText.value)
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    outputText.value = ''
  }
}

// Live preview
function previewUnicode() {
  if (!unicodeInput.value.trim()) {
    unicodePreview.value = ''
    return
  }
  try {
    unicodePreview.value = unicodeToText(unicodeInput.value)
  } catch {
    unicodePreview.value = ''
  }
}

function previewHtml() {
  if (!htmlInput.value.trim()) {
    htmlPreview.value = ''
    return
  }
  try {
    htmlPreview.value = htmlEntityToText(htmlInput.value)
  } catch {
    htmlPreview.value = ''
  }
}

function copyResult() {
  if (!outputText.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(outputText.value, toast)
}

function clearAll() {
  inputText.value = ''
  outputText.value = ''
  unicodeInput.value = ''
  htmlInput.value = ''
  unicodePreview.value = ''
  htmlPreview.value = ''
}
</script>
