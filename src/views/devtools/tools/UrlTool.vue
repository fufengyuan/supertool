<template>
  <div>
    <h3 class="text-lg font-bold text-base-content mb-5">🔗 URL 编码/解码</h3>

    <!-- Mode Toggle -->
    <div class="flex flex-wrap gap-2.5 mb-3">
      <div>
        <span class="label-text text-xs font-medium opacity-60 mb-1 block">模式</span>
        <div class="join">
          <button
            class="btn btn-ghost join-item"
            :class="{ 'btn-active': mode === 'encode' }"
            @click="mode = 'encode'"
          >编码</button>
          <button
            class="btn btn-ghost join-item"
            :class="{ 'btn-active': mode === 'decode' }"
            @click="mode = 'decode'"
          >解码</button>
        </div>
      </div>
    </div>

    <!-- Input -->
    <div class="mb-5">
      <span class="label-text text-xs font-medium opacity-60 mb-1 block">输入</span>
      <textarea
        v-model="inputText"
        class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]"
        :placeholder="mode === 'encode' ? '输入要编码的 URL 或文本...' : '输入要解码的 URL 编码字符串...'"
      ></textarea>
    </div>

    <div class="flex flex-wrap gap-2.5 mb-3">
      <button class="btn btn-primary" @click="process">
        {{ mode === 'encode' ? '编码' : '解码' }}
      </button>
      <button class="btn btn-ghost" @click="copyResult">复制结果</button>
      <button class="btn btn-ghost" @click="clearAll">清空</button>
    </div>

    <!-- Output -->
    <div class="mb-5">
      <span class="label-text text-xs font-medium opacity-60 mb-1 block">输出</span>
      <div class="bg-base-200 border border-base-content/10 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all max-h-72 overflow-y-auto">{{ outputText || '结果将显示在这里...' }}</div>
    </div>

    <hr class="border-base-content/10 my-5" />

    <!-- Quick Examples -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">快速示例</h4>
      <div class="flex flex-wrap gap-2">
        <button class="btn btn-ghost text-xs !px-3 !py-1.5" v-for="ex in examples" :key="ex.text" @click="useExample(ex)">
          {{ ex.label }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const mode = ref<'encode' | 'decode'>('encode')
const inputText = ref('')
const outputText = ref('')

const examples = [
  { label: '中文', text: '你好世界' },
  { label: 'URL', text: 'https://example.com/search?q=hello world' },
  { label: 'Query', text: 'name=张三&age=25&city=北京' },
  { label: '特殊字符', text: 'a=b&c=d e+f/g' },
  { label: 'Emoji', text: 'Hello 👋 World 🌍' },
]

function process() {
  if (!inputText.value.trim()) {
    toast.warning('请输入文本')
    return
  }

  try {
    if (mode.value === 'encode') {
      // Encode each component separately to handle URLs properly
      outputText.value = encodeURIComponent(inputText.value)
    } else {
      outputText.value = decodeURIComponent(inputText.value.trim())
    }
  } catch (e: any) {
    toast.error(`${mode.value === 'encode' ? '编码' : '解码'}失败: ${e.message}`)
    outputText.value = ''
  }
}

function copyResult() {
  if (!outputText.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(outputText.value, toast)
}

function clearAll() {
  inputText.value = ''
  outputText.value = ''
}

function useExample(ex: { label: string; text: string }) {
  inputText.value = ex.text
  process()
}
</script>
