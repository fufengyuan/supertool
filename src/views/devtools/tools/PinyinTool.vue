<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5">🔤 汉字转拼音</h3>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">输入汉字</h4>
      <textarea
        v-model="chineseInput"
        class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]"
        placeholder="请输入中文文本..."
        rows="3"
        @input="convertPinyin"
      ></textarea>

      <div class="flex flex-wrap gap-2.5 mb-3 mt-3 items-end">
        <div>
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">转换模式</span>
          <select v-model="pinyinMode" class="select select-bordered" @change="convertPinyin">
            <option value="tone">带声调 (nǐ hǎo)</option>
            <option value="notone">无声调 (ni hao)</option>
            <option value="initials">首字母 (nh)</option>
          </select>
        </div>
        <div>
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">分隔符</span>
          <select v-model="separator" class="select select-bordered" @change="convertPinyin">
            <option value=" ">空格</option>
            <option value=",">逗号</option>
            <option value="-">横线</option>
            <option value="">无分隔</option>
          </select>
        </div>
        <button class="btn btn-primary" @click="convertPinyin">转换</button>
        <button class="btn btn-ghost" @click="copyText(pinyinOutput, toast)"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 复制</button>
      </div>

      <div v-if="pinyinOutput" class="bg-base-200 border border-base-content/10 rounded-box p-3 text-base whitespace-pre-wrap break-all">{{ pinyinOutput }}</div>
    </div>

    <hr class="border-base-content/10 my-5" />

    <!-- Examples -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">示例</h4>
      <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2">
        <div
          v-for="ex in examples"
          :key="ex.text"
          class="p-2 bg-base-200 border border-base-content/10 rounded-box cursor-pointer transition-all duration-150 flex flex-col gap-1 hover:border-primary hover:bg-primary/10"
          @click="chineseInput = ex.text; convertPinyin()"
        >
          <span class="text-sm text-base-content font-medium">{{ ex.text }}</span>
          <span class="text-xs opacity-60">{{ ex.pinyin }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { pinyin } from 'pinyin-pro'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const chineseInput = ref('你好世界')
const pinyinMode = ref<'tone' | 'notone' | 'initials'>('tone')
const separator = ref(' ')
const pinyinOutput = ref('')

function convertPinyin() {
  if (!chineseInput.value.trim()) { pinyinOutput.value = ''; return }
  try {
    let result: string
    if (pinyinMode.value === 'initials') {
      result = pinyin(chineseInput.value, {
        pattern: 'first',
        toneType: 'none',
        type: 'array',
      }).join(separator.value)
    } else {
      result = pinyin(chineseInput.value, {
        pattern: 'pinyin',
        toneType: pinyinMode.value === 'tone' ? 'symbol' : 'none',
        type: 'array',
      }).join(separator.value)
    }
    pinyinOutput.value = result
  } catch (e: any) {
    pinyinOutput.value = `错误: ${e.message || '转换失败'}`
    toast.error('拼音转换失败')
  }
}

const examples = [
  { text: '你好世界', pinyin: 'nǐ hǎo shì jiè' },
  { text: '今天天气真好', pinyin: 'jīn tiān tiān qì zhēn hǎo' },
  { text: '中华人民共和国', pinyin: 'zhōng huá rén mín gòng hé guó' },
  { text: '北京欢迎你', pinyin: 'běi jīng huān yíng nǐ' },
]
</script>
