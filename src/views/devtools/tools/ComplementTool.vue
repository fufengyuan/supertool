<template>
  <div class="tool-panel">
    <h3>💾 原码 / 反码 / 补码</h3>

    <div class="tool-section">
      <div class="tool-row">
        <div>
          <label class="tool-label">位宽</label>
          <select v-model.number="bitWidth" class="tool-select" @change="convert">
            <option :value="8">8 位</option>
            <option :value="16">16 位</option>
            <option :value="32" selected>32 位</option>
            <option :value="64">64 位</option>
          </select>
        </div>
        <div style="flex: 1">
          <label class="tool-label">十进制数</label>
          <input
            v-model.number="input"
            class="tool-input"
            type="number"
            placeholder="输入十进制整数..."
            @input="convert"
          />
        </div>
      </div>

      <div v-if="error" class="error-box">{{ error }}</div>

      <div v-if="result" class="results-grid" style="margin-top: 16px">
        <div class="result-card">
          <div class="result-label">原码 (Sign-Magnitude)</div>
          <div class="result-value binary-value">{{ result.original }}</div>
          <div class="result-copy-hint" @click="copyValue(result.original)">点击复制</div>
        </div>
        <div class="result-card">
          <div class="result-label">反码 (One's Complement)</div>
          <div class="result-value binary-value">{{ result.inverse }}</div>
          <div class="result-copy-hint" @click="copyValue(result.inverse)">点击复制</div>
        </div>
        <div class="result-card">
          <div class="result-label">补码 (Two's Complement)</div>
          <div class="result-value binary-value">{{ result.complement }}</div>
          <div class="result-copy-hint" @click="copyValue(result.complement)">点击复制</div>
        </div>
      </div>

      <div v-if="result" class="info-block">
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">十六进制</span>
            <span class="info-value">{{ result.hex }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">八进制</span>
            <span class="info-value">{{ result.oct }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">无符号值</span>
            <span class="info-value">{{ result.unsigned }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const input = ref<number | null>(null)
const bitWidth = ref(32)
const error = ref('')

interface ComplementResult {
  original: string
  inverse: string
  complement: string
  hex: string
  oct: string
  unsigned: string
}

const result = ref<ComplementResult | null>(null)

function computeComplement(value: number, bits: number): ComplementResult | string {
  const maxVal = Math.pow(2, bits - 1) - 1
  const minVal = -Math.pow(2, bits - 1)

  if (value > maxVal || value < minVal) {
    return `超出 ${bits} 位有符号整数范围 [${minVal}, ${maxVal}]`
  }

  if (!Number.isInteger(value)) {
    return '请输入整数'
  }

  // For small bit widths, use BigInt to avoid precision issues
  const isNegative = value < 0
  const absVal = Math.abs(value)

  // Calculate unsigned representation for the complement
  let unsignedVal: number
  if (isNegative) {
    unsignedVal = Math.pow(2, bits) - absVal
  } else {
    unsignedVal = value
  }

  // For 64-bit, we need to be careful with JavaScript number precision
  // Use BigInt for the full calculation
  const bigBits = BigInt(bits)
  const bigAbsVal = BigInt(absVal)
  const bigTwo = BigInt(2)
  const bigPow2 = bigTwo ** bigBits

  // Original code (原码): sign bit + magnitude
  let original = ''
  if (isNegative) {
    original = '1' + absVal.toString(2).padStart(bits - 1, '0')
  } else {
    original = '0' + absVal.toString(2).padStart(bits - 1, '0')
  }

  // Inverse code (反码): for positive = original; for negative = sign bit + inverted magnitude bits
  let inverse = ''
  if (!isNegative) {
    inverse = original
  } else {
    // Sign bit stays 1, magnitude bits are inverted
    const magnitudeBits = absVal.toString(2).padStart(bits - 1, '0')
    const inverted = magnitudeBits.split('').map(b => b === '0' ? '1' : '0').join('')
    inverse = '1' + inverted
  }

  // Complement code (补码): for positive = original; for negative = inverse + 1
  let complement = ''
  if (!isNegative) {
    complement = original
  } else {
    // Calculate using BigInt for precision
    const complementBits = (bigPow2 - bigAbsVal).toString(2).padStart(bits, '0')
    complement = complementBits.slice(-bits) // Take only the last 'bits' characters
  }

  // Hex and octal from complement
  const hex = '0x' + (unsignedVal >>> 0).toString(16).toUpperCase()
  const oct = '0o' + (unsignedVal >>> 0).toString(8)

  return {
    original,
    inverse,
    complement,
    hex,
    oct,
    unsigned: String(unsignedVal),
  }
}

function computeComplement64(value: number, bits: 64): ComplementResult | string {
  if (!Number.isInteger(value)) {
    return '请输入整数'
  }

  const isNegative = value < 0
  const absVal = BigInt(Math.abs(value))
  const bigTwo = BigInt(2)
  const bigPow2 = bigTwo ** BigInt(bits)

  // Original code
  let original = ''
  if (isNegative) {
    original = '1' + absVal.toString(2).padStart(bits - 1, '0')
  } else {
    original = '0' + absVal.toString(2).padStart(bits - 1, '0')
  }

  // Inverse code
  let inverse = ''
  if (!isNegative) {
    inverse = original
  } else {
    const magnitudeBits = absVal.toString(2).padStart(bits - 1, '0')
    const inverted = magnitudeBits.split('').map(b => b === '0' ? '1' : '0').join('')
    inverse = '1' + inverted
  }

  // Complement code
  let complement = ''
  if (!isNegative) {
    complement = original
  } else {
    complement = (bigPow2 - absVal).toString(2)
    while (complement.length < bits) complement = '0' + complement
    complement = complement.slice(-bits)
  }

  // Hex
  const unsignedVal = isNegative ? bigPow2 - absVal : absVal
  const hexStr = unsignedVal.toString(16).toUpperCase()
  const hex = '0x' + hexStr

  // Octal
  const octStr = unsignedVal.toString(8)
  const oct = '0o' + octStr

  return {
    original,
    inverse,
    complement,
    hex,
    oct,
    unsigned: unsignedVal.toString(),
  }
}

function convert() {
  error.value = ''
  result.value = null

  if (input.value === null || input.value === undefined) return

  const value = input.value
  const bits = bitWidth.value

  let res: ComplementResult | string

  if (bits === 64) {
    res = computeComplement64(value, 64)
  } else {
    res = computeComplement(value, bits)
  }

  if (typeof res === 'string') {
    error.value = res
  } else {
    result.value = res
  }
}

function copyValue(value: string) {
  if (!value) return
  copyText(value, toast)
}

watch([input, bitWidth], () => {
  convert()
})
</script>

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
  margin: 0 0 20px 0;
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.result-card {
  padding: 12px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.result-card:hover {
  border-color: var(--color-primary);
}

.result-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-primary);
  margin-bottom: 6px;
}

.binary-value {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--color-base-content);
  word-break: break-all;
  letter-spacing: 0.5px;
}

.result-copy-hint {
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-top: 4px;
}

.error-box {
  margin-top: 12px;
  padding: 10px 12px;
  background: #fee2e2;
  border: 1px solid #fca5a5;
  border-radius: 8px;
  color: #dc2626;
  font-size: 13px;
}

.info-block {
  margin-top: 16px;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

.info-item {
  display: flex;
  flex-direction: column;
  padding: 8px 12px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
}

.info-label {
  font-size: 11px;
  font-weight: 500;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-bottom: 4px;
}

.info-value {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--color-base-content);
  word-break: break-all;
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--color-base-content); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--color-primary); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-input:focus { border-color: var(--color-primary); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
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
.tool-label { font-size: 12px; font-weight: 500; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-select:focus { border-color: var(--color-primary); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--color-base-content); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); margin: 20px 0; }
</style>
