<template>
  <div class="w-[260px] min-w-[260px] bg-[#181825] border-l border-[#313244] flex flex-col overflow-y-auto overflow-x-hidden">
    <div class="flex justify-between items-center px-3 py-2 bg-[#11111b] border-b border-[#313244] text-[12px] font-semibold text-[#cdd6f4] flex-shrink-0">
      <span class="flex items-center gap-1"><SvgIcon name="barChart" size="12" /> 系统监控</span>
      <button @click="$emit('toggle')" class="bg-transparent border-none text-[#6c7086] cursor-pointer p-0.5 rounded flex items-center hover:bg-[rgba(205,214,244,0.1)] hover:text-[#cdd6f4]" title="隐藏">
        <SvgIcon name="chevronRight" size="12" />
      </button>
    </div>

    <div v-if="loading" class="p-5 text-center text-xs text-[#6c7086]">加载中...</div>
    <div v-else-if="error" class="p-5 text-center text-xs text-[#f38ba8]">{{ error }}</div>
    <div v-else class="p-2 flex flex-col gap-2.5">
      <!-- 系统信息 -->
      <div class="bg-[#1e1e2e] rounded-lg p-2.5">
        <div class="text-[11px] font-semibold text-[#a6adc8] mb-1.5 flex justify-between items-center">系统信息</div>
        <div class="grid gap-1">
          <div class="flex justify-between text-[11px]">
            <span class="text-[#6c7086]">主机名</span>
            <span class="text-[#cdd6f4] max-w-[120px] truncate text-right">{{ info.hostname }}</span>
          </div>
          <div class="flex justify-between text-[11px]">
            <span class="text-[#6c7086]">操作系统</span>
            <span class="text-[#cdd6f4] max-w-[120px] truncate text-right">{{ info.os }}</span>
          </div>
          <div class="flex justify-between text-[11px]">
            <span class="text-[#6c7086]">内核</span>
            <span class="text-[#cdd6f4] max-w-[120px] truncate text-right">{{ info.kernel }}</span>
          </div>
          <div class="flex justify-between text-[11px]">
            <span class="text-[#6c7086]">运行时间</span>
            <span class="text-[#cdd6f4] max-w-[120px] truncate text-right">{{ info.uptime }}</span>
          </div>
          <div class="flex justify-between text-[11px]">
            <span class="text-[#6c7086]">负载</span>
            <span class="text-[#cdd6f4] max-w-[120px] truncate text-right">{{ info.loadavg }}</span>
          </div>
        </div>
      </div>

      <!-- CPU -->
      <div class="bg-[#1e1e2e] rounded-lg p-2.5">
        <div class="text-[11px] font-semibold text-[#a6adc8] mb-1.5 flex justify-between items-center">
          CPU <span class="text-[#cdd6f4] font-medium">{{ stats.cpu }}%</span>
        </div>
        <div class="h-10 mt-1">
          <canvas ref="cpuCanvas" width="220" height="40" class="w-full h-10"></canvas>
        </div>
      </div>

      <!-- 内存 -->
      <div class="bg-[#1e1e2e] rounded-lg p-2.5">
        <div class="text-[11px] font-semibold text-[#a6adc8] mb-1.5 flex justify-between items-center">
          内存 <span class="text-[#cdd6f4] font-medium">{{ stats.memoryUsed }}/{{ stats.memoryTotal }} GB ({{ stats.memoryPercent }}%)</span>
        </div>
        <div class="h-1.5 bg-[#313244] rounded overflow-hidden mt-1">
          <div class="h-full rounded transition-[width] duration-300 bg-gradient-to-r from-[#89b4fa] to-[#74c7ec]" :style="{ width: stats.memoryPercent + '%' }"></div>
        </div>
      </div>

      <!-- Swap -->
      <div class="bg-[#1e1e2e] rounded-lg p-2.5" v-if="stats.swapTotal > 0">
        <div class="text-[11px] font-semibold text-[#a6adc8] mb-1.5 flex justify-between items-center">
          Swap <span class="text-[#cdd6f4] font-medium">{{ stats.swapUsed }}/{{ stats.swapTotal }} GB ({{ stats.swapPercent }}%)</span>
        </div>
        <div class="h-1.5 bg-[#313244] rounded overflow-hidden mt-1">
          <div class="h-full rounded transition-[width] duration-300 bg-gradient-to-r from-[#f9e2af] to-[#fab387]" :style="{ width: stats.swapPercent + '%' }"></div>
        </div>
      </div>

      <!-- 磁盘 -->
      <div class="bg-[#1e1e2e] rounded-lg p-2.5">
        <div class="text-[11px] font-semibold text-[#a6adc8] mb-1.5 flex justify-between items-center">磁盘</div>
        <div v-for="disk in disks" :key="disk.mount" class="mb-1.5 last:mb-0">
          <div class="flex justify-between text-[10px] mb-0.5">
            <span class="text-[#a6adc8]">{{ disk.mount }}</span>
            <span class="text-[#cdd6f4]">{{ disk.used }}/{{ disk.total }} GB ({{ disk.percent }}%)</span>
          </div>
          <div class="h-1.5 bg-[#313244] rounded overflow-hidden mt-1">
            <div class="h-full rounded transition-[width] duration-300 bg-gradient-to-r from-[#a6e3a1] to-[#94e2d5]" :style="{ width: disk.percent + '%' }"></div>
          </div>
        </div>
      </div>

      <!-- 网络 -->
      <div class="bg-[#1e1e2e] rounded-lg p-2.5">
        <div class="text-[11px] font-semibold text-[#a6adc8] mb-1.5 flex justify-between items-center">网络</div>
        <div class="grid grid-cols-2 gap-1">
          <div class="flex justify-between text-[11px]">
            <span class="text-[#6c7086]">↑ 上传</span>
            <span class="font-medium text-[#f38ba8]">{{ formatSpeed(stats.netUp) }}</span>
          </div>
          <div class="flex justify-between text-[11px]">
            <span class="text-[#6c7086]">↓ 下载</span>
            <span class="font-medium text-[#a6e3a1]">{{ formatSpeed(stats.netDown) }}</span>
          </div>
        </div>
        <div class="h-10 mt-1">
          <canvas ref="netCanvas" width="220" height="40" class="w-full h-10"></canvas>
        </div>
      </div>

      <!-- Top 进程 -->
      <div class="bg-[#1e1e2e] rounded-lg p-2.5">
        <div class="text-[11px] font-semibold text-[#a6adc8] mb-1.5 flex justify-between items-center">Top 进程</div>
        <div class="text-[10px]">
          <div class="grid grid-cols-[40px_40px_40px_1fr] gap-1 text-[#6c7086] font-semibold py-1 border-b border-[#313244]">
            <span>PID</span>
            <span>CPU</span>
            <span>MEM</span>
            <span>命令</span>
          </div>
          <div v-for="p in processes" :key="p.pid" class="grid grid-cols-[40px_40px_40px_1fr] gap-1 py-0.5 text-[#cdd6f4] hover:bg-[rgba(137,180,250,0.05)]">
            <span>{{ p.pid }}</span>
            <span>{{ p.cpu }}%</span>
            <span>{{ p.mem }}%</span>
            <span class="truncate" :title="p.cmd">{{ p.cmd }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'

interface MonitorStats {
  cpu: number
  memoryUsed: number
  memoryTotal: number
  memoryPercent: number
  swapUsed: number
  swapTotal: number
  swapPercent: number
  netUp: number
  netDown: number
}

interface MonitorInfo {
  hostname: string
  os: string
  kernel: string
  uptime: string
  loadavg: string
}

interface DiskInfo {
  mount: string
  used: number
  total: number
  percent: number
}

interface ProcessInfo {
  pid: string
  cpu: string
  mem: string
  cmd: string
}

const props = defineProps<{
  serverId: string
}>()

const emit = defineEmits(['toggle'])

const loading = ref(true)
const error = ref('')
const stats = ref<MonitorStats>({ cpu: 0, memoryUsed: 0, memoryTotal: 0, memoryPercent: 0, swapUsed: 0, swapTotal: 0, swapPercent: 0, netUp: 0, netDown: 0 })
const info = ref<MonitorInfo>({ hostname: '-', os: '-', kernel: '-', uptime: '-', loadavg: '-' })
const disks = ref<DiskInfo[]>([])
const processes = ref<ProcessInfo[]>([])

const cpuCanvas = ref<HTMLCanvasElement | null>(null)
const netCanvas = ref<HTMLCanvasElement | null>(null)

const cpuHistory = ref<number[]>([])
const netHistory = ref<{ up: number; down: number }[]>([])
const MAX_HISTORY = 30

let timer: ReturnType<typeof setInterval> | null = null
let prevNetTotal: { rx: number; tx: number } | null = null

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec < 1024) {return bytesPerSec.toFixed(0) + ' B/s'}
  if (bytesPerSec < 1024 * 1024) {return (bytesPerSec / 1024).toFixed(1) + ' KB/s'}
  return (bytesPerSec / 1024 / 1024).toFixed(1) + ' MB/s'
}

async function refresh() {
  try {
    const commands = [
      'top -bn1 | grep "%Cpu" | head -1',
      "free -b | grep '^Mem:'",
      "free -b | grep '^Swap:'",
      "cat /proc/net/dev | tail -n +3 | awk '{rx+=$2; tx+=$10} END {print rx, tx}'",
      'hostname',
      "cat /etc/os-release 2>/dev/null | grep PRETTY_NAME | cut -d= -f2 | tr -d '\"' || uname -s",
      'uname -r',
      'cat /proc/uptime | cut -d" " -f1 | cut -d. -f1',
      'cat /proc/loadavg | cut -d" " -f1-3',
      "df -BG --output=target,size,used,pcent 2>/dev/null | grep '^/' | grep -v loop",
      "ps aux --sort=-%cpu | head -6",
    ]

    const resp = await getTauriAPI().getServerMonitor(props.serverId, commands)
    if (!resp?.success || !resp.results) { error.value = '获取数据失败'; return }
    const r = (cmd: string) => {
      const result = resp.results?.[cmd]
      return (typeof result === 'string' ? result : result?.output || '').trim()
    }

    // CPU
    const cpuRaw = r(commands[0])
    let cpu = 0
    const cpuMatch = cpuRaw.match(/(\d+\.?\d*)\s*id/)
    if (cpuMatch) {cpu = Math.round(100 - parseFloat(cpuMatch[1]))}

    // Memory
    const memParts = r(commands[1]).split(/\s+/)
    const memTotal = parseInt(memParts[1]) || 0
    const memUsed = parseInt(memParts[2]) || 0

    // Swap
    const swapParts = r(commands[2]).split(/\s+/)
    const swapTotal = parseInt(swapParts[1]) || 0
    const swapUsed = parseInt(swapParts[2]) || 0

    // Network
    const netParts = r(commands[3]).split(/\s+/)
    const rx = parseInt(netParts[0]) || 0
    const tx = parseInt(netParts[1]) || 0
    let netUp = 0, netDown = 0
    if (prevNetTotal) {
      netUp = Math.max(0, tx - prevNetTotal.tx)
      netDown = Math.max(0, rx - prevNetTotal.rx)
    }
    prevNetTotal = { rx, tx }

    stats.value = {
      cpu,
      memoryUsed: Math.round(memUsed / 1024 / 1024 / 1024 * 10) / 10,
      memoryTotal: Math.round(memTotal / 1024 / 1024 / 1024 * 10) / 10,
      memoryPercent: memTotal > 0 ? Math.round(memUsed / memTotal * 100) : 0,
      swapUsed: swapTotal > 0 ? Math.round(swapUsed / 1024 / 1024 / 1024 * 10) / 10 : 0,
      swapTotal: Math.round(swapTotal / 1024 / 1024 / 1024 * 10) / 10,
      swapPercent: swapTotal > 0 ? Math.round(swapUsed / swapTotal * 100) : 0,
      netUp, netDown,
    }

    cpuHistory.value.push(cpu)
    if (cpuHistory.value.length > MAX_HISTORY) {cpuHistory.value.shift()}
    netHistory.value.push({ up: netUp, down: netDown })
    if (netHistory.value.length > MAX_HISTORY) {netHistory.value.shift()}

    drawSparklines()

    // System info
    const uptimeSec = parseInt(r(commands[7])) || 0
    const d = Math.floor(uptimeSec / 86400)
    const h = Math.floor((uptimeSec % 86400) / 3600)
    const m = Math.floor((uptimeSec % 3600) / 60)
    info.value = {
      hostname: r(commands[4]) || '-',
      os: r(commands[5]) || '-',
      kernel: r(commands[6]) || '-',
      uptime: d > 0 ? `${d}天${h}时${m}分` : `${h}时${m}分`,
      loadavg: r(commands[8]) || '-',
    }

    // Disks
    const diskLines = r(commands[9]).split('\n').filter(l => l.trim())
    disks.value = diskLines.map(line => {
      const dm = line.match(/^(\S+)\s+(\d+)G\s+(\d+)G\s+(\d+)%/)
      return dm ? { mount: dm[1], used: parseInt(dm[3]), total: parseInt(dm[2]), percent: parseInt(dm[4]) } : null
    }).filter(Boolean) as DiskInfo[]

    // Processes
    const procLines = r(commands[10]).split('\n').slice(1)
    processes.value = procLines.map(line => {
      const parts = line.trim().split(/\s+/)
      return parts.length >= 11 ? { pid: parts[1], cpu: parts[2], mem: parts[3], cmd: parts[10] } : null
    }).filter(Boolean) as ProcessInfo[]

    loading.value = false
    error.value = ''
  } catch {
    error.value = '获取数据失败'
  }
}

function drawSparklines() {
  drawLine(cpuCanvas.value, cpuHistory.value, '#89b4fa')
  drawLine(netCanvas.value, netHistory.value.map(h => h.down), '#a6e3a1')
}

function drawLine(canvas: HTMLCanvasElement | null, data: number[], color: string) {
  if (!canvas || data.length < 2) {return}
  const ctx = canvas.getContext('2d')
  if (!ctx) {return}
  const w = canvas.width, h = canvas.height
  ctx.clearRect(0, 0, w, h)

  const max = Math.max(...data, 1)
  const step = w / (MAX_HISTORY - 1)
  const startX = (MAX_HISTORY - data.length) * step

  const grad = ctx.createLinearGradient(0, 0, 0, h)
  grad.addColorStop(0, color + '40')
  grad.addColorStop(1, color + '05')

  ctx.beginPath()
  ctx.moveTo(startX, h)
  data.forEach((v, i) => {
    ctx.lineTo(startX + i * step, h - (v / max) * (h - 4))
  })
  ctx.lineTo(startX + (data.length - 1) * step, h)
  ctx.closePath()
  ctx.fillStyle = grad
  ctx.fill()

  ctx.beginPath()
  data.forEach((v, i) => {
    const x = startX + i * step
    const y = h - (v / max) * (h - 4)
    i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)
  })
  ctx.strokeStyle = color
  ctx.lineWidth = 1.5
  ctx.stroke()
}

watch(() => props.serverId, () => {
  prevNetTotal = null
  cpuHistory.value = []
  netHistory.value = []
  loading.value = true
  refresh()
})

onMounted(() => {
  refresh()
  timer = setInterval(refresh, 2000)
})

onUnmounted(() => {
  if (timer) {clearInterval(timer)}
})
</script>
