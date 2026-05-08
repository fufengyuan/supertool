<template>
  <div class="tool-panel">
    <h3>🅰️ ASCII 编码转换</h3>

    <div class="tool-section">
      <label class="tool-label">输入文本</label>
      <input
        v-model="input"
        class="tool-input"
        placeholder="输入任意文本..."
        @input="convert"
      />

      <div v-if="results.length > 0" class="ascii-table-wrapper">
        <table class="ascii-table">
          <thead>
            <tr>
              <th>字符</th>
              <th>十进制 (DEC)</th>
              <th>十六进制 (HEX)</th>
              <th>八进制 (OCT)</th>
              <th>二进制 (BIN)</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(r, idx) in results" :key="idx" :class="{ space: r.char === ' ' }">
              <td class="char-cell">{{ r.char === ' ' ? '⎵' : r.char }}</td>
              <td>{{ r.dec }}</td>
              <td>{{ r.hex }}</td>
              <td>{{ r.oct }}</td>
              <td>{{ r.bin }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="input" class="combined-results" style="margin-top: 16px">
        <div class="combined-item">
          <span class="combined-label">十进制</span>
          <span class="combined-value">{{ decStr }}</span>
          <button class="tool-btn copy-btn" @click="doCopy(decStr)">📋</button>
        </div>
        <div class="combined-item">
          <span class="combined-label">十六进制</span>
          <span class="combined-value">{{ hexStr }}</span>
          <button class="tool-btn copy-btn" @click="doCopy(hexStr)">📋</button>
        </div>
        <div class="combined-item">
          <span class="combined-label">八进制</span>
          <span class="combined-value">{{ octStr }}</span>
          <button class="tool-btn copy-btn" @click="doCopy(octStr)">📋</button>
        </div>
        <div class="combined-item">
          <span class="combined-label">二进制</span>
          <span class="combined-value">{{ binStr }}</span>
          <button class="tool-btn copy-btn" @click="doCopy(binStr)">📋</button>
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

interface CharInfo {
  char: string
  dec: string
  hex: string
  oct: string
  bin: string
}

const results = computed<CharInfo[]>(() => {
  if (!input.value) return []
  return Array.from(input.value).map(ch => {
    const code = ch.charCodeAt(0)
    return {
      char: ch,
      dec: String(code),
      hex: '0x' + code.toString(16).toUpperCase().padStart(2, '0'),
      oct: '0o' + code.toString(8).padStart(3, '0'),
      bin: code.toString(2).padStart(8, '0'),
    }
  })
})

const decStr = computed(() => results.value.map(r => r.dec).join(' '))
const hexStr = computed(() => results.value.map(r => r.hex).join(' '))
const octStr = computed(() => results.value.map(r => r.oct).join(' '))
const binStr = computed(() => results.value.map(r => r.bin).join(' '))

function convert() {
  // reactive via computed
}

function doCopy(text: string) {
  if (!text) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(text, toast)
}
</script>

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
  margin: 0 0 20px 0;
}

.ascii-table-wrapper {
  margin-top: 16px;
  overflow-x: auto;
  max-height: 300px;
  overflow-y: auto;
}

.ascii-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.ascii-table th {
  position: sticky;
  top: 0;
  background: var(--color-base-200);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-primary);
  text-align: left;
  padding: 8px 12px;
  border-bottom: 2px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  z-index: 1;
}

.ascii-table td {
  padding: 6px 12px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  color: var(--color-base-content);
}

.ascii-table tr.space td {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.char-cell {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-primary);
}

.combined-results {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.combined-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
}

.combined-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-primary);
  min-width: 90px;
  flex-shrink: 0;
}

.combined-value {
  flex: 1;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  color: var(--color-base-content);
  word-break: break-all;
}

.copy-btn {
  padding: 4px 8px !important;
  font-size: 12px !important;
  flex-shrink: 0;
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
