<template>
  <div>
    <h3 class="text-lg font-bold text-base-content mb-5"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg> JWT 解码</h3>

    <div class="mb-5">
      <span class="label-text text-xs font-medium opacity-60 mb-1 block">JWT Token</span>
      <textarea
        v-model="input"
        class="textarea textarea-bordered w-full font-mono text-sm min-h-[120px]"
        placeholder="输入 JWT token (eyJ...)"
        rows="3"
        @input="decode"
      ></textarea>

      <div class="flex flex-wrap gap-2.5 mb-3 mt-3">
        <button class="btn btn-primary" @click="decode">解码</button>
        <button class="btn btn-ghost" @click="copyHeader"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 复制 Header</button>
        <button class="btn btn-ghost" @click="copyPayload"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 复制 Payload</button>
        <button class="btn btn-ghost" @click="clear">清空</button>
      </div>

      <div v-if="error" class="mt-3 p-3 bg-error/10 border border-error/30 rounded-box text-error text-sm">{{ error }}</div>

      <div v-if="header" class="mt-4">
        <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> Header</h4>
        <div class="bg-base-200 border border-base-content/10 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all max-h-72 overflow-y-auto">{{ header }}</div>
        <div v-if="headerObj" class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2 mt-3">
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">算法 (alg)</span>
            <span class="text-sm text-base-content break-all">{{ headerObj.alg || '—' }}</span>
          </div>
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">类型 (typ)</span>
            <span class="text-sm text-base-content break-all">{{ headerObj.typ || '—' }}</span>
          </div>
        </div>
      </div>

      <div v-if="payload" class="mt-4">
        <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> Payload</h4>
        <div class="bg-base-200 border border-base-content/10 rounded-box p-3 font-mono text-sm whitespace-pre-wrap break-all max-h-72 overflow-y-auto">{{ payload }}</div>
        <div v-if="payloadObj" class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2 mt-3">
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">签发者 (iss)</span>
            <span class="text-sm text-base-content break-all">{{ payloadObj.iss || '—' }}</span>
          </div>
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">主体 (sub)</span>
            <span class="text-sm text-base-content break-all">{{ payloadObj.sub || '—' }}</span>
          </div>
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">签发时间 (iat)</span>
            <span class="text-sm text-base-content break-all">{{ payloadObj.iat ? formatDate(payloadObj.iat) : '—' }}</span>
          </div>
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">过期时间 (exp)</span>
            <span class="text-sm text-base-content break-all">{{ payloadObj.exp ? formatDate(payloadObj.exp) : '—' }}</span>
          </div>
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">生效时间 (nbf)</span>
            <span class="text-sm text-base-content break-all">{{ payloadObj.nbf ? formatDate(payloadObj.nbf) : '—' }}</span>
          </div>
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">JWT ID (jti)</span>
            <span class="text-sm text-base-content break-all">{{ payloadObj.jti || '—' }}</span>
          </div>
          <div class="flex flex-col p-2 bg-base-200 border border-base-content/10 rounded-box">
            <span class="text-xs font-medium opacity-60 mb-1">受众 (aud)</span>
            <span class="text-sm text-base-content break-all">{{ payloadObj.aud || '—' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

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
  return atob(base64)
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
