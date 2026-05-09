<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5">🖧 IP 网络计算器</h3>

    <div class="mb-5">
      <label class="text-xs font-medium text-base-content/60 mb-1 block">IP 地址 / CIDR</label>
      <input
        v-model="input"
        class="input input-bordered w-full font-mono text-xs"
        placeholder="输入 IP/CIDR，如 192.168.1.0/24 或 10.0.0.0/8"
        @keyup.enter="calculate"
      />

      <div class="flex gap-2.5 mb-3 flex-wrap items-center mt-3">
        <button class="btn btn-primary btn-sm" @click="calculate">计算</button>
        <button class="btn btn-ghost btn-sm" @click="copyAll"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> 复制全部</button>
        <button class="btn btn-ghost btn-sm" @click="clear">清空</button>
      </div>

      <div v-if="error" class="error-box">{{ error }}</div>

      <div v-if="result" class="results-grid">
        <div class="result-card">
          <div class="result-label">IP 地址</div>
          <div class="result-value">{{ result.ip }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">CIDR</div>
          <div class="result-value">/{{ result.cidr }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">子网掩码</div>
          <div class="result-value">{{ result.mask }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">反掩码 (Wildcard)</div>
          <div class="result-value">{{ result.wildcard }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">网络地址</div>
          <div class="result-value">{{ result.network }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">广播地址</div>
          <div class="result-value">{{ result.broadcast }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">首个可用 IP</div>
          <div class="result-value">{{ result.firstUsable }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">末个可用 IP</div>
          <div class="result-value">{{ result.lastUsable }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">可用主机数</div>
          <div class="result-value">{{ result.hostCount }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">IP 类型</div>
          <div class="result-value">{{ result.ipClass }}</div>
        </div>
      </div>

      <!-- Binary representation -->
      <div v-if="result" class="binary-section" style="margin-top: 20px">
        <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">二进制表示</h4>
        <div class="binary-table">
          <div class="binary-row">
            <span class="binary-label">IP 地址:</span>
            <span class="binary-value">{{ result.ipBinary }}</span>
          </div>
          <div class="binary-row">
            <span class="binary-label">子网掩码:</span>
            <span class="binary-value">{{ result.maskBinary }}</span>
          </div>
          <div class="binary-row">
            <span class="binary-label">网络地址:</span>
            <span class="binary-value">{{ result.networkBinary }}</span>
          </div>
        </div>
      </div>

      <!-- IP conversion -->
      <div v-if="result" class="conversion-section" style="margin-top: 20px">
        <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">IP 进制转换</h4>
        <div class="conversion-grid">
          <div class="conversion-item">
            <span class="conversion-label">十进制</span>
            <span class="conversion-value">{{ result.ipDecimal }}</span>
            <button class="btn btn-ghost btn-sm" @click="doCopy(result.ipDecimal)"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg></button>
          </div>
          <div class="conversion-item">
            <span class="conversion-label">十六进制</span>
            <span class="conversion-value">{{ result.ipHex }}</span>
            <button class="btn btn-ghost btn-sm" @click="doCopy(result.ipHex)"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg></button>
          </div>
          <div class="conversion-item">
            <span class="conversion-label">八进制</span>
            <span class="conversion-value">{{ result.ipOctal }}</span>
            <button class="btn btn-ghost btn-sm" @click="doCopy(result.ipOctal)"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg></button>
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
const error = ref('')

interface IpResult {
  ip: string
  cidr: number
  mask: string
  wildcard: string
  network: string
  broadcast: string
  firstUsable: string
  lastUsable: string
  hostCount: string
  ipClass: string
  ipBinary: string
  maskBinary: string
  networkBinary: string
  ipDecimal: string
  ipHex: string
  ipOctal: string
}

const result = ref<IpResult | null>(null)

function ipToLong(ip: string): number {
  const parts = ip.split('.').map(Number)
  return ((parts[0] << 24) | (parts[1] << 16) | (parts[2] << 8) | parts[3]) >>> 0
}

function longToIp(long: number): string {
  return [
    (long >>> 24) & 0xFF,
    (long >>> 16) & 0xFF,
    (long >>> 8) & 0xFF,
    long & 0xFF,
  ].join('.')
}

function longToBinary(long: number): string {
  return long.toString(2).padStart(32, '0').replace(/(.{8})/g, '$1.').slice(0, -1)
}

function isValidIp(ip: string): boolean {
  const parts = ip.split('.')
  if (parts.length !== 4) return false
  return parts.every(p => {
    const n = parseInt(p, 10)
    return n >= 0 && n <= 255 && String(n) === p
  })
}

function getIpClass(ip: string): string {
  const first = parseInt(ip.split('.')[0], 10)
  if (first >= 1 && first <= 126) return 'A 类'
  if (first >= 128 && first <= 191) return 'B 类'
  if (first >= 192 && first <= 223) return 'C 类'
  if (first >= 224 && first <= 239) return 'D 类 (组播)'
  if (first >= 240 && first <= 255) return 'E 类 (保留)'
  return '未知'
}

function calculate() {
  error.value = ''
  result.value = null

  if (!input.value.trim()) {
    toast.warning('请输入 IP/CIDR')
    return
  }

  let ip: string
  let cidr: number

  const parts = input.value.trim().split('/')
  ip = parts[0]

  if (!isValidIp(ip)) {
    error.value = '无效的 IP 地址格式'
    return
  }

  if (parts[1] !== undefined) {
    cidr = parseInt(parts[1], 10)
    if (isNaN(cidr) || cidr < 0 || cidr > 32) {
      error.value = 'CIDR 前缀必须在 0-32 之间'
      return
    }
  } else {
    // Guess CIDR based on IP class
    const first = parseInt(ip.split('.')[0], 10)
    if (first >= 1 && first <= 126) cidr = 8
    else if (first >= 128 && first <= 191) cidr = 16
    else if (first >= 192 && first <= 223) cidr = 24
    else cidr = 24
  }

  const ipLong = ipToLong(ip)
  const maskLong = cidr === 0 ? 0 : (~0 << (32 - cidr)) >>> 0
  const wildcardLong = (~maskLong) >>> 0
  const networkLong = (ipLong & maskLong) >>> 0
  const broadcastLong = (networkLong | wildcardLong) >>> 0
  const firstUsableLong = cidr >= 31 ? networkLong : (networkLong + 1) >>> 0
  const lastUsableLong = cidr >= 31 ? broadcastLong : (broadcastLong - 1) >>> 0
  const hostCount = cidr >= 31 ? (cidr === 32 ? '1' : '2') : String(Math.pow(2, 32 - cidr) - 2)

  result.value = {
    ip,
    cidr,
    mask: longToIp(maskLong),
    wildcard: longToIp(wildcardLong),
    network: longToIp(networkLong),
    broadcast: longToIp(broadcastLong),
    firstUsable: longToIp(firstUsableLong),
    lastUsable: longToIp(lastUsableLong),
    hostCount,
    ipClass: getIpClass(ip),
    ipBinary: longToBinary(ipLong),
    maskBinary: longToBinary(maskLong),
    networkBinary: longToBinary(networkLong),
    ipDecimal: String(ipLong >>> 0),
    ipHex: '0x' + (ipLong >>> 0).toString(16).toUpperCase().padStart(8, '0'),
    ipOctal: '0o' + (ipLong >>> 0).toString(8),
  }
}

function copyAll() {
  if (!result.value) {
    toast.warning('没有可复制的结果')
    return
  }
  const text = [
    `IP 地址: ${result.value.ip}/${result.value.cidr}`,
    `子网掩码: ${result.value.mask}`,
    `反掩码: ${result.value.wildcard}`,
    `网络地址: ${result.value.network}`,
    `广播地址: ${result.value.broadcast}`,
    `首个可用 IP: ${result.value.firstUsable}`,
    `末个可用 IP: ${result.value.lastUsable}`,
    `可用主机数: ${result.value.hostCount}`,
  ].join('\n')
  copyText(text, toast)
}

function doCopy(text: string) {
  copyText(text, toast)
}

function clear() {
  input.value = ''
  error.value = ''
  result.value = null
}
</script>