<template>
  <div>
    <h3 class="text-lg font-bold text-base-content mb-5">随机字符生成器</h3>

    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">生成选项</h4>
      <div class="flex flex-wrap gap-2.5 mb-3">
        <div>
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">长度</span>
          <input v-model.number="length" type="number" class="input input-bordered w-24" min="1" max="10000" />
        </div>
        <div>
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">数量</span>
          <input v-model.number="count" type="number" class="input input-bordered w-24" min="1" max="100" />
        </div>
        <div>
          <span class="label-text text-xs font-medium opacity-60 mb-1 block">分隔符</span>
          <input v-model="separator" class="input input-bordered w-20" placeholder="\n" />
        </div>
      </div>

      <div class="flex flex-wrap gap-2.5 mb-3">
        <label class="label cursor-pointer gap-1.5 text-sm text-base-content">
          <input type="checkbox" v-model="uppercase" class="checkbox" /> 大写字母 (A-Z)
        </label>
        <label class="label cursor-pointer gap-1.5 text-sm text-base-content">
          <input type="checkbox" v-model="lowercase" class="checkbox" /> 小写字母 (a-z)
        </label>
        <label class="label cursor-pointer gap-1.5 text-sm text-base-content">
          <input type="checkbox" v-model="digits" class="checkbox" /> 数字 (0-9)
        </label>
        <label class="label cursor-pointer gap-1.5 text-sm text-base-content">
          <input type="checkbox" v-model="symbols" class="checkbox" /> 特殊符号 (!@#$%...)
        </label>
        <label class="label cursor-pointer gap-1.5 text-sm text-base-content">
          <input type="checkbox" v-model="spaces" class="checkbox" /> 空格
        </label>
      </div>

      <div class="flex flex-wrap gap-2.5 mb-3">
        <label class="label cursor-pointer gap-1.5 text-sm text-base-content">
          <input type="checkbox" v-model="excludeSimilar" class="checkbox" /> 排除相似字符 (0O, 1lI)
        </label>
        <label class="label cursor-pointer gap-1.5 text-sm text-base-content">
          <input type="checkbox" v-model="noRepeat" class="checkbox" /> 字符不重复
        </label>
      </div>
    </div>

    <div class="flex flex-wrap gap-2.5 mb-3">
      <button class="btn btn-primary" @click="generate" :disabled="!hasCharset">生成</button>
      <button class="btn btn-ghost" @click="copyResults" :disabled="!results">复制结果</button>
      <button class="btn btn-ghost" @click="clearAll">清空</button>
    </div>

    <div class="mb-5" v-if="results">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">生成结果</h4>
      <textarea v-model="results" class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]" readonly rows="8" placeholder="生成的随机字符将显示在这里..."></textarea>
      <div class="mt-1.5 text-xs opacity-60">
        共生成 {{ resultCount }} 个，总字符数 {{ totalChars }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const length = ref(16)
const count = ref(5)
const separator = ref('\\n')
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
  if (uppercase.value) {chars += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'}
  if (lowercase.value) {chars += 'abcdefghijklmnopqrstuvwxyz'}
  if (digits.value) {chars += '0123456789'}
  if (symbols.value) {chars += '!@#$%^&*()_+-=[]{}|;:,.<>?/~`'}
  if (spaces.value) {chars += ' '}

  if (excludeSimilar.value) {
    for (const c of similarChars) {
      chars = chars.replace(c, '')
    }
  }

  return chars
})

const hasCharset = computed(() => charset.value.length > 0)

const resultCount = computed(() => {
  if (!results.value) {return 0}
  return results.value.split(getSeparator()).length
})

const totalChars = computed(() => {
  if (!results.value) {return 0}
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
  if (!results.value) {return}
  await copyText(results.value, toast)
}

function clearAll() {
  results.value = ''
}
</script>
