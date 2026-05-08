<template>
  <div class="tool-panel">
    <h3>🔀 变量名格式转换</h3>

    <div class="tool-section">
      <label class="tool-label">输入变量名</label>
      <input
        v-model="input"
        class="tool-input"
        placeholder="输入变量名 (任意格式)..."
        @input="convert"
      />

      <div class="results-grid" style="margin-top: 16px">
        <div
          v-for="item in results"
          :key="item.label"
          class="result-card"
          @click="copyValue(item.value)"
        >
          <div class="result-label">{{ item.label }}</div>
          <div class="result-value">{{ item.value || '—' }}</div>
          <div class="result-copy-hint">点击复制</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const input = ref('')

function parseWords(name: string): string[] {
  if (!name.trim()) return []
  // Try to split by common separators first
  let s = name.trim()
  
  // If contains underscores, hyphens, spaces, or dots
  if (s.includes('_') || s.includes('-') || s.includes(' ') || s.includes('.')) {
    return s.split(/[\s_\-\.]+/).filter(Boolean).map(w => w.toLowerCase())
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
  if (current) words.push(current.toLowerCase())
  
  // Filter out empty strings
  return words.filter(Boolean)
}

function toCamelCase(words: string[]): string {
  if (words.length === 0) return ''
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
