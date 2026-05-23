<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5"><SvgIcon name="terminal" size="14" class="inline-block align-text-bottom" /> ASCII 编码转换</h3>

    <div class="mb-5">
      <label class="label-text text-xs text-base-content/60 mb-1 block">输入文本</label>
      <input
        v-model="input"
        class="input input-bordered w-full text-xs bg-base-200"
        placeholder="输入任意文本..."
        @input="convert"
      />

      <div v-if="results.length > 0" class="mt-4 overflow-x-auto max-h-72 overflow-y-auto">
        <table class="w-full border-collapse text-xs">
          <thead>
            <tr>
              <th class="sticky top-0 bg-base-200 text-xs font-semibold text-primary text-left px-3 py-2 border-b-2 border-base-content/10 z-10">字符</th>
              <th class="sticky top-0 bg-base-200 text-xs font-semibold text-primary text-left px-3 py-2 border-b-2 border-base-content/10 z-10">十进制 (DEC)</th>
              <th class="sticky top-0 bg-base-200 text-xs font-semibold text-primary text-left px-3 py-2 border-b-2 border-base-content/10 z-10">十六进制 (HEX)</th>
              <th class="sticky top-0 bg-base-200 text-xs font-semibold text-primary text-left px-3 py-2 border-b-2 border-base-content/10 z-10">八进制 (OCT)</th>
              <th class="sticky top-0 bg-base-200 text-xs font-semibold text-primary text-left px-3 py-2 border-b-2 border-base-content/10 z-10">二进制 (BIN)</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(r, idx) in results" :key="idx" :class="{ 'text-base-content/60': r.char === ' ' }">
              <td class="text-base font-semibold text-primary px-3 py-1.5 border-b border-base-content/10 font-mono text-base-content">{{ r.char === ' ' ? '⎵' : r.char }}</td>
              <td class="px-3 py-1.5 border-b border-base-content/10 font-mono text-base-content">{{ r.dec }}</td>
              <td class="px-3 py-1.5 border-b border-base-content/10 font-mono text-base-content">{{ r.hex }}</td>
              <td class="px-3 py-1.5 border-b border-base-content/10 font-mono text-base-content">{{ r.oct }}</td>
              <td class="px-3 py-1.5 border-b border-base-content/10 font-mono text-base-content">{{ r.bin }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="input" class="flex flex-col gap-2" style="margin-top: 16px">
        <div class="flex items-center gap-2.5 p-2 bg-base-200 border border-base-content/10 rounded-box">
          <span class="text-xs font-semibold text-primary min-w-[90px] flex-shrink-0">十进制</span>
          <span class="flex-1 font-mono text-xs text-base-content break-all">{{ decStr }}</span>
          <button class="btn btn-ghost btn-xs flex-shrink-0" @click="doCopy(decStr)"><SvgIcon name="file" size="14" class="align-text-bottom" /></button>
        </div>
        <div class="flex items-center gap-2.5 p-2 bg-base-200 border border-base-content/10 rounded-box">
          <span class="text-xs font-semibold text-primary min-w-[90px] flex-shrink-0">十六进制</span>
          <span class="flex-1 font-mono text-xs text-base-content break-all">{{ hexStr }}</span>
          <button class="btn btn-ghost btn-xs flex-shrink-0" @click="doCopy(hexStr)"><SvgIcon name="file" size="14" class="align-text-bottom" /></button>
        </div>
        <div class="flex items-center gap-2.5 p-2 bg-base-200 border border-base-content/10 rounded-box">
          <span class="text-xs font-semibold text-primary min-w-[90px] flex-shrink-0">八进制</span>
          <span class="flex-1 font-mono text-xs text-base-content break-all">{{ octStr }}</span>
          <button class="btn btn-ghost btn-xs flex-shrink-0" @click="doCopy(octStr)"><SvgIcon name="file" size="14" class="align-text-bottom" /></button>
        </div>
        <div class="flex items-center gap-2.5 p-2 bg-base-200 border border-base-content/10 rounded-box">
          <span class="text-xs font-semibold text-primary min-w-[90px] flex-shrink-0">二进制</span>
          <span class="flex-1 font-mono text-xs text-base-content break-all">{{ binStr }}</span>
          <button class="btn btn-ghost btn-xs flex-shrink-0" @click="doCopy(binStr)"><SvgIcon name="file" size="14" class="align-text-bottom" /></button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const input = ref('')

interface CharInfo {
  char: string
  dec: string
  hex: string
  oct: string
  bin: string
}

const results = computed<CharInfo[]>(() => {
  if (!input.value) {return []}
  return Array.from(input.value).map(ch => {
    const code = ch.charCodeAt(0)
    return {
      char: ch,
      dec: String(code),
      hex: '0x' + code.toString(16).toUpperCase().padStart(2, '0'),
      oct: '0o' + code.toString(8).padStart(3, '0'),
      bin: code.toString(2).padStart(8, '0'),
    }
  })
})

const decStr = computed(() => results.value.map(r => r.dec).join(' '))
const hexStr = computed(() => results.value.map(r => r.hex).join(' '))
const octStr = computed(() => results.value.map(r => r.oct).join(' '))
const binStr = computed(() => results.value.map(r => r.bin).join(' '))

function convert() {
  // reactive via computed
}

function doCopy(text: string) {
  if (!text) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(text, toast)
}
</script>