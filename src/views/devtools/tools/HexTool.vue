<template>
  <div class="tool-panel">
    <h3>🔢 Hex / Base64 转换</h3>

    <!-- String ↔ Hex -->
    <div class="tool-section">
      <h4>String ↔ Hex</h4>
      <label class="tool-label">输入</label>
      <textarea
        v-model="stringHexInput"
        class="tool-textarea"
        placeholder="输入字符串或 Hex..."
      ></textarea>

      <div class="tool-row" style="margin-top: 12px;">
        <button class="tool-btn primary" @click="stringToHex">String → Hex</button>
        <button class="tool-btn" @click="hexToString">Hex → String</button>
        <button class="tool-btn" @click="copyHexResult">复制结果</button>
        <button class="tool-btn" @click="clearStringHex">清空</button>
      </div>

      <label class="tool-label">输出</label>
      <div class="tool-result">{{ stringHexOutput || '结果将显示在这里...' }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Hex ↔ Base64 -->
    <div class="tool-section">
      <h4>Hex ↔ Base64</h4>
      <label class="tool-label">输入</label>
      <textarea
        v-model="hexBase64Input"
        class="tool-textarea"
        placeholder="输入 Hex 或 Base64..."
      ></textarea>

      <div class="tool-row" style="margin-top: 12px;">
        <button class="tool-btn primary" @click="hexToBase64">Hex → Base64</button>
        <button class="tool-btn" @click="base64ToHex">Base64 → Hex</button>
        <button class="tool-btn" @click="copyBase64Result">复制结果</button>
        <button class="tool-btn" @click="clearHexBase64">清空</button>
      </div>

      <label class="tool-label">输出</label>
      <div class="tool-result">{{ hexBase64Output || '结果将显示在这里...' }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Quick Convert -->
    <div class="tool-section">
      <h4>快速转换</h4>
      <label class="tool-label">输入文本</label>
      <textarea
        v-model="quickInput"
        class="tool-textarea"
        placeholder="输入任意文本..."
      ></textarea>

      <div class="tool-row" style="margin-top: 12px;">
        <button class="tool-btn primary" @click="quickConvert">转换</button>
        <button class="tool-btn" @click="clearQuick">清空</button>
      </div>

      <div v-if="quickResults" class="quick-results">
        <div class="quick-result-item">
          <span class="quick-label">Hex:</span>
          <span class="quick-value">{{ quickResults.hex }}</span>
          <button class="tool-btn copy-btn" @click="doCopy(quickResults.hex)">📋</button>
        </div>
        <div class="quick-result-item">
          <span class="quick-label">Base64:</span>
          <span class="quick-value">{{ quickResults.base64 }}</span>
          <button class="tool-btn copy-btn" @click="doCopy(quickResults.base64)">📋</button>
        </div>
        <div class="quick-result-item">
          <span class="quick-label">Unicode:</span>
          <span class="quick-value">{{ quickResults.unicode }}</span>
          <button class="tool-btn copy-btn" @click="doCopy(quickResults.unicode)">📋</button>
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

// String ↔ Hex
const stringHexInput = ref('')
const stringHexOutput = ref('')

// Hex ↔ Base64
const hexBase64Input = ref('')
const hexBase64Output = ref('')

// Quick Convert
const quickInput = ref('')
const quickResults = ref<{ hex: string; base64: string; unicode: string } | null>(null)

// String → Hex
function stringToHex() {
  if (!stringHexInput.value) {
    toast.warning('请输入文本')
    return
  }
  try {
    stringHexOutput.value = stringHexInput.value
      .split('')
      .map(c => c.charCodeAt(0).toString(16).padStart(2, '0'))
      .join(' ')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    stringHexOutput.value = ''
  }
}

// Hex → String
function hexToString() {
  if (!stringHexInput.value.trim()) {
    toast.warning('请输入 Hex')
    return
  }
  try {
    const hex = stringHexInput.value.replace(/\s+/g, '')
    if (hex.length % 2 !== 0) {
      toast.error('Hex 长度必须为偶数')
      stringHexOutput.value = ''
      return
    }
    if (!/^[0-9a-fA-F]+$/.test(hex)) {
      toast.error('无效的 Hex 字符串')
      stringHexOutput.value = ''
      return
    }
    stringHexOutput.value = hex
      .match(/.{2}/g)!
      .map(h => String.fromCharCode(parseInt(h, 16)))
      .join('')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    stringHexOutput.value = ''
  }
}

function copyHexResult() {
  if (!stringHexOutput.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(stringHexOutput.value, toast)
}

function clearStringHex() {
  stringHexInput.value = ''
  stringHexOutput.value = ''
}

// Hex → Base64
function hexToBase64() {
  if (!hexBase64Input.value.trim()) {
    toast.warning('请输入 Hex')
    return
  }
  try {
    const hex = hexBase64Input.value.replace(/\s+/g, '')
    if (hex.length % 2 !== 0) {
      toast.error('Hex 长度必须为偶数')
      hexBase64Output.value = ''
      return
    }
    if (!/^[0-9a-fA-F]+$/.test(hex)) {
      toast.error('无效的 Hex 字符串')
      hexBase64Output.value = ''
      return
    }
    const bytes = hex.match(/.{2}/g)!.map(h => parseInt(h, 16))
    let binary = ''
    for (const b of bytes) {
      binary += String.fromCharCode(b)
    }
    hexBase64Output.value = btoa(binary)
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    hexBase64Output.value = ''
  }
}

// Base64 → Hex
function base64ToHex() {
  if (!hexBase64Input.value.trim()) {
    toast.warning('请输入 Base64')
    return
  }
  try {
    const binary = atob(hexBase64Input.value.trim())
    hexBase64Output.value = binary
      .split('')
      .map(c => c.charCodeAt(0).toString(16).padStart(2, '0'))
      .join(' ')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    hexBase64Output.value = ''
  }
}

function copyBase64Result() {
  if (!hexBase64Output.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(hexBase64Output.value, toast)
}

function clearHexBase64() {
  hexBase64Input.value = ''
  hexBase64Output.value = ''
}

// Quick Convert
function quickConvert() {
  if (!quickInput.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  try {
    const text = quickInput.value
    const hex = text
      .split('')
      .map(c => c.charCodeAt(0).toString(16).padStart(2, '0'))
      .join(' ')

    const binary = text
      .split('')
      .map(c => String.fromCharCode(c.charCodeAt(0)))
      .join('')
    const base64 = btoa(unescape(encodeURIComponent(text)))

    const unicode = text
      .split('')
      .map(c => {
        const code = c.codePointAt(0) || 0
        return '\\u' + code.toString(16).padStart(4, '0')
      })
      .join('')

    quickResults.value = { hex, base64, unicode }
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    quickResults.value = null
  }
}

function doCopy(text: string) {
  if (!text) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(text, toast)
}

function clearQuick() {
  quickInput.value = ''
  quickResults.value = null
}
</script>


