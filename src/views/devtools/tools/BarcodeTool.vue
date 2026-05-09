<template>
  <div class="barcode-tool">
    <h3 class="text-lg font-bold text-base-content mb-5"><SvgIcon name="barChart" size="14" class="align-text-bottom" /> 条形码生成</h3>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">条形码设置</h4>
      <div class="flex gap-2.5 mb-3 flex-wrap items-center">
        <input
          v-model="barcodeInput"
          type="text"
          class="tool-input mono"
          placeholder="输入条形码内容..."
          @input="debouncedGenerate"
        />
        <select v-model="barcodeFormat" class="select select-bordered select-sm" @change="generateBarcode">
          <option value="CODE128">CODE128</option>
          <option value="CODE39">CODE39</option>
          <option value="EAN13">EAN13</option>
          <option value="EAN8">EAN8</option>
          <option value="UPC">UPC</option>
          <option value="ITF14">ITF14</option>
          <option value="MSI">MSI</option>
          <option value="pharmacode">Pharmacode</option>
          <option value="codabar">Codabar</option>
        </select>
      </div>

      <div class="flex gap-2.5 mb-3 flex-wrap items-center">
        <div>
          <label class="text-xs font-medium text-base-content/60 mb-1 block">宽度</label>
          <input v-model.number="barcodeWidth" type="number" class="tool-input mono" style="width: 80px" min="1" max="4" />
        </div>
        <div>
          <label class="text-xs font-medium text-base-content/60 mb-1 block">高度</label>
          <input v-model.number="barcodeHeight" type="number" class="tool-input mono" style="width: 80px" min="20" max="200" />
        </div>
        <label class="flex items-center gap-1.5 text-xs text-base-content cursor-pointer">
          <input type="checkbox" v-model="barcodeShowText" />
          显示文字
        </label>
        <button class="btn btn-primary btn-sm" @click="generateBarcode">生成</button>
        <button class="btn btn-ghost btn-sm" @click="downloadBarcode"><SvgIcon name="arrowDown" size="14" class="align-text-bottom" /> 下载 PNG</button>
      </div>

      <!-- Barcode Canvas -->
      <div class="barcode-output">
        <canvas ref="barcodeCanvas" class="barcode-canvas"></canvas>
      </div>
      <div v-if="barcodeError" class="tool-result error">{{ barcodeError }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, onMounted, nextTick } from 'vue'
import JsBarcode from 'jsbarcode'
import { copyText, downloadFile } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const barcodeInput = ref('123456789012')
const barcodeFormat = ref('CODE128')
const barcodeWidth = ref(2)
const barcodeHeight = ref(100)
const barcodeShowText = ref(true)
const barcodeError = ref('')
const barcodeCanvas = ref<HTMLCanvasElement | null>(null)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

function debouncedGenerate() {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => generateBarcode(), 300)
}

function generateBarcode() {
  barcodeError.value = ''
  if (!barcodeInput.value.trim()) return

  nextTick(() => {
    if (!barcodeCanvas.value) return
    try {
      JsBarcode(barcodeCanvas.value, barcodeInput.value.trim(), {
        format: barcodeFormat.value,
        width: barcodeWidth.value,
        height: barcodeHeight.value,
        displayValue: barcodeShowText.value,
        font: 'monospace',
        fontSize: 16,
        textMargin: 8,
        margin: 10,
        background: '#ffffff',
        lineColor: '#000000',
      })
    } catch (e: any) {
      barcodeError.value = `生成失败: ${e.message || '无效的条形码数据'}`
      toast.error(`条形码生成失败: ${e.message}`)
    }
  })
}

function downloadBarcode() {
  if (!barcodeCanvas.value) { toast.error('请先生成条形码'); return }
  try {
    const dataUrl = barcodeCanvas.value.toDataURL('image/png')
    const base64 = dataUrl.split(',')[1]
    const byteCharacters = atob(base64)
    const byteNumbers = new Array(byteCharacters.length)
    for (let i = 0; i < byteCharacters.length; i++) {
      byteNumbers[i] = byteCharacters.charCodeAt(i)
    }
    const byteArray = new Uint8Array(byteNumbers)
    const blob = new Blob([byteArray], { type: 'image/png' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `barcode_${barcodeFormat.value}_${barcodeInput.value.trim()}.png`
    a.click()
    URL.revokeObjectURL(url)
    toast.success('条形码已下载')
  } catch (e: any) {
    toast.error(`下载失败: ${e.message}`)
  }
}

onMounted(() => {
  generateBarcode()
})
</script>