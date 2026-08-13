<template>
  <ToolPage
    icon="unlock"
    name="JWT 解码"
    description="解析 JWT Token 的 Header 与 Payload，展示常用声明字段"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="key" size="12" /> JWT Token</h4>
      <textarea
        v-model="input"
        class="textarea textarea-bordered w-full font-mono text-sm bg-base-200/60 min-h-[110px] resize-none"
        placeholder="输入 JWT token (eyJ...)"
        @input="decode"
      ></textarea>
      <div class="flex flex-wrap gap-2 mt-3">
        <button class="btn btn-primary btn-sm" @click="decode">解码</button>
        <button class="btn btn-outline btn-sm" @click="copyHeader" :disabled="!header">复制 Header</button>
        <button class="btn btn-outline btn-sm" @click="copyPayload" :disabled="!payload">复制 Payload</button>
        <button class="btn btn-ghost btn-sm ml-auto" @click="clear" :disabled="!input">清空</button>
      </div>
    </div>

    <div v-if="error" class="p-3.5 bg-error/10 border border-error/30 rounded-xl text-error text-sm flex items-center gap-2">
      <SvgIcon name="alertTriangle" size="15" /> {{ error }}
    </div>

    <!-- Header -->
    <div v-if="header" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="fileText" size="12" /> Header</h4>
      <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-56 overflow-y-auto">{{ header }}</div>
      <div v-if="headerObj" class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2 mt-3">
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">算法 (alg)</span>
          <span class="text-sm text-base-content break-all font-mono">{{ headerObj.alg || '—' }}</span>
        </div>
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">类型 (typ)</span>
          <span class="text-sm text-base-content break-all font-mono">{{ headerObj.typ || '—' }}</span>
        </div>
      </div>
    </div>

    <!-- Payload -->
    <div v-if="payload" class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="fileText" size="12" /> Payload</h4>
      <div class="p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-56 overflow-y-auto">{{ payload }}</div>
      <div v-if="payloadObj" class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2 mt-3">
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">签发者 (iss)</span>
          <span class="text-sm text-base-content break-all">{{ payloadObj.iss || '—' }}</span>
        </div>
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">主体 (sub)</span>
          <span class="text-sm text-base-content break-all">{{ payloadObj.sub || '—' }}</span>
        </div>
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">签发时间 (iat)</span>
          <span class="text-sm text-base-content break-all">{{ payloadObj.iat ? formatDate(payloadObj.iat) : '—' }}</span>
        </div>
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">过期时间 (exp)</span>
          <span class="text-sm text-base-content break-all">{{ payloadObj.exp ? formatDate(payloadObj.exp) : '—' }}</span>
        </div>
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">生效时间 (nbf)</span>
          <span class="text-sm text-base-content break-all">{{ payloadObj.nbf ? formatDate(payloadObj.nbf) : '—' }}</span>
        </div>
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">JWT ID (jti)</span>
          <span class="text-sm text-base-content break-all font-mono">{{ payloadObj.jti || '—' }}</span>
        </div>
        <div class="flex flex-col p-2.5 bg-base-200/60 border border-base-content/10 rounded-lg">
          <span class="text-[11px] font-medium text-base-content/50 mb-1">受众 (aud)</span>
          <span class="text-sm text-base-content break-all">{{ payloadObj.aud || '—' }}</span>
        </div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

const toast = useToast()
const input = ref('')
const header = ref('')
const payload = ref('')
const headerObj = ref<Record<string, any> | null>(null)
const payloadObj = ref<Record<string, any> | null>(null)
const error = ref('')

function base64UrlDecode(str: string): string {
  // Replace URL-safe chars with standard base64 chars
  let base64 = str.replace(/-/g, '+').replace(/_/g, '/')
  // Pad with '=' if needed
  while (base64.length % 4) {
    base64 += '='
  }
  // JWT header/payload 是对 UTF-8 字节做的 base64url，atob 得到 Latin-1 字符串，
  // 必须按 UTF-8 解码，否则中文 payload（如 name:张三）会乱码
  const bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0))
  return new TextDecoder('utf-8').decode(bytes)
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })
}

function decode() {
  error.value = ''
  header.value = ''
  payload.value = ''
  headerObj.value = null
  payloadObj.value = null

  if (!input.value.trim()) {
    toast.warning('请输入 JWT token')
    return
  }

  const token = input.value.trim()
  const parts = token.split('.')

  if (parts.length !== 3) {
    error.value = '无效的 JWT 格式：必须包含 3 个部分（Header.Payload.Signature）'
    return
  }

  try {
    const headerStr = base64UrlDecode(parts[0])
    headerObj.value = JSON.parse(headerStr)
    header.value = JSON.stringify(headerObj.value, null, 2)
  } catch (e: any) {
    error.value = `Header 解码失败: ${e.message}`
    return
  }

  try {
    const payloadStr = base64UrlDecode(parts[1])
    payloadObj.value = JSON.parse(payloadStr)
    payload.value = JSON.stringify(payloadObj.value, null, 2)
  } catch (e: any) {
    error.value = `Payload 解码失败: ${e.message}`
    return
  }
}

function copyHeader() {
  if (!header.value) {
    toast.warning('没有 Header 内容可复制')
    return
  }
  copyText(header.value, toast)
}

function copyPayload() {
  if (!payload.value) {
    toast.warning('没有 Payload 内容可复制')
    return
  }
  copyText(payload.value, toast)
}

function clear() {
  input.value = ''
  header.value = ''
  payload.value = ''
  headerObj.value = null
  payloadObj.value = null
  error.value = ''
}
</script>
