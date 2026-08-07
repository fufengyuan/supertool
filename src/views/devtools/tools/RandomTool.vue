<template>
  <ToolPage
    icon="sparkles"
    name="随机生成器"
    description="自定义字符集批量生成随机字符串，支持排除相似字符与不重复模式"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-3 flex items-center gap-1.5"><SvgIcon name="filter" size="12" /> 生成选项</h4>
      <div class="flex flex-wrap gap-3 mb-3">
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">长度</span>
          <input v-model.number="length" type="number" class="input input-bordered input-sm w-24 bg-base-200/60" min="1" max="10000" />
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">数量</span>
          <input v-model.number="count" type="number" class="input input-bordered input-sm w-24 bg-base-200/60" min="1" max="100" />
        </div>
        <div>
          <span class="text-[11px] font-medium text-base-content/50 mb-1 block">分隔符</span>
          <input v-model="separator" class="input input-bordered input-sm w-20 bg-base-200/60" placeholder="\n" />
        </div>
      </div>
      <div class="flex flex-wrap gap-x-4 gap-y-2 mb-3">
        <label class="flex items-center gap-1.5 text-xs text-base-content/80 cursor-pointer select-none">
          <input type="checkbox" v-model="uppercase" class="checkbox checkbox-sm checkbox-primary" /> 大写字母 (A-Z)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content/80 cursor-pointer select-none">
          <input type="checkbox" v-model="lowercase" class="checkbox checkbox-sm checkbox-primary" /> 小写字母 (a-z)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content/80 cursor-pointer select-none">
          <input type="checkbox" v-model="digits" class="checkbox checkbox-sm checkbox-primary" /> 数字 (0-9)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content/80 cursor-pointer select-none">
          <input type="checkbox" v-model="symbols" class="checkbox checkbox-sm checkbox-primary" /> 特殊符号 (!@#$%...)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content/80 cursor-pointer select-none">
          <input type="checkbox" v-model="spaces" class="checkbox checkbox-sm checkbox-primary" /> 空格
        </label>
      </div>
      <div class="flex flex-wrap gap-x-4 gap-y-2">
        <label class="flex items-center gap-1.5 text-xs text-base-content/80 cursor-pointer select-none">
          <input type="checkbox" v-model="excludeSimilar" class="checkbox checkbox-sm checkbox-primary" /> 排除相似字符 (0O, 1lI)
        </label>
        <label class="flex items-center gap-1.5 text-xs text-base-content/80 cursor-pointer select-none">
          <input type="checkbox" v-model="noRepeat" class="checkbox checkbox-sm checkbox-primary" /> 字符不重复
        </label>
      </div>
      <div class="flex gap-2 mt-4 pt-3 border-t border-base-content/10">
        <button class="btn btn-primary btn-sm" @click="generate" :disabled="!hasCharset">生成</button>
        <button class="btn btn-outline btn-sm" @click="copyResults" :disabled="!results"><SvgIcon name="copy" size="11" /> 复制</button>
        <button class="btn btn-ghost btn-sm" @click="clearAll" :disabled="!results">清空</button>
      </div>
    </div>

    <div v-if="results" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex items-center justify-between mb-2.5">
        <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="sparkles" size="12" /> 生成结果</h4>
        <span class="text-[11px] text-base-content/40">{{ resultCount }} 条 · 总 {{ totalChars }} 字符</span>
      </div>
      <textarea v-model="results" readonly class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px] resize-none bg-base-200/60" placeholder="生成的随机字符将显示在这里..."></textarea>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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
