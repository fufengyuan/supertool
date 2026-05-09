<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5"><SvgIcon name="tag" size="14" class="align-text-bottom" /> HTML 实体编码/解码</h3>

    <div class="mb-5">
      <label class="label-text text-xs text-base-content/60 mb-1 block">输入</label>
      <textarea
        v-model="input"
        class="textarea textarea-bordered w-full text-xs bg-base-200 font-mono min-h-[120px]"
        placeholder="输入需要编码或解码的文本..."
        rows="5"
      ></textarea>

      <div class="flex flex-wrap gap-2.5 mb-3">
        <button class="btn btn-primary btn-sm" @click="encode">编码 →</button>
        <button class="btn btn-ghost btn-sm" @click="decode">← 解码</button>
        <button class="btn btn-ghost btn-sm" @click="copyResult"><SvgIcon name="file" size="14" class="align-text-bottom" /> 复制结果</button>
        <button class="btn btn-ghost btn-sm" @click="clear">清空</button>
      </div>

      <label class="label-text text-xs text-base-content/60 mb-1 block" style="margin-top: 12px">输出</label>
      <div class="mt-2.5 p-2.5 bg-base-200 border border-base-content/10 rounded-box font-mono text-xs text-base-content whitespace-pre-wrap break-all max-h-72 overflow-y-auto">{{ output || '结果将显示在这里...' }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

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