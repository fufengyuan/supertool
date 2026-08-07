<template>
  <ToolPage
    icon="ban"
    name="原码 / 反码 / 补码"
    description="8/16/32/64 位有符号整数的原码、反码、补码与进制换算"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex flex-wrap items-end gap-3">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">位宽</span>
          <select v-model.number="bitWidth" class="select select-bordered select-sm" @change="convert">
            <option :value="8">8 位</option>
            <option :value="16">16 位</option>
            <option :value="32">32 位</option>
            <option :value="64">64 位</option>
          </select>
        </div>
        <div class="flex-1 min-w-[220px]">
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">十进制整数</span>
          <input
            v-model.number="input"
            class="input input-bordered w-full font-mono text-sm bg-base-200/60"
            type="number"
            placeholder="输入十进制整数..."
            @input="convert"
          />
        </div>
      </div>
      <div v-if="error" class="mt-3 p-2.5 bg-error/10 border border-error/30 rounded-lg text-error text-xs">{{ error }}</div>
    </div>

    <div v-if="result" class="grid grid-cols-1 sm:grid-cols-3 gap-3">
      <div
        v-for="card in resultCards"
        :key="card.label"
        class="bg-base-100 border border-base-content/10 rounded-xl p-4"
      >
        <div class="text-[11px] font-semibold text-primary mb-2">{{ card.label }}</div>
        <div class="font-mono text-sm text-base-content break-all bg-base-200/60 border border-base-content/10 rounded-lg p-2.5 mb-2">{{ card.value }}</div>
        <button class="btn btn-outline btn-xs w-full" @click="copyValue(card.value)"><SvgIcon name="copy" size="10" /> 复制</button>
      </div>
    </div>

    <div v-if="result" class="grid grid-cols-1 sm:grid-cols-3 gap-3">
      <div v-for="info in infoRows" :key="info.label" class="flex items-center justify-between bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
        <span class="text-xs text-base-content/50">{{ info.label }}</span>
        <span class="font-mono text-sm text-base-content break-all text-right">{{ info.value }}</span>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed, watch } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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

const resultCards = computed(() => {
  const r = result.value
  if (!r) { return [] }
  return [
    { label: '原码 (Sign-Magnitude)', value: r.original },
    { label: "反码 (One's Complement)", value: r.inverse },
    { label: "补码 (Two's Complement)", value: r.complement },
  ]
})

const infoRows = computed(() => {
  const r = result.value
  if (!r) { return [] }
  return [
    { label: '十六进制', value: r.hex },
    { label: '八进制', value: r.oct },
    { label: '无符号值', value: r.unsigned },
  ]
})

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
    while (complement.length < bits) {complement = '0' + complement}
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

  if (input.value === null || input.value === undefined) {return}

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
  if (!value) {return}
  copyText(value, toast)
}

watch([input, bitWidth], () => {
  convert()
})
</script>
