<template>
  <div class="server-monitor">
    <div class="monitor-header">
      <span>📊 系统监控</span>
      <button @click="$emit('toggle')" class="btn-collapse" title="隐藏">
        <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
      </button>
    </div>

    <div v-if="loading" class="monitor-loading">加载中...</div>
    <div v-else-if="error" class="monitor-error">{{ error }}</div>
    <div v-else class="monitor-content">
      <!-- 系统信息 -->
      <div class="monitor-section">
        <div class="section-title">系统信息</div>
        <div class="info-grid">
          <div class="info-item"><span class="label">主机名</span><span class="value">{{ info.hostname }}</span></div>
          <div class="info-item"><span class="label">操作系统</span><span class="value">{{ info.os }}</span></div>
          <div class="info-item"><span class="label">内核</span><span class="value">{{ info.kernel }}</span></div>
          <div class="info-item"><span class="label">运行时间</span><span class="value">{{ info.uptime }}</span></div>
          <div class="info-item"><span class="label">负载</span><span class="value">{{ info.loadavg }}</span></div>
        </div>
      </div>

      <!-- CPU -->
      <div class="monitor-section">
        <div class="section-title">CPU <span class="section-value">{{ stats.cpu }}%</span></div>
        <div class="sparkline-container">
          <canvas ref="cpuCanvas" width="220" height="40" class="sparkline"></canvas>
        </div>
      </div>

      <!-- 内存 -->
      <div class="monitor-section">
        <div class="section-title">内存 <span class="section-value">{{ stats.memoryUsed }}/{{ stats.memoryTotal }} GB ({{ stats.memoryPercent }}%)</span></div>
        <div class="progress-bar"><div class="progress-fill memory" :style="{ width: stats.memoryPercent + '%' }"></div></div>
      </div>

      <!-- Swap -->
      <div class="monitor-section" v-if="stats.swapTotal > 0">
        <div class="section-title">Swap <span class="section-value">{{ stats.swapUsed }}/{{ stats.swapTotal }} GB ({{ stats.swapPercent }}%)</span></div>
        <div class="progress-bar"><div class="progress-fill swap" :style="{ width: stats.swapPercent + '%' }"></div></div>
      </div>

      <!-- 磁盘 -->
      <div class="monitor-section">
        <div class="section-title">磁盘</div>
        <div v-for="disk in disks" :key="disk.mount" class="disk-item">
          <div class="disk-header">
            <span class="disk-name">{{ disk.mount }}</span>
            <span class="disk-value">{{ disk.used }}/{{ disk.total }} GB ({{ disk.percent }}%)</span>
          </div>
          <div class="progress-bar"><div class="progress-fill disk" :style="{ width: disk.percent + '%' }"></div></div>
        </div>
      </div>

      <!-- 网络 -->
      <div class="monitor-section">
        <div class="section-title">网络</div>
        <div class="network-grid">
          <div class="net-item">
            <span class="net-label">↑ 上传</span>
            <span class="net-value upload">{{ formatSpeed(stats.netUp) }}</span>
          </div>
          <div class="net-item">
            <span class="net-label">↓ 下载</span>
            <span class="net-value download">{{ formatSpeed(stats.netDown) }}</span>
          </div>
        </div>
        <div class="sparkline-container">
          <canvas ref="netCanvas" width="220" height="40" class="sparkline"></canvas>
        </div>
      </div>

      <!-- Top 进程 -->
      <div class="monitor-section">
        <div class="section-title">Top 进程</div>
        <div class="process-list">
          <div class="process-header">
            <span class="proc-pid">PID</span>
            <span class="proc-cpu">CPU</span>
            <span class="proc-mem">MEM</span>
            <span class="proc-cmd">命令</span>
          </div>
          <div v-for="p in processes" :key="p.pid" class="process-row">
            <span class="proc-pid">{{ p.pid }}</span>
            <span class="proc-cpu">{{ p.cpu }}%</span>
            <span class="proc-mem">{{ p.mem }}%</span>
            <span class="proc-cmd" :title="p.cmd">{{ p.cmd }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
console.log("[components/server/ServerMonitor.vue] component loaded")
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
  if (bytesPerSec < 1024) return bytesPerSec.toFixed(0) + ' B/s'
  if (bytesPerSec < 1024 * 1024) return (bytesPerSec / 1024).toFixed(1) + ' KB/s'
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
    const r = (cmd: string) => (resp.results![cmd] || '').trim()

    // CPU
    const cpuRaw = r(commands[0])
    let cpu = 0
    const cpuMatch = cpuRaw.match(/(\d+\.?\d*)\s*id/)
    if (cpuMatch) cpu = Math.round(100 - parseFloat(cpuMatch[1]))

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
    if (cpuHistory.value.length > MAX_HISTORY) cpuHistory.value.shift()
    netHistory.value.push({ up: netUp, down: netDown })
    if (netHistory.value.length > MAX_HISTORY) netHistory.value.shift()

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
  if (!canvas || data.length < 2) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return
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
  if (timer) clearInterval(timer)
})
</script>

<style scoped>
.server-monitor {
  width: 260px;
  min-width: 260px;
  background: #181825;
  border-left: 1px solid #313244;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  overflow-x: hidden;
}

.monitor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #11111b;
  border-bottom: 1px solid #313244;
  font-size: 12px;
  font-weight: 600;
  color: #cdd6f4;
  flex-shrink: 0;
}

.btn-collapse {
  background: transparent;
  border: none;
  color: #6c7086;
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  display: flex;
  align-items: center;
}

.btn-collapse:hover {
  background: rgba(205, 214, 244, 0.1);
  color: #cdd6f4;
}

.monitor-loading, .monitor-error {
  padding: 20px;
  text-align: center;
  font-size: 12px;
  color: #6c7086;
}

.monitor-error { color: #f38ba8; }

.monitor-content {
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.monitor-section {
  background: #1e1e2e;
  border-radius: 8px;
  padding: 10px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  color: #a6adc8;
  margin-bottom: 6px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-value { color: #cdd6f4; font-weight: 500; }

.info-grid { display: grid; gap: 4px; }

.info-item {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.info-item .label { color: #6c7086; }

.info-item .value {
  color: #cdd6f4;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
}

.sparkline-container { height: 40px; margin-top: 4px; }
.sparkline { width: 100%; height: 40px; }

.progress-bar {
  height: 6px;
  background: #313244;
  border-radius: 3px;
  overflow: hidden;
  margin-top: 4px;
}

.progress-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-fill.memory { background: linear-gradient(90deg, #89b4fa, #74c7ec); }
.progress-fill.swap { background: linear-gradient(90deg, #f9e2af, #fab387); }
.progress-fill.disk { background: linear-gradient(90deg, #a6e3a1, #94e2d5); }

.disk-item { margin-bottom: 6px; }
.disk-item:last-child { margin-bottom: 0; }

.disk-header {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  margin-bottom: 2px;
}

.disk-name { color: #a6adc8; }
.disk-value { color: #cdd6f4; }

.network-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px;
}

.net-item {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.net-label { color: #6c7086; }
.net-value { font-weight: 500; }
.net-value.upload { color: #f38ba8; }
.net-value.download { color: #a6e3a1; }

.process-list { font-size: 10px; }

.process-header {
  display: grid;
  grid-template-columns: 40px 40px 40px 1fr;
  gap: 4px;
  color: #6c7086;
  font-weight: 600;
  padding: 4px 0;
  border-bottom: 1px solid #313244;
}

.process-row {
  display: grid;
  grid-template-columns: 40px 40px 40px 1fr;
  gap: 4px;
  padding: 3px 0;
  color: #cdd6f4;
}

.process-row:hover { background: rgba(137, 180, 250, 0.05); }
.proc-cmd { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
