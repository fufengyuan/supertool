<template>
  <div class="flex flex-col h-full bg-base-200 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 bg-base-100 border-b border-base-200">
      <div class="flex items-center gap-2.5">
        <span class="text-2xl">🔒</span>
        <h2 class="text-lg font-semibold m-0 text-base-content">VPN 管理</h2>
      </div>
      <div role="tablist" class="tabs tabs-box">
        <button role="tab" class="tab" :class="{ 'tab-active': activeProtocol === 'openvpn' }" @click="activeProtocol = 'openvpn'">
          🔐 OpenVPN
        </button>
        <button role="tab" class="tab" :class="{ 'tab-active': activeProtocol === 'wireguard' }" @click="activeProtocol = 'wireguard'">
          ⚡ WireGuard
        </button>
      </div>
    </div>

    <!-- OpenVPN Tab -->
    <div v-if="activeProtocol === 'openvpn'" class="flex-1 overflow-hidden flex flex-col min-h-0">
      <!-- OpenVPN not available banner -->
      <div v-if="!openvpnAvailable && !checking" class="flex items-start gap-3 px-5 py-4 mx-5 mt-3 bg-warning/10 border border-warning/30 rounded-box text-warning">
        <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        <div>
          <strong class="block mb-1">OpenVPN 不可用</strong>
          <p class="m-0 text-[13px]">内置 OpenVPN 二进制加载失败，请检查应用完整性或尝试重新安装</p>
        </div>
      </div>

      <!-- Connection Status Bar -->
      <div v-if="ovpnStatus.state !== 'disconnected' || ovpnStatus.connected" 
           class="flex items-center justify-between px-5 py-2.5 mx-5 mt-3 rounded-box text-sm"
           :class="{
             'bg-info/10 border border-info/30 text-info': ovpnStatus.state === 'connecting',
             'bg-success/10 border border-success/30 text-success': ovpnStatus.state === 'connected',
             'bg-warning/10 border border-warning/30 text-warning': ovpnStatus.state === 'disconnecting',
             'bg-error/10 border border-error/30 text-error': ovpnStatus.state === 'error'
           }">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full"
                :class="{
                  'bg-info animate-pulse': ovpnStatus.state === 'connecting',
                  'bg-success': ovpnStatus.state === 'connected',
                  'bg-warning': ovpnStatus.state === 'disconnecting',
                  'bg-error': ovpnStatus.state === 'error'
                }"></span>
          <span class="font-medium">
            <template v-if="ovpnStatus.state === 'connecting'">🔌 正在连接 {{ ovpnStatus.configName }}...</template>
            <template v-else-if="ovpnStatus.state === 'connected'">
              ✅ 已连接 — {{ ovpnStatus.configName }}
              <span v-if="ovpnStatus.remote" class="font-normal opacity-70 ml-1">({{ ovpnStatus.remote }})</span>
              <span class="text-[13px] font-medium opacity-70 ml-2 font-mono">{{ ovpnDuration }}</span>
            </template>
            <template v-else-if="ovpnStatus.state === 'disconnecting'">⏳ 正在断开连接...</template>
            <template v-else-if="ovpnStatus.state === 'error'">❌ 连接错误 — {{ ovpnStatus.configName }}</template>
          </span>
        </div>
        <div class="flex items-center gap-4">
          <span v-if="ovpnStatus.connected && ovpnTraffic" class="text-[13px] font-medium opacity-80 whitespace-nowrap">
            ↑ {{ ovpnTraffic.bytesSentHuman }} ↓ {{ ovpnTraffic.bytesReceivedHuman }}
          </span>
          <div class="flex gap-2">
            <button v-if="ovpnStatus.connected" class="btn btn-sm btn-error" @click="ovpnDisconnect">⏏️ 断开连接</button>
            <button v-if="ovpnStatus.state === 'error'" class="btn btn-sm btn-primary" @click="ovpnReconnect">🔄 重新连接</button>
          </div>
        </div>
      </div>

      <!-- Config List -->
      <div class="flex flex-1 overflow-hidden gap-4 mx-5 mb-5 mt-3 min-h-0">
        <div class="w-[420px] shrink-0 flex flex-col bg-base-100 rounded-box overflow-hidden">
          <div class="flex items-center justify-between px-4 py-2.5 text-xs font-semibold text-base-content/60 border-b border-base-200">
            <span>OpenVPN 配置文件 ({{ ovpnConfigs.length }})</span>
            <div class="flex gap-1">
              <button class="btn btn-xs btn-ghost" @click="checkOpenVPN" :disabled="checking">{{ checking ? '检测中...' : '🔍 检测' }}</button>
              <button class="btn btn-xs btn-primary" @click="importOvpn" :disabled="!openvpnAvailable">📥 导入</button>
            </div>
          </div>
          <div class="flex-1 overflow-y-auto p-2">
            <div v-for="cfg in ovpnConfigs" :key="cfg.id" 
                 class="flex items-center justify-between px-3 py-2.5 mb-1 rounded-box cursor-pointer transition-colors border-l-[3px] border-l-transparent hover:bg-base-200"
                 :class="{
                   'bg-primary/10 border-l-primary': ovpnStatus.configId === cfg.id,
                   'bg-info/20 border-l-info': ovpnStatus.state === 'connecting' && ovpnStatus.configId === cfg.id
                 }"
                 @click="selectOvpnConfig(cfg)">
              <div class="flex-1 min-w-0">
                <span class="block text-sm font-medium text-base-content truncate">{{ cfg.name }}</span>
                <span class="block text-[11px] text-base-content/60 truncate mt-0.5" :title="cfg.filePath">{{ cfg.filePath }}</span>
              </div>
              <div class="flex gap-1 shrink-0">
                <button v-if="ovpnStatus.configId === cfg.id && ovpnStatus.connected" class="btn btn-ghost btn-xs px-1" title="已连接">✅</button>
                <button v-else-if="ovpnStatus.state === 'connecting' && ovpnStatus.configId === cfg.id" class="btn btn-ghost btn-xs px-1" title="连接中">⏳</button>
                <button v-else class="btn btn-ghost btn-xs px-1" @click.stop="ovpnConnect(cfg)" title="连接">▶️</button>
                <button class="btn btn-ghost btn-xs px-1" @click.stop="ovpnDelete(cfg)" title="删除">🗑️</button>
              </div>
            </div>
            <div v-if="ovpnConfigs.length === 0" class="flex flex-col items-center justify-center py-10 px-5 text-center text-base-content/60">
              <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
              <p class="my-3">暂无 OpenVPN 配置文件</p>
              <button class="btn btn-sm btn-primary" @click="importOvpn" :disabled="!openvpnAvailable">导入 .ovpn 文件</button>
            </div>
          </div>
        </div>
        <div class="flex-1 flex flex-col bg-base-100 rounded-box overflow-hidden min-w-0">
          <div class="flex items-center justify-between px-4 py-2.5 text-xs font-semibold text-base-content/60 border-b border-base-200">
            <span>连接日志</span>
            <button class="btn btn-xs btn-ghost" @click="ovpnLogs = []">清空</button>
          </div>
          <div class="flex-1 overflow-y-auto p-3 font-mono text-xs leading-relaxed bg-[#1e1e2e] text-[#cdd6f4]" ref="ovpnLogRef">
            <div v-for="(line, i) in ovpnLogs" :key="i" class="whitespace-pre-wrap break-all" :class="getLogClass(line)">{{ line }}</div>
            <div v-if="ovpnLogs.length === 0" class="flex items-center justify-center h-full text-[#6c7086] text-[13px]">等待连接...</div>
          </div>
        </div>
      </div>

      <!-- Sudo Password Dialog -->
      <div v-if="showPasswordDialog" class="modal modal-open" @click.self="cancelPasswordDialog">
        <div class="modal-box max-w-[600px]">
          <div class="text-lg font-semibold text-base-content mb-3">🔐 需要 sudo 密码</div>
          <div class="text-sm text-base-content mb-4 leading-relaxed">OpenVPN 需要 root 权限创建 TUN 设备。请输入你的系统密码：</div>
          <div class="mb-4">
            <input ref="passwordInputRef" v-model="sudoPassword" type="password" placeholder="输入密码" @keydown.enter="submitPassword" @keydown.escape="showPasswordDialog = false" class="input input-bordered w-full" autofocus />
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

    <!-- WireGuard Tab -->
    <div v-if="activeProtocol === 'wireguard'" class="flex-1 overflow-hidden flex flex-col min-h-0">
      <!-- WireGuard Status Bar -->
      <div v-if="wgStatus.connected" class="flex items-center justify-between px-5 py-2.5 mx-5 mt-3 rounded-box text-sm bg-success/10 border border-success/30 text-success">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-success"></span>
          <span class="font-medium">✅ 已连接 — {{ wgStatus.configName }}</span>
          <span class="text-[13px] font-medium opacity-70 font-mono">{{ wgDuration }}</span>
        </div>
        <div class="flex items-center gap-4">
          <span class="text-[13px] font-medium opacity-80 whitespace-nowrap">↑ {{ formatBytes(wgStatus.bytesSent) }} ↓ {{ formatBytes(wgStatus.bytesReceived) }}</span>
          <button class="btn btn-sm btn-error" @click="wgDisconnect">⏏️ 断开</button>
        </div>
      </div>
      <div v-if="wgStatus.state === 'connecting'" class="flex items-center px-5 py-2.5 mx-5 mt-3 rounded-box text-sm bg-info/10 border border-info/30 text-info">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-info animate-pulse"></span>
          <span class="font-medium">🔌 正在连接 {{ wgStatus.configName }}...</span>
        </div>
      </div>
      <div v-if="wgStatus.state === 'error'" class="flex items-center px-5 py-2.5 mx-5 mt-3 rounded-box text-sm bg-error/10 border border-error/30 text-error">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-error"></span>
          <span class="font-medium">❌ 连接错误 — {{ wgStatus.configName }}</span>
        </div>
      </div>

      <!-- Config List + Log -->
      <div class="flex flex-1 overflow-hidden gap-4 mx-5 mb-5 mt-3 min-h-0">
        <div class="w-[420px] shrink-0 flex flex-col bg-base-100 rounded-box overflow-hidden">
          <div class="flex items-center justify-between px-4 py-2.5 text-xs font-semibold text-base-content/60 border-b border-base-200">
            <span>WireGuard 配置 ({{ wgConfigs.length }})</span>
            <div class="flex gap-1">
              <button class="btn btn-xs btn-primary" @click="showWgForm = true; editingWg = null">📥 添加</button>
            </div>
          </div>
          <div class="flex-1 overflow-y-auto p-2">
            <div v-for="cfg in wgConfigs" :key="cfg.id" 
                 class="flex items-center justify-between px-3 py-2.5 mb-1 rounded-box cursor-pointer transition-colors border-l-[3px] border-l-transparent hover:bg-base-200"
                 :class="{
                   'bg-primary/10 border-l-primary': wgStatus.configId === cfg.id,
                   'bg-info/20 border-l-info': wgStatus.state === 'connecting' && wgStatus.configId === cfg.id
                 }"
                 @click="selectWgConfig(cfg)">
              <div class="flex-1 min-w-0">
                <span class="block text-sm font-medium text-base-content truncate">{{ cfg.name }}</span>
                <span class="block text-[11px] text-base-content/60 truncate mt-0.5">{{ cfg.peerEndpoint }}</span>
              </div>
              <div class="flex gap-1 shrink-0">
                <button v-if="wgStatus.configId === cfg.id && wgStatus.connected" class="btn btn-ghost btn-xs px-1" title="已连接">✅</button>
                <button v-else-if="wgStatus.state === 'connecting' && wgStatus.configId === cfg.id" class="btn btn-ghost btn-xs px-1" title="连接中">⏳</button>
                <button v-else class="btn btn-ghost btn-xs px-1" @click.stop="wgConnect(cfg)" title="连接">▶️</button>
                <button class="btn btn-ghost btn-xs px-1" @click.stop="editWgConfig(cfg)" title="编辑">✏️</button>
                <button class="btn btn-ghost btn-xs px-1" @click.stop="wgDelete(cfg)" title="删除">🗑️</button>
              </div>
            </div>
            <div v-if="wgConfigs.length === 0" class="flex flex-col items-center justify-center py-10 px-5 text-center text-base-content/60">
              <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
              <p class="my-3">暂无 WireGuard 配置</p>
              <button class="btn btn-sm btn-primary" @click="showWgForm = true; editingWg = null">添加 WireGuard 配置</button>
            </div>
          </div>
        </div>
        <div class="flex-1 flex flex-col bg-base-100 rounded-box overflow-hidden min-w-0">
          <div class="flex items-center justify-between px-4 py-2.5 text-xs font-semibold text-base-content/60 border-b border-base-200">
            <span>连接日志</span>
            <button class="btn btn-xs btn-ghost" @click="wgLogs = []">清空</button>
          </div>
          <div class="flex-1 overflow-y-auto p-3 font-mono text-xs leading-relaxed bg-[#1e1e2e] text-[#cdd6f4]" ref="wgLogRef">
            <div v-for="(line, i) in wgLogs" :key="i" class="whitespace-pre-wrap break-all" :class="getLogClass(line)">{{ line }}</div>
            <div v-if="wgLogs.length === 0" class="flex items-center justify-center h-full text-[#6c7086] text-[13px]">等待连接...</div>
          </div>
        </div>
      </div>
    </div>

    <!-- WireGuard Add/Edit Dialog -->
    <div v-if="showWgForm" class="modal modal-open" @click.self="showWgForm = false">
      <div class="modal-box max-w-[600px] max-h-[90vh] overflow-y-auto">
        <div class="text-lg font-semibold text-base-content mb-3">⚡ {{ editingWg ? '编辑' : '添加' }} WireGuard 配置</div>
        <div class="flex flex-col gap-2.5 text-sm text-base-content mb-4">
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">名称</label>
            <input v-model="wgForm.name" class="input input-bordered" placeholder="我的 WireGuard" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">私钥</label>
            <div class="flex gap-1">
              <input v-model="wgForm.privateKey" class="input input-bordered flex-1" placeholder="Base64 私钥" />
              <button class="btn btn-xs btn-ghost" @click="generateKeypair" :disabled="generatingKeys">{{ generatingKeys ? '生成中...' : '🔑 生成' }}</button>
            </div>
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">公钥</label>
            <input v-model="wgForm.publicKey" class="input input-bordered" placeholder="自动派生或手动填写" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">接口地址</label>
            <input v-model="wgForm.address" class="input input-bordered" placeholder="10.0.0.2/32" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">DNS (可选)</label>
            <input v-model="wgForm.dns" class="input input-bordered" placeholder="1.1.1.1" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">MTU (可选)</label>
            <input v-model="wgForm.mtu" type="number" class="input input-bordered" placeholder="1420" />
          </div>
          <hr class="border-base-200 my-1" />
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">对端公钥 *</label>
            <input v-model="wgForm.peerPublicKey" class="input input-bordered" placeholder="Base64 公钥" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">对端地址 *</label>
            <input v-model="wgForm.peerEndpoint" class="input input-bordered" placeholder="server.com:51820" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">允许 IP</label>
            <input v-model="wgForm.peerAllowedIPs" class="input input-bordered" placeholder="0.0.0.0/0" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">Keepalive (秒)</label>
            <input v-model="wgForm.peerPersistentKeepalive" type="number" class="input input-bordered" placeholder="25" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">PSK (可选)</label>
            <input v-model="wgForm.presharedKey" class="input input-bordered" placeholder="Base64 预共享密钥" />
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-3">
          <button class="btn btn-sm btn-ghost" @click="showWgForm = false">取消</button>
          <button class="btn btn-sm btn-primary" @click="saveWgConfig" :disabled="!wgForm.name || !wgForm.peerPublicKey || !wgForm.peerEndpoint">{{ editingWg ? '保存' : '添加' }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast'

const toast = useToast()

// ============ Protocol Tabs ============
const activeProtocol = ref<'openvpn' | 'wireguard'>('openvpn')

// ============ OpenVPN State ============
interface OvpnConfig { id: string; name: string; filePath: string; content: string; createdAt: string; updatedAt: string }
interface OvpnStatus { connected: boolean; configId: string | null; configName: string | null; state: string; log: string[]; connectedSince?: string; remote?: string }

const ovpnConfigs = ref<OvpnConfig[]>([])
const ovpnStatus = ref<OvpnStatus>({ connected: false, configId: null, configName: null, state: 'disconnected', log: [] })
const ovpnLogs = ref<string[]>([])
const openvpnAvailable = ref(true)
const checking = ref(false)
const ovpnDuration = ref('')
const ovpnTraffic = ref<any>(null)
const ovpnLogRef = ref<HTMLElement | null>(null)
let ovpnPolling: any = null
let ovpnDurationTimer: any = null
let ovpnTrafficTimer: any = null

// Password dialog
const showPasswordDialog = ref(false)
const sudoPassword = ref('')
const passwordInputRef = ref<HTMLInputElement | null>(null)
let pendingOvpnConfig: OvpnConfig | null = null

// ============ WireGuard State ============
interface WgConfig { id: string; name: string; privateKey: string; publicKey: string; address: string; dns?: string; mtu?: number; peerPublicKey: string; peerEndpoint: string; peerAllowedIPs: string; peerPersistentKeepalive?: number; presharedKey?: string; createdAt: string; updatedAt: string }
interface WgStatus { connected: boolean; configId: string | null; configName: string | null; state: string; log: string[]; connectedSince?: string; bytesSent: number; bytesReceived: number; latestHandshake?: string }

const wgConfigs = ref<WgConfig[]>([])
const wgStatus = ref<WgStatus>({ connected: false, configId: null, configName: null, state: 'disconnected', log: [], bytesSent: 0, bytesReceived: 0 })
const wgLogs = ref<string[]>([])
const wgDuration = ref('')
const wgLogRef = ref<HTMLElement | null>(null)
let wgPolling: any = null
let wgDurationTimer: any = null

// WireGuard form
const showWgForm = ref(false)
const editingWg = ref<WgConfig | null>(null)
const generatingKeys = ref(false)
const wgForm = ref<Record<string, any>>({ name: '', privateKey: '', publicKey: '', address: '10.0.0.2/32', dns: '', mtu: 1420, peerPublicKey: '', peerEndpoint: '', peerAllowedIPs: '0.0.0.0/0', peerPersistentKeepalive: 25, presharedKey: '' })

// ============ Lifecycle ============
onMounted(async () => {
  console.log('[VPNManager] mounted')
  await loadOvpnAll()
  await loadWgAll()
  await checkOpenVPN()
  await loadOvpnStatus()
  await loadWgStatus()
  ovpnPolling = setInterval(loadOvpnStatus, 2000)
  ovpnTrafficTimer = setInterval(loadOvpnTraffic, 3000)
  ovpnDurationTimer = setInterval(updateOvpnDuration, 1000)
  wgPolling = setInterval(loadWgStatus, 2000)
  wgDurationTimer = setInterval(updateWgDuration, 1000)
})

onUnmounted(() => {
  if (ovpnPolling) clearInterval(ovpnPolling)
  if (ovpnDurationTimer) clearInterval(ovpnDurationTimer)
  if (ovpnTrafficTimer) clearInterval(ovpnTrafficTimer)
  if (wgPolling) clearInterval(wgPolling)
  if (wgDurationTimer) clearInterval(wgDurationTimer)
})

watch(ovpnLogs, () => { nextTick(() => { if (ovpnLogRef.value) ovpnLogRef.value.scrollTop = ovpnLogRef.value.scrollHeight }) })
watch(wgLogs, () => { nextTick(() => { if (wgLogRef.value) wgLogRef.value.scrollTop = wgLogRef.value.scrollHeight }) })

// ============ OpenVPN Methods ============
async function loadOvpnAll() { try { ovpnConfigs.value = await getTauriAPI().openvpnGetAll() } catch(e:any) { console.error(e) } }
async function loadOvpnStatus() { try { const s = await getTauriAPI().openvpnGetStatus(); if (s) { ovpnStatus.value = s; if (s.log?.length) ovpnLogs.value = s.log } } catch {} }
async function loadOvpnTraffic() { if (!ovpnStatus.value.connected) { ovpnTraffic.value = null; return }; try { ovpnTraffic.value = await getTauriAPI().openvpnGetTrafficStats() } catch {} }
async function checkOpenVPN() { checking.value = true; try { const r = await getTauriAPI().openvpnCheckAvailable(); openvpnAvailable.value = r.available; if (!r.available) console.warn('OpenVPN not available:', r.error) } catch { openvpnAvailable.value = false } finally { checking.value = false } }
function updateOvpnDuration() { if (!ovpnStatus.value.connected || !ovpnStatus.value.connectedSince) { ovpnDuration.value = ''; return }; const s = (Date.now() - new Date(ovpnStatus.value.connectedSince).getTime()) / 1000; const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = Math.floor(s % 60); ovpnDuration.value = h > 0 ? `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}` : `${m}:${String(sec).padStart(2, '0')}` }

async function importOvpn() { try { const r: any = await getTauriAPI().importOvpnFile(); if (r.canceled || !r.filePaths?.length) return; const fp = r.filePaths[0]; const fn = fp.split('/').pop() || fp.split('\\\\').pop() || 'config'; const name = fn.replace(/\\.(ovpn|conf)$/i, ''); if (ovpnConfigs.value.find(c => c.filePath === fp)) { toast.warning('该配置文件已存在'); return }; const content = await getTauriAPI().readFileContent(fp); const v = await getTauriAPI().openvpnValidateConfig(content); if (!v.valid) { toast.error('配置文件无效: ' + (v.error || '请检查文件格式')); return }; await getTauriAPI().openvpnAdd({ name, filePath: fp, content }); await loadOvpnAll(); toast.success(`已导入: ${name}`) } catch(e:any) { toast.error('导入失败: ' + e.message) } }

async function ovpnConnect(cfg: OvpnConfig, password?: string) { if (ovpnStatus.value.connected) { await ovpnDisconnect(); await new Promise(r => setTimeout(r, 500)) }; try { const r: any = await getTauriAPI().openvpnConnect(cfg.id, cfg.name, cfg.content, password); if (!r?.success) { if (r?.needsPassword) { pendingOvpnConfig = cfg; showPasswordDialog.value = true; await nextTick(); passwordInputRef.value?.focus() } else { toast.error('连接失败: ' + (r?.error || '未知错误')) } }; await loadOvpnStatus() } catch(e:any) { toast.error('连接失败: ' + e.message) } }

function cancelPasswordDialog() { showPasswordDialog.value = false; pendingOvpnConfig = null; sudoPassword.value = '' }
async function submitPassword() { if (!sudoPassword.value || !pendingOvpnConfig) return; showPasswordDialog.value = false; const pwd = sudoPassword.value; sudoPassword.value = ''; try { const r: any = await getTauriAPI().openvpnRetryWithPassword(pwd); if (!r?.success) toast.error('连接失败: ' + (r?.error || '密码错误或超时')); await loadOvpnStatus() } catch(e:any) { toast.error('连接失败: ' + e.message) } }

async function ovpnDisconnect() { try { await getTauriAPI().openvpnDisconnect(); await loadOvpnStatus(); toast.info('已断开连接') } catch(e:any) { toast.error('断开失败: ' + e.message) } }
async function ovpnReconnect() { if (ovpnStatus.value.configId) { const cfg = ovpnConfigs.value.find(c => c.id === ovpnStatus.value.configId); if (cfg) await ovpnConnect(cfg) } }
function selectOvpnConfig(_cfg: OvpnConfig) {}
async function ovpnDelete(cfg: OvpnConfig) { if (!confirm(`确定要删除 "${cfg.name}" 吗？`)) return; try { await getTauriAPI().openvpnDelete(cfg.id); await loadOvpnAll(); toast.success('已删除') } catch(e:any) { toast.error('删除失败: ' + e.message) } }

// ============ WireGuard Methods ============
async function loadWgAll() { try { wgConfigs.value = await getTauriAPI().wireguardGetAll() } catch(e:any) { console.error(e) } }
async function loadWgStatus() { try { const s = await getTauriAPI().wireguardGetStatus(); if (s) { wgStatus.value = s; if (s.log?.length) wgLogs.value = s.log } } catch {} }
function updateWgDuration() { if (!wgStatus.value.connected || !wgStatus.value.connectedSince) { wgDuration.value = ''; return }; const s = (Date.now() - new Date(wgStatus.value.connectedSince).getTime()) / 1000; const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = Math.floor(s % 60); wgDuration.value = h > 0 ? `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}` : `${m}:${String(sec).padStart(2, '0')}` }

async function generateKeypair() { generatingKeys.value = true; try { const r = await getTauriAPI().wireguardGenerateKeypair(); if (r) { wgForm.value.privateKey = r.privateKey; wgForm.value.publicKey = r.publicKey; toast.success('密钥对已生成') } } catch(e:any) { toast.error('生成失败: ' + e.message) } finally { generatingKeys.value = false } }

function editWgConfig(cfg: WgConfig) { editingWg.value = cfg; wgForm.value = { name: cfg.name, privateKey: cfg.privateKey || '', publicKey: cfg.publicKey || '', address: cfg.address || '10.0.0.2/32', dns: cfg.dns || '', mtu: cfg.mtu, peerPublicKey: cfg.peerPublicKey, peerEndpoint: cfg.peerEndpoint, peerAllowedIPs: cfg.peerAllowedIPs || '0.0.0.0/0', peerPersistentKeepalive: cfg.peerPersistentKeepalive, presharedKey: cfg.presharedKey || '' }; showWgForm.value = true }

async function saveWgConfig() { try { const data = { ...wgForm.value, mtu: wgForm.value.mtu ? Number(wgForm.value.mtu) : null, peerPersistentKeepalive: wgForm.value.peerPersistentKeepalive ? Number(wgForm.value.peerPersistentKeepalive) : null }; if (editingWg.value) { await getTauriAPI().wireguardUpdate({ ...data, id: editingWg.value.id }) } else { await getTauriAPI().wireguardAdd(data) }; showWgForm.value = false; editingWg.value = null; await loadWgAll(); toast.success(editingWg.value ? '已更新' : '已添加') } catch(e:any) { toast.error('保存失败: ' + e.message) } }

async function wgConnect(cfg: WgConfig) { if (wgStatus.value.connected) { await wgDisconnect(); await new Promise(r => setTimeout(r, 500)) }; try { const r = await getTauriAPI().wireguardConnect(cfg.id, cfg.name, cfg.privateKey, cfg.peerPublicKey, cfg.peerEndpoint, cfg.presharedKey || undefined); if (!r?.success) toast.error('连接失败: ' + (r?.error || '未知错误')); await loadWgStatus() } catch(e:any) { toast.error('连接失败: ' + e.message) } }

async function wgDisconnect() { try { await getTauriAPI().wireguardDisconnect(); await loadWgStatus(); toast.info('已断开') } catch(e:any) { toast.error('断开失败: ' + e.message) } }
function selectWgConfig(_cfg: WgConfig) {}

async function wgDelete(cfg: WgConfig) { if (!confirm(`确定要删除 "${cfg.name}" 吗？`)) return; try { await getTauriAPI().wireguardDelete(cfg.id); await loadWgAll(); toast.success('已删除') } catch(e:any) { toast.error('删除失败: ' + e.message) } }

function formatBytes(bytes: number): string { if (bytes < 1024) return bytes + ' B'; if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'; return (bytes / 1048576).toFixed(1) + ' MB' }

function getLogClass(line: string): string {
  if (line.includes('✅') || line.includes('成功') || line.includes('Completed')) return 'text-[#a6e3a1]'
  if (line.includes('❌') || line.includes('错误') || line.includes('Error') || line.includes('FAILED')) return 'text-[#f38ba8]'
  if (line.includes('⏳') || line.includes('连接中')) return 'text-[#f9e2af]'
  return ''
}
</script>
