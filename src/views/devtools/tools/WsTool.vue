<template>
  <div class="flex flex-col gap-0 p-0">
    <h3 class="text-lg font-bold text-base-content m-0 mb-5">🔌 WebSocket 调试</h3>

    <div class="mb-5">
      <label class="text-xs font-medium text-base-content/60 mb-1 block">WebSocket URL</label>
      <input
        v-model="url"
        class="input w-full px-3 py-2 text-sm text-base-content bg-base-200 rounded-md border border-base-content/10 outline-none focus:border-primary"
        placeholder="ws:// 或 wss://..."
      />

      <div class="flex gap-2.5 mt-3 flex-wrap">
        <button
          v-if="status !== 'connected'"
          class="btn btn-primary btn-sm"
          @click="connect"
          :disabled="status === 'connecting'"
        >
          {{ status === 'connecting' ? '连接中...' : '🔗 连接' }}
        </button>
        <button
          v-else
          class="btn btn-error btn-sm"
          @click="disconnect"
        >
          ⛔ 断开
        </button>
        <button class="btn btn-ghost btn-sm" @click="clearLog">🗑️ 清空日志</button>
      </div>

      <!-- Status indicator -->
      <div class="flex items-center gap-2 px-3 py-2 mt-3 bg-base-200 border border-base-content/10 rounded-md">
        <span class="w-2.5 h-2.5 rounded-full" :class="status === 'connected' ? 'bg-green-500 shadow-[0_0_6px_#22c55e]' : status === 'connecting' ? 'bg-amber-500 animate-pulse' : 'bg-red-500'"></span>
        <span class="text-xs text-base-content">{{ statusText }}</span>
      </div>

      <!-- Message log -->
      <div ref="logContainer" class="mt-3 h-[300px] overflow-y-auto p-3 bg-base-200 border border-base-content/10 rounded-lg font-mono text-xs">
        <div v-if="messages.length === 0" class="text-base-content/60 text-center py-10">
          连接 WebSocket 服务器后，消息将显示在这里
        </div>
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="flex gap-2 py-1 border-b border-base-content/10 last:border-b-0 leading-5"
        >
          <span class="text-base-content/60 shrink-0 min-w-[65px]">{{ msg.time }}</span>
          <span class="shrink-0 min-w-[55px]" :class="msg.type === 'sent' ? 'text-blue-500' : msg.type === 'received' ? 'text-green-500' : msg.type === 'system' ? 'text-amber-500' : 'text-red-500'">{{ msg.typeLabel }}</span>
          <span class="flex-1 text-base-content break-all whitespace-pre-wrap">{{ msg.content }}</span>
        </div>
      </div>

      <!-- Message input -->
      <div class="flex gap-2 mt-3">
        <input
          v-model="messageInput"
          class="input flex-1 px-3 py-2 text-sm text-base-content bg-base-200 rounded-md border border-base-content/10 outline-none focus:border-primary"
          placeholder="输入要发送的消息..."
          @keyup.enter="sendMessage"
          :disabled="status !== 'connected'"
        />
        <button
          class="btn btn-primary btn-sm"
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


