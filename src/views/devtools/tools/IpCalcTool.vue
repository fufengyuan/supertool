<template>
  <ToolPage
    icon="server"
    name="IP 网络计算器"
    description="CIDR 子网计算：掩码、网络/广播地址、可用主机、二进制与进制转换"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex gap-2 items-center">
        <input
          v-model="input"
          class="input input-bordered input-sm w-full font-mono bg-base-200/60"
          placeholder="输入 IP/CIDR，如 192.168.1.0/24 或 10.0.0.0/8"
          @keyup.enter="calculate"
        />
        <button class="btn btn-primary btn-sm shrink-0" @click="calculate">计算</button>
        <button class="btn btn-outline btn-sm shrink-0" @click="copyAll" :disabled="!result"><SvgIcon name="copy" size="12" /> 复制</button>
        <button class="btn btn-ghost btn-xs shrink-0" @click="clear" :disabled="!input && !result">清空</button>
      </div>
      <div v-if="error" class="mt-3 p-3 bg-error/10 border border-error/25 rounded-lg text-error text-sm">{{ error }}</div>

      <div v-if="result" class="mt-4">
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2.5">
          <div v-for="cell in resultCards" :key="cell.label" class="flex flex-col p-3 bg-base-200/60 border border-base-content/10 rounded-xl">
            <span class="text-[11px] font-medium text-base-content/50 mb-1">{{ cell.label }}</span>
            <span class="text-sm text-base-content font-mono break-all">{{ cell.value }}</span>
          </div>
        </div>

        <div class="mt-4">
          <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="code" size="12" /> 二进制表示</h4>
          <div class="flex flex-col gap-1.5">
            <div v-for="row in binaryRows" :key="row.label" class="flex items-center gap-2.5 p-2 bg-base-200/60 border border-base-content/10 rounded-lg">
              <span class="text-[11px] text-base-content/50 w-20 shrink-0">{{ row.label }}</span>
              <span class="font-mono text-[11px] text-base-content break-all">{{ row.value }}</span>
            </div>
          </div>
        </div>

        <div class="mt-4">
          <h4 class="text-xs font-semibold text-base-content/70 mb-2.5 flex items-center gap-1.5"><SvgIcon name="pencil" size="12" /> IP 进制转换</h4>
          <div class="flex flex-col gap-1.5">
            <div v-for="conv in conversionRows" :key="conv.label" class="flex items-center gap-2.5 p-2 bg-base-200/60 border border-base-content/10 rounded-lg group">
              <span class="text-[11px] text-base-content/50 w-16 shrink-0">{{ conv.label }}</span>
              <span class="flex-1 font-mono text-xs text-base-content break-all">{{ conv.value }}</span>
              <button class="btn btn-ghost btn-xs shrink-0 opacity-0 group-hover:opacity-100 transition-opacity" @click="doCopy(conv.value)"><SvgIcon name="copy" size="11" /></button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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

const resultCards = computed(() => {
  const r = result.value
  if (!r) { return [] }
  return [
    { label: 'IP 地址', value: r.ip },
    { label: 'CIDR', value: `/${r.cidr}` },
    { label: '子网掩码', value: r.mask },
    { label: '反掩码 (Wildcard)', value: r.wildcard },
    { label: '网络地址', value: r.network },
    { label: '广播地址', value: r.broadcast },
    { label: '首个可用 IP', value: r.firstUsable },
    { label: '末个可用 IP', value: r.lastUsable },
    { label: '可用主机数', value: r.hostCount },
    { label: 'IP 类型', value: r.ipClass },
  ]
})

const binaryRows = computed(() => {
  const r = result.value
  if (!r) { return [] }
  return [
    { label: 'IP 地址', value: r.ipBinary },
    { label: '子网掩码', value: r.maskBinary },
    { label: '网络地址', value: r.networkBinary },
  ]
})

const conversionRows = computed(() => {
  const r = result.value
  if (!r) { return [] }
  return [
    { label: '十进制', value: r.ipDecimal },
    { label: '十六进制', value: r.ipHex },
    { label: '八进制', value: r.ipOctal },
  ]
})

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
  if (parts.length !== 4) {return false}
  return parts.every(p => {
    const n = parseInt(p, 10)
    return n >= 0 && n <= 255 && String(n) === p
  })
}

function getIpClass(ip: string): string {
  const first = parseInt(ip.split('.')[0], 10)
  if (first >= 1 && first <= 126) {return 'A 类'}
  if (first >= 128 && first <= 191) {return 'B 类'}
  if (first >= 192 && first <= 223) {return 'C 类'}
  if (first >= 224 && first <= 239) {return 'D 类 (组播)'}
  if (first >= 240 && first <= 255) {return 'E 类 (保留)'}
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
    if (first >= 1 && first <= 126) {cidr = 8}
    else if (first >= 128 && first <= 191) {cidr = 16}
    else if (first >= 192 && first <= 223) {cidr = 24}
    else {cidr = 24}
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