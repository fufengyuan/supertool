<template>
  <ToolPage
    icon="grid"
    name="条形码生成"
    description="CODE128 / EAN / UPC / ITF14 等 9 种格式，支持下载 PNG"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="grid" size="12" /> 条形码设置</h4>
      <div class="flex gap-2 mb-3 flex-wrap items-center">
        <input
          v-model="barcodeInput"
          type="text"
          class="input input-bordered input-sm font-mono flex-1 min-w-[180px] bg-base-200/60"
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
      <div class="flex gap-2 mb-3 flex-wrap items-center">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">宽度</span>
          <input v-model.number="barcodeWidth" type="number" class="input input-bordered input-sm font-mono w-20 bg-base-200/60" min="1" max="4" />
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">高度</span>
          <input v-model.number="barcodeHeight" type="number" class="input input-bordered input-sm font-mono w-20 bg-base-200/60" min="20" max="200" />
        </div>
        <label class="flex items-center gap-1.5 text-xs text-base-content/70 cursor-pointer select-none self-end pb-2">
          <input type="checkbox" v-model="barcodeShowText" class="checkbox checkbox-sm checkbox-primary" />
          显示文字
        </label>
        <button class="btn btn-primary btn-sm self-end" @click="generateBarcode">生成</button>
        <button class="btn btn-outline btn-sm self-end" @click="downloadBarcode"><SvgIcon name="download" size="12" /> 下载 PNG</button>
      </div>
      <div v-if="barcodeError" class="p-2.5 bg-error/10 border border-error/25 rounded-lg text-error text-xs">{{ barcodeError }}</div>
    </div>

    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4 flex items-center justify-center min-h-[220px]">
      <canvas ref="barcodeCanvas" class="max-w-full"></canvas>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, onMounted, nextTick } from 'vue'
import JsBarcode from 'jsbarcode'
import { copyText, downloadFile } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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
  if (debounceTimer) {clearTimeout(debounceTimer)}
  debounceTimer = setTimeout(() => generateBarcode(), 300)
}

function generateBarcode() {
  barcodeError.value = ''
  if (!barcodeInput.value.trim()) {return}

  nextTick(() => {
    if (!barcodeCanvas.value) {return}
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
    const byteNumbers = Array.from({ length: byteCharacters.length }, (_, i) => byteCharacters.charCodeAt(i))
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