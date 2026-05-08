<template>
  <div class="qrcode-tool">
    <h3>📱 二维码工具</h3>

    <!-- Generate QR -->
    <div class="tool-section">
      <h4>生成二维码</h4>
      <textarea
        v-model="qrInput"
        class="tool-textarea"
        placeholder="输入文本、URL 等内容..."
        rows="3"
      ></textarea>
      <div class="tool-row" style="margin-top: 12px">
        <div>
          <label class="tool-label">尺寸</label>
          <input v-model.number="qrSize" type="number" class="tool-input mono" style="width: 80px" min="100" max="1000" step="50" />
        </div>
        <div>
          <label class="tool-label">容错级别</label>
          <select v-model="qrErrorLevel" class="tool-select">
            <option value="L">L (7%)</option>
            <option value="M" selected>M (15%)</option>
            <option value="Q">Q (25%)</option>
            <option value="H">H (30%)</option>
          </select>
        </div>
        <button class="tool-btn primary" @click="generateQR" :disabled="qrGenerating" style="align-self: flex-end">
          {{ qrGenerating ? '生成中...' : '生成' }}
        </button>
      </div>

      <div v-if="qrGenerating" class="loading-box">
        <div class="spinner"></div>
        <span>正在生成二维码...</span>
      </div>

      <div v-if="qrDataUrl" class="qr-output">
        <img :src="qrDataUrl" alt="QR Code" class="qr-image" />
        <button class="tool-btn" @click="downloadQR">⬇️ 下载 PNG</button>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- Parse QR -->
    <div class="tool-section">
      <h4>解析二维码</h4>
      <div class="tool-row">
        <input
          type="file"
          accept="image/*"
          class="tool-input"
          @change="handleFileUpload"
          :disabled="parseLoading"
        />
      </div>
      <div v-if="parseLoading" class="loading-box">
        <div class="spinner"></div>
        <span>正在解析二维码...</span>
      </div>
      <div v-if="parseResult" class="tool-result">
        {{ parseResult }}
        <div class="tool-row" style="margin-top: 8px">
          <button class="tool-btn" @click="doCopy(parseResult)">📋 复制</button>
        </div>
      </div>
      <div v-if="parseError" class="tool-result error">{{ parseError }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import qr from 'qrcode-generator'
import jsQR from 'jsqr'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

/* Generate QR */
const qrInput = ref('https://example.com')
const qrSize = ref(256)
const qrErrorLevel = ref<'L' | 'M' | 'Q' | 'H'>('M')
const qrDataUrl = ref('')
const qrGenerating = ref(false)

const errorCorrectionMap: Record<string, 'L' | 'M' | 'Q' | 'H'> = { L: 'L', M: 'M', Q: 'Q', H: 'H' }

function generateQR() {
  if (!qrInput.value.trim()) { toast.error('请输入内容'); return }
  qrGenerating.value = true
  try {
    const typeNumber = 0
    const ecLevel = errorCorrectionMap[qrErrorLevel.value]
    const qrcode = qr(typeNumber, ecLevel)
    qrcode.addData(qrInput.value.trim())
    qrcode.make()

    // Create canvas and render QR code
    const cellCount = qrcode.getModuleCount()
    const cellSize = Math.max(1, Math.floor(qrSize.value / cellCount))
    const margin = cellSize * 2
    const canvasSize = cellCount * cellSize + margin * 2

    const canvas = document.createElement('canvas')
    canvas.width = canvasSize
    canvas.height = canvasSize
    const ctx = canvas.getContext('2d')
    if (!ctx) { toast.error('Canvas 不可用'); return }

    // Fill background
    ctx.fillStyle = '#FFFFFF'
    ctx.fillRect(0, 0, canvasSize, canvasSize)

    // Draw modules
    ctx.fillStyle = '#000000'
    for (let row = 0; row < cellCount; row++) {
      for (let col = 0; col < cellCount; col++) {
        if (qrcode.isDark(row, col)) {
          ctx.fillRect(margin + col * cellSize, margin + row * cellSize, cellSize, cellSize)
        }
      }
    }

    qrDataUrl.value = canvas.toDataURL('image/png')
  } catch (e: any) {
    toast.error(`生成失败: ${e.message}`)
    qrDataUrl.value = ''
  } finally {
    qrGenerating.value = false
  }
}

function downloadQR() {
  if (!qrDataUrl.value) return
  const a = document.createElement('a')
  a.href = qrDataUrl.value
  a.download = 'qrcode.png'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
}

/* Parse QR using jsQR */
const parseResult = ref('')
const parseError = ref('')
const parseLoading = ref(false)

function doCopy(text: string) {
  copyText(text, toast)
}

async function handleFileUpload(event: Event) {
  parseResult.value = ''
  parseError.value = ''

  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  parseLoading.value = true
  try {
    const img = new Image()
    const objectUrl = URL.createObjectURL(file)

    await new Promise<void>((resolve, reject) => {
      img.onload = () => resolve()
      img.onerror = () => reject(new Error('图片加载失败'))
      img.src = objectUrl
    })

    // Draw image to canvas to get pixel data
    const canvas = document.createElement('canvas')
    canvas.width = img.width
    canvas.height = img.height
    const ctx = canvas.getContext('2d')
    if (!ctx) { throw new Error('Canvas 不可用') }
    ctx.drawImage(img, 0, 0)

    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
    const code = jsQR(imageData.data, imageData.width, imageData.height)

    URL.revokeObjectURL(objectUrl)

    if (code && code.data) {
      parseResult.value = code.data
    } else {
      parseError.value = '未检测到二维码内容，请尝试其他图片'
    }
  } catch (e: any) {
    parseError.value = `解析失败: ${e.message || '未知错误'}`
    toast.error('二维码解析失败')
  } finally {
    parseLoading.value = false
    target.value = ''
  }
}
</script>

<style scoped>
.qrcode-tool { max-width: 700px; }
.qrcode-tool h3 { font-size: 18px; font-weight: 700; color: var(--color-base-content); margin: 0 0 20px 0; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--color-base-content); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--color-primary); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-input:focus { border-color: var(--color-primary); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; align-items: flex-end; }
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
.tool-result.error { border-color: #e74c3c; color: #e74c3c; }
.tool-label { font-size: 12px; font-weight: 500; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-select:focus { border-color: var(--color-primary); }
.tool-divider { border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); margin: 20px 0; }
.mono { font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; }
.qr-output { margin-top: 16px; display: flex; flex-direction: column; align-items: center; gap: 12px; }
.qr-image { border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; max-width: 300px; background: white; padding: 8px; }
.loading-box { display: flex; align-items: center; gap: 12px; margin-top: 16px; padding: 12px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); font-size: 13px; }
.spinner { width: 20px; height: 20px; border: 2px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-top-color: var(--color-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
