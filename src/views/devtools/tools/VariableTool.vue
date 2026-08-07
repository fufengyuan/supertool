<template>
  <ToolPage
    icon="gitBranch"
    name="变量名转换"
    description="camelCase / snake_case / PascalCase / kebab-case 等 6 种格式实时互转，点击复制"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="arrowDown" size="12" /> 输入变量名</h4>
      <input
        v-model="input"
        class="input input-bordered w-full font-mono text-sm bg-base-200/60"
        placeholder="输入变量名（任意格式，如 userLoginCount / user_login_count）..."
        @input="convert"
      />
    </div>

    <div v-if="input" class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div
        v-for="item in results"
        :key="item.label"
        class="group bg-base-100 border border-base-content/10 rounded-xl p-4 cursor-pointer hover:border-primary/40 hover:shadow-md transition-all"
        title="点击复制"
        @click="copyValue(item.value)"
      >
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-[11px] font-medium text-base-content/50 font-mono">{{ item.label }}</span>
          <SvgIcon name="copy" size="12" class="opacity-0 group-hover:opacity-60 transition-opacity text-base-content/50" />
        </div>
        <div class="text-sm text-base-content font-mono break-all">{{ item.value || '—' }}</div>
      </div>
    </div>
    <div v-else class="py-16 text-center text-xs text-base-content/40 bg-base-100/50 border border-dashed border-base-content/15 rounded-xl">
      输入变量名后，这里实时显示 6 种命名格式
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
const input = ref('')

function parseWords(name: string): string[] {
  if (!name.trim()) {return []}
  // Try to split by common separators first
  let s = name.trim()

  // If contains underscores, hyphens, spaces, or dots
  if (s.includes('_') || s.includes('-') || s.includes(' ') || s.includes('.')) {
    return s.split(/[\s_\-.]+/).filter(Boolean).map(w => w.toLowerCase())
  }

  // Otherwise, try to split by camelCase/PascalCase boundaries
  const words: string[] = []
  let current = ''
  for (let i = 0; i < s.length; i++) {
    const ch = s[i]
    if (/[A-Z]/.test(ch) && current.length > 0) {
      words.push(current.toLowerCase())
      current = ch
    } else {
      current += ch
    }
  }
  if (current) {words.push(current.toLowerCase())}

  // Filter out empty strings
  return words.filter(Boolean)
}

function toCamelCase(words: string[]): string {
  if (words.length === 0) {return ''}
  return words[0] + words.slice(1).map(w => w.charAt(0).toUpperCase() + w.slice(1)).join('')
}

function toPascalCase(words: string[]): string {
  return words.map(w => w.charAt(0).toUpperCase() + w.slice(1)).join('')
}

function toSnakeCase(words: string[]): string {
  return words.join('_').toLowerCase()
}

function toConstantCase(words: string[]): string {
  return words.join('_').toUpperCase()
}

function toKebabCase(words: string[]): string {
  return words.join('-').toLowerCase()
}

function toSentenceCase(words: string[]): string {
  return words.map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ').toLowerCase().replace(/^./, m => m.toUpperCase())
}

const results = computed(() => {
  const words = parseWords(input.value)
  if (words.length === 0) {
    return formats.map(f => ({ ...f, value: '' }))
  }
  return formats.map(f => ({
    label: f.label,
    value: f.fn(words),
  }))
})

const formats = [
  { label: 'camelCase', fn: toCamelCase },
  { label: 'snake_case', fn: toSnakeCase },
  { label: 'CONSTANT_CASE', fn: toConstantCase },
  { label: 'PascalCase', fn: toPascalCase },
  { label: 'kebab-case', fn: toKebabCase },
  { label: 'Sentence case', fn: toSentenceCase },
]

function convert() {
  // reactive via computed
}

function copyValue(value: string) {
  if (!value) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(value, toast)
}
</script>
