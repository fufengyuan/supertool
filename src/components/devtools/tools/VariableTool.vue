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

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: oklch(var(--bc));
  margin: 0 0 20px 0;
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 12px;
}

.result-card {
  padding: 12px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.result-card:hover {
  border-color: oklch(var(--p));
}

.result-label {
  font-size: 12px;
  font-weight: 600;
  color: oklch(var(--p));
  margin-bottom: 6px;
}

.result-value {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  color: oklch(var(--bc));
  word-break: break-all;
  min-height: 20px;
}

.result-copy-hint {
  font-size: 11px;
  color: oklch(var(--bc) / 0.6);
  margin-top: 4px;
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: oklch(var(--bc)); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: oklch(var(--p)); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); outline: none; }
.tool-input:focus { border-color: oklch(var(--p)); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: oklch(var(--b1)); color: oklch(var(--bc)); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: oklch(var(--p)); color: oklch(var(--p)); }
.tool-btn.primary { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: oklch(var(--b2)); border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: oklch(var(--bc)); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: oklch(var(--bc) / 0.6); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); outline: none; }
.tool-select:focus { border-color: oklch(var(--p)); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: oklch(var(--bc)); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid oklch(var(--bc) / 0.1); margin: 20px 0; }
</style>
