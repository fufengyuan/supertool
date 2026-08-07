<template>
  <ToolPage
    icon="hash"
    name="Hex 转换"
    description="String ↔ Hex、Hex ↔ Base64 互转，以及一键多格式转换"
    @back="$emit('back')"
  >
    <!-- String ↔ Hex -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="fileText" size="12" /> String ↔ Hex</h4>
      <textarea
        v-model="stringHexInput"
        class="textarea textarea-bordered w-full font-mono text-sm bg-base-200/60 min-h-[100px] resize-none"
        placeholder="输入字符串或 Hex..."
      ></textarea>
      <div class="flex flex-wrap gap-2 mt-3">
        <button class="btn btn-primary btn-sm" @click="stringToHex">String → Hex</button>
        <button class="btn btn-outline btn-sm" @click="hexToString">Hex → String</button>
        <button class="btn btn-ghost btn-sm ml-auto" @click="clearStringHex" :disabled="!stringHexInput">清空</button>
      </div>
      <div v-if="stringHexOutput" class="mt-3">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-[11px] font-medium text-base-content/50">输出</span>
          <button class="btn btn-primary btn-xs" @click="copyHexResult"><SvgIcon name="copy" size="11" /> 复制</button>
        </div>
        <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-52 overflow-y-auto">{{ stringHexOutput }}</div>
      </div>
    </div>

    <!-- Hex ↔ Base64 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="refresh" size="12" /> Hex ↔ Base64</h4>
      <textarea
        v-model="hexBase64Input"
        class="textarea textarea-bordered w-full font-mono text-sm bg-base-200/60 min-h-[100px] resize-none"
        placeholder="输入 Hex 或 Base64..."
      ></textarea>
      <div class="flex flex-wrap gap-2 mt-3">
        <button class="btn btn-primary btn-sm" @click="hexToBase64">Hex → Base64</button>
        <button class="btn btn-outline btn-sm" @click="base64ToHex">Base64 → Hex</button>
        <button class="btn btn-ghost btn-sm ml-auto" @click="clearHexBase64" :disabled="!hexBase64Input">清空</button>
      </div>
      <div v-if="hexBase64Output" class="mt-3">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-[11px] font-medium text-base-content/50">输出</span>
          <button class="btn btn-primary btn-xs" @click="copyBase64Result"><SvgIcon name="copy" size="11" /> 复制</button>
        </div>
        <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-52 overflow-y-auto">{{ hexBase64Output }}</div>
      </div>
    </div>

    <!-- 快速转换 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="sparkles" size="12" /> 快速转换</h4>
      <textarea
        v-model="quickInput"
        class="textarea textarea-bordered w-full font-mono text-sm bg-base-200/60 min-h-[80px] resize-none"
        placeholder="输入任意文本，一键得到 Hex / Base64 / Unicode..."
      ></textarea>
      <div class="flex flex-wrap gap-2 mt-3">
        <button class="btn btn-primary btn-sm" @click="quickConvert">转换</button>
        <button class="btn btn-ghost btn-sm" @click="clearQuick" :disabled="!quickInput">清空</button>
      </div>
      <div v-if="quickResults" class="flex flex-col gap-2.5 mt-3">
        <div v-for="(item, key) in quickRows" :key="key" class="flex items-start gap-2.5 p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="shrink-0 text-[11px] font-medium text-base-content/50 w-14 pt-0.5">{{ item.label }}</span>
          <span class="flex-1 font-mono text-xs text-base-content whitespace-pre-wrap break-all">{{ item.value }}</span>
          <button class="btn btn-ghost btn-xs shrink-0" @click="doCopy(item.value)"><SvgIcon name="copy" size="11" /></button>
        </div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

// String ↔ Hex
const stringHexInput = ref('')
const stringHexOutput = ref('')

// Hex ↔ Base64
const hexBase64Input = ref('')
const hexBase64Output = ref('')

// Quick Convert
const quickInput = ref('')
const quickResults = ref<{ hex: string; base64: string; unicode: string } | null>(null)

const quickRows = computed(() => {
  if (!quickResults.value) { return [] }
  return [
    { label: 'Hex:', value: quickResults.value.hex },
    { label: 'Base64:', value: quickResults.value.base64 },
    { label: 'Unicode:', value: quickResults.value.unicode },
  ]
})

// String → Hex
function stringToHex() {
  if (!stringHexInput.value) {
    toast.warning('请输入文本')
    return
  }
  try {
    stringHexOutput.value = stringHexInput.value
      .split('')
      .map(c => c.charCodeAt(0).toString(16).padStart(2, '0'))
      .join(' ')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    stringHexOutput.value = ''
  }
}

// Hex → String
function hexToString() {
  if (!stringHexInput.value.trim()) {
    toast.warning('请输入 Hex')
    return
  }
  try {
    const hex = stringHexInput.value.replace(/\s+/g, '')
    if (hex.length % 2 !== 0) {
      toast.error('Hex 长度必须为偶数')
      stringHexOutput.value = ''
      return
    }
    if (!/^[0-9a-fA-F]+$/.test(hex)) {
      toast.error('无效的 Hex 字符串')
      stringHexOutput.value = ''
      return
    }
    stringHexOutput.value = hex
      .match(/.{2}/g)!
      .map(h => String.fromCharCode(parseInt(h, 16)))
      .join('')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    stringHexOutput.value = ''
  }
}

function copyHexResult() {
  if (!stringHexOutput.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(stringHexOutput.value, toast)
}

function clearStringHex() {
  stringHexInput.value = ''
  stringHexOutput.value = ''
}

// Hex → Base64
function hexToBase64() {
  if (!hexBase64Input.value.trim()) {
    toast.warning('请输入 Hex')
    return
  }
  try {
    const hex = hexBase64Input.value.replace(/\s+/g, '')
    if (hex.length % 2 !== 0) {
      toast.error('Hex 长度必须为偶数')
      hexBase64Output.value = ''
      return
    }
    if (!/^[0-9a-fA-F]+$/.test(hex)) {
      toast.error('无效的 Hex 字符串')
      hexBase64Output.value = ''
      return
    }
    const bytes = hex.match(/.{2}/g)!.map(h => parseInt(h, 16))
    let binary = ''
    for (const b of bytes) {
      binary += String.fromCharCode(b)
    }
    hexBase64Output.value = btoa(binary)
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    hexBase64Output.value = ''
  }
}

// Base64 → Hex
function base64ToHex() {
  if (!hexBase64Input.value.trim()) {
    toast.warning('请输入 Base64')
    return
  }
  try {
    const binary = atob(hexBase64Input.value.trim())
    hexBase64Output.value = binary
      .split('')
      .map(c => c.charCodeAt(0).toString(16).padStart(2, '0'))
      .join(' ')
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    hexBase64Output.value = ''
  }
}

function copyBase64Result() {
  if (!hexBase64Output.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(hexBase64Output.value, toast)
}

function clearHexBase64() {
  hexBase64Input.value = ''
  hexBase64Output.value = ''
}

// Quick Convert
function quickConvert() {
  if (!quickInput.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  try {
    const text = quickInput.value
    const hex = text
      .split('')
      .map(c => c.charCodeAt(0).toString(16).padStart(2, '0'))
      .join(' ')

    const binary = text
      .split('')
      .map(c => String.fromCharCode(c.charCodeAt(0)))
      .join('')
    const base64 = btoa(unescape(encodeURIComponent(text)))

    const unicode = text
      .split('')
      .map(c => {
        const code = c.codePointAt(0) || 0
        return '\\u' + code.toString(16).padStart(4, '0')
      })
      .join('')

    quickResults.value = { hex, base64, unicode }
  } catch (e: any) {
    toast.error(`转换失败: ${e.message}`)
    quickResults.value = null
  }
}

function doCopy(text: string) {
  if (!text) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(text, toast)
}

function clearQuick() {
  quickInput.value = ''
  quickResults.value = null
}
</script>
