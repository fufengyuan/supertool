<template>
  <div class="tool-panel">
    <h3>🔐 哈希计算</h3>

    <!-- Single Input -->
    <div class="tool-section">
      <h4>文本哈希</h4>
      <label class="tool-label">输入文本</label>
      <textarea
        v-model="inputText"
        class="tool-textarea"
        placeholder="输入需要计算哈希的文本..."
      ></textarea>

      <div class="tool-row" style="margin-top: 12px;">
        <button class="tool-btn primary" @click="computeHashes">计算哈希</button>
        <button class="tool-btn" @click="clearSingle">清空</button>
      </div>

      <div v-if="singleResults" class="hash-results">
        <div class="hash-result-item" v-for="(hash, algo) in singleResults" :key="algo">
          <span class="hash-algo">{{ algo }}</span>
          <div class="hash-value">{{ hash }}</div>
          <button class="tool-btn copy-btn" @click="copyHash(hash)" title="复制">📋</button>
        </div>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- Batch Processing -->
    <div class="tool-section">
      <h4>批量哈希</h4>
      <label class="tool-label">每行一条文本，分别计算哈希</label>
      <textarea
        v-model="batchInput"
        class="tool-textarea"
        placeholder="第一行文本&#10;第二行文本&#10;第三行文本"
      ></textarea>

      <div class="tool-row" style="margin-top: 12px;">
        <button class="tool-btn primary" @click="computeBatch">批量计算</button>
        <button class="tool-btn" @click="clearBatch">清空</button>
      </div>

      <div v-if="batchResults.length" class="batch-results">
        <div class="batch-result-row" v-for="(row, idx) in batchResults" :key="idx">
          <span class="batch-index">{{ idx + 1 }}</span>
          <span class="batch-input">{{ row.input }}</span>
          <span class="batch-hash sha256">{{ row.sha256 }}</span>
          <span class="batch-hash sm3">{{ row.sm3 }}</span>
          <button class="tool-btn copy-btn" @click="copyHash(row.sha256 + '\\n' + row.sm3)" title="复制两个哈希">📋</button>
        </div>
      </div>
    </div>

    <hr class="tool-divider" />

    <!-- File Hash -->
    <div class="tool-section">
      <h4>文件哈希</h4>
      <input type="file" ref="fileInput" @change="handleFileHash" class="tool-file-input" />
      <div v-if="fileHashResults" class="hash-results">
        <div class="hash-result-item" v-for="(hash, algo) in fileHashResults" :key="algo">
          <span class="hash-algo">{{ algo }}</span>
          <div class="hash-value">{{ hash }}</div>
          <button class="tool-btn copy-btn" @click="copyHash(hash)" title="复制">📋</button>
        </div>
      </div>
      <div v-if="fileHashing" class="loading-text">正在计算文件哈希...</div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref } from 'vue'
import CryptoJS from 'crypto-js'
import { sm3 } from 'sm-crypto'
import { copyText, readFileAsArrayBuffer } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const inputText = ref('')
const singleResults = ref<Record<string, string> | null>(null)
const batchInput = ref('')
const batchResults = ref<{ input: string; sha256: string; sm3: string }[]>([])
const fileHashResults = ref<Record<string, string> | null>(null)
const fileHashing = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

function computeHashes() {
  if (!inputText.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  try {
    const text = inputText.value
    singleResults.value = {
      MD5: CryptoJS.MD5(text).toString(),
      SHA1: CryptoJS.SHA1(text).toString(),
      SHA224: CryptoJS.SHA224(text).toString(),
      SHA256: CryptoJS.SHA256(text).toString(),
      SHA384: CryptoJS.SHA384(text).toString(),
      SHA512: CryptoJS.SHA512(text).toString(),
      SHA3: CryptoJS.SHA3(text).toString(),
      RIPEMD160: CryptoJS.RIPEMD160(text).toString(),
      SM3: sm3(text),
    }
  } catch (e: any) {
    toast.error(`哈希计算失败: ${e.message}`)
  }
}

function clearSingle() {
  inputText.value = ''
  singleResults.value = null
}

function computeBatch() {
  const lines = batchInput.value.split('\n').filter(l => l.trim())
  if (!lines.length) {
    toast.warning('请输入至少一行文本')
    return
  }
  try {
    batchResults.value = lines.map(line => ({
      input: line,
      sha256: CryptoJS.SHA256(line).toString(),
      sm3: sm3(line),
    }))
    toast.success(`已处理 ${lines.length} 行`)
  } catch (e: any) {
    toast.error(`批量处理失败: ${e.message}`)
  }
}

function clearBatch() {
  batchInput.value = ''
  batchResults.value = []
}

async function handleFileHash(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  fileHashing.value = true
  fileHashResults.value = null

  try {
    const buffer = await readFileAsArrayBuffer(file)
    const wordArray = CryptoJS.lib.WordArray.create(new Uint8Array(buffer))

    const sm3Text = new TextDecoder().decode(new Uint8Array(buffer))
    fileHashResults.value = {
      MD5: CryptoJS.MD5(wordArray).toString(),
      SHA1: CryptoJS.SHA1(wordArray).toString(),
      SHA256: CryptoJS.SHA256(wordArray).toString(),
      SHA512: CryptoJS.SHA512(wordArray).toString(),
      SM3: sm3(sm3Text),
    }
  } catch (e: any) {
    toast.error(`文件哈希计算失败: ${e.message}`)
  } finally {
    fileHashing.value = false
    input.value = ''
  }
}

function copyHash(hash: string) {
  copyText(hash, toast)
}
</script>

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--main-text);
  margin: 0 0 20px 0;
}

.hash-results {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.hash-result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.hash-algo {
  font-size: 12px;
  font-weight: 600;
  color: var(--primary-color);
  min-width: 60px;
  flex-shrink: 0;
}

.hash-value {
  flex: 1;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  color: var(--main-text);
  word-break: break-all;
  overflow: hidden;
  text-overflow: ellipsis;
}

.copy-btn {
  padding: 4px 8px !important;
  font-size: 12px !important;
  flex-shrink: 0;
}

.batch-results {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 300px;
  overflow-y: auto;
}

.batch-result-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 12px;
}

.batch-index {
  color: var(--main-text-secondary);
  min-width: 24px;
  text-align: center;
}

.batch-input {
  color: var(--main-text);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.batch-hash {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  color: var(--primary-color);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tool-file-input {
  padding: 8px 0;
  font-size: 13px;
  color: var(--main-text);
}

.loading-text {
  margin-top: 12px;
  font-size: 13px;
  color: var(--main-text-secondary);
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--main-text); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid var(--border-color); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--input-bg); color: var(--main-text); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--primary-color); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid var(--border-color); border-radius: 6px; font-size: 13px; background: var(--input-bg); color: var(--main-text); outline: none; }
.tool-input:focus { border-color: var(--primary-color); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid var(--border-color); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: var(--card-bg); color: var(--main-text); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: var(--primary-color); color: var(--primary-color); }
.tool-btn.primary { background: var(--primary-color); color: white; border-color: var(--primary-color); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: var(--primary-color); color: white; border-color: var(--primary-color); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: var(--input-bg); border: 1px solid var(--border-color); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: var(--main-text); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: var(--main-text-secondary); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid var(--border-color); border-radius: 6px; font-size: 13px; background: var(--input-bg); color: var(--main-text); outline: none; }
.tool-select:focus { border-color: var(--primary-color); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--main-text); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid var(--border-color); margin: 20px 0; }
</style>
