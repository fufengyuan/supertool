<template>
  <div>
    <h3 class="text-lg font-bold text-base-content mb-5"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg> Unicode 转换</h3>

    <!-- Input -->
    <div class="mb-5">
      <span class="label-text text-xs font-medium opacity-60 mb-1 block">输入</span>
      <textarea
        v-model="inputText"
        class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]"
        placeholder="输入文本，如：Hello 你好 👋"
      ></textarea>
    </div>

    <!-- Action Buttons -->
    <div class="flex flex-wrap gap-2.5 mb-3">
      <button class="btn btn-primary" @click="toUnicode">String → Unicode</button>
      <button class="btn btn-ghost" @click="fromUnicode">Unicode → String</button>
      <button class="btn btn-ghost" @click="toHtmlEntity">String → HTML实体</button>
      <button class="btn btn-ghost" @click="fromHtmlEntity">HTML实体 → String</button>
      <button class="btn btn-ghost" @click="toCssEntity">String → CSS实体</button>
      <button class="btn btn-ghost" @click="fromCssEntity">CSS实体 → String</button>
    </div>

    <div class="flex flex-wrap gap-2.5 mb-3">
      <button class="btn btn-ghost" @click="copyResult">复制结果</button>
      <button class="btn btn-ghost" @click="clearAll">清空</button>
    </div>

    <!-- Output -->
    <div class="mb-5">
      <span class="label-text text-xs font-medium opacity-60 mb-1 block">输出</span>
      <div class="bg-base-200 border border-base-content/10 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all max-h-72 overflow-y-auto">{{ outputText || '结果将显示在这里...' }}</div>
    </div>

    <hr class="border-base-content/10 my-5" />

    <!-- Live Preview -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">实时预览</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <div class="flex-1">
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">Unicode → 文本</span>
          <textarea
            v-model="unicodeInput"
            class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]"
            placeholder="输入 Unicode，如：\u4f60\u597d"
            @input="previewUnicode"
          ></textarea>
        </div>
        <div class="flex-1">
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">HTML实体 → 文本</span>
          <textarea
            v-model="htmlInput"
            class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]"
            placeholder="输入 HTML 实体，如：&#20320;&#22909;"
            @input="previewHtml"
          ></textarea>
        </div>
      </div>
      <div v-if="unicodePreview || htmlPreview" class="mt-3 flex flex-col gap-2">
        <div v-if="unicodePreview" class="flex gap-2 items-center">
          <span class="text-xs font-semibold text-primary min-w-[100px]">Unicode 预览:</span>
          <span class="text-base text-base-content">{{ unicodePreview }}</span>
        </div>
        <div v-if="htmlPreview" class="flex gap-2 items-center">
          <span class="text-xs font-semibold text-primary min-w-[100px]">HTML实体 预览:</span>
          <span class="text-base text-base-content" v-html="htmlPreview"></span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

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

// Unicode → String
function fromUnicode() {
  if (!inputText.value.trim()) {
    toast.warning('请输入 Unicode 字符串')
    return
  }
  try {
    outputText.value = inputText.value.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => {
      return String.fromCharCode(parseInt(hex, 16))
    })
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

// HTML Entity → String
function fromHtmlEntity() {
  if (!inputText.value.trim()) {
    toast.warning('请输入 HTML 实体字符串')
    return
  }
  try {
    outputText.value = inputText.value.replace(/&#(\d+);/g, (_, dec) => {
      return String.fromCodePoint(parseInt(dec, 10))
    })
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

// CSS Entity → String
function fromCssEntity() {
  if (!inputText.value.trim()) {
    toast.warning('请输入 CSS 实体字符串')
    return
  }
  try {
    outputText.value = inputText.value.replace(/\\([0-9a-fA-F]{4})/g, (_, hex) => {
      return String.fromCharCode(parseInt(hex, 16))
    })
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
    unicodePreview.value = unicodeInput.value.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => {
      return String.fromCharCode(parseInt(hex, 16))
    })
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
    htmlPreview.value = htmlInput.value.replace(/&#(\d+);/g, (_, dec) => {
      return String.fromCodePoint(parseInt(dec, 10))
    })
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
