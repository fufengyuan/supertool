<template>
  <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4">
    <div class="flex items-center justify-between mb-2 gap-2">
      <div class="flex items-center gap-2 min-w-0">
        <span class="text-xs font-semibold text-base-content/70 truncate"><slot name="title" /></span>
        <span v-if="detected !== 'text'" class="badge badge-xs badge-ghost shrink-0">{{ detected.toUpperCase() }}</span>
      </div>
      <div class="flex items-center gap-1 shrink-0">
        <select
          v-model="fmt"
          class="select select-bordered select-xs w-[92px] bg-base-200/60 text-[11px]"
          title="报文格式（默认自动识别）"
        >
          <option v-for="o in FORMAT_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
        <button class="btn btn-ghost btn-xs" :disabled="!content" @click="pretty = !pretty"
          :title="pretty ? '切换为原始（单行）' : '切换为美化（缩进）'">
          {{ pretty ? '原始' : '美化' }}
        </button>
        <button class="btn btn-primary btn-xs" :disabled="!content" @click="copyText(view.text, toast)">
          <SvgIcon name="copy" size="11" /> 复制
        </button>
      </div>
    </div>
    <div
      class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all overflow-y-auto min-h-[140px]"
    >{{ view.text || placeholder || '结果将显示在这里...' }}</div>
    <div v-if="view.error" class="text-[11px] text-warning mt-1.5">{{ view.error }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { copyText, detectFormat, formatText, FORMAT_OPTIONS, type TextFormat } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const props = defineProps<{
  content: string
  placeholder?: string
}>()

const toast = useToast()
const fmt = ref<TextFormat>('auto')
const pretty = ref(true)

/** 自动识别出的格式，仅在识别成功时展示角标 */
const detected = computed(() => (props.content ? detectFormat(props.content) : 'text'))

const view = computed(() => {
  const raw = props.content || ''
  if (!raw) {return { text: '', error: '' }}
  try {
    return { text: formatText(raw, fmt.value, pretty.value), error: '' }
  } catch (e: any) {
    // 手动指定格式却解析不了时，退回原文而不是空白，并说明原因
    return { text: raw, error: `${fmt.value.toUpperCase()} 解析失败，已按原文显示：${e?.message || e}` }
  }
})
</script>
