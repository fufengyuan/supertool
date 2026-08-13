<template>
  <ToolPage
    icon="lock"
    name="哈希计算"
    description="MD5 / SHA 系列 / SHA3 / RIPEMD160 / SM3，支持文本、批量与文件哈希"
    @back="$emit('back')"
  >
    <!-- 文本哈希 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex items-center justify-between mb-2.5">
        <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="fileText" size="12" /> 文本哈希</h4>
        <label class="flex items-center gap-1 text-[11px] text-base-content/50">
          输出格式
          <select v-model="outputFormat" class="select select-xs select-bordered bg-base-200/60 w-[90px]" title="哈希摘要的输出编码：Hex（默认，常见）或 Base64（API 签名/二进制场景常用）">
            <option value="hex">Hex</option>
            <option value="base64">Base64</option>
          </select>
        </label>
      </div>
      <textarea
        v-model="inputText"
        class="textarea textarea-bordered w-full min-h-[100px] font-mono text-xs bg-base-200/60 resize-none"
        placeholder="输入需要计算哈希的文本..."
      ></textarea>
      <div class="flex gap-2 mt-3">
        <button class="btn btn-primary btn-sm" @click="computeHashes">计算哈希</button>
        <button class="btn btn-ghost btn-sm" @click="clearSingle" :disabled="!inputText">清空</button>
      </div>
      <div v-if="singleResults" class="flex flex-col gap-2 mt-4">
        <div
          v-for="(hash, algo) in singleResults"
          :key="algo"
          class="flex items-center gap-2.5 p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg group"
        >
          <span class="text-[11px] font-semibold text-primary w-20 shrink-0 font-mono">{{ algo }}</span>
          <span class="flex-1 font-mono text-xs text-base-content break-all">{{ hash }}</span>
          <button class="btn btn-ghost btn-xs shrink-0 opacity-0 group-hover:opacity-100 transition-opacity" @click="copyHash(hash)" title="复制"><SvgIcon name="copy" size="12" /></button>
        </div>
      </div>
    </div>

    <!-- 批量哈希 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="list" size="12" /> 批量哈希</h4>
      <p class="text-[11px] text-base-content/50 mb-2">每行一条文本，分别计算 SHA256 与 SM3</p>
      <textarea
        v-model="batchInput"
        class="textarea textarea-bordered w-full min-h-[100px] font-mono text-xs bg-base-200/60 resize-none"
        placeholder="第一行文本&#10;第二行文本&#10;第三行文本"
      ></textarea>
      <div class="flex gap-2 mt-3">
        <button class="btn btn-primary btn-sm" @click="computeBatch">批量计算</button>
        <button class="btn btn-ghost btn-sm" @click="clearBatch" :disabled="!batchInput">清空</button>
      </div>
      <div v-if="batchResults.length" class="flex flex-col gap-1.5 mt-4 max-h-56 overflow-y-auto">
        <div v-for="(row, idx) in batchResults" :key="idx" class="flex items-center gap-2.5 p-2 bg-base-200/60 border border-base-content/10 rounded-lg group">
          <span class="text-base-content/40 text-[11px] w-6 text-right shrink-0">{{ idx + 1 }}</span>
          <div class="flex-1 min-w-0">
            <div class="text-xs text-base-content/70 truncate">{{ row.input }}</div>
            <div class="font-mono text-[11px] text-base-content/80 break-all">SHA256: {{ row.sha256 }}</div>
            <div class="font-mono text-[11px] text-base-content/80 break-all">SM3: {{ row.sm3 }}</div>
          </div>
          <button class="btn btn-ghost btn-xs shrink-0 opacity-0 group-hover:opacity-100 transition-opacity" @click="copyHash(row.sha256 + '\n' + row.sm3)" title="复制两个哈希"><SvgIcon name="copy" size="12" /></button>
        </div>
      </div>
    </div>

    <!-- 文件哈希 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="upload" size="12" /> 文件哈希</h4>
      <input type="file" ref="fileInput" @change="handleFileHash" class="file-input file-input-bordered file-input-sm w-full max-w-xs" />
      <div v-if="fileHashing" class="mt-3 text-xs text-base-content/60 flex items-center gap-1.5">
        <span class="loading loading-spinner loading-xs" /> 正在计算文件哈希...
      </div>
      <div v-if="fileHashResults" class="flex flex-col gap-2 mt-4">
        <div
          v-for="(hash, algo) in fileHashResults"
          :key="algo"
          class="flex items-center gap-2.5 p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg group"
        >
          <span class="text-[11px] font-semibold text-primary w-16 shrink-0 font-mono">{{ algo }}</span>
          <span class="flex-1 font-mono text-xs text-base-content break-all">{{ hash }}</span>
          <button class="btn btn-ghost btn-xs shrink-0 opacity-0 group-hover:opacity-100 transition-opacity" @click="copyHash(hash)" title="复制"><SvgIcon name="copy" size="12" /></button>
        </div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import CryptoJS from 'crypto-js'
import { sm3 } from 'sm-crypto'
import { copyText, readFileAsArrayBuffer } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()
const inputText = ref('')
const singleResults = ref<Record<string, string> | null>(null)
const batchInput = ref('')
const batchResults = ref<{ input: string; sha256: string; sm3: string }[]>([])
const fileHashResults = ref<Record<string, string> | null>(null)
const fileHashing = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
// 摘要输出编码：hex（默认）/ base64（API 签名、二进制场景）
const outputFormat = ref<'hex' | 'base64'>('hex')

// sm-crypto 的 sm3 只输出 hex 字符串，需 base64 时手动转换
function hexToBase64(hex: string): string {
  const clean = hex.replace(/\s+/g, '')
  const bytes = new Uint8Array(clean.length / 2)
  for (let i = 0; i < clean.length; i += 2) {bytes[i / 2] = parseInt(clean.substring(i, i + 2), 16)}
  let bin = ''
  for (let i = 0; i < bytes.length; i++) {bin += String.fromCharCode(bytes[i])}
  return btoa(bin)
}

// 统一摘要输出：CryptoJS WordArray 或 sm3 的 hex 字符串 → 按所选格式编码
function formatDigest(digest: any): string {
  if (typeof digest === 'string') {
    return outputFormat.value === 'hex' ? digest : hexToBase64(digest)
  }
  return outputFormat.value === 'hex' ? digest.toString() : digest.toString(CryptoJS.enc.Base64)
}

function computeHashes() {
  if (!inputText.value.trim()) {
    toast.warning('请输入文本')
    return
  }
  try {
    const text = inputText.value
    singleResults.value = {
      MD5: formatDigest(CryptoJS.MD5(text)),
      SHA1: formatDigest(CryptoJS.SHA1(text)),
      SHA224: formatDigest(CryptoJS.SHA224(text)),
      SHA256: formatDigest(CryptoJS.SHA256(text)),
      SHA384: formatDigest(CryptoJS.SHA384(text)),
      SHA512: formatDigest(CryptoJS.SHA512(text)),
      SHA3: formatDigest(CryptoJS.SHA3(text)),
      RIPEMD160: formatDigest(CryptoJS.RIPEMD160(text)),
      SM3: formatDigest(sm3(text)),
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
      sha256: formatDigest(CryptoJS.SHA256(line)),
      sm3: formatDigest(sm3(line)),
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
  if (!file) {return}

  fileHashing.value = true
  fileHashResults.value = null

  try {
    const buffer = await readFileAsArrayBuffer(file)
    const bytes = new Uint8Array(buffer)
    const wordArray = CryptoJS.lib.WordArray.create(bytes)
    // sm-crypto 的 sm3 接受 Array<number>，避免 TextDecoder 破坏二进制数据
    const byteArray = Array.from(bytes)

    fileHashResults.value = {
      MD5: formatDigest(CryptoJS.MD5(wordArray)),
      SHA1: formatDigest(CryptoJS.SHA1(wordArray)),
      SHA256: formatDigest(CryptoJS.SHA256(wordArray)),
      SHA512: formatDigest(CryptoJS.SHA512(wordArray)),
      SM3: formatDigest(sm3(byteArray)),
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
