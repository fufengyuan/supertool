<template>
  <div class="random-tool">
    <h3>随机字符生成器</h3>

    <div class="tool-section">
      <h4>生成选项</h4>
      <div class="tool-row">
        <div>
          <label class="tool-label">长度</label>
          <input v-model.number="length" type="number" class="tool-input" style="width: 100px;" min="1" max="10000" />
        </div>
        <div>
          <label class="tool-label">数量</label>
          <input v-model.number="count" type="number" class="tool-input" style="width: 100px;" min="1" max="100" />
        </div>
        <div>
          <label class="tool-label">分隔符</label>
          <input v-model="separator" class="tool-input" style="width: 80px;" placeholder="\n" />
        </div>
      </div>

      <div class="tool-row">
        <label class="tool-checkbox">
          <input type="checkbox" v-model="uppercase" /> 大写字母 (A-Z)
        </label>
        <label class="tool-checkbox">
          <input type="checkbox" v-model="lowercase" /> 小写字母 (a-z)
        </label>
        <label class="tool-checkbox">
          <input type="checkbox" v-model="digits" /> 数字 (0-9)
        </label>
        <label class="tool-checkbox">
          <input type="checkbox" v-model="symbols" /> 特殊符号 (!@#$%...)
        </label>
        <label class="tool-checkbox">
          <input type="checkbox" v-model="spaces" /> 空格
        </label>
      </div>

      <div class="tool-row">
        <label class="tool-checkbox">
          <input type="checkbox" v-model="excludeSimilar" /> 排除相似字符 (0O, 1lI)
        </label>
        <label class="tool-checkbox">
          <input type="checkbox" v-model="noRepeat" /> 字符不重复
        </label>
      </div>
    </div>

    <div class="tool-row">
      <button class="tool-btn primary" @click="generate" :disabled="!hasCharset">生成</button>
      <button class="tool-btn" @click="copyResults" :disabled="!results">复制结果</button>
      <button class="tool-btn" @click="clearAll">清空</button>
    </div>

    <div class="tool-section" v-if="results">
      <h4>生成结果</h4>
      <textarea v-model="results" class="tool-textarea" readonly rows="8" placeholder="生成的随机字符将显示在这里..."></textarea>
      <div class="result-info">
        共生成 {{ resultCount }} 个，总字符数 {{ totalChars }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const length = ref(16)
const count = ref(5)
const separator = ref('\n')
const uppercase = ref(true)
const lowercase = ref(true)
const digits = ref(true)
const symbols = ref(false)
const spaces = ref(false)
const excludeSimilar = ref(false)
const noRepeat = ref(false)
const results = ref('')

const similarChars = '0O1lI|'

const charset = computed(() => {
  let chars = ''
  if (uppercase.value) chars += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'
  if (lowercase.value) chars += 'abcdefghijklmnopqrstuvwxyz'
  if (digits.value) chars += '0123456789'
  if (symbols.value) chars += '!@#$%^&*()_+-=[]{}|;:,.<>?/~`'
  if (spaces.value) chars += ' '

  if (excludeSimilar.value) {
    for (const c of similarChars) {
      chars = chars.replace(c, '')
    }
  }

  return chars
})

const hasCharset = computed(() => charset.value.length > 0)

const resultCount = computed(() => {
  if (!results.value) return 0
  return results.value.split(getSeparator()).length
})

const totalChars = computed(() => {
  if (!results.value) return 0
  return results.value.replace(/\n/g, '').length
})

function getSeparator(): string {
  return separator.value === '\\n' ? '\n' : separator.value === '\\t' ? '\t' : separator.value || '\n'
}

function generate() {
  if (!hasCharset.value) {
    toast.warning('请至少选择一种字符类型')
    return
  }

  if (noRepeat.value && length.value > charset.value.length) {
    toast.warning(`字符不重复模式下，长度不能超过字符集大小 (${charset.value.length})`)
    length.value = charset.value.length
    return
  }

  const chars = charset.value
  const sep = getSeparator()
  const generated: string[] = []

  for (let i = 0; i < count.value; i++) {
    let result = ''
    if (noRepeat.value) {
      // Shuffle and pick
      const shuffled = [...chars].sort(() => Math.random() - 0.5)
      result = shuffled.slice(0, length.value).join('')
    } else {
      for (let j = 0; j < length.value; j++) {
        result += chars[Math.floor(Math.random() * chars.length)]
      }
    }
    generated.push(result)
  }

  results.value = generated.join(sep)
  toast.success('生成成功')
}

async function copyResults() {
  if (!results.value) return
  await copyText(results.value, toast)
}

function clearAll() {
  results.value = ''
}
</script>

<style scoped>

.result-info {
  margin-top: 6px;
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
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
