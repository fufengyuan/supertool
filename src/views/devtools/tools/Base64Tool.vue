<template>
  <div class="tool-panel">
    <h3>📝 BASE64 编码/解码</h3>

    <!-- Mode Toggle -->
    <div class="tool-row">
      <div>
        <label class="tool-label">模式</label>
        <div class="tool-btn-group">
          <button
            class="tool-btn"
            :class="{ active: mode === 'encode' }"
            @click="mode = 'encode'"
          >编码</button>
          <button
            class="tool-btn"
            :class="{ active: mode === 'decode' }"
            @click="mode = 'decode'"
          >解码</button>
        </div>
      </div>
    </div>

    <!-- Text Input -->
    <div class="tool-section">
      <label class="tool-label">输入</label>
      <textarea
        v-model="inputText"
        class="tool-textarea"
        :placeholder="mode === 'encode' ? '输入要编码的文本...' : '输入要解码的 Base64...'"
      ></textarea>
    </div>

    <div class="tool-row">
      <button class="tool-btn primary" @click="process">
        {{ mode === 'encode' ? '编码' : '解码' }}
      </button>
      <button class="tool-btn" @click="copyResult">复制结果</button>
      <button class="tool-btn" @click="clearAll">清空</button>
    </div>

    <!-- Output -->
    <div class="tool-section">
      <label class="tool-label">输出</label>
      <div class="tool-result">{{ outputText || '结果将显示在这里...' }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- File Encoding -->
    <div class="tool-section">
      <h4>文件编码</h4>
      <input type="file" ref="fileInput" @change="handleFile" class="tool-file-input" />
      <div v-if="fileBase64" class="file-result">
        <div class="file-info">
          <span class="file-name">{{ fileName }}</span>
          <span class="file-size">{{ fileSize }}</span>
        </div>
        <div class="tool-result file-base64">{{ fileBase64 }}</div>
        <div class="tool-row" style="margin-top: 8px;">
          <button class="tool-btn" @click="copyFileBase64">复制 Base64</button>
          <button class="tool-btn" @click="downloadBase64File">下载文件</button>
        </div>
      </div>
      <div v-if="fileProcessing" class="loading-text">正在处理文件...</div>
    </div>

    <!-- File Decode (Base64 → File) -->
    <div class="tool-section">
      <h4>文件解码</h4>
      <label class="tool-label">输入 Base64 字符串</label>
      <textarea
        v-model="fileDecodeInput"
        class="tool-textarea"
        placeholder="粘贴 Base64 字符串..."
      ></textarea>
      <div class="tool-row" style="margin-top: 8px;">
        <button class="tool-btn primary" @click="decodeFile">解码并下载</button>
      </div>
      <div class="tool-row" style="margin-top: 4px;">
        <label class="tool-label">文件名</label>
        <input v-model="decodeFileName" class="tool-input" placeholder="decoded_file.txt" style="max-width: 300px;" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { copyText, downloadFile, readFileAsArrayBuffer } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const mode = ref<'encode' | 'decode'>('encode')
const inputText = ref('')
const outputText = ref('')

// File encoding
const fileInput = ref<HTMLInputElement | null>(null)
const fileBase64 = ref('')
const fileName = ref('')
const fileSize = ref('')
const fileProcessing = ref(false)

// File decoding
const fileDecodeInput = ref('')
const decodeFileName = ref('decoded_file')

function process() {
  if (!inputText.value.trim()) {
    toast.warning('请输入文本')
    return
  }

  try {
    if (mode.value === 'encode') {
      outputText.value = btoa(unescape(encodeURIComponent(inputText.value)))
    } else {
      outputText.value = decodeURIComponent(escape(atob(inputText.value.trim())))
    }
  } catch (e: any) {
    toast.error(`${mode.value === 'encode' ? '编码' : '解码'}失败: ${e.message}`)
    outputText.value = ''
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
}

async function handleFile(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  fileProcessing.value = true
  fileBase64.value = ''
  fileName.value = file.name
  fileSize.value = formatFileSize(file.size)

  try {
    const buffer = await readFileAsArrayBuffer(file)
    const binary = new Uint8Array(buffer)
    let binaryString = ''
    for (let i = 0; i < binary.length; i++) {
      binaryString += String.fromCharCode(binary[i])
    }
    fileBase64.value = btoa(binaryString)
  } catch (e: any) {
    toast.error(`文件读取失败: ${e.message}`)
  } finally {
    fileProcessing.value = false
    input.value = ''
  }
}

function copyFileBase64() {
  if (!fileBase64.value) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(fileBase64.value, toast)
}

function downloadBase64File() {
  if (!fileBase64.value) {
    toast.warning('没有可下载的内容')
    return
  }
  try {
    const binary = atob(fileBase64.value)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i)
    }
    const blob = new Blob([bytes])
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = fileName.value || 'download'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    toast.error(`下载失败: ${e.message}`)
  }
}

function decodeFile() {
  if (!fileDecodeInput.value.trim()) {
    toast.warning('请输入 Base64 字符串')
    return
  }

  try {
    const binary = atob(fileDecodeInput.value.trim())
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i)
    }
    const blob = new Blob([bytes])
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = decodeFileName.value || 'decoded_file'
    a.click()
    URL.revokeObjectURL(url)
    toast.success('文件已下载')
  } catch (e: any) {
    toast.error(`解码失败: ${e.message}`)
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}
</script>


