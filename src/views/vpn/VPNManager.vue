<template>
  <div class="flex flex-col h-full bg-base-200 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 bg-base-100 border-b border-base-200">
      <div class="flex items-center gap-2.5">
        <span class="text-2xl"><SvgIcon name="lock" size="14" class="inline-block align-text-bottom" /></span>
        <h2 class="text-lg font-semibold m-0 text-base-content">VPN 管理</h2>
        <span class="badge badge-sm badge-ghost">WireGuard</span>
      </div>
    </div>

    <!-- WireGuard Status Bar -->
    <div v-if="wgStatus.connected" class="flex items-center justify-between px-5 py-2.5 mx-5 mt-3 rounded-box text-sm bg-success/10 border border-success/30 text-success">
      <div class="flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full bg-success"></span>
        <span class="font-medium"><SvgIcon name="check" size="14" />  已连接 — {{ wgStatus.configName }}</span>
        <span class="text-[13px] font-medium opacity-70 font-mono">{{ wgDuration }}</span>
      </div>
      <div class="flex items-center gap-4">
        <span class="text-[13px] font-medium opacity-80 whitespace-nowrap">↑ {{ formatBytes(wgStatus.bytesSent) }} ↓ {{ formatBytes(wgStatus.bytesReceived) }}</span>
        <button class="btn btn-sm btn-error" @click="wgDisconnect"><SvgIcon name="power" size="14" class="inline-block" /> 断开</button>
      </div>
    </div>
    <div v-if="wgStatus.state === 'connecting'" class="flex items-center px-5 py-2.5 mx-5 mt-3 rounded-box text-sm bg-info/10 border border-info/30 text-info">
      <div class="flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full bg-info animate-pulse"></span>
        <span class="font-medium"><SvgIcon name="plug" size="14" class="inline-block align-text-bottom" /> 正在连接 {{ wgStatus.configName }}...</span>
      </div>
    </div>
    <div v-if="wgStatus.state === 'error'" class="flex items-center px-5 py-2.5 mx-5 mt-3 rounded-box text-sm bg-error/10 border border-error/30 text-error">
      <div class="flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full bg-error"></span>
        <span class="font-medium"><SvgIcon name="x" size="14" />  连接错误 — {{ wgStatus.configName }}</span>
      </div>
    </div>

    <!-- Config List + Log -->
    <div class="flex flex-1 overflow-hidden gap-4 mx-5 mb-5 mt-3 min-h-0">
      <div class="w-[420px] shrink-0 flex flex-col bg-base-100 rounded-box overflow-hidden">
        <div class="flex items-center justify-between px-4 py-2.5 text-xs font-semibold text-base-content/60 border-b border-base-200">
          <span>WireGuard 配置 ({{ wgConfigs.length }})</span>
          <div class="flex gap-1">
            <button class="btn btn-xs btn-primary" @click="showWgForm = true; editingWg = null"><SvgIcon name="download" size="14" />  添加</button>
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
              <button v-if="wgStatus.configId === cfg.id && wgStatus.connected" class="btn btn-ghost btn-xs px-1" title="已连接"><SvgIcon name="check" size="14" /> </button>
              <button v-else-if="wgStatus.state === 'connecting' && wgStatus.configId === cfg.id" class="btn btn-ghost btn-xs px-1" title="连接中"><SvgIcon name="clock" size="14" /> </button>
              <button v-else class="btn btn-ghost btn-xs px-1" @click.stop="wgConnect(cfg)" title="连接"><SvgIcon name="play" size="14" class="inline-block" /></button>
              <button class="btn btn-ghost btn-xs px-1" @click.stop="editWgConfig(cfg)" title="编辑"><SvgIcon name="pencil" size="14" /> </button>
              <button class="btn btn-ghost btn-xs px-1" @click.stop="wgDelete(cfg)" title="删除"><SvgIcon name="trash" size="14" /> </button>
            </div>
          </div>
          <div v-if="wgConfigs.length === 0" class="flex flex-col items-center justify-center py-10 px-5 text-center text-base-content/60">
            <SvgIcon name="file" size="40" stroke-width="1.5" />
            <p class="my-3">暂无 WireGuard 配置</p>
            <button class="btn btn-sm btn-primary" @click="showWgForm = true; editingWg = null">添加 WireGuard 配置</button>
          </div>
        </div>
      </div>
      <div class="flex-1 flex flex-col bg-base-100 rounded-box overflow-hidden min-w-0 relative">
        <div class="flex items-center justify-between px-4 py-2.5 text-xs font-semibold text-base-content/60 border-b border-base-200">
          <span>连接日志</span>
          <button class="btn btn-xs btn-ghost" @click="wgUserScrolledUp = false; wgLogs = []">清空</button>
        </div>
        <div class="flex-1 overflow-y-auto p-3 font-mono text-xs leading-relaxed bg-[#1e1e2e] text-[#cdd6f4]" ref="wgLogRef" @scroll="onWgLogScroll">
          <div v-for="(line, i) in wgLogs" :key="i" class="whitespace-pre-wrap break-all" :class="getLogClass(line)">{{ line }}</div>
          <div v-if="wgLogs.length === 0" class="flex items-center justify-center h-full text-[#6c7086] text-[13px]">等待连接...</div>
        </div>
        <button
          v-if="wgUserScrolledUp"
          @click="scrollWgToBottom"
          class="btn btn-primary btn-sm rounded-full absolute bottom-2 right-2 z-10 shadow-lg hover:scale-105 transition-all"
          title="回到底部"
        >
          <SvgIcon name="arrowDown" size="14" /> 回到底部
        </button>
      </div>
    </div>

    <!-- WireGuard Add/Edit Dialog -->
    <div v-if="showWgForm" class="modal modal-open">
      <div class="modal-box relative max-w-[600px] max-h-[90vh] overflow-y-auto">
        <button @click="showWgForm = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭"><SvgIcon name="x" size="16" /></button>
        <div class="text-lg font-semibold text-base-content mb-3"><SvgIcon name="zap" size="14" />  {{ editingWg ? '编辑' : '添加' }} WireGuard 配置</div>
        <div class="flex flex-col gap-2.5 text-sm text-base-content mb-4">
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">名称</label>
            <input v-model="wgForm.name" class="input input-bordered" placeholder="我的 WireGuard" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-semibold text-base-content/60">私钥</label>
            <div class="flex gap-1">
              <input v-model="wgForm.privateKey" class="input input-bordered flex-1" placeholder="Base64 私钥" />
              <button class="btn btn-xs btn-ghost" @click="generateKeypair" :disabled="generatingKeys"><template v-if="generatingKeys">生成中...</template><template v-else><SvgIcon name="key" size="14" />  生成</template></button>
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

    <!-- 删除确认弹层（替代 window.confirm——Tauri 中原生 confirm 不弹窗） -->
    <div v-if="wgDeleteTarget" class="modal modal-open">
      <div class="modal-box">
        <div class="text-lg font-semibold text-base-content mb-3"><SvgIcon name="alertTriangle" size="14" /> 确认删除</div>
        <p class="text-sm text-base-content/70 mb-4">确定要删除 WireGuard 配置「{{ wgDeleteTarget.name }}」吗？此操作不可撤销。</p>
        <div class="flex justify-end gap-2">
          <button class="btn btn-sm btn-ghost" @click="wgDeleteTarget = null">取消</button>
          <button class="btn btn-sm btn-error" @click="confirmWgDelete">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'VPNManager' })
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const toast = useToast()

// ============ WireGuard State ============
interface WgConfig { id: string; name: string; privateKey: string; publicKey: string; address: string; dns?: string; mtu?: number; peerPublicKey: string; peerEndpoint: string; peerAllowedIPs: string; peerPersistentKeepalive?: number; presharedKey?: string; createdAt: string; updatedAt: string }
interface WgStatus { connected: boolean; configId: string | null; configName: string | null; state: string; log: string[]; connectedSince?: string; bytesSent: number; bytesReceived: number; latestHandshake?: string }

const wgConfigs = ref<WgConfig[]>([])
const wgStatus = ref<WgStatus>({ connected: false, configId: null, configName: null, state: 'disconnected', log: [], bytesSent: 0, bytesReceived: 0 })
const wgLogs = ref<string[]>([])
const wgDuration = ref('')
const wgLogRef = ref<HTMLElement | null>(null)
const wgUserScrolledUp = ref(false)
function onWgLogScroll() {
  const el = wgLogRef.value
  if (!el) {return}
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 50
  wgUserScrolledUp.value = !atBottom
}
function scrollWgToBottom() {
  wgUserScrolledUp.value = false
  if (wgLogRef.value) {wgLogRef.value.scrollTop = wgLogRef.value.scrollHeight}
}
let wgPolling: any = null
let wgDurationTimer: any = null

// WireGuard form
const showWgForm = ref(false)
const editingWg = ref<WgConfig | null>(null)
const generatingKeys = ref(false)
const wgForm = ref<Record<string, any>>({ name: '', privateKey: '', publicKey: '', address: '10.0.0.2/32', dns: '', mtu: 1420, peerPublicKey: '', peerEndpoint: '', peerAllowedIPs: '0.0.0.0/0', peerPersistentKeepalive: 25, presharedKey: '' })

// ============ Lifecycle ============
onMounted(async () => {
  await loadWgAll()
  await loadWgStatus()
  wgPolling = setInterval(loadWgStatus, 2000)
  wgDurationTimer = setInterval(updateWgDuration, 1000)
})

onUnmounted(() => {
  if (wgPolling) {clearInterval(wgPolling)}
  if (wgDurationTimer) {clearInterval(wgDurationTimer)}
})

watch(wgLogs, () => { nextTick(() => { if (wgLogRef.value && !wgUserScrolledUp.value) {wgLogRef.value.scrollTop = wgLogRef.value.scrollHeight} }) })

// ============ WireGuard Methods ============
async function loadWgAll() { try { wgConfigs.value = await getTauriAPI().wireguardGetAll() } catch(e:any) { console.error(e) } }
let wgStatusLoading = false
async function loadWgStatus() {
  // 防重入：2 秒轮询 + 手动触发可能重叠，慢响应时跳过本次
  if (wgStatusLoading) {return}
  wgStatusLoading = true
  try { const s = await getTauriAPI().wireguardGetStatus(); if (s) { wgStatus.value = s; if (s.log?.length) {wgLogs.value = s.log} } } catch {} finally { wgStatusLoading = false }
}
function updateWgDuration() { if (!wgStatus.value.connected || !wgStatus.value.connectedSince) { wgDuration.value = ''; return }; const s = (Date.now() - new Date(wgStatus.value.connectedSince).getTime()) / 1000; const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = Math.floor(s % 60); wgDuration.value = h > 0 ? `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}` : `${m}:${String(sec).padStart(2, '0')}` }

async function generateKeypair() { generatingKeys.value = true; try { const r = await getTauriAPI().wireguardGenerateKeypair(); if (r) { wgForm.value.privateKey = r.privateKey; wgForm.value.publicKey = r.publicKey; toast.success('密钥对已生成') } } catch(e:any) { toast.error('生成失败: ' + e.message) } finally { generatingKeys.value = false } }

function editWgConfig(cfg: WgConfig) { editingWg.value = cfg; wgForm.value = { name: cfg.name, privateKey: cfg.privateKey || '', publicKey: cfg.publicKey || '', address: cfg.address || '10.0.0.2/32', dns: cfg.dns || '', mtu: cfg.mtu, peerPublicKey: cfg.peerPublicKey, peerEndpoint: cfg.peerEndpoint, peerAllowedIPs: cfg.peerAllowedIPs || '0.0.0.0/0', peerPersistentKeepalive: cfg.peerPersistentKeepalive, presharedKey: cfg.presharedKey || '' }; showWgForm.value = true }

async function saveWgConfig() { try { const data = { ...wgForm.value, mtu: wgForm.value.mtu !== '' && wgForm.value.mtu != null ? Number(wgForm.value.mtu) : null, peerPersistentKeepalive: wgForm.value.peerPersistentKeepalive !== '' && wgForm.value.peerPersistentKeepalive != null ? Number(wgForm.value.peerPersistentKeepalive) : null }; if (editingWg.value) { await getTauriAPI().wireguardUpdate({ ...data, id: editingWg.value.id }) } else { await getTauriAPI().wireguardAdd(data)    }; const wasEditing = !!editingWg.value; showWgForm.value = false; editingWg.value = null; await loadWgAll(); toast.success(wasEditing ? '已更新' : '已添加')} catch(e:any) { toast.error('保存失败: ' + e.message) } }

async function wgConnect(cfg: WgConfig) { if (wgStatus.value.connected) { await wgDisconnect(); await new Promise(r => setTimeout(r, 500)) }; try { const r = await getTauriAPI().wireguardConnect(cfg.id, cfg.name, cfg.privateKey, cfg.peerPublicKey, cfg.peerEndpoint, cfg.presharedKey || undefined, cfg.address, cfg.mtu); if (!r?.success) {toast.error('连接失败: ' + (r?.error || '未知错误'));} await loadWgStatus() } catch(e:any) { toast.error('连接失败: ' + e.message) } }

async function wgDisconnect() { try { await getTauriAPI().wireguardDisconnect(); await loadWgStatus(); toast.info('已断开') } catch(e:any) { toast.error('断开失败: ' + e.message) } }
function selectWgConfig(cfg: WgConfig) {
  // 点击配置项：回填表单打开编辑（此前为空函数，点了没反应）
  editingWg.value = cfg
  wgForm.value = {
    name: cfg.name,
    privateKey: cfg.privateKey || '',
    publicKey: cfg.publicKey || '',
    address: cfg.address || '10.0.0.2/32',
    dns: cfg.dns || '',
    mtu: cfg.mtu ?? 1420,
    peerPublicKey: cfg.peerPublicKey || '',
    peerEndpoint: cfg.peerEndpoint || '',
    peerAllowedIPs: cfg.peerAllowedIPs || '0.0.0.0/0',
    peerPersistentKeepalive: cfg.peerPersistentKeepalive ?? 25,
    presharedKey: cfg.presharedKey || '',
  }
  showWgForm.value = true
}

// 待删除的 WireGuard 配置（替代 window.confirm——Tauri 中原生 confirm 不弹窗）
const wgDeleteTarget = ref<WgConfig | null>(null)

async function wgDelete(cfg: WgConfig) {
  wgDeleteTarget.value = cfg
}

async function confirmWgDelete() {
  if (!wgDeleteTarget.value) {return}
  const cfg = wgDeleteTarget.value
  wgDeleteTarget.value = null
  try {
    await getTauriAPI().wireguardDelete(cfg.id)
    await loadWgAll()
    toast.success('已删除')
  } catch (e: any) { toast.error('删除失败: ' + e.message) }
}

function formatBytes(bytes: number): string { if (bytes < 1024) {return bytes + ' B';} if (bytes < 1048576) {return (bytes / 1024).toFixed(1) + ' KB';} return (bytes / 1048576).toFixed(1) + ' MB' }

function getLogClass(line: string): string {
  if (line.includes('✅') || line.includes('成功') || line.includes('Completed')) {return 'text-[#a6e3a1]'}
  if (line.includes('❌') || line.includes('⚠️') || line.includes('错误') || line.includes('Error') || line.includes('FAILED')) {return 'text-[#f38ba8]'}
  if (line.includes('⏳') || line.includes('连接中')) {return 'text-[#f9e2af]'}
  return ''
}
</script>
