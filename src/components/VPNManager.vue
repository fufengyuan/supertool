<template>
  <div class="vpn-manager">
    <!-- Header -->
    <div class="vpn-header">
      <div class="vpn-header-left">
        <span class="vpn-icon">🔒</span>
        <h2 class="vpn-title">VPN 管理</h2>
      </div>
      <div class="vpn-header-tabs">
        <button class="tab-btn" :class="{ active: activeProtocol === 'openvpn' }" @click="activeProtocol = 'openvpn'">
          🔐 OpenVPN
        </button>
        <button class="tab-btn" :class="{ active: activeProtocol === 'wireguard' }" @click="activeProtocol = 'wireguard'">
          ⚡ WireGuard
        </button>
      </div>
    </div>

    <!-- OpenVPN Tab -->
    <div v-if="activeProtocol === 'openvpn'" class="protocol-content">
      <!-- OpenVPN not available banner -->
      <div v-if="!openvpnAvailable && !checking" class="vpn-banner">
        <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        <div class="banner-text">
          <strong>OpenVPN 不可用</strong>
          <p>内置 OpenVPN 二进制加载失败，请检查应用完整性或尝试重新安装</p>
        </div>
      </div>

      <!-- Connection Status Bar -->
      <div v-if="ovpnStatus.state !== 'disconnected' || ovpnStatus.connected" class="status-bar" :class="ovpnStatus.state">
        <div class="status-left">
          <span class="status-dot" :class="ovpnStatus.state"></span>
          <span class="status-text">
            <template v-if="ovpnStatus.state === 'connecting'">🔌 正在连接 {{ ovpnStatus.configName }}...</template>
            <template v-else-if="ovpnStatus.state === 'connected'">
              ✅ 已连接 — {{ ovpnStatus.configName }}
              <span v-if="ovpnStatus.remote" class="status-remote">({{ ovpnStatus.remote }})</span>
              <span class="status-duration">{{ ovpnDuration }}</span>
            </template>
            <template v-else-if="ovpnStatus.state === 'disconnecting'">⏳ 正在断开连接...</template>
            <template v-else-if="ovpnStatus.state === 'error'">❌ 连接错误 — {{ ovpnStatus.configName }}</template>
          </span>
        </div>
        <div class="status-right">
          <span v-if="ovpnStatus.connected && ovpnTraffic" class="traffic-stats">
            ↑ {{ ovpnTraffic.bytesSentHuman }} ↓ {{ ovpnTraffic.bytesReceivedHuman }}
          </span>
          <div class="status-actions">
            <button v-if="ovpnStatus.connected" class="btn btn-sm btn-danger" @click="ovpnDisconnect">⏏️ 断开连接</button>
            <button v-if="ovpnStatus.state === 'error'" class="btn btn-sm btn-primary" @click="ovpnReconnect">🔄 重新连接</button>
          </div>
        </div>
      </div>

      <!-- Config List -->
      <div class="vpn-layout">
        <div class="vpn-sidebar">
          <div class="vpn-sidebar-header">
            <span>OpenVPN 配置文件 ({{ ovpnConfigs.length }})</span>
            <div class="vpn-sidebar-actions">
              <button class="btn btn-xs btn-ghost" @click="checkOpenVPN" :disabled="checking">{{ checking ? '检测中...' : '🔍 检测' }}</button>
              <button class="btn btn-xs btn-primary" @click="importOvpn" :disabled="!openvpnAvailable">📥 导入</button>
            </div>
          </div>
          <div class="vpn-config-list">
            <div v-for="cfg in ovpnConfigs" :key="cfg.id" class="vpn-config-item" :class="{ active: ovpnStatus.configId === cfg.id, 'is-connecting': ovpnStatus.state === 'connecting' && ovpnStatus.configId === cfg.id }" @click="selectOvpnConfig(cfg)">
              <div class="config-info">
                <span class="config-name">{{ cfg.name }}</span>
                <span class="config-path" :title="cfg.filePath">{{ cfg.filePath }}</span>
              </div>
              <div class="config-actions">
                <button v-if="ovpnStatus.configId === cfg.id && ovpnStatus.connected" class="config-btn btn-connected" title="已连接">✅</button>
                <button v-else-if="ovpnStatus.state === 'connecting' && ovpnStatus.configId === cfg.id" class="config-btn btn-connecting" title="连接中">⏳</button>
                <button v-else class="config-btn btn-connect" @click.stop="ovpnConnect(cfg)" title="连接">▶️</button>
                <button class="config-btn btn-delete" @click.stop="ovpnDelete(cfg)" title="删除">🗑️</button>
              </div>
            </div>
            <div v-if="ovpnConfigs.length === 0" class="vpn-empty">
              <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
              <p>暂无 OpenVPN 配置文件</p>
              <button class="btn btn-sm btn-primary" @click="importOvpn" :disabled="!openvpnAvailable">导入 .ovpn 文件</button>
            </div>
          </div>
        </div>
        <div class="vpn-main">
          <div class="vpn-log-header"><span>连接日志</span><button class="btn btn-xs btn-ghost" @click="ovpnLogs = []">清空</button></div>
          <div class="vpn-log" ref="ovpnLogRef">
            <div v-for="(line, i) in ovpnLogs" :key="i" class="log-line" :class="getLogClass(line)">{{ line }}</div>
            <div v-if="ovpnLogs.length === 0" class="log-empty">等待连接...</div>
          </div>
        </div>
      </div>

      <!-- Sudo Password Dialog -->
      <div v-if="showPasswordDialog" class="password-overlay" @click.self="cancelPasswordDialog">
        <div class="password-dialog">
          <div class="password-dialog-header">🔐 需要 sudo 密码</div>
          <div class="password-dialog-body">OpenVPN 需要 root 权限创建 TUN 设备。请输入你的系统密码：</div>
          <div class="password-dialog-input"><input ref="passwordInputRef" v-model="sudoPassword" type="password" placeholder="输入密码" @keydown.enter="submitPassword" @keydown.escape="showPasswordDialog = false" class="password-input" autofocus /></div>
          <div class="password-dialog-actions">
            <button class="btn btn-sm btn-ghost" @click="showPasswordDialog = false">取消</button>
            <button class="btn btn-sm btn-primary" @click="submitPassword" :disabled="!sudoPassword">确认连接</button>
          </div>
          <div class="password-dialog-hint">提示：可以配置免密 sudo：<code>echo "$USER ALL=(root) NOPASSWD: $(which openvpn)" | sudo tee /etc/sudoers.d/openvpn</code></div>
        </div>
      </div>
    </div>

    <!-- WireGuard Tab -->
    <div v-if="activeProtocol === 'wireguard'" class="protocol-content">
      <!-- WireGuard Status Bar -->
      <div v-if="wgStatus.connected" class="status-bar connected">
        <div class="status-left">
          <span class="status-dot connected"></span>
          <span class="status-text">✅ 已连接 — {{ wgStatus.configName }}</span>
          <span class="status-duration">{{ wgDuration }}</span>
        </div>
        <div class="status-right">
          <span class="traffic-stats">↑ {{ formatBytes(wgStatus.bytesSent) }} ↓ {{ formatBytes(wgStatus.bytesReceived) }}</span>
          <button class="btn btn-sm btn-danger" @click="wgDisconnect">⏏️ 断开</button>
        </div>
      </div>
      <div v-if="wgStatus.state === 'connecting'" class="status-bar connecting">
        <div class="status-left"><span class="status-dot connecting"></span><span class="status-text">🔌 正在连接 {{ wgStatus.configName }}...</span></div>
      </div>
      <div v-if="wgStatus.state === 'error'" class="status-bar error">
        <div class="status-left"><span class="status-dot error"></span><span class="status-text">❌ 连接错误 — {{ wgStatus.configName }}</span></div>
      </div>

      <!-- Config List + Log -->
      <div class="vpn-layout">
        <div class="vpn-sidebar">
          <div class="vpn-sidebar-header">
            <span>WireGuard 配置 ({{ wgConfigs.length }})</span>
            <div class="vpn-sidebar-actions">
              <button class="btn btn-xs btn-primary" @click="showWgForm = true; editingWg = null">📥 添加</button>
            </div>
          </div>
          <div class="vpn-config-list">
            <div v-for="cfg in wgConfigs" :key="cfg.id" class="vpn-config-item" :class="{ active: wgStatus.configId === cfg.id, 'is-connecting': wgStatus.state === 'connecting' && wgStatus.configId === cfg.id }" @click="selectWgConfig(cfg)">
              <div class="config-info">
                <span class="config-name">{{ cfg.name }}</span>
                <span class="config-path">{{ cfg.peerEndpoint }}</span>
              </div>
              <div class="config-actions">
                <button v-if="wgStatus.configId === cfg.id && wgStatus.connected" class="config-btn btn-connected" title="已连接">✅</button>
                <button v-else-if="wgStatus.state === 'connecting' && wgStatus.configId === cfg.id" class="config-btn btn-connecting" title="连接中">⏳</button>
                <button v-else class="config-btn btn-connect" @click.stop="wgConnect(cfg)" title="连接">▶️</button>
                <button class="config-btn btn-edit" @click.stop="editWgConfig(cfg)" title="编辑">✏️</button>
                <button class="config-btn btn-delete" @click.stop="wgDelete(cfg)" title="删除">🗑️</button>
              </div>
            </div>
            <div v-if="wgConfigs.length === 0" class="vpn-empty">
              <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
              <p>暂无 WireGuard 配置</p>
              <button class="btn btn-sm btn-primary" @click="showWgForm = true; editingWg = null">添加 WireGuard 配置</button>
            </div>
          </div>
        </div>
        <div class="vpn-main">
          <div class="vpn-log-header"><span>连接日志</span><button class="btn btn-xs btn-ghost" @click="wgLogs = []">清空</button></div>
          <div class="vpn-log" ref="wgLogRef">
            <div v-for="(line, i) in wgLogs" :key="i" class="log-line" :class="getLogClass(line)">{{ line }}</div>
            <div v-if="wgLogs.length === 0" class="log-empty">等待连接...</div>
          </div>
        </div>
      </div>
    </div>

    <!-- WireGuard Add/Edit Dialog -->
    <div v-if="showWgForm" class="password-overlay" @click.self="showWgForm = false">
      <div class="password-dialog" style="max-height:90vh;overflow-y:auto">
        <div class="password-dialog-header">⚡ {{ editingWg ? '编辑' : '添加' }} WireGuard 配置</div>
        <div class="password-dialog-body" style="display:flex;flex-direction:column;gap:10px">
          <div class="form-row"><label>名称</label><input v-model="wgForm.name" class="password-input" placeholder="我的 WireGuard" /></div>
          <div class="form-row"><label>私钥</label>
            <div style="display:flex;gap:4px"><input v-model="wgForm.privateKey" class="password-input" placeholder="Base64 私钥" style="flex:1" /><button class="btn btn-xs btn-ghost" @click="generateKeypair" :disabled="generatingKeys">{{ generatingKeys ? '生成中...' : '🔑 生成' }}</button></div>
          </div>
          <div class="form-row"><label>公钥</label><input v-model="wgForm.publicKey" class="password-input" placeholder="自动派生或手动填写" /></div>
          <div class="form-row"><label>接口地址</label><input v-model="wgForm.address" class="password-input" placeholder="10.0.0.2/32" /></div>
          <div class="form-row"><label>DNS (可选)</label><input v-model="wgForm.dns" class="password-input" placeholder="1.1.1.1" /></div>
          <div class="form-row"><label>MTU (可选)</label><input v-model="wgForm.mtu" type="number" class="password-input" placeholder="1420" /></div>
          <hr style="border-color:bg-base-200;margin:4px 0" />
          <div class="form-row"><label>对端公钥 *</label><input v-model="wgForm.peerPublicKey" class="password-input" placeholder="Base64 公钥" /></div>
          <div class="form-row"><label>对端地址 *</label><input v-model="wgForm.peerEndpoint" class="password-input" placeholder="server.com:51820" /></div>
          <div class="form-row"><label>允许 IP</label><input v-model="wgForm.peerAllowedIPs" class="password-input" placeholder="0.0.0.0/0" /></div>
          <div class="form-row"><label>Keepalive (秒)</label><input v-model="wgForm.peerPersistentKeepalive" type="number" class="password-input" placeholder="25" /></div>
          <div class="form-row"><label>PSK (可选)</label><input v-model="wgForm.presharedKey" class="password-input" placeholder="Base64 预共享密钥" /></div>
        </div>
        <div class="password-dialog-actions" style="margin-top:12px">
          <button class="btn btn-sm btn-ghost" @click="showWgForm = false">取消</button>
          <button class="btn btn-sm btn-primary" @click="saveWgConfig" :disabled="!wgForm.name || !wgForm.peerPublicKey || !wgForm.peerEndpoint">{{ editingWg ? '保存' : '添加' }}</button>
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

async function importOvpn() { try { const r: any = await getTauriAPI().importOvpnFile(); if (r.canceled || !r.filePaths?.length) return; const fp = r.filePaths[0]; const fn = fp.split('/').pop() || fp.split('\\').pop() || 'config'; const name = fn.replace(/\.(ovpn|conf)$/i, ''); if (ovpnConfigs.value.find(c => c.filePath === fp)) { toast.warning('该配置文件已存在'); return }; const content = await getTauriAPI().readFileContent(fp); const v = await getTauriAPI().openvpnValidateConfig(content); if (!v.valid) { toast.error('配置文件无效: ' + (v.error || '请检查文件格式')); return }; await getTauriAPI().openvpnAdd({ name, filePath: fp, content }); await loadOvpnAll(); toast.success(`已导入: ${name}`) } catch(e:any) { toast.error('导入失败: ' + e.message) } }

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

function getLogClass(line: string): string { if (line.includes('✅') || line.includes('成功') || line.includes('Completed')) return 'log-success'; if (line.includes('❌') || line.includes('错误') || line.includes('Error') || line.includes('FAILED')) return 'log-error'; if (line.includes('⏳') || line.includes('连接中')) return 'log-warning'; return '' }
</script>

<style scoped>
.vpn-manager { display: flex; flex-direction: column; height: 100%; background: var(--color-base-200); overflow: hidden; }
.vpn-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; background: var(--color-base-100); border-bottom: 1px solid var(--color-base-200); }
.vpn-header-left { display: flex; align-items: center; gap: 10px; }
.vpn-icon { font-size: 24px; }
.vpn-title { font-size: 18px; font-weight: 600; margin: 0; color: var(--color-base-content); }
.vpn-header-tabs { display: flex; gap: 4px; }
.tab-btn { padding: 6px 16px; border: 1px solid var(--color-base-200); border-radius: 8px; background: transparent; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.tab-btn.active { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }
.tab-btn:hover:not(.active) { background: var(--color-base-200); }
.protocol-content { flex: 1; overflow: hidden; display: flex; flex-direction: column; min-height: 0; }
.vpn-banner { display: flex; align-items: flex-start; gap: 12px; padding: 16px 20px; margin: 12px 20px; background: #fff3cd; border: 1px solid #ffc107; border-radius: 8px; color: #856404; }
.banner-text strong { display: block; margin-bottom: 4px; }
.banner-text p { margin: 0; font-size: 13px; }
.status-bar { display: flex; align-items: center; justify-content: space-between; padding: 10px 20px; margin: 12px 20px 0; border-radius: 8px; font-size: 14px; }
.status-bar.connecting { background: #e3f2fd; border: 1px solid #90caf9; color: #1565c0; }
.status-bar.connected { background: #e8f5e9; border: 1px solid #a5d6a7; color: #2e7d32; }
.status-bar.disconnecting { background: #fff3e0; border: 1px solid #ffcc80; color: #e65100; }
.status-bar.error { background: #ffebee; border: 1px solid #ef9a9a; color: #c62828; }
.status-left { display: flex; align-items: center; gap: 8px; }
.status-dot { width: 10px; height: 10px; border-radius: 50%; background: #999; }
.status-dot.connecting { background: #2196f3; animation: pulse 1s infinite; }
.status-dot.connected { background: #4caf50; }
.status-dot.disconnecting { background: #ff9800; }
.status-dot.error { background: #f44336; }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.3; } }
.status-text { font-weight: 500; }
.status-remote { font-weight: 400; opacity: 0.7; margin-left: 4px; }
.status-duration { font-size: 13px; font-weight: 500; opacity: 0.7; margin-left: 8px; font-family: 'SF Mono', 'Fira Code', monospace; }
.status-right { display: flex; align-items: center; gap: 16px; }
.traffic-stats { font-size: 13px; font-weight: 500; opacity: 0.8; white-space: nowrap; }
.status-actions { display: flex; gap: 8px; }
.vpn-layout { display: flex; flex: 1; overflow: hidden; margin: 12px 20px 20px; gap: 16px; min-height: 0; }
.vpn-sidebar { width: 420px; flex-shrink: 0; display: flex; flex-direction: column; background: var(--color-base-100); border-radius: 10px; overflow: hidden; }
.vpn-sidebar-header { display: flex; align-items: center; justify-content: space-between; padding: 10px 16px; font-size: 13px; font-weight: 600; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); border-bottom: 1px solid var(--color-base-200); }
.vpn-sidebar-actions { display: flex; gap: 4px; }
.vpn-config-list { flex: 1; overflow-y: auto; padding: 8px; }
.vpn-config-item { display: flex; align-items: center; justify-content: space-between; padding: 10px 12px; margin-bottom: 4px; border-radius: 8px; cursor: pointer; transition: background 0.15s; border-left: 3px solid transparent; }
.vpn-config-item:hover { background: var(--color-base-200); }
.vpn-config-item.active { background: color-mix(in oklab, var(--color-primary) 10%, transparent); border-left-color: var(--color-primary); }
.vpn-config-item.is-connecting { background: #e3f2fd; border-left-color: #2196f3; }
.config-info { flex: 1; min-width: 0; }
.config-name { display: block; font-size: 14px; font-weight: 500; color: var(--color-base-content); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.config-path { display: block; font-size: 11px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 2px; }
.config-actions { display: flex; gap: 4px; flex-shrink: 0; }
.config-btn { background: none; border: none; cursor: pointer; padding: 4px; border-radius: 4px; font-size: 14px; transition: background 0.15s; }
.config-btn:hover { background: rgba(0,0,0,0.1); }
.vpn-empty { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 40px 20px; text-align: center; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }
.vpn-empty p { margin: 12px 0 16px; }
.vpn-main { flex: 1; display: flex; flex-direction: column; background: var(--color-base-100); border-radius: 10px; overflow: hidden; min-width: 0; }
.vpn-log-header { display: flex; align-items: center; justify-content: space-between; padding: 10px 16px; font-size: 13px; font-weight: 600; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); border-bottom: 1px solid var(--color-base-200); }
.vpn-log { flex: 1; overflow-y: auto; padding: 12px; font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace; font-size: 12px; line-height: 1.5; background: #1e1e2e; color: #cdd6f4; }
.log-line { white-space: pre-wrap; word-break: break-all; }
.log-line.log-success { color: #a6e3a1; }
.log-line.log-error { color: #f38ba8; }
.log-line.log-warning { color: #f9e2af; }
.log-empty { display: flex; align-items: center; justify-content: center; height: 100%; color: #6c7086; font-size: 13px; }

.btn { display: inline-flex; align-items: center; gap: 4px; padding: 6px 12px; border: none; border-radius: 6px; font-size: 13px; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 4px 10px; font-size: 12px; }
.btn-xs { padding: 2px 8px; font-size: 11px; }
.btn-primary { background: var(--color-primary); color: white; }
.btn-primary:hover:not(:disabled) { background: color-mix(in oklab, var(--color-primary) 10%, transparent); }
.btn-ghost { background: transparent; color: var(--color-base-content); border: 1px solid var(--color-base-200); }
.btn-ghost:hover:not(:disabled) { background: var(--color-base-200); }
.btn-danger { background: var(--color-error); color: white; }
.btn-danger:hover:not(:disabled) { opacity: 0.9; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }

.form-row { display: flex; flex-direction: column; gap: 4px; }
.form-row label { font-size: 12px; font-weight: 600; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }

.password-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 9999; }
.password-dialog { background: var(--color-base-100); border-radius: 12px; padding: 24px; width: 600px; max-width: 90vw; box-shadow: 0 20px 60px rgba(0,0,0,0.3); border: 1px solid var(--color-base-200); }
.password-dialog-header { font-size: 18px; font-weight: 600; color: var(--color-base-content); margin-bottom: 12px; }
.password-dialog-body { font-size: 14px; color: var(--color-base-content); margin-bottom: 16px; line-height: 1.5; }
.password-dialog-input { margin-bottom: 16px; }
.password-input { width: 100%; padding: 10px 12px; border: 1px solid var(--color-base-200); border-radius: 8px; font-size: 14px; background: var(--color-base-200); color: var(--color-base-content); outline: none; transition: border-color 0.15s; box-sizing: border-box; }
.password-input:focus { border-color: var(--color-primary); box-shadow: 0 0 0 3px rgba(99,102,241,0.1); }
.password-dialog-actions { display: flex; justify-content: flex-end; gap: 8px; margin-bottom: 12px; }
.password-dialog-hint { font-size: 11px; color: var(--color-base-content); opacity: 0.7; line-height: 1.5; padding: 8px 10px; background: rgba(0,0,0,0.03); border-radius: 6px; }
.password-dialog-hint code { font-size: 10px; background: rgba(0,0,0,0.08); padding: 1px 4px; border-radius: 3px; word-break: break-all; }
</style>
