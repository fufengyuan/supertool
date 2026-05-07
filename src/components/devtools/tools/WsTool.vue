<template>
  <div class="tool-panel">
    <h3>🔌 WebSocket 调试</h3>

    <div class="tool-section">
      <label class="tool-label">WebSocket URL</label>
      <input
        v-model="url"
        class="tool-input"
        placeholder="ws:// 或 wss://..."
      />

      <div class="tool-row" style="margin-top: 12px">
        <button
          v-if="status !== 'connected'"
          class="tool-btn primary"
          @click="connect"
          :disabled="status === 'connecting'"
        >
          {{ status === 'connecting' ? '连接中...' : '🔗 连接' }}
        </button>
        <button
          v-else
          class="tool-btn danger"
          @click="disconnect"
        >
          ⛔ 断开
        </button>
        <button class="tool-btn" @click="clearLog">🗑️ 清空日志</button>
      </div>

      <!-- Status indicator -->
      <div class="status-bar">
        <span class="status-dot" :class="status"></span>
        <span class="status-text">{{ statusText }}</span>
      </div>

      <!-- Message log -->
      <div ref="logContainer" class="message-log">
        <div v-if="messages.length === 0" class="empty-log">
          连接 WebSocket 服务器后，消息将显示在这里
        </div>
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="message-item"
          :class="msg.type"
        >
          <span class="message-time">{{ msg.time }}</span>
          <span class="message-type">{{ msg.typeLabel }}</span>
          <span class="message-content">{{ msg.content }}</span>
        </div>
      </div>

      <!-- Message input -->
      <div class="send-row">
        <input
          v-model="messageInput"
          class="tool-input"
          placeholder="输入要发送的消息..."
          @keyup.enter="sendMessage"
          :disabled="status !== 'connected'"
        />
        <button
          class="tool-btn primary"
          @click="sendMessage"
          :disabled="status !== 'connected'"
        >
          📤 发送
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
// styles in <style scoped>
import { useToast } from '@/composables/useToast'

const toast = useToast()
const url = ref('ws://localhost:8080')
const messageInput = ref('')
const messages = ref<Array<{
  id: string
  type: 'sent' | 'received' | 'system' | 'error'
  typeLabel: string
  content: string
  time: string
}>>([])
const status = ref<'disconnected' | 'connecting' | 'connected'>('disconnected')
const logContainer = ref<HTMLElement | null>(null)

let ws: WebSocket | null = null

const statusText = {
  disconnected: '未连接',
  connecting: '连接中...',
  connected: '已连接',
}[status.value] as string

watch(status, () => {
  // Update status text reactively - handled in template
})

function getNow(): string {
  return new Date().toLocaleTimeString('zh-CN', { hour12: false })
}

function addMessage(type: 'sent' | 'received' | 'system' | 'error', content: string) {
  const typeLabels: Record<string, string> = {
    sent: '→ 发送',
    received: '← 接收',
    system: '⚙ 系统',
    error: '✗ 错误',
  }
  messages.value.push({
    id: Date.now().toString(36) + Math.random().toString(36).slice(2, 5),
    type,
    typeLabel: typeLabels[type],
    content,
    time: getNow(),
  })
  scrollToBottom()
}

async function scrollToBottom() {
  await nextTick()
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

function connect() {
  if (!url.value.trim()) {
    toast.warning('请输入 WebSocket URL')
    return
  }

  const wsUrl = url.value.trim()
  if (!wsUrl.startsWith('ws://') && !wsUrl.startsWith('wss://')) {
    toast.warning('URL 必须以 ws:// 或 wss:// 开头')
    return
  }

  status.value = 'connecting'
  addMessage('system', `正在连接 ${wsUrl}...`)

  try {
    ws = new WebSocket(wsUrl)

    ws.onopen = () => {
      status.value = 'connected'
      addMessage('system', `✅ 已连接到 ${wsUrl}`)
    }

    ws.onmessage = (event) => {
      const data = typeof event.data === 'string' ? event.data : '[Binary]'
      addMessage('received', data)
    }

    ws.onclose = (event) => {
      status.value = 'disconnected'
      ws = null
      addMessage('system', `❌ 连接已关闭 (代码: ${event.code}, 原因: ${event.reason || '无'})`)
    }

    ws.onerror = (error) => {
      status.value = 'disconnected'
      addMessage('error', 'WebSocket 错误')
    }
  } catch (e: any) {
    status.value = 'disconnected'
    addMessage('error', `连接失败: ${e.message}`)
  }
}

function disconnect() {
  if (ws) {
    ws.close(1000, '用户主动断开')
    ws = null
  }
}

function sendMessage() {
  if (!messageInput.value.trim()) {
    toast.warning('请输入消息内容')
    return
  }
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    toast.warning('WebSocket 未连接')
    return
  }

  const msg = messageInput.value
  try {
    ws.send(msg)
    addMessage('sent', msg)
    messageInput.value = ''
  } catch (e: any) {
    addMessage('error', `发送失败: ${e.message}`)
  }
}

function clearLog() {
  messages.value = []
}
</script>

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--main-text);
  margin: 0 0 20px 0;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin-top: 12px;
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: 6px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--main-text-secondary);
}

.status-dot.connected {
  background: #22c55e;
  box-shadow: 0 0 6px #22c55e;
}

.status-dot.connecting {
  background: #f59e0b;
  animation: pulse 1s infinite;
}

.status-dot.disconnected {
  background: #ef4444;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.status-text {
  font-size: 13px;
  color: var(--main-text);
}

.message-log {
  margin-top: 12px;
  height: 300px;
  overflow-y: auto;
  padding: 12px;
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
}

.empty-log {
  color: var(--main-text-secondary);
  text-align: center;
  padding: 40px 0;
}

.message-item {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  border-bottom: 1px solid var(--border-color);
  line-height: 1.5;
}

.message-item:last-child {
  border-bottom: none;
}

.message-time {
  color: var(--main-text-secondary);
  flex-shrink: 0;
  min-width: 65px;
}

.message-type {
  flex-shrink: 0;
  min-width: 55px;
}

.message-item.sent .message-type {
  color: #3b82f6;
}

.message-item.received .message-type {
  color: #22c55e;
}

.message-item.system .message-type {
  color: #f59e0b;
}

.message-item.error .message-type {
  color: #ef4444;
}

.message-content {
  flex: 1;
  color: var(--main-text);
  word-break: break-all;
  white-space: pre-wrap;
}

.send-row {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.send-row .tool-input {
  flex: 1;
}

.tool-btn.danger {
  background: #ef4444;
  color: white;
  border-color: #ef4444;
}

.tool-btn.danger:hover {
  opacity: 0.9;
}

.tool-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
