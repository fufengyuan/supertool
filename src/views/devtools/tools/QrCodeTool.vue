<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5">📱 二维码工具</h3>

    <!-- Generate QR -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">生成二维码</h4>
      <textarea
        v-model="qrInput"
        class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]"
        placeholder="输入文本、URL 等内容..."
        rows="3"
      ></textarea>
      <div class="flex flex-wrap gap-2.5 mb-3 mt-3 items-end">
        <div>
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">尺寸</span>
          <input v-model.number="qrSize" type="number" class="input input-bordered font-mono w-20" min="100" max="1000" step="50" />
        </div>
        <div>
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">容错级别</span>
          <select v-model="qrErrorLevel" class="select select-bordered">
            <option value="L">L (7%)</option>
            <option value="M" selected>M (15%)</option>
            <option value="Q">Q (25%)</option>
            <option value="H">H (30%)</option>
          </select>
        </div>
        <button class="btn btn-primary self-end" @click="generateQR" :disabled="qrGenerating">
          {{ qrGenerating ? '生成中...' : '生成' }}
        </button>
      </div>

      <div v-if="qrGenerating" class="flex items-center gap-3 mt-4 p-3 opacity-60 text-sm">
        <div class="w-5 h-5 border-2 border-base-content/10 border-t-primary rounded-full animate-spin"></div>
        <span>正在生成二维码...</span>
      </div>

      <div v-if="qrDataUrl" class="mt-4 flex flex-col items-center gap-3">
        <img :src="qrDataUrl" alt="QR Code" class="border border-base-content/10 rounded-box max-w-[300px] bg-white p-2" />
        <button class="btn btn-ghost" @click="downloadQR"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><line x1="12" y1="5" x2="12" y2="19"/><polyline points="19 12 12 19 5 12"/></svg> 下载 PNG</button>
      </div>
    </div>

    <hr class="border-base-content/10 my-5" />

    <!-- Parse QR -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">解析二维码</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <input
          type="file"
          accept="image/*"
          class="input input-bordered"
          @change="handleFileUpload"
          :disabled="parseLoading"
        />
      </div>
      <div v-if="parseLoading" class="flex items-center gap-3 mt-4 p-3 opacity-60 text-sm">
        <div class="w-5 h-5 border-2 border-base-content/10 border-t-primary rounded-full animate-spin"></div>
        <span>正在解析二维码...</span>
      </div>
      <div v-if="parseResult" class="bg-base-200 border border-base-content/10 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all max-h-72 overflow-y-auto mt-2">
        {{ parseResult }}
        <div class="flex flex-wrap gap-2.5 mb-3 mt-2">
          <button class="btn btn-ghost" @click="doCopy(parseResult)"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 复制</button>
        </div>
      </div>
      <div v-if="parseError" class="bg-base-200 border border-error/30 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all max-h-72 overflow-y-auto mt-2 text-error">{{ parseError }}</div>
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
