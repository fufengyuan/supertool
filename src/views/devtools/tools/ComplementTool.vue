<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg> 原码 / 反码 / 补码</h3>

    <div class="mb-5">
      <div class="flex gap-2.5 mb-3 flex-wrap items-center">
        <div>
          <label class="text-xs font-medium text-base-content/60 mb-1 block">位宽</label>
          <select v-model.number="bitWidth" class="select select-bordered select-sm" @change="convert">
            <option :value="8">8 位</option>
            <option :value="16">16 位</option>
            <option :value="32" selected>32 位</option>
            <option :value="64">64 位</option>
          </select>
        </div>
        <div style="flex: 1">
          <label class="text-xs font-medium text-base-content/60 mb-1 block">十进制数</label>
          <input
            v-model.number="input"
            class="input input-bordered w-full font-mono text-xs"
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