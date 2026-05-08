<template>
  <div class="tool-panel">
    <h3>🌐 Unicode 转换</h3>

    <!-- Input -->
    <div class="tool-section">
      <label class="tool-label">输入</label>
      <textarea
        v-model="inputText"
        class="tool-textarea"
        placeholder="输入文本，如：Hello 你好 👋"
      ></textarea>
    </div>

    <!-- Action Buttons -->
    <div class="tool-row">
      <button class="tool-btn primary" @click="toUnicode">String → Unicode</button>
      <button class="tool-btn" @click="fromUnicode">Unicode → String</button>
      <button class="tool-btn" @click="toHtmlEntity">String → HTML实体</button>
      <button class="tool-btn" @click="fromHtmlEntity">HTML实体 → String</button>
      <button class="tool-btn" @click="toCssEntity">String → CSS实体</button>
      <button class="tool-btn" @click="fromCssEntity">CSS实体 → String</button>
    </div>

    <div class="tool-row">
      <button class="tool-btn" @click="copyResult">复制结果</button>
      <button class="tool-btn" @click="clearAll">清空</button>
    </div>

    <!-- Output -->
    <div class="tool-section">
      <label class="tool-label">输出</label>
      <div class="tool-result">{{ outputText || '结果将显示在这里...' }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Live Preview -->
    <div class="tool-section">
      <h4>实时预览</h4>
      <div class="tool-row">
        <div style="flex: 1;">
          <label class="tool-label">Unicode → 文本</label>
          <textarea
            v-model="unicodeInput"
            class="tool-textarea"
            placeholder="输入 Unicode，如：\u4f60\u597d"
            @input="previewUnicode"
          ></textarea>
        </div>
        <div style="flex: 1;">
          <label class="tool-label">HTML实体 → 文本</label>
          <textarea
            v-model="htmlInput"
            class="tool-textarea"
            placeholder="输入 HTML 实体，如：&#20320;&#22909;"
            @input="previewHtml"
          ></textarea>
        </div>
      </div>
      <div class="preview-results" v-if="unicodePreview || htmlPreview">
        <div v-if="unicodePreview" class="preview-item">
          <span class="preview-label">Unicode 预览:</span>
          <span class="preview-value">{{ unicodePreview }}</span>
        </div>
        <div v-if="htmlPreview" class="preview-item">
          <span class="preview-label">HTML实体 预览:</span>
          <span class="preview-value" v-html="htmlPreview"></span>
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

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: oklch(var(--bc));
  margin: 0 0 20px 0;
}

.preview-results {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-item {
  display: flex;
  gap: 8px;
  align-items: center;
}

.preview-label {
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--p));
  min-width: 100px;
}

.preview-value {
  font-size: 16px;
  color: oklch(var(--bc));
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: oklch(var(--bc)); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: oklch(var(--p)); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); outline: none; }
.tool-input:focus { border-color: oklch(var(--p)); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: oklch(var(--b1)); color: oklch(var(--bc)); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: oklch(var(--p)); color: oklch(var(--p)); }
.tool-btn.primary { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: oklch(var(--b2)); border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: oklch(var(--bc)); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: oklch(var(--bc) / 0.6); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); outline: none; }
.tool-select:focus { border-color: oklch(var(--p)); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: oklch(var(--bc)); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid oklch(var(--bc) / 0.1); margin: 20px 0; }
</style>
