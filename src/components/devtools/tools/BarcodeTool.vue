<template>
  <div class="barcode-tool">
    <h3>📊 条形码生成</h3>

    <div class="tool-section">
      <h4>条形码设置</h4>
      <div class="tool-row">
        <input
          v-model="barcodeInput"
          type="text"
          class="tool-input mono"
          placeholder="输入条形码内容..."
          @input="debouncedGenerate"
        />
        <select v-model="barcodeFormat" class="tool-select" @change="generateBarcode">
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

      <div class="tool-row">
        <div>
          <label class="tool-label">宽度</label>
          <input v-model.number="barcodeWidth" type="number" class="tool-input mono" style="width: 80px" min="1" max="4" />
        </div>
        <div>
          <label class="tool-label">高度</label>
          <input v-model.number="barcodeHeight" type="number" class="tool-input mono" style="width: 80px" min="20" max="200" />
        </div>
        <label class="tool-checkbox">
          <input type="checkbox" v-model="barcodeShowText" />
          显示文字
        </label>
        <button class="tool-btn primary" @click="generateBarcode">生成</button>
        <button class="tool-btn" @click="downloadBarcode">⬇️ 下载 PNG</button>
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

<style scoped>

.barcode-tool {
  max-width: 700px;
}

.barcode-tool h3 {
  font-size: 18px;
  font-weight: 700;
  color: oklch(var(--bc));
  margin: 0 0 20px 0;
}

.tool-section h4 {
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin: 0 0 10px 0;
}

.tool-row {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
  flex-wrap: wrap;
  align-items: flex-end;
}

.tool-result {
  margin-top: 10px;
  padding: 10px 12px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: oklch(var(--bc));
  white-space: pre-wrap;
  word-break: break-all;
}

.tool-result.error {
  border-color: #e74c3c;
  color: #e74c3c;
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

.barcode-output {
  margin-top: 16px;
  padding: 16px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  display: flex;
  justify-content: center;
  overflow-x: auto;
}

.barcode-canvas {
  max-width: 100%;
}

.tool-btn {
  padding: 7px 16px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
  transition: all 0.15s;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tool-btn:hover {
  border-color: oklch(var(--p));
  color: oklch(var(--p));
}

.tool-btn.primary {
  background: oklch(var(--p));
  color: white;
  border-color: oklch(var(--p));
}

.tool-btn.primary:hover {
  opacity: 0.9;
}

.tool-select {
  padding: 7px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  outline: none;
}

.tool-label {
  font-size: 12px;
  font-weight: 500;
  color: oklch(var(--bc) / 0.6);
  margin-bottom: 4px;
  display: block;
}

.tool-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: oklch(var(--bc));
  cursor: pointer;
  align-self: flex-end;
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
