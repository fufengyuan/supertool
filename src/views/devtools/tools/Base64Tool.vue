<template>
  <ToolPage
    icon="file"
    name="BASE64 编码/解码"
    description="文本与 BASE64 互转，支持文件编码/解码，可选 URL-safe 模式"
    @back="$emit('back')"
  >
    <!-- 模式与选项 -->
    <div class="flex flex-wrap items-center gap-x-4 gap-y-2 bg-base-100 border border-base-content/10 rounded-xl px-4 py-3">
      <div class="join">
        <button class="btn btn-sm join-item" :class="mode === 'encode' ? 'btn-primary' : 'btn-ghost'" @click="mode = 'encode'">编码</button>
        <button class="btn btn-sm join-item" :class="mode === 'decode' ? 'btn-primary' : 'btn-ghost'" @click="mode = 'decode'">解码</button>
      </div>
      <label class="flex items-center gap-2 text-xs text-base-content/70 cursor-pointer select-none">
        <input type="checkbox" v-model="urlSafe" class="checkbox checkbox-sm checkbox-primary" />
        URL-safe（`- _`，去除 `=` 填充）
      </label>
      <span v-if="parseError" class="text-error text-xs flex items-center gap-1">
        <SvgIcon name="alertTriangle" size="13" /> {{ parseError }}
      </span>
      <button class="btn btn-ghost btn-sm ml-auto" @click="swapMode" :disabled="!inputText" title="把输出作为输入并切换模式">
        <SvgIcon name="refresh" size="13" /> 互换
      </button>
    </div>

    <!-- 文本输入/输出 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[240px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5">
            <SvgIcon name="arrowDown" size="12" /> 输入
          </h4>
          <button class="btn btn-ghost btn-xs" @click="clearAll" :disabled="!inputText">清空</button>
        </div>
        <textarea
          v-model="inputText"
          class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60 focus:bg-base-200"
          :placeholder="mode === 'encode' ? '输入要编码的文本...' : '输入要解码的 Base64...'"
        ></textarea>
      </div>
      <div class="flex flex-col bg-base-100 border border-base-content/10 rounded-xl p-4 min-h-[240px]">
        <div class="flex items-center justify-between mb-2.5">
          <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5">
            <SvgIcon name="arrowUp" size="12" /> 输出
          </h4>
          <div class="flex gap-1.5">
            <button class="btn btn-ghost btn-xs" @click="useAsInput" :disabled="!outputText" title="把输出作为新的输入">→ 输入</button>
            <button class="btn btn-primary btn-xs" @click="copyResult" :disabled="!outputText">
              <SvgIcon name="copy" size="11" /> 复制
            </button>
          </div>
        </div>
        <textarea
          v-model="outputText"
          readonly
          class="textarea textarea-bordered w-full font-mono text-sm flex-1 resize-none bg-base-200/60"
          placeholder="结果将显示在这里..."
        ></textarea>
      </div>
    </div>

    <!-- 文件编码 / 解码 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
        <h4 class="text-xs font-semibold text-base-content/70 mb-3 flex items-center gap-1.5">
          <SvgIcon name="upload" size="12" /> 文件 → Base64
        </h4>
        <input type="file" ref="fileInput" @change="handleFile" class="file-input file-input-bordered file-input-sm w-full max-w-xs" />
        <div v-if="fileProcessing" class="mt-2 text-xs text-base-content/50 flex items-center gap-1.5">
          <span class="loading loading-spinner loading-xs" /> 正在处理文件...
        </div>
        <div v-if="fileBase64" class="mt-2">
          <div class="text-xs text-base-content/60 mb-1.5">
            <span class="font-mono">{{ fileName }}</span>
            <span class="text-base-content/40">（{{ fileSize }}）</span>
          </div>
          <div class="p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-[180px] overflow-y-auto">{{ fileBase64 }}</div>
          <div class="flex gap-2 mt-2.5">
            <button class="btn btn-ghost btn-xs" @click="copyFileBase64">复制 Base64</button>
            <button class="btn btn-outline btn-xs" @click="downloadBase64File">
              <SvgIcon name="download" size="11" /> 下载文件
            </button>
          </div>
        </div>
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
        <h4 class="text-xs font-semibold text-base-content/70 mb-3 flex items-center gap-1.5">
          <SvgIcon name="download" size="12" /> Base64 → 文件
        </h4>
        <textarea
          v-model="fileDecodeInput"
          class="textarea textarea-bordered w-full font-mono text-xs bg-base-200/60 min-h-[90px] mb-2.5"
          placeholder="粘贴 Base64 字符串..."
        ></textarea>
        <div class="flex flex-wrap gap-2 items-center">
          <input v-model="decodeFileName" class="input input-bordered input-sm font-mono text-xs flex-1 min-w-[160px]" placeholder="decoded_file.txt" />
          <button class="btn btn-primary btn-sm" @click="decodeFile" :disabled="!fileDecodeInput.trim()">
            <SvgIcon name="download" size="12" /> 解码并下载
          </button>
        </div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import { copyText, readFileAsArrayBuffer } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()

const mode = ref<'encode' | 'decode'>('encode')
const urlSafe = ref(false)
const inputText = ref('')

const result = computed<{ output: string; error: string }>(() => {
  const input = inputText.value.trim()
  if (!input) {
    return { output: '', error: '' }
  }

  try {
    if (mode.value === 'encode') {
      const encoded = btoa(unescape(encodeURIComponent(inputText.value)))
      return {
        output: urlSafe.value ? encoded.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '') : encoded,
        error: '',
      }
    } else {
      let str = input
      if (urlSafe.value) {
        str = str.replace(/-/g, '+').replace(/_/g, '/')
        // 补齐 padding
        const pad = str.length % 4
        if (pad) {str += '='.repeat(4 - pad)}
      }
      return { output: decodeURIComponent(escape(atob(str))), error: '' }
    }
  } catch (e: any) {
    return { output: '', error: `${mode.value === 'encode' ? '编码' : '解码'}失败: ${e.message}` }
  }
})

const outputText = computed(() => result.value.output)
const parseError = computed(() => result.value.error)

function copyResult() {
  if (!outputText.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(outputText.value, toast)
}

function useAsInput() {
  if (!outputText.value) {return}
  inputText.value = outputText.value
}

function swapMode() {
  if (outputText.value) {
    inputText.value = outputText.value
  }
  mode.value = mode.value === 'encode' ? 'decode' : 'encode'
}

function clearAll() {
  inputText.value = ''
}

// File encoding
const fileInput = ref<HTMLInputElement | null>(null)
const fileBase64 = ref('')
const fileName = ref('')
const fileSize = ref('')
const fileProcessing = ref(false)

// File decoding
const fileDecodeInput = ref('')
const decodeFileName = ref('decoded_file')

async function handleFile(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) {return}

  fileProcessing.value = true
  fileBase64.value = ''
  fileName.value = file.name
  fileSize.value = formatFileSize(file.size)

  try {
    const buffer = await readFileAsArrayBuffer(file)
    const binary = new Uint8Array(buffer)
    let binaryString = ''
    const chunkSize = 8192
    for (let i = 0; i < binary.length; i += chunkSize) {
      binaryString += String.fromCharCode.apply(null, Array.from(binary.subarray(i, i + chunkSize)) as unknown as number[])
    }
    fileBase64.value = btoa(binaryString)
  } catch (e: any) {
    toast.error(`文件读取失败: ${e.message}`)
  } finally {
    fileProcessing.value = false
    input.value = ''
  }
}

function copyFileBase64() {
  if (!fileBase64.value) {
    toast.warning('没有可复制的内容')
    return
  }
  copyText(fileBase64.value, toast)
}

function downloadBase64File() {
  if (!fileBase64.value) {
    toast.warning('没有可下载的内容')
    return
  }
  try {
    const binary = atob(fileBase64.value)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i)
    }
    const blob = new Blob([bytes])
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = fileName.value || 'download'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    toast.error(`下载失败: ${e.message}`)
  }
}

function decodeFile() {
  if (!fileDecodeInput.value.trim()) {
    toast.warning('请输入 Base64 字符串')
    return
  }

  try {
    let str = fileDecodeInput.value.trim()
    if (urlSafe.value) {
      str = str.replace(/-/g, '+').replace(/_/g, '/')
      const pad = str.length % 4
      if (pad) {str += '='.repeat(4 - pad)}
    }
    const binary = atob(str)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i)
    }
    const blob = new Blob([bytes])
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = decodeFileName.value || 'decoded_file'
    a.click()
    URL.revokeObjectURL(url)
    toast.success('文件已下载')
  } catch (e: any) {
    toast.error(`解码失败: ${e.message}`)
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) {return bytes + ' B'}
  if (bytes < 1024 * 1024) {return (bytes / 1024).toFixed(2) + ' KB'}
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}
</script>
