<template>
  <ToolPage
    icon="camera"
    name="二维码工具"
    description="文本 / URL 生成二维码，支持图片解析二维码，可下载 PNG"
    @back="$emit('back')"
  >
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- 生成二维码 -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
        <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="camera" size="12" /> 生成二维码</h4>
        <textarea
          v-model="qrInput"
          class="textarea textarea-bordered w-full font-mono text-sm bg-base-200/60 min-h-[90px] resize-none"
          placeholder="输入文本、URL 等内容..."
        ></textarea>
        <div class="flex flex-wrap gap-2 mt-3 items-end">
          <div>
            <span class="text-[11px] font-medium text-base-content/50 mb-1 block">尺寸</span>
            <input v-model.number="qrSize" type="number" class="input input-bordered input-sm font-mono w-20 bg-base-200/60" min="100" max="1000" step="50" />
          </div>
          <div>
            <span class="text-[11px] font-medium text-base-content/50 mb-1 block">容错级别</span>
            <select v-model="qrErrorLevel" class="select select-bordered select-sm">
              <option value="L">L (7%)</option>
              <option value="M">M (15%)</option>
              <option value="Q">Q (25%)</option>
              <option value="H">H (30%)</option>
            </select>
          </div>
          <button class="btn btn-primary btn-sm" @click="generateQR" :disabled="qrGenerating">
            {{ qrGenerating ? '生成中...' : '生成' }}
          </button>
        </div>
        <div v-if="qrGenerating" class="flex items-center gap-3 mt-4 p-3 text-sm text-base-content/60">
          <span class="loading loading-spinner loading-sm text-primary"></span>
          <span>正在生成二维码...</span>
        </div>
        <div v-if="qrDataUrl" class="mt-4 flex flex-col items-center gap-3">
          <img :src="qrDataUrl" alt="QR Code" class="border border-base-content/10 rounded-xl max-w-[280px] bg-white p-2" />
          <button class="btn btn-outline btn-sm" @click="downloadQR"><SvgIcon name="download" size="12" /> 下载 PNG</button>
        </div>
      </div>

      <!-- 解析二维码 -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
        <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="search" size="12" /> 解析二维码</h4>
        <div class="flex flex-wrap gap-2 mb-3">
          <input
            type="file"
            accept="image/*"
            class="file-input file-input-bordered file-input-sm w-full max-w-xs"
            @change="handleFileUpload"
            :disabled="parseLoading"
          />
        </div>
        <div v-if="parseLoading" class="flex items-center gap-3 mt-4 p-3 text-sm text-base-content/60">
          <span class="loading loading-spinner loading-sm text-primary"></span>
          <span>正在解析二维码...</span>
        </div>
        <div v-if="parseResult" class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-sm whitespace-pre-wrap break-all max-h-60 overflow-y-auto mt-2">
          {{ parseResult }}
          <div class="flex flex-wrap gap-2 mt-2">
            <button class="btn btn-primary btn-xs" @click="doCopy(parseResult)"><SvgIcon name="copy" size="11" /> 复制</button>
          </div>
        </div>
        <div v-if="parseError" class="p-3 bg-error/10 border border-error/30 rounded-lg font-mono text-sm whitespace-pre-wrap break-all max-h-60 overflow-y-auto mt-2 text-error">{{ parseError }}</div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import qr from 'qrcode-generator'
import jsQR from 'jsqr'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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
  if (!qrDataUrl.value) {return}
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
  if (!file) {return}

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
