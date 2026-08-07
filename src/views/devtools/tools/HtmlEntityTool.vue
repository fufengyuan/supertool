<template>
  <ToolPage
    icon="tag"
    name="HTML 实体编码/解码"
    description="将文本转为 HTML 实体（&amp; &lt; 等），或反向解码"
    @back="$emit('back')"
  >
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[240px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入</h4>
          <button class="btn btn-ghost btn-xs" @click="clear" :disabled="!input">清空</button>
        </div>
        <textarea
          v-model="input"
          class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 focus:bg-base-200"
          placeholder="输入需要编码或解码的文本..."
        ></textarea>
        <div class="flex flex-wrap gap-2 mt-3">
          <button class="btn btn-primary btn-sm" @click="encode">编码 →</button>
          <button class="btn btn-outline btn-sm" @click="decode">← 解码</button>
        </div>
      </div>
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[240px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 输出</h4>
          <button class="btn btn-primary btn-xs" @click="copyResult" :disabled="!output"><SvgIcon name="copy" size="11" /> 复制</button>
        </div>
        <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-sm whitespace-pre-wrap break-all overflow-y-auto min-h-[120px]">{{ output || '结果将显示在这里...' }}</div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()
const input = ref('')
const output = ref('')

function encode() {
  if (!input.value) {
    toast.warning('请输入文本')
    return
  }
  output.value = input.value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function decode() {
  if (!input.value) {
    toast.warning('请输入文本')
    return
  }
  output.value = input.value
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&')
}

function copyResult() {
  if (!output.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(output.value, toast)
}

function clear() {
  input.value = ''
  output.value = ''
}
</script>
