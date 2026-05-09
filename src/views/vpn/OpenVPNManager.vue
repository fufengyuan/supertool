<template>
  <div class="flex flex-col h-full bg-base-200 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 bg-base-100 border-b border-base-200">
      <div class="flex items-center gap-2.5">
        <span class="text-2xl"><SvgIcon name="lock" size="14" class="inline-block align-text-bottom" /></span>
        <h2 class="text-lg font-semibold m-0 text-base-content">OpenVPN 客户端</h2>
      </div>
      <div class="flex gap-2">
        <button class="btn btn-sm btn-ghost" @click="checkOpenVPN" :disabled="checking">
          <template v-if="checking">检测中...</template><template v-else><SvgIcon name="search" size="14" />  检测 OpenVPN</template>
        </button>
        <button class="btn btn-sm btn-primary" @click="importConfig" :disabled="!openvpnAvailable">
          <SvgIcon name="download" size="14" />  导入 .ovpn
        </button>
      </div>
    </div>

    <!-- OpenVPN not available banner -->
    <div v-if="!openvpnAvailable && !checking" class="flex items-start gap-3 px-5 py-4 mx-5 mt-3 bg-warning/10 border border-warning/30 rounded-box text-warning">
      <SvgIcon name="alertCircle" size="24" />
      <div>
        <strong class="block mb-1">OpenVPN 不可用</strong>
        <p class="m-0 text-[13px]">内置 OpenVPN 二进制加载失败，请检查应用完整性或尝试重新安装</p>
      </div>
    </div>

    <!-- Connection Status Bar -->
    <div v-if="status.state !== 'disconnected' || status.connected" 
         class="flex items-center justify-between px-5 py-2.5 mx-5 mt-3 rounded-box text-sm"
         :class="{
           'bg-info/10 border border-info/30 text-info': status.state === 'connecting',
           'bg-success/10 border border-success/30 text-success': status.state === 'connected',
           'bg-warning/10 border border-warning/30 text-warning': status.state === 'disconnecting',
           'bg-error/10 border border-error/30 text-error': status.state === 'error'
         }">
      <div class="flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full"
              :class="{
                'bg-info animate-pulse': status.state === 'connecting',
                'bg-success': status.state === 'connected',
                'bg-warning': status.state === 'disconnecting',
                'bg-error': status.state === 'error'
              }"></span>
        <span class="font-medium">
          <template v-if="status.state === 'connecting'">
            <SvgIcon name="plug" size="14" class="inline-block align-text-bottom" /> 正在连接 {{ status.configName }}...
          </template>
          <template v-else-if="status.state === 'connected'">
            <SvgIcon name="check" size="14" />  已连接 — {{ status.configName }}
            <span v-if="status.remote" class="font-normal opacity-70 ml-1">({{ status.remote }})</span>
            <span class="text-[13px] font-medium opacity-70 ml-2 font-mono">{{ connectionDuration }}</span>
          </template>
          <template v-else-if="status.state === 'disconnecting'">
            <SvgIcon name="clock" size="14" />  正在断开连接...
          </template>
          <template v-else-if="status.state === 'error'">
            <SvgIcon name="x" size="14" />  连接错误 — {{ status.configName }}
          </template>
        </span>
      </div>
      <div class="flex items-center gap-4">
        <span v-if="status.connected && trafficStats" class="text-[13px] font-medium opacity-80 whitespace-nowrap">
          ↑ {{ trafficStats.bytesSentHuman }} ↓ {{ trafficStats.bytesReceivedHuman }}
        </span>
        <div class="flex gap-2">
          <button v-if="status.connected" class="btn btn-sm btn-error" @click="disconnect">
            <SvgIcon name="power" size="14" class="inline-block" /> 断开连接
          </button>
          <button v-if="status.state === 'error'" class="btn btn-sm btn-primary" @click="reconnect">
<SvgIcon name="refresh" size="14" />  重新连接
          </button>
        </div>
      </div>
    </div>

    <!-- Main Layout -->
    <div class="flex flex-1 overflow-hidden gap-4 mx-5 mb-5 mt-3 min-h-0">
      <!-- Left: Config List -->
      <div class="w-[420px] shrink-0 flex flex-col bg-base-100 rounded-box overflow-hidden">
        <div class="px-4 py-3 text-xs font-semibold text-base-content/60 border-b border-base-200">
          <span>配置文件 ({{ configs.length }})</span>
        </div>
        <div class="flex-1 overflow-y-auto p-2">
          <div
            v-for="cfg in configs"
            :key="cfg.id"
            class="flex items-center justify-between px-3 py-2.5 mb-1 rounded-box cursor-pointer transition-colors border-l-[3px] border-l-transparent hover:bg-base-200"
            :class="{
              'bg-primary/10 border-l-primary': status.configId === cfg.id,
              'bg-info/20 border-l-info': status.state === 'connecting' && status.configId === cfg.id
            }"
            @click="selectConfig(cfg)"
          >
            <div class="flex-1 min-w-0">
              <span class="block text-sm font-medium text-base-content truncate">{{ cfg.name }}</span>
              <span class="block text-[11px] text-base-content/60 truncate mt-0.5" :title="cfg.filePath">{{ cfg.filePath }}</span>
            </div>
            <div class="flex gap-1 shrink-0">
              <button
                v-if="status.configId === cfg.id && status.connected"
                class="btn btn-ghost btn-xs px-1"
                title="已连接"
              ><SvgIcon name="check" size="14" /> </button>
              <button
                v-else-if="status.state === 'connecting' && status.configId === cfg.id"
                class="btn btn-ghost btn-xs px-1"
                title="连接中"
              ><SvgIcon name="clock" size="14" /> </button>
              <button
                v-else
                class="btn btn-ghost btn-xs px-1"
                @click.stop="connectConfig(cfg)"
                title="连接"
              ><SvgIcon name="play" size="14" /></button>
              <button
                class="btn btn-ghost btn-xs px-1"
                @click.stop="deleteConfig(cfg)"
                title="删除"
              ><SvgIcon name="trash" size="14" /></button>
            </div>
          </div>
          <div v-if="configs.length === 0" class="flex flex-col items-center justify-center py-10 px-5 text-center text-base-content/60">
            <SvgIcon name="file" size="40" stroke-width="1.5" />
            <p class="my-3">暂无配置文件</p>
            <button class="btn btn-sm btn-primary" @click="importConfig" :disabled="!openvpnAvailable">
              导入 .ovpn 文件
            </button>
          </div>
        </div>
      </div>

      <!-- Right: Log Viewer -->
      <div class="flex-1 flex flex-col bg-base-100 rounded-box overflow-hidden min-w-0">
        <div class="flex items-center justify-between px-4 py-2.5 text-xs font-semibold text-base-content/60 border-b border-base-200">
          <span>连接日志</span>
          <button class="btn btn-xs btn-ghost" @click="logs = []">清空</button>
        </div>
        <div class="flex-1 overflow-y-auto p-3 font-mono text-xs leading-relaxed bg-[#1e1e2e] text-[#cdd6f4]" ref="logRef">
          <div
            v-for="(line, i) in logs"
            :key="i"
            class="whitespace-pre-wrap break-all"
            :class="getLogClass(line)"
          >{{ line }}</div>
          <div v-if="logs.length === 0" class="flex items-center justify-center h-full text-[#6c7086] text-[13px]">等待连接...</div>
        </div>
      </div>
    </div>

    <!-- Sudo Password Dialog -->
    <div v-if="showPasswordDialog" class="modal modal-open" @click.self="cancelPasswordDialog">
      <div class="modal-box max-w-[600px]">
        <div class="text-lg font-semibold text-base-content mb-3">
          <SvgIcon name="lock" size="14" class="inline-block align-text-bottom" /> 需要 sudo 密码
        </div>
        <div class="text-sm text-base-content mb-4 leading-relaxed">
          OpenVPN 需要 root 权限创建 TUN 设备。请输入你的系统密码：
        </div>
        <div class="mb-4">
          <input
            ref="passwordInputRef"
            v-model="sudoPassword"
            type="password"
            placeholder="输入密码"
            @keydown.enter="submitPassword"
            @keydown.escape="showPasswordDialog = false"
            class="input input-bordered w-full"
            autofocus
          />
        </div>
        <div class="flex justify-end gap-2 mb-3">
          <button class="btn btn-sm btn-ghost" @click="showPasswordDialog = false">取消</button>
          <button class="btn btn-sm btn-primary" @click="submitPassword" :disabled="!sudoPassword">确认连接</button>
        </div>
        <div class="text-[11px] text-base-content/70 leading-relaxed p-2 bg-black/[0.03] rounded-box">
          提示：可以配置免密 sudo：<code class="text-[10px] bg-black/10 px-1 py-0.5 rounded break-all">echo "$USER ALL=(root) NOPASSWD: $(which openvpn)" | sudo tee /etc/sudoers.d/openvpn</code>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const toast = useToast()

interface Config {
  id: string
  name: string
  filePath: string
  content: string
  createdAt: string
  updatedAt: string
}

// Sudo password state
const showPasswordDialog = ref(false)
const sudoPassword = ref('')
const passwordInputRef = ref<HTMLInputElement | null>(null)
let pendingConfig: Config | null = null

interface VPNStatus {
  connected: boolean
  configId: string | null
  configName: string | null
  state: 'disconnected' | 'connecting' | 'connected' | 'error' | 'disconnecting'
  log: string[]
  connectedSince?: string
  remote?: string
  bytesSent?: number
  bytesReceived?: number
}

const configs = ref<Config[]>([])
const status = ref<VPNStatus>({
  connected: false,
  configId: null,
  configName: null,
  state: 'disconnected',
  log: [],
})
const logs = ref<string[]>([])
const openvpnAvailable = ref(true)
const checking = ref(false)
const logRef = ref<HTMLElement | null>(null)
const connectionDuration = ref('')
const trafficStats = ref<{ bytesSent: number; bytesReceived: number; bytesSentHuman: string; bytesReceivedHuman: string } | null>(null)
let pollingTimer: ReturnType<typeof setInterval> | null = null
let durationTimer: ReturnType<typeof setInterval> | null = null
let trafficTimer: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
    console.log("[components/OpenVPNManager.vue] mounted")
  await loadConfigs()
  await checkOpenVPN()
  await loadStatus()
  // Poll status every 2 seconds
  pollingTimer = setInterval(loadStatus, 2000)
  // Traffic stats every 3 seconds
  trafficTimer = setInterval(loadTrafficStats, 3000)
  // Duration timer every second
  durationTimer = setInterval(updateDuration, 1000)
})

onUnmounted(() => {
  if (pollingTimer) clearInterval(pollingTimer)
  if (durationTimer) clearInterval(durationTimer)
  if (trafficTimer) clearInterval(trafficTimer)
})

function updateDuration() {
  if (!status.value.connected || !status.value.connectedSince) {
    connectionDuration.value = ''
    return
  }
  const start = new Date(status.value.connectedSince).getTime()
  const now = Date.now()
  const diff = Math.floor((now - start) / 1000)
  const hours = Math.floor(diff / 3600)
  const minutes = Math.floor((diff % 3600) / 60)
  const seconds = diff % 60
  if (hours > 0) {
    connectionDuration.value = `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
  } else {
    connectionDuration.value = `${minutes}:${String(seconds).padStart(2, '0')}`
  }
}

async function loadTrafficStats() {
  if (!status.value.connected) {
    trafficStats.value = null
    return
  }
  try {
    trafficStats.value = await getTauriAPI().openvpnGetTrafficStats()
  } catch {
    // Ignore
  }
}

// Auto-scroll logs
watch(logs, () => {
  nextTick(() => {
    if (logRef.value) {
      logRef.value.scrollTop = logRef.value.scrollHeight
    }
  })
})

async function loadConfigs() {
  try {
    console.log("[loadConfigs] called")
    configs.value = await getTauriAPI().openvpnGetAll()
  } catch (e: any) {
    console.error('Failed to load VPN configs:', e)
  }
}

async function loadStatus() {
  try {
    console.log("[loadStatus] called")
    const s = await getTauriAPI().openvpnGetStatus()
    if (!s) return
    const prevConnected = status.value.connected
    status.value = s
    // Update logs
    if (s.log && s.log.length > 0) {
      logs.value = s.log
    }
  } catch {
    // Ignore polling errors
  }
}

async function checkOpenVPN() {
  checking.value = true
  try {
    console.log("[checkOpenVPN] called")
    const result = await getTauriAPI().openvpnCheckAvailable()
    openvpnAvailable.value = result.available
    if (!result.available) {
      console.warn('OpenVPN not available:', result.error)
    }
  } catch {
    openvpnAvailable.value = false
  } finally {
    checking.value = false
  }
}

async function importConfig() {
  try {
    console.log("[importConfig] called")
    const result: any = await getTauriAPI().importOvpnFile()
    if (result.canceled || !result.filePaths?.length) return

    const filePath = result.filePaths[0]
    const fileName = filePath.split('/').pop() || filePath.split('\\\\').pop() || 'config'
    const name = fileName.replace(/\\.(ovpn|conf)$/i, '')

    // Check for duplicate
    const existing = configs.value.find(c => c.filePath === filePath)
    if (existing) {
      toast.warning('该配置文件已存在')
      return
    }

    // Read file content via main process
    const content = await getTauriAPI().readFileContent(filePath) as string

    // Validate config before saving
    const validation = await getTauriAPI().openvpnValidateConfig(content)
    if (!validation.valid) {
      toast.error('配置文件无效: ' + (validation.error || '请检查文件格式'))
      return
    }

    await getTauriAPI().openvpnAdd({ name, filePath, content })
    await loadConfigs()
    toast.success(`已导入: ${name}`)
  } catch (e: any) {
    toast.error('导入失败: ' + e.message)
  }
}

async function connectConfig(cfg: Config, password?: string) {
  if (status.value.connected) {
    console.log("[connectConfig] called")
    await disconnect()
    await new Promise(r => setTimeout(r, 500))
  }
  try {
    const result: any = await getTauriAPI().openvpnConnect(cfg.id, cfg.name, cfg.content, password)
    if (!result?.success) {
      if (result?.needsPassword) {
        // Need sudo password — show dialog
        pendingConfig = cfg
        showPasswordDialog.value = true
        // Focus password input
        await nextTick()
        passwordInputRef.value?.focus()
      } else {
        toast.error('连接失败: ' + (result?.error || '未知错误'))
      }
    }
    await loadStatus()
  } catch (e: any) {
    toast.error('连接失败: ' + e.message)
  }
}

function cancelPasswordDialog() {
  showPasswordDialog.value = false
  pendingConfig = null
  sudoPassword.value = ''
}

async function submitPassword() {
    console.log("[cancelPasswordDialog] called")
  if (!sudoPassword.value || !pendingConfig) return
  showPasswordDialog.value = false
  const pwd = sudoPassword.value
  sudoPassword.value = '' // Clear from memory ASAP
  try {
    console.log("[submitPassword] called")
    const result: any = await getTauriAPI().openvpnRetryWithPassword(pwd)
    if (!result?.success) {
      toast.error('连接失败: ' + (result?.error || '密码错误或超时'))
    }
    await loadStatus()
  } catch (e: any) {
    toast.error('连接失败: ' + e.message)
  }
}

async function disconnect() {
  try {
    console.log("[disconnect] called")
    await getTauriAPI().openvpnDisconnect()
    await loadStatus()
    toast.info('已断开连接')
  } catch (e: any) {
    toast.error('断开失败: ' + e.message)
  }
}

async function reconnect() {
  if (status.value.configId) {
    console.log("[reconnect] called")
    const cfg = configs.value.find(c => c.id === status.value.configId)
    if (cfg) {
      await connectConfig(cfg)
    }
  }
}

function selectConfig(cfg: Config) {
  // Just highlight, no action
}

async function deleteConfig(cfg: Config) {
  if (!confirm(`确定要删除 "${cfg.name}" 吗？`)) return
  try {
    await getTauriAPI().openvpnDelete(cfg.id)
    await loadConfigs()
    toast.success('已删除')
  } catch (e: any) {
    toast.error('删除失败: ' + e.message)
  }
}

function getLogClass(line: string): string {
  if (line.includes('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg> ') || line.includes('成功') || line.includes('Completed')) return 'text-[#a6e3a1]'
  if (line.includes('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg> ') || line.includes('错误') || line.includes('Error') || line.includes('FAILED')) return 'text-[#f38ba8]'
  if (line.includes('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> ') || line.includes('连接中')) return 'text-[#f9e2af]'
  return ''
}
</script>
