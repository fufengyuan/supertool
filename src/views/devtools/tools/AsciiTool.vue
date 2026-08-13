<template>
  <ToolPage
    icon="terminal"
    name="ASCII 编码"
    description="逐字符查看十进制 / 十六进制 / 八进制 / 二进制编码，支持整串复制"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入文本</h4>
      <input
        v-model="input"
        class="input input-bordered w-full font-mono text-sm bg-base-200/60"
        placeholder="输入任意文本..."
        @input="convert"
      />
    </div>

    <!-- 字符对照表 -->
    <div v-if="results.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-0 overflow-hidden">
      <div class="flex items-center justify-between px-4 py-2.5 border-b border-base-content/10">
        <h4 class="text-xs font-semibold text-base-content/70">字符对照表</h4>
        <span class="text-[11px] text-base-content/40">{{ results.length }} 个字符</span>
      </div>
      <div class="overflow-x-auto max-h-64 overflow-y-auto">
        <table class="w-full border-collapse text-xs">
          <thead>
            <tr>
              <th class="sticky top-0 bg-base-100 text-xs font-semibold text-primary text-left px-3 py-2 border-b border-base-content/10 z-10">字符</th>
              <th class="sticky top-0 bg-base-100 text-xs font-semibold text-primary text-left px-3 py-2 border-b border-base-content/10 z-10">十进制 (DEC)</th>
              <th class="sticky top-0 bg-base-100 text-xs font-semibold text-primary text-left px-3 py-2 border-b border-base-content/10 z-10">十六进制 (HEX)</th>
              <th class="sticky top-0 bg-base-100 text-xs font-semibold text-primary text-left px-3 py-2 border-b border-base-content/10 z-10">八进制 (OCT)</th>
              <th class="sticky top-0 bg-base-100 text-xs font-semibold text-primary text-left px-3 py-2 border-b border-base-content/10 z-10">二进制 (BIN)</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(r, idx) in results" :key="idx" :class="{ 'text-base-content/50': r.char === ' ' }">
              <td class="text-base font-semibold px-3 py-1.5 border-b border-base-content/5 font-mono text-base-content">{{ r.char === ' ' ? '⎵' : r.char }}</td>
              <td class="px-3 py-1.5 border-b border-base-content/5 font-mono text-base-content">{{ r.dec }}</td>
              <td class="px-3 py-1.5 border-b border-base-content/5 font-mono text-base-content">{{ r.hex }}</td>
              <td class="px-3 py-1.5 border-b border-base-content/5 font-mono text-base-content">{{ r.oct }}</td>
              <td class="px-3 py-1.5 border-b border-base-content/5 font-mono text-base-content">{{ r.bin }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 整串结果 -->
    <div v-if="input" class="bg-base-100 border border-base-content/10 rounded-xl p-4 flex flex-col gap-2.5">
      <h4 class="text-xs font-semibold text-base-content/70">整串编码</h4>
      <div v-for="row in summaryRows" :key="row.label" class="flex items-center gap-2.5 p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
        <span class="text-xs font-semibold text-primary min-w-[70px] flex-shrink-0">{{ row.label }}</span>
        <span class="flex-1 font-mono text-xs text-base-content break-all">{{ row.value }}</span>
        <button class="btn btn-ghost btn-xs flex-shrink-0" @click="doCopy(row.value)"><SvgIcon name="copy" size="12" /></button>
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
  // Array.from 按码点迭代，codePointAt 取完整码点（emoji/非 BMP 字符不再是代理单元）
  return Array.from(input.value).map(ch => {
    const code = ch.codePointAt(0) || 0
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

const summaryRows = computed(() => [
  { label: '十进制', value: decStr.value },
  { label: '十六进制', value: hexStr.value },
  { label: '八进制', value: octStr.value },
  { label: '二进制', value: binStr.value },
])

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
