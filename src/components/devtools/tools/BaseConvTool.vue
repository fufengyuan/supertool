<template>
  <div class="tool-panel">
    <h3>🔣 进制转换</h3>

    <div class="tool-section">
      <div class="tool-row">
        <div>
          <label class="tool-label">输入进制</label>
          <select v-model.number="inputBase" class="tool-select">
            <option v-for="b in baseOptions" :key="b" :value="b">{{ b }}</option>
          </select>
        </div>
      </div>

      <label class="tool-label" style="margin-top: 12px">输入数值</label>
      <input
        v-model="input"
        class="tool-input"
        :placeholder="`请输入 ${inputBase} 进制数...`"
        @input="convert"
      />

      <div class="results-grid">
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
import { baseConvert, copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const input = ref('')
const inputBase = ref(10)

const baseOptions = computed(() => {
  const bases: number[] = []
  for (let i = 2; i <= 64; i++) {
    bases.push(i)
  }
  return bases
})

const targetBases = [
  { label: '二进制 (2)', base: 2 },
  { label: '八进制 (8)', base: 8 },
  { label: '十进制 (10)', base: 10 },
  { label: '十六进制 (16)', base: 16 },
  { label: 'Base32 (32)', base: 32 },
  { label: 'Base64 (64)', base: 64 },
]

const results = computed(() => {
  if (!input.value.trim()) return targetBases.map(b => ({ ...b, value: '' }))
  return targetBases.map(b => ({
    ...b,
    value: baseConvert(input.value, inputBase.value, b.base),
  }))
})

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
  margin-top: 16px;
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
