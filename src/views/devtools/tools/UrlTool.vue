<template>
  <div class="tool-panel">
    <h3>🔗 URL 编码/解码</h3>

    <!-- Mode Toggle -->
    <div class="tool-row">
      <div>
        <label class="tool-label">模式</label>
        <div class="tool-btn-group">
          <button
            class="tool-btn"
            :class="{ active: mode === 'encode' }"
            @click="mode = 'encode'"
          >编码</button>
          <button
            class="tool-btn"
            :class="{ active: mode === 'decode' }"
            @click="mode = 'decode'"
          >解码</button>
        </div>
      </div>
    </div>

    <!-- Input -->
    <div class="tool-section">
      <label class="tool-label">输入</label>
      <textarea
        v-model="inputText"
        class="tool-textarea"
        :placeholder="mode === 'encode' ? '输入要编码的 URL 或文本...' : '输入要解码的 URL 编码字符串...'"
      ></textarea>
    </div>

    <div class="tool-row">
      <button class="tool-btn primary" @click="process">
        {{ mode === 'encode' ? '编码' : '解码' }}
      </button>
      <button class="tool-btn" @click="copyResult">复制结果</button>
      <button class="tool-btn" @click="clearAll">清空</button>
    </div>

    <!-- Output -->
    <div class="tool-section">
      <label class="tool-label">输出</label>
      <div class="tool-result">{{ outputText || '结果将显示在这里...' }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Quick Examples -->
    <div class="tool-section">
      <h4>快速示例</h4>
      <div class="examples">
        <button class="tool-btn example-btn" v-for="ex in examples" :key="ex.text" @click="useExample(ex)">
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

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
  margin: 0 0 20px 0;
}

.examples {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.example-btn {
  font-size: 12px !important;
  padding: 5px 12px !important;
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--color-base-content); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--color-primary); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-input:focus { border-color: var(--color-primary); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: var(--color-base-100); color: var(--color-base-content); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.tool-btn.primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: var(--color-primary); color: white; border-color: var(--color-primary); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: var(--color-base-200); border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: var(--color-base-content); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-select:focus { border-color: var(--color-primary); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--color-base-content); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); margin: 20px 0; }
</style>
