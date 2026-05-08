<template>
  <div class="uuid-tool">
    <h3>🆔 UUID 生成器</h3>

    <div class="tool-section">
      <h4>批量生成 UUID v4</h4>
      <div class="tool-row">
        <div>
          <label class="tool-label">生成数量</label>
          <input
            v-model.number="uuidCount"
            type="number"
            class="tool-input mono"
            style="width: 80px"
            min="1"
            max="100"
          />
        </div>
        <button class="tool-btn primary" @click="generateUUIDs">🔄 生成</button>
        <button class="tool-btn" @click="copyText(uuidOutput, toast)">📋 复制全部</button>
        <button class="tool-btn" @click="downloadUUIDs">⬇️ 下载 TXT</button>
      </div>

      <div v-if="uuids.length > 0" class="tool-result uuid-list">
        <div v-for="(u, i) in uuids" :key="i" class="uuid-item">{{ u }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import { copyText, downloadFile } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const uuidCount = ref(5)
const uuids = ref<string[]>([])
const uuidOutput = ref('')

function generateUUIDs() {
  let count = uuidCount.value
  if (count < 1) count = 1
  if (count > 100) {
    count = 100
    toast.warning('最多生成 100 个')
  }
  uuids.value = []
  for (let i = 0; i < count; i++) {
    uuids.value.push(uuidv4())
  }
  uuidOutput.value = uuids.value.join('\n')
  toast.success(`已生成 ${count} 个 UUID`)
}

function downloadUUIDs() {
  if (uuids.value.length === 0) { toast.error('请先生成 UUID'); return }
  downloadFile(uuidOutput.value, 'uuids.txt', 'text/plain')
  toast.success('UUID 文件已下载')
}
</script>

<style scoped>

.uuid-tool {
  max-width: 700px;
}

.uuid-tool h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
  margin: 0 0 20px 0;
}

.tool-section h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-base-content);
  margin: 0 0 10px 0;
}

.tool-row {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
  flex-wrap: wrap;
  align-items: flex-end;
}

.tool-result {
  margin-top: 10px;
  padding: 10px 12px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--color-base-content);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 400px;
  overflow-y: auto;
}

.uuid-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.uuid-item {
  padding: 2px 0;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.uuid-item:last-child {
  border-bottom: none;
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

.tool-btn {
  padding: 7px 16px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  background: var(--color-base-100);
  color: var(--color-base-content);
  transition: all 0.15s;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tool-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.tool-btn.primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.tool-btn.primary:hover {
  opacity: 0.9;
}

.tool-divider {
  border: none;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  margin: 20px 0;
}

.tool-select {
  padding: 7px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  outline: none;
}

.tool-label {
  font-size: 12px;
  font-weight: 500;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-bottom: 4px;
  display: block;
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
