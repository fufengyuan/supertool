<template>
  <div class="openvpn-manager">
    <!-- Header -->
    <div class="vpn-header">
      <div class="vpn-header-left">
        <span class="vpn-icon">🔒</span>
        <h2 class="vpn-title">OpenVPN 客户端</h2>
      </div>
      <div class="vpn-header-actions">
        <button class="btn btn-sm btn-ghost" @click="checkOpenVPN" :disabled="checking">
          {{ checking ? '检测中...' : '🔍 检测 OpenVPN' }}
        </button>
        <button class="btn btn-sm btn-primary" @click="importConfig" :disabled="!openvpnAvailable">
          📥 导入 .ovpn
        </button>
      </div>
    </div>

    <!-- OpenVPN not available banner -->
    <div v-if="!openvpnAvailable && !checking" class="vpn-banner">
      <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      <div class="banner-text">
        <strong>OpenVPN 不可用</strong>
        <p>内置 OpenVPN 二进制 加载失败，请检查应用完整性或尝试重新安装</p>
      </div>
    </div>

    <!-- Connection Status Bar -->
    <div v-if="status.state !== 'disconnected' || status.connected" class="status-bar" :class="status.state">
      <div class="status-left">
        <span class="status-dot" :class="status.state"></span>
        <span class="status-text">
          <template v-if="status.state === 'connecting'">
            🔌 正在连接 {{ status.configName }}...
          </template>
          <template v-else-if="status.state === 'connected'">
            ✅ 已连接 — {{ status.configName }}
            <span v-if="status.remote" class="status-remote">({{ status.remote }})</span>
            <span class="status-duration">{{ connectionDuration }}</span>
          </template>
          <template v-else-if="status.state === 'disconnecting'">
            ⏳ 正在断开连接...
          </template>
          <template v-else-if="status.state === 'error'">
            ❌ 连接错误 — {{ status.configName }}
          </template>
        </span>
      </div>
      <div class="status-right">
        <span v-if="status.connected && trafficStats" class="traffic-stats">
          ↑ {{ trafficStats.bytesSentHuman }} ↓ {{ trafficStats.bytesReceivedHuman }}
        </span>
        <div class="status-actions">
          <button v-if="status.connected" class="btn btn-sm btn-danger" @click="disconnect">
            ⏏️ 断开连接
          </button>
          <button v-if="status.state === 'error'" class="btn btn-sm btn-primary" @click="reconnect">
            🔄 重新连接
          </button>
        </div>
      </div>
    </div>

    <!-- Main Layout -->
    <div class="vpn-layout">
      <!-- Left: Config List -->
      <div class="vpn-sidebar">
        <div class="vpn-sidebar-header">
          <span>配置文件 ({{ configs.length }})</span>
        </div>
        <div class="vpn-config-list">
          <div
            v-for="cfg in configs"
            :key="cfg.id"
            class="vpn-config-item"
            :class="{
              active: status.configId === cfg.id,
              'is-connecting': status.state === 'connecting' && status.configId === cfg.id,
            }"
            @click="selectConfig(cfg)"
          >
            <div class="config-info">
              <span class="config-name">{{ cfg.name }}</span>
              <span class="config-path" :title="cfg.filePath">{{ cfg.filePath }}</span>
            </div>
            <div class="config-actions">
              <button
                v-if="status.configId === cfg.id && status.connected"
                class="config-btn btn-connected"
                title="已连接"
              >✅</button>
              <button
                v-else-if="status.state === 'connecting' && status.configId === cfg.id"
                class="config-btn btn-connecting"
                title="连接中"
              >⏳</button>
              <button
                v-else
                class="config-btn btn-connect"
                @click.stop="connectConfig(cfg)"
                title="连接"
              >▶️</button>
              <button class="config-btn btn-delete" @click.stop="deleteConfig(cfg)" title="删除">🗑️</button>
            </div>
          </div>
          <div v-if="configs.length === 0" class="vpn-empty">
            <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            <p>暂无配置文件</p>
            <button class="btn btn-sm btn-primary" @click="importConfig" :disabled="!openvpnAvailable">
              导入 .ovpn 文件
            </button>
          </div>
        </div>
      </div>

      <!-- Right: Log Viewer -->
      <div class="vpn-main">
        <div class="vpn-log-header">
          <span>连接日志</span>
          <button class="btn btn-xs btn-ghost" @click="logs = []">清空</button>
        </div>
        <div class="vpn-log" ref="logRef">
          <div
            v-for="(line, i) in logs"
            :key="i"
            class="log-line"
            :class="getLogClass(line)"
          >{{ line }}</div>
          <div v-if="logs.length === 0" class="log-empty">等待连接...</div>
        </div>
      </div>
    </div>

    <!-- Sudo Password Dialog -->
    <div v-if="showPasswordDialog" class="password-overlay" @click.self="cancelPasswordDialog">
      <div class="password-dialog">
        <div class="password-dialog-header">
          🔐 需要 sudo 密码
        </div>
        <div class="password-dialog-body">
          OpenVPN 需要 root 权限创建 TUN 设备。请输入你的系统密码：
        </div>
        <div class="password-dialog-input">
          <input
            ref="passwordInputRef"
            v-model="sudoPassword"
            type="password"
            placeholder="输入密码"
            @keydown.enter="submitPassword"
            @keydown.escape="showPasswordDialog = false"
            class="password-input"
            autofocus
          />
        </div>
        <div class="password-dialog-actions">
          <button class="btn btn-sm btn-ghost" @click="showPasswordDialog = false">取消</button>
          <button class="btn btn-sm btn-primary" @click="submitPassword" :disabled="!sudoPassword">确认连接</button>
        </div>
        <div class="password-dialog-hint">
          提示：可以配置免密 sudo：<code>echo "$USER ALL=(root) NOPASSWD: $(which openvpn)" | sudo tee /etc/sudoers.d/openvpn</code>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import { useToast } from '../composables/useToast'

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
    const name = fileName.replace(/\.(ovpn|conf)$/i, '')

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
  if (line.includes('✅') || line.includes('成功') || line.includes('Completed')) return 'log-success'
  if (line.includes('❌') || line.includes('错误') || line.includes('Error') || line.includes('FAILED')) return 'log-error'
  if (line.includes('⏳') || line.includes('连接中')) return 'log-warning'
  return ''
}
</script>

<style scoped>
.openvpn-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--main-bg);
  overflow: hidden;
}

/* Header */
.vpn-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: var(--card-bg);
  border-bottom: 1px solid var(--input-bg);
}

.vpn-header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.vpn-icon {
  font-size: 24px;
}

.vpn-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: var(--main-text);
}

.vpn-header-actions {
  display: flex;
  gap: 8px;
}

/* Banner */
.vpn-banner {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px 20px;
  margin: 12px 20px;
  background: #fff3cd;
  border: 1px solid #ffc107;
  border-radius: 8px;
  color: #856404;
}

.banner-text strong {
  display: block;
  margin-bottom: 4px;
}

.banner-text p {
  margin: 0;
  font-size: 13px;
}

.banner-text code {
  background: rgba(0,0,0,0.08);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
}

.banner-text a {
  color: #0066cc;
}

/* Status Bar */
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 20px;
  margin: 12px 20px 0;
  border-radius: 8px;
  font-size: 14px;
}

.status-bar.connecting {
  background: #e3f2fd;
  border: 1px solid #90caf9;
  color: #1565c0;
}

.status-bar.connected {
  background: #e8f5e9;
  border: 1px solid #a5d6a7;
  color: #2e7d32;
}

.status-bar.disconnecting {
  background: #fff3e0;
  border: 1px solid #ffcc80;
  color: #e65100;
}

.status-bar.error {
  background: #ffebee;
  border: 1px solid #ef9a9a;
  color: #c62828;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #999;
}

.status-dot.connecting {
  background: #2196f3;
  animation: pulse 1s infinite;
}

.status-dot.connected {
  background: #4caf50;
}

.status-dot.disconnecting {
  background: #ff9800;
}

.status-dot.error {
  background: #f44336;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.status-text {
  font-weight: 500;
}

.status-remote {
  font-weight: 400;
  opacity: 0.7;
  margin-left: 4px;
}

.status-actions {
  display: flex;
  gap: 8px;
}

.status-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.traffic-stats {
  font-size: 13px;
  font-weight: 500;
  opacity: 0.8;
  white-space: nowrap;
}

.status-duration {
  font-size: 13px;
  font-weight: 500;
  opacity: 0.7;
  margin-left: 8px;
  font-family: 'SF Mono', 'Fira Code', monospace;
}

/* Layout */
.vpn-layout {
  display: flex;
  flex: 1;
  overflow: hidden;
  margin: 12px 20px 20px;
  gap: 16px;
  min-height: 0;
}

/* Config List Sidebar */
.vpn-sidebar {
  width: 420px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--card-bg);
  border-radius: 10px;
  overflow: hidden;
}

.vpn-sidebar-header {
  padding: 12px 16px;
  font-size: 13px;
  font-weight: 600;
  color: var(--main-text-secondary);
  border-bottom: 1px solid var(--input-bg);
}

.vpn-config-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.vpn-config-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  margin-bottom: 4px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
  border-left: 3px solid transparent;
}

.vpn-config-item:hover {
  background: var(--input-bg);
}

.vpn-config-item.active {
  background: var(--primary-light);
  border-left-color: var(--primary-color);
}

.vpn-config-item.is-connecting {
  background: #e3f2fd;
  border-left-color: #2196f3;
}

.config-info {
  flex: 1;
  min-width: 0;
}

.config-name {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--main-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.config-path {
  display: block;
  font-size: 11px;
  color: var(--main-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.config-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.config-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  font-size: 14px;
  transition: background 0.15s;
}

.config-btn:hover {
  background: rgba(0,0,0,0.1);
}

.vpn-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  text-align: center;
  color: var(--main-text-secondary);
}

.vpn-empty p {
  margin: 12px 0 16px;
}

/* Log Viewer */
.vpn-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--card-bg);
  border-radius: 10px;
  overflow: hidden;
  min-width: 0;
}

.vpn-log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 600;
  color: var(--main-text-secondary);
  border-bottom: 1px solid var(--input-bg);
}

.vpn-log {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 12px;
  line-height: 1.5;
  background: #1e1e2e;
  color: #cdd6f4;
}

.log-line {
  white-space: pre-wrap;
  word-break: break-all;
}

.log-line.log-success {
  color: #a6e3a1;
}

.log-line.log-error {
  color: #f38ba8;
}

.log-line.log-warning {
  color: #f9e2af;
}

.log-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #6c7086;
  font-size: 13px;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn-xs {
  padding: 2px 8px;
  font-size: 11px;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-light);
}

.btn-ghost {
  background: transparent;
  color: var(--main-text);
  border: 1px solid var(--input-bg);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--input-bg);
}

.btn-danger {
  background: var(--danger-color);
  color: white;
}

.btn-danger:hover:not(:disabled) {
  opacity: 0.9;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Password Dialog */
.password-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.password-dialog {
  background: var(--card-bg, #fff);
  border-radius: 12px;
  padding: 24px;
  width: 600px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  border: 1px solid var(--input-bg, #e5e7eb);
}

.password-dialog-header {
  font-size: 18px;
  font-weight: 600;
  color: var(--main-text, #1f2937);
  margin-bottom: 12px;
}

.password-dialog-body {
  font-size: 14px;
  color: var(--main-text, #4b5563);
  margin-bottom: 16px;
  line-height: 1.5;
}

.password-dialog-input {
  margin-bottom: 16px;
}

.password-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--input-bg, #d1d5db);
  border-radius: 8px;
  font-size: 14px;
  background: var(--input-bg, #f9fafb);
  color: var(--main-text, #1f2937);
  outline: none;
  transition: border-color 0.15s;
  box-sizing: border-box;
}

.password-input:focus {
  border-color: var(--primary-color, #6366f1);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.password-dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-bottom: 12px;
}

.password-dialog-hint {
  font-size: 11px;
  color: var(--main-text, #6b7280);
  opacity: 0.7;
  line-height: 1.5;
  padding: 8px 10px;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 6px;
}

.password-dialog-hint code {
  font-size: 10px;
  background: rgba(0, 0, 0, 0.08);
  padding: 1px 4px;
  border-radius: 3px;
  word-break: break-all;
}
</style>
