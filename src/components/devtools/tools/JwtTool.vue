<template>
  <div class="tool-panel">
    <h3>🔑 JWT 解码</h3>

    <div class="tool-section">
      <label class="tool-label">JWT Token</label>
      <textarea
        v-model="input"
        class="tool-textarea"
        placeholder="输入 JWT token (eyJ...)"
        rows="3"
        @input="decode"
      ></textarea>

      <div class="tool-row" style="margin-top: 12px">
        <button class="tool-btn primary" @click="decode">解码</button>
        <button class="tool-btn" @click="copyHeader">📋 复制 Header</button>
        <button class="tool-btn" @click="copyPayload">📋 复制 Payload</button>
        <button class="tool-btn" @click="clear">清空</button>
      </div>

      <div v-if="error" class="error-box">{{ error }}</div>

      <div v-if="header" class="result-block">
        <h4>📋 Header</h4>
        <div class="tool-result">{{ header }}</div>
        <div v-if="headerObj" class="info-grid">
          <div class="info-item">
            <span class="info-label">算法 (alg)</span>
            <span class="info-value">{{ headerObj.alg || '—' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">类型 (typ)</span>
            <span class="info-value">{{ headerObj.typ || '—' }}</span>
          </div>
        </div>
      </div>

      <div v-if="payload" class="result-block">
        <h4>📋 Payload</h4>
        <div class="tool-result">{{ payload }}</div>
        <div v-if="payloadObj" class="info-grid">
          <div class="info-item">
            <span class="info-label">签发者 (iss)</span>
            <span class="info-value">{{ payloadObj.iss || '—' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">主体 (sub)</span>
            <span class="info-value">{{ payloadObj.sub || '—' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">签发时间 (iat)</span>
            <span class="info-value">{{ payloadObj.iat ? formatDate(payloadObj.iat) : '—' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">过期时间 (exp)</span>
            <span class="info-value">{{ payloadObj.exp ? formatDate(payloadObj.exp) : '—' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">生效时间 (nbf)</span>
            <span class="info-value">{{ payloadObj.nbf ? formatDate(payloadObj.nbf) : '—' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">JWT ID (jti)</span>
            <span class="info-value">{{ payloadObj.jti || '—' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">受众 (aud)</span>
            <span class="info-value">{{ payloadObj.aud || '—' }}</span>
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

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--main-text);
  margin: 0 0 20px 0;
}

.result-block {
  margin-top: 16px;
}

.result-block h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--main-text);
  margin: 0 0 10px 0;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
  margin-top: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  padding: 8px 12px;
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: 6px;
}

.info-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--main-text-secondary);
  margin-bottom: 4px;
}

.info-value {
  font-size: 13px;
  color: var(--main-text);
  word-break: break-all;
}

.error-box {
  margin-top: 12px;
  padding: 10px 12px;
  background: #fee2e2;
  border: 1px solid #fca5a5;
  border-radius: 8px;
  color: #dc2626;
  font-size: 13px;
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
