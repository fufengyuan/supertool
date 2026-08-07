<template>
  <ToolPage
    icon="keyboard"
    name="汉字转拼音"
    description="声调 / 无声调 / 首字母三种模式，可自定义分隔符"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入汉字</h4>
      <textarea
        v-model="chineseInput"
        class="textarea textarea-bordered w-full font-mono text-sm bg-base-200/60 min-h-[100px] resize-none"
        placeholder="请输入中文文本..."
        @input="convertPinyin"
      ></textarea>
      <div class="flex flex-wrap gap-3 mt-3 items-end">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">转换模式</span>
          <select v-model="pinyinMode" class="select select-bordered select-sm" @change="convertPinyin">
            <option value="tone">带声调 (nǐ hǎo)</option>
            <option value="notone">无声调 (ni hao)</option>
            <option value="initials">首字母 (nh)</option>
          </select>
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">分隔符</span>
          <select v-model="separator" class="select select-bordered select-sm" @change="convertPinyin">
            <option value=" ">空格</option>
            <option value=",">逗号</option>
            <option value="-">横线</option>
            <option value="">无分隔</option>
          </select>
        </div>
        <button class="btn btn-primary btn-sm" @click="convertPinyin">转换</button>
      </div>
    </div>

    <div v-if="pinyinOutput" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex items-center justify-between mb-2.5">
        <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 拼音结果</h4>
        <button class="btn btn-primary btn-xs" @click="copyText(pinyinOutput, toast)"><SvgIcon name="copy" size="11" /> 复制</button>
      </div>
      <div class="p-3.5 bg-base-200/60 border border-base-content/10 rounded-lg text-base text-base-content whitespace-pre-wrap break-all">{{ pinyinOutput }}</div>
    </div>

    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="sparkles" size="12" /> 示例</h4>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
        <div
          v-for="ex in examples"
          :key="ex.text"
          class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg cursor-pointer transition-all duration-150 flex flex-col gap-1 hover:border-primary hover:bg-primary/10"
          @click="chineseInput = ex.text; convertPinyin()"
        >
          <span class="text-sm text-base-content font-medium">{{ ex.text }}</span>
          <span class="text-xs text-base-content/60">{{ ex.pinyin }}</span>
        </div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import { pinyin } from 'pinyin-pro'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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
