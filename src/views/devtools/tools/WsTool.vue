<template>
  <ToolPage
    icon="plug"
    name="WebSocket 调试"
    description="连接 ws/wss 服务器，收发消息、JSON 自动格式化、消息统计"
    :offline="false"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex gap-2">
        <input
          v-model="url"
          class="input input-sm flex-1 px-3 font-mono bg-base-200/60"
          placeholder="ws:// 或 wss://..."
          @keydown.enter="connect"
        />
        <button v-if="status !== 'connected'" class="btn btn-primary btn-sm" @click="connect" :disabled="status === 'connecting'">
          <template v-if="status === 'connecting'">连接中...</template><template v-else><SvgIcon name="link" size="13" /> 连接</template>
        </button>
        <button v-else class="btn btn-error btn-sm" @click="disconnect">断开</button>
      </div>

      <div v-if="urlHistory.length > 0" class="mt-2 flex flex-wrap gap-1.5">
        <span class="text-[11px] text-base-content/40 leading-6">历史：</span>
        <button
          v-for="h in urlHistory"
          :key="h"
          class="btn btn-ghost btn-xs font-mono opacity-60 hover:opacity-100"
          @click="url = h; connect()"
        >{{ h.replace(/^wss?:\/\//, '') }}</button>
      </div>

      <div class="flex items-center gap-2 px-3 py-2 mt-3 bg-base-200/60 border border-base-content/10 rounded-lg">
        <span class="w-2.5 h-2.5 rounded-full" :class="status === 'connected' ? 'bg-green-500 shadow-[0_0_6px_#22c55e]' : status === 'connecting' ? 'bg-amber-500 animate-pulse' : 'bg-red-500'"></span>
        <span class="text-xs text-base-content">{{ statusText }}</span>
        <span v-if="status === 'connected'" class="text-xs text-base-content/40 ml-auto">已收 {{ stats.received }} · 已发 {{ stats.sent }}</span>
      </div>

      <div ref="logContainer" class="mt-3 h-[300px] overflow-y-auto p-3 bg-base-200/60 border border-base-content/10 rounded-lg font-mono text-xs">
        <div v-if="messages.length === 0" class="text-base-content/50 text-center py-10">
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

      <div class="flex gap-2 mt-3">
        <input
          v-model="messageInput"
          class="input input-sm flex-1 px-3 font-mono bg-base-200/60"
          placeholder="输入要发送的消息..."
          @keyup.enter="sendMessage"
          :disabled="status !== 'connected'"
        />
        <button class="btn btn-primary btn-sm" @click="sendMessage" :disabled="status !== 'connected'">发送</button>
        <button class="btn btn-ghost btn-sm" @click="clearLog" :disabled="messages.length === 0"><SvgIcon name="trash" size="13" /></button>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed, nextTick, onUnmounted } from 'vue'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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
const urlHistory = ref<string[]>(loadUrlHistory())
const stats = ref({ sent: 0, received: 0 })

let ws: WebSocket | null = null

const statusText = computed(() => {
  switch (status.value) {
    case 'connected': return '已连接'
    case 'connecting': return '连接中...'
    default: return '未连接'
  }
})

function getNow(): string {
  return new Date().toLocaleTimeString('zh-CN', { hour12: false })
}

function loadUrlHistory(): string[] {
  try {
    return JSON.parse(localStorage.getItem('devtools:ws-history') || '[]')
  } catch { return [] }
}

function saveUrlHistory(url: string) {
  const filtered = urlHistory.value.filter(u => u !== url)
  urlHistory.value = [url, ...filtered].slice(0, 8)
  localStorage.setItem('devtools:ws-history', JSON.stringify(urlHistory.value))
}

function tryFormatJson(str: string): string {
  try {
    const parsed = JSON.parse(str)
    if (typeof parsed === 'object' && parsed !== null) {
      return JSON.stringify(parsed, null, 2)
    }
  } catch { /* not json */ }
  return str
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
  if (type === 'sent') { stats.value.sent++ }
  else if (type === 'received') { stats.value.received++ }
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

  // Cleanup existing connection
  if (ws) {
    ws.onclose = null
    ws.onerror = null
    ws.onmessage = null
    ws.onopen = null
    ws.close()
    ws = null
  }

  status.value = 'connecting'
  stats.value = { sent: 0, received: 0 }
  saveUrlHistory(wsUrl)
  addMessage('system', `正在连接 ${wsUrl}...`)

  try {
    ws = new WebSocket(wsUrl)

    ws.onopen = () => {
      status.value = 'connected'
      addMessage('system', `已连接到 ${wsUrl}`)
    }

    ws.onmessage = (event) => {
      const data = typeof event.data === 'string' ? tryFormatJson(event.data) : '[Binary]'
      addMessage('received', data)
    }

    ws.onclose = (event) => {
      status.value = 'disconnected'
      ws = null
      addMessage('system', `连接已关闭 (代码: ${event.code}${event.reason ? ', 原因: ' + event.reason : ''})`)
    }

    ws.onerror = () => {
      addMessage('error', 'WebSocket 连接错误')
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
  stats.value = { sent: 0, received: 0 }
}

onUnmounted(() => {
  if (ws) {
    ws.onclose = null
    ws.onerror = null
    ws.onmessage = null
    ws.onopen = null
    ws.close()
    ws = null
  }
})
</script>
