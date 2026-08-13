<template>
  <ToolPage
    icon="link"
    name="URL 编码/解码"
    description="URL / URI 组件编码与解码，内置常用示例"
    @back="$emit('back')"
  >
    <!-- 模式与操作 -->
    <div class="flex flex-wrap items-center gap-3 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
      <div class="join">
        <button class="btn btn-sm join-item" :class="mode === 'encode' ? 'btn-primary' : 'btn-ghost'" @click="mode = 'encode'">编码</button>
        <button class="btn btn-sm join-item" :class="mode === 'decode' ? 'btn-primary' : 'btn-ghost'" @click="mode = 'decode'">解码</button>
      </div>
      <label class="flex items-center gap-1 text-[11px] text-base-content/50">
        编码策略
        <select v-model="encodeMode" class="select select-xs select-bordered bg-base-200/60 w-[170px]" title="Component：空格→%20（encodeURIComponent，默认）；URI：保留 / : ? & = 等结构字符（encodeURI）；Form：空格→+（application/x-www-form-urlencoded / qs 库风格）">
          <option value="component">Component（%20）</option>
          <option value="uri">URI（保留结构字符）</option>
          <option value="form">Form（空格→+）</option>
        </select>
      </label>
      <button class="btn btn-primary btn-sm ml-auto" @click="process">
        {{ mode === 'encode' ? '编码' : '解码' }}
      </button>
    </div>

    <!-- 输入输出 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[220px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入</h4>
          <button class="btn btn-ghost btn-xs" @click="clearAll" :disabled="!inputText">清空</button>
        </div>
        <textarea
          v-model="inputText"
          class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 focus:bg-base-200"
          :placeholder="mode === 'encode' ? '输入要编码的 URL 或文本...' : '输入要解码的 URL 编码字符串...'"
        ></textarea>
      </div>
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[220px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="arrowUp" size="12" /> 输出</h4>
          <button class="btn btn-primary btn-xs" @click="copyResult" :disabled="!outputText"><SvgIcon name="copy" size="11" /> 复制</button>
        </div>
        <div class="flex-1 p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-sm whitespace-pre-wrap break-all overflow-y-auto min-h-[100px]">{{ outputText || '结果将显示在这里...' }}</div>
      </div>
    </div>

    <!-- 快速示例 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="sparkles" size="12" /> 快速示例</h4>
      <div class="flex flex-wrap gap-2">
        <button class="btn btn-outline btn-xs" v-for="ex in examples" :key="ex.text" @click="useExample(ex)">
          {{ ex.label }}
        </button>
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

const mode = ref<'encode' | 'decode'>('encode')
const inputText = ref('')
const outputText = ref('')
// 编码策略：component = encodeURIComponent（空格→%20）；uri = encodeURI（保留结构字符）；form = 空格→+
const encodeMode = ref<'component' | 'uri' | 'form'>('component')

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
      // 按所选策略编码：component 逐组件转义；uri 保留 URL 结构字符；form 用 + 表示空格
      if (encodeMode.value === 'uri') {
        outputText.value = encodeURI(inputText.value)
      } else if (encodeMode.value === 'form') {
        outputText.value = encodeURIComponent(inputText.value).replace(/%20/g, '+')
      } else {
        outputText.value = encodeURIComponent(inputText.value)
      }
    } else {
      // 解码：form 策略先把 + 还原为空格（其他策略 + 是字面量，不处理）
      const str = encodeMode.value === 'form' ? inputText.value.replace(/\+/g, ' ') : inputText.value
      outputText.value = encodeMode.value === 'uri'
        ? decodeURI(str.trim())
        : decodeURIComponent(str.trim())
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
