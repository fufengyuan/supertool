<template>
  <div
    class="terminal-panel fixed inset-0 w-screen h-screen bg-[#1e1e2e] flex flex-col z-[1000] shadow-2xl overflow-hidden min-w-[400px] min-h-[300px] transition-[left,top,width,height,border-radius] duration-200 ease-in-out"
    @mousedown="focusActiveTerminal"
  >
    <!-- 窗口标题栏 -->
    <div
      class="flex justify-between items-center px-4 py-[10px] bg-[#181825] border-b border-[#313244] cursor-move select-none shrink-0"
      @mousedown="startDrag"
    >
      <div class="flex items-center gap-2 text-[#cdd6f4] min-w-0">
        <SvgIcon name="terminal" size="16" />
        <span class="font-semibold text-sm">终端</span>
      </div>
      <div class="flex items-center gap-1.5 shrink-0">
        <button
          @click.stop="showMonitor = !showMonitor"
          class="btn btn-ghost btn-xs h-7 w-7 min-h-0 rounded-md p-0"
          :class="{ 'text-[#89b4fa] bg-[rgba(137,180,250,0.15)]': showMonitor }"
          :title="showMonitor ? '隐藏监控' : '显示监控'"
        >
          <SvgIcon name="monitor" size="14" />
        </button>
        <button
          @click.stop="quickOpenSftp"
          class="btn btn-ghost btn-xs h-7 w-7 min-h-0 rounded-md p-0 text-[#a6e3a1]"
          title="在当前路径打开 SFTP"
        >
          <SvgIcon name="folder" size="14" />
        </button>
        <button
          @click.stop="maximizeToggle"
          class="btn btn-ghost btn-xs h-7 w-7 min-h-0 rounded-md p-0"
          :title="isMaximized ? '还原' : '最大化'"
        >
          <SvgIcon v-if="!isMaximized" name="stopSquare" size="14" />
          <SvgIcon v-else name="stopSquare" size="14" />
        </button>
        <button
          @click.stop="$emit('close')"
          class="btn btn-xs h-7 w-7 min-h-0 rounded-full bg-[#f38ba8] text-white hover:bg-[#e04560] hover:scale-110 border-none p-0"
          title="关闭终端"
        >
          <SvgIcon name="x" size="18" />
        </button>
      </div>
    </div>

    <!-- 标签栏 -->
    <div class="flex items-stretch bg-[#11111b] border-b border-[#313244] overflow-x-auto overflow-y-hidden shrink-0">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="group flex items-center gap-1.5 px-3 py-2 min-w-0 max-w-[220px] bg-transparent text-[#6c7086] text-xs font-medium cursor-pointer border-r border-[#313244] transition-all duration-150 whitespace-nowrap relative hover:bg-[#1e1e2e] hover:text-[#cdd6f4]"
        :class="[activeTabId === tab.id ? 'bg-[#1e1e2e] text-[#cdd6f4]' : '']"
        @click="switchTab(tab.id)"
      >
        <span
          class="w-1.5 h-1.5 rounded-full shrink-0"
          :class="{
            'bg-[#f9e2af] animate-pulse': tab.status === 'connecting',
            'bg-[#a6e3a1] shadow-[0_0_4px_#a6e3a1]': tab.status === 'connected',
            'bg-[#f38ba8]': tab.status === 'disconnected' || tab.status === 'error'
          }"
        ></span>
        <span class="truncate flex-1 min-w-0">{{ tab.server.name }}</span>
        <button
          @click.stop="duplicateTab(tab)"
          class="bg-transparent border-none text-[#6c7086] w-4 h-4 rounded flex items-center justify-center cursor-pointer opacity-0 group-hover:opacity-70 hover:!opacity-100 hover:text-[#a6e3a1] hover:bg-[rgba(166,227,161,0.15)] transition-all duration-150 shrink-0 p-0"
          title="复制标签"
        >
          <SvgIcon name="file" size="10" />
        </button>
        <button
          @click.stop="reconnectTab(tab)"
          class="bg-transparent border-none text-[#6c7086] w-[18px] h-[18px] rounded flex items-center justify-center cursor-pointer opacity-0 group-hover:opacity-100 hover:text-[#89b4fa] hover:bg-[rgba(137,180,250,0.15)] transition-all duration-150 shrink-0 p-0"
          title="重连"
        >
          <SvgIcon :class="{ 'animate-spin': tab.status === 'connecting' }" name="refresh" size="12" stroke-width="2.5" />
        </button>
        <button
          @click.stop="closeTab(tab.id)"
          class="bg-transparent border-none text-[#6c7086] w-4 h-4 rounded flex items-center justify-center cursor-pointer opacity-0 group-hover:opacity-70 hover:!opacity-100 hover:text-[#f38ba8] hover:bg-[rgba(243,139,168,0.15)] transition-all duration-150 shrink-0 p-0"
          title="关闭"
        >
          <SvgIcon name="x" size="10" stroke-width="3" />
        </button>
        <!-- Active tab bottom indicator -->
        <span v-if="activeTabId === tab.id" class="absolute bottom-0 left-0 right-0 h-0.5 bg-[#89b4fa]"></span>
      </div>
      <!-- 添加标签：服务器选择器 -->
      <div class="flex items-center w-9 min-w-9 bg-transparent text-[#6c7086] text-xs font-medium cursor-pointer border-r-0 transition-all duration-150 relative hover:bg-[#1e1e2e] hover:text-[#89b4fa]">
        <button @click="onAddTab" class="flex items-center justify-center w-9 h-full border-none bg-transparent text-[#6c7086] cursor-pointer transition-all duration-150 hover:text-[#89b4fa] p-0" title="添加标签 / 选择服务器">
          <SvgIcon name="plus" size="14" />
        </button>
      </div>
    </div>

    <!-- 服务器选择下拉框（放在标签栏外，避免被 overflow-y:hidden 裁剪） -->
    <div v-if="showServerSelect" class="fixed top-0 right-0 z-[1000] min-w-[280px] max-w-[340px] max-h-[400px] overflow-y-auto bg-base-100 border border-base-content/10 rounded-box shadow-2xl p-2 mt-[38px] mr-4" @click.stop>
      <GroupedServerSelector
        v-model="selectedServerId"
        mode="single"
        :servers="allServers"
        :groups="groups"
      />
    </div>

    <!-- 终端内容区：每个标签一个容器，CSS 控制显隐 -->
    <div class="flex-1 min-h-0 flex overflow-hidden">
      <div class="terminal-body flex-1 min-h-0 overflow-hidden bg-[#1e1e2e] relative min-w-0">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          :ref="el => setTabContainer(tab.id, el)"
          class="absolute inset-0 hidden"
          :class="{ '!block': activeTabId === tab.id }"
        ></div>
      </div>

      <!-- 服务器监控面板 -->
      <ServerMonitor
        v-if="showMonitor && activeServerId"
        :server-id="activeServerId"
        @toggle="showMonitor = false"
      />
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import SvgIcon from '@/components/ui/SvgIcon.vue'
import * as logger from '../../services/logger'
import { getTauriAPI } from '../../utils/tauri-api'
import { ref, onMounted, onUnmounted, nextTick, watch, computed } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'

import type { Server } from '../../types'
import ServerMonitor from './ServerMonitor.vue'
import GroupedServerSelector from './GroupedServerSelector.vue'

interface TerminalTab {
  id: string
  server: Server
  sessionId: string | null
  status: 'connecting' | 'connected' | 'disconnected' | 'error'
  term: Terminal | null
  fitAddon: FitAddon | null
  cleanupData: (() => void) | null
  cleanupClose: (() => void) | null
}

const props = defineProps<{
  server: Server
  servers?: Server[]
}>()

const emit = defineEmits(['close', 'openSftp'])

const activeTabId = ref('')
const isMaximized = ref(true) // 默认最大化
const showServerSelect = ref(false)
const selectedServerId = ref('')
const showMonitor = ref(true) // 默认显示监控面板

const tabs = ref<TerminalTab[]>([])
// Per-tab container elements: tabId → HTMLElement
const tabContainers = new Map<string, HTMLElement>()
let resizeObserver: ResizeObserver | null = null
let currentTerm: Terminal | null = null

// Track last sent terminal dimensions to avoid redundant resize calls
let lastResizeRows = 0
let lastResizeCols = 0

// Watch for server selection via v-model
watch(selectedServerId, (newId) => {
  if (!newId) return
  const srv = props.servers?.find(s => s.id === newId)
  if (srv) {
    addTab(srv)
    selectedServerId.value = ''
    showServerSelect.value = false
  }
})

// Collect all servers and groups from props and connected servers
const allServers = computed<Server[]>(() => {
  const result: Server[] = []
  if (props.servers) result.push(...props.servers)
  return result
})

interface GroupNode {
  id: string
  name: string
  color: string
  parentId?: string | null
}

// Load groups from API when selector is shown
const groups = ref<GroupNode[]>([])

async function loadGroups() {
  if (groups.value.length > 0) return
  try {
    groups.value = await getTauriAPI().getServerGroups()
  } catch (e) {
    console.error('[Terminal] Failed to load groups:', e)
  }
}

// Set up ResizeObserver to sync terminal size with container
function setupResizeObserver() {
  resizeObserver = new ResizeObserver(() => {
    const tab = tabs.value.find(t => t.id === activeTabId.value)
    if (!tab || !tab.term || !tab.sessionId || tab.status !== 'connected') return

    tab.fitAddon?.fit()
    const { rows, cols } = tab.term
    if (rows > 0 && cols > 0 && (rows !== lastResizeRows || cols !== lastResizeCols)) {
      lastResizeRows = rows
      lastResizeCols = cols
      getTauriAPI().resizeTerminal(tab.sessionId, cols, rows)
    }
  })

  // Observe the terminal-body container
  const body = document.querySelector('.terminal-body')
  if (body) resizeObserver.observe(body)
}

// 当前活跃标签的服务器 ID
const activeServerId = computed(() => {
  const tab = tabs.value.find(t => t.id === activeTabId.value)
  return tab?.server.id || ''
})

function setTabContainer(tabId: string, el: unknown) {
  if (el instanceof HTMLElement) {
    tabContainers.set(tabId, el)
  } else {
    tabContainers.delete(tabId)
  }
}

// 初始化第一个标签
onMounted(() => {
  addTab(props.server)
  setupResizeObserver()
  // 不需要调 maximizeToggle — CSS 默认就是 100vw×100vh 全屏
  // connectTab 成功后会自行调用 fit()

  // 键盘快捷键
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  tabs.value.forEach(tab => cleanupTab(tab))
  resizeObserver?.disconnect()
  window.removeEventListener('keydown', handleKeyDown)
})

function handleKeyDown(e: KeyboardEvent) {
  // Cmd/Ctrl+Shift+T: 复制当前标签
  if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'T') {
    e.preventDefault()
    onAddTab()
    return
  }
  // Cmd/Ctrl+Shift+M: 切换监控面板
  if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'M') {
    e.preventDefault()
    showMonitor.value = !showMonitor.value
    return
  }
  // Cmd/Ctrl+W: 关闭当前标签（仅在终端面板打开且终端聚焦时）
  if ((e.metaKey || e.ctrlKey) && e.key === 'w') {
    const activeTab = tabs.value.find(t => t.id === activeTabId.value)
    if (activeTab && document.activeElement?.closest('.terminal-body')) {
      e.preventDefault()
      closeTab(activeTabId.value)
    }
  }
  // Cmd/Ctrl+1~9: 切换到指定标签
  if ((e.metaKey || e.ctrlKey) && e.key >= '1' && e.key <= '9') {
    e.preventDefault()
    const idx = parseInt(e.key) - 1
    if (idx < tabs.value.length) {
      switchTab(tabs.value[idx].id)
    }
  }
}

function addTab(server: Server) {
  const tab: TerminalTab = {
    id: `tab_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
    server,
    sessionId: null,
    status: 'connecting',
    term: null,
    fitAddon: null,
    cleanupData: null,
    cleanupClose: null,
  }
  tabs.value.push(tab)
  activeTabId.value = tab.id
  connectTab(tab)
  // DO NOT call fitActiveTerminal here — connectTab is async and handles
  // open/fit after SSH connection succeeds. Calling fitActiveTerminal early
  // would open the terminal before SSH is ready, then connectTab clears the
  // container and opens again, causing a conflict where the terminal vanishes.
}

function duplicateTab(tab: TerminalTab) {
  // Create a new tab with the same server — each tab gets its own SSH shell session
  addTab(tab.server)
}

async function connectTab(tab: TerminalTab) {
  logger.info(`[Terminal] connectTab START: ${tab.id} ${tab.server.name} ${tab.server.host}`)
  tab.status = 'connecting'

  // 如果已有终端实例，先清理
  if (tab.term) {
    logger.info('[Terminal] Disposing old terminal instance')
    tab.term.dispose()
    tab.term = null
    tab.fitAddon = null
  }
  if (tab.cleanupData) { tab.cleanupData(); tab.cleanupData = null }
  if (tab.cleanupClose) { tab.cleanupClose(); tab.cleanupClose = null }
  tab.sessionId = null

  // 创建新的 xterm 实例
  const term = new Terminal({
    cursorBlink: true,
    cursorStyle: 'block',
    fontSize: 14,
    fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Menlo', 'Courier New', monospace",
    theme: {
      background: '#1e1e2e',
      foreground: '#cdd6f4',
      cursor: '#f5e0dc',
      cursorAccent: '#1e1e2e',
      selectionBackground: 'rgba(137, 180, 250, 0.3)',
      selectionForeground: '#cdd6f4',
      black: '#45475a',
      red: '#f38ba8',
      green: '#a6e3a1',
      yellow: '#f9e2af',
      blue: '#89b4fa',
      magenta: '#f5c2e7',
      cyan: '#94e2d5',
      white: '#bac2de',
      brightBlack: '#585b70',
      brightRed: '#f38ba8',
      brightGreen: '#a6e3a1',
      brightYellow: '#f9e2af',
      brightBlue: '#89b4fa',
      brightMagenta: '#f5c2e7',
      brightCyan: '#94e2d5',
      brightWhite: '#a6adc8',
    },
    scrollback: 5000,
  })

  const fitAddon = new FitAddon()
  term.loadAddon(fitAddon)

  tab.term = term
  tab.fitAddon = fitAddon

  // 先在终端显示"正在连接"（不 open DOM，避免空终端闪烁）
  term.writeln(`\x1b[1;36m正在连接 ${tab.server.name} (${tab.server.host})...\x1b[0m`)
  logger.info('[Terminal] Created xterm instance, showing "connecting"')

  // 监听输入
  term.onData(async (data: string) => {
    console.log('[Terminal] onData:', JSON.stringify(data.slice(0, 50)))
    logger.info(`[Terminal] onData: ${JSON.stringify(data.slice(0, 50))}`)
    if (tab.sessionId && tab.status === 'connected') {
      try {
        await getTauriAPI().writeTerminal(tab.sessionId, data)
      } catch (e: any) {
        console.error('[Terminal] writeTerminal failed:', e)
        term.writeln(`\x1b[1;31m[写入失败: ${e?.message || e}]\x1b[0m`)
      }
    }
  })

  const MAX_RETRIES = 2
  let lastError: Error | null = null

  for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
    logger.info(`[Terminal] connectTab attempt ${attempt}/${MAX_RETRIES} for tab ${tab.id}`)
    if (attempt > 1) {
      term.writeln(`\x1b[1;33m⏳ 自动重试 (${attempt}/${MAX_RETRIES})...\x1b[0m`)
      logger.info('[Terminal] Waiting 1.5s before retry...')
      await new Promise(r => setTimeout(r, 1500))
    }

    try {
      // 步骤1：建立 SSH 连接
      logger.info(`[Terminal] Step 1: Checking isServerConnected for ${tab.server.id}`)
      const connStatus = await getTauriAPI().isServerConnected(tab.server.id)
      const isConnected = connStatus?.connected === true
      logger.info(`[Terminal] isServerConnected result: ${isConnected}`)

      if (!isConnected) {
        logger.info('[Terminal] Not connected, calling connectServer...')
        const connResult = await getTauriAPI().connectServer(tab.server.id)
        logger.info(`[Terminal] connectServer result: ${JSON.stringify(connResult)}`)
        if (!connResult?.success) {
          throw new Error(connResult?.error || 'SSH 连接失败')
        }
        term.writeln(`\x1b[1;32m✓ SSH 已连接\x1b[0m`)
      } else {
        term.writeln(`\x1b[1;33m⚡ 复用已有 SSH 连接\x1b[0m`)
      }

      // 步骤2：注册监听器
      logger.info('[Terminal] Step 2: Registering terminal data listeners')
      /* TODO(tauri-events): tab.cleanupData = getTauriAPI().onTerminalData((data) => {
        logger.info(`[Terminal] onTerminalData received: ${data.terminalId}, data length: ${data.data?.length}`)
        if (data.terminalId === tab.sessionId && tab.term) {
          tab.term.write(data.data)
        }
      })
      */
      /* TODO(tauri-events): tab.cleanupClose = getTauriAPI().onTerminalClose((data) => {
        logger.info('[Terminal] onTerminalClose:', data.terminalId)
        if (data.terminalId === tab.sessionId) {
          tab.status = 'disconnected'
          if (tab.term) {
            tab.term.writeln('')
            tab.term.writeln('\x1b[90m═══════════════════════════════════════\x1b[0m')
            tab.term.writeln('\x1b[1;33m⚡ 连接断开，自动重连中...\x1b[0m')
          }
          // 自动重连：最多 2 次，间隔 1.5s
          autoReconnectTab(tab)
        }
      })
      */logger.info('[Terminal] Step 2: Listeners registered')

      // 步骤3：创建终端会话（shell）
      // 先用默认 80x24 创建 shell，open + fit 后再发送真实尺寸同步
      const sid = `term_${tab.id}_${Date.now()}`
      tab.sessionId = sid
      logger.info(`[Terminal] Step 3: Creating terminal session, sid=${sid}`)
      const result = await getTauriAPI().createTerminal(tab.server.id, sid, 24, 80)
      logger.info('[Terminal] createTerminal result:', JSON.stringify(result))
      if (!result?.success) {
        throw new Error(result?.error || '创建终端会话失败')
      }

      // 步骤4：连接成功，打开终端到 DOM
      logger.info('[Terminal] Step 4: Connection success, opening terminal to DOM')
      tab.status = 'connected'

      // 使用 nextTick + requestAnimationFrame 确保 DOM 容器已经渲染且有尺寸
      await nextTick()
      await new Promise(r => requestAnimationFrame(r))

      const container = tabContainers.get(tab.id)
      if (tab.id === activeTabId.value && container) {
        logger.info('[Terminal] Calling term.open() on DOM container for tab', tab.id)
        term.open(container)
        logger.info(`[Terminal] term.open() done, element: ${term.element ? 'exists' : 'null'}`)
        logger.info(`[Terminal] Container dimensions: ${container.clientWidth}x${container.clientHeight}`)
        fitAddon.fit()
        term.focus()
        currentTerm = term
        // 点击终端区域自动聚焦，确保键盘输入始终被捕获
        container.addEventListener('click', () => term.focus(), { once: false })
        logger.info(`[Terminal] fitAddon.fit() done, term dimensions: ${term.rows}x${term.cols}`)

        // 同步真实 PTY 尺寸到远端 shell（发送 SIGWINCH）
        // 确保尺寸有效再发送，避免 setWindow(0,0) 导致远端 shell 挂起
        if (term.rows > 0 && term.cols > 0) {
          lastResizeRows = term.rows
          lastResizeCols = term.cols
          getTauriAPI().resizeTerminal(sid, term.cols, term.rows)
        }
      }

      term.writeln(`\x1b[1;32m✓ 已连接到 ${tab.server.name}\x1b[0m`)
      term.writeln('\x1b[90m═══════════════════════════════════════\x1b[0m')
      term.writeln('')
      logger.info('[Terminal] connectTab SUCCESS for', tab.server.name)

      // 启动终端输出轮询（200ms 间隔，从 PTY 拉取输出写入 xterm）
      const pollInterval = setInterval(async () => {
        if (tab.status !== 'connected' || !tab.sessionId || !tab.term) {
          clearInterval(pollInterval)
          return
        }
        try {
          const result = await getTauriAPI().readTerminal(tab.sessionId)
          // tauriCall 解包 read_terminal 返回的 {success, data:"..."} 后 result 已是字符串
          const data = typeof result === 'string' ? result : result?.data
          if (data && typeof data === 'string' && data.length > 0) {
            tab.term.write(data)
          }
        } catch (e) {
          // 终端可能已关闭，静默忽略
        }
      }, 200)
      tab.cleanupData = () => clearInterval(pollInterval)

      return // 成功，直接返回
    } catch (error) {
      lastError = error as Error
      console.error(`[Terminal] Connection attempt ${attempt} failed:`, lastError)
      term.writeln(`\x1b[1;31m✗ 尝试 ${attempt} 失败: ${lastError.message}\x1b[0m`)
      // 清理本次尝试的监听器，避免累积
      if (tab.cleanupData) { tab.cleanupData(); tab.cleanupData = null }
      if (tab.cleanupClose) { tab.cleanupClose(); tab.cleanupClose = null }
      tab.sessionId = null
    }
  }

  // 所有重试都失败了
  console.error('[Terminal] All retries failed for', tab.server.name, lastError)
  tab.status = 'error'
  term.writeln(`\x1b[1;31m✗ 连接失败: ${lastError?.message}\x1b[0m`)
  const errorContainer = tabContainers.get(tab.id)
  if (tab.id === activeTabId.value && errorContainer && !term.element) {
    term.open(errorContainer)
    fitAddon.fit()
    term.focus()
    currentTerm = term
  }
}

async function reconnectTab(tab: TerminalTab) {
  if (tab.status === 'connecting') return
  // 不要操作旧终端，connectTab 会 dispose 并创建新实例
  await connectTab(tab)
  // connectTab 内部已处理 active tab 的 open/fit，无需重复调用
}

// 自动重连：最多 MAX_RETRIES 次，间隔 1.5s，失败后给提示
async function autoReconnectTab(tab: TerminalTab) {
  if (tab.status === 'connecting') return
  const MAX_RETRIES = 2

  for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
    logger.info(`[Terminal] autoReconnectTab attempt ${attempt}/${MAX_RETRIES} for tab ${tab.id}`)
    if (attempt > 1) {
      if (tab.term) tab.term.writeln(`\x1b[1;33m⏳ 自动重试 (${attempt}/${MAX_RETRIES})...\x1b[0m`)
      await new Promise(r => setTimeout(r, 1500))
    }
    try {
      const isConnected = await getTauriAPI().isServerConnected(tab.server.id)
      if (!isConnected) {
        const connResult = await getTauriAPI().connectServer(tab.server.id)
        if (!connResult?.success) throw new Error(connResult?.error || 'SSH 连接失败')
      }

      // 重新注册监听器（旧的可能已经失效）
      if (tab.cleanupData) { tab.cleanupData(); tab.cleanupData = null }
      if (tab.cleanupClose) { tab.cleanupClose(); tab.cleanupClose = null }

      /* TODO(tauri-events): tab.cleanupData = getTauriAPI().onTerminalData((data) => {
        if (data.terminalId === tab.sessionId && tab.term) tab.term.write(data.data)
      })
      */tab.cleanupClose = getTauriAPI().onTerminalClose((data) => {
        if (data.terminalId === tab.sessionId) {
          tab.status = 'disconnected'
          if (tab.term) {
            tab.term.writeln('\x1b[90m═══════════════════════════════════════\x1b[0m')
            tab.term.writeln('\x1b[1;33m⚡ 连接断开，自动重连中...\x1b[0m')
          }
          autoReconnectTab(tab)
        }
      })

      // 重新创建终端会话
      const sid = `term_${tab.id}_${Date.now()}`
      tab.sessionId = sid
      const result = await getTauriAPI().createTerminal(tab.server.id, sid, 24, 80)
      if (!result?.success) throw new Error(result?.error || '创建终端会话失败')

      tab.status = 'connected'

      // 同步真实尺寸（终端已在 DOM 上）
      if (tab.term) {
        tab.term.writeln('\x1b[1;32m✓ 自动重连成功\x1b[0m')
        tab.term.writeln('')
        tab.fitAddon?.fit()
        if (tab.term.rows > 0 && tab.term.cols > 0) {
          lastResizeRows = tab.term.rows
          lastResizeCols = tab.term.cols
          getTauriAPI().resizeTerminal(sid, tab.term.rows, tab.term.cols)
        }
      }
      logger.info('[Terminal] autoReconnectTab SUCCESS for', tab.server.name)
      return
    } catch (error) {
      console.error(`[Terminal] Auto-reconnect attempt ${attempt} failed:`, error)
      if (tab.term) tab.term.writeln(`\x1b[1;31m✗ 重连尝试 ${attempt} 失败: ${(error as Error).message}\x1b[0m`)
      if (tab.cleanupData) { tab.cleanupData(); tab.cleanupData = null }
      if (tab.cleanupClose) { tab.cleanupClose(); tab.cleanupClose = null }
      tab.sessionId = null
    }
  }

  // 所有重试都失败了
  tab.status = 'error'
  if (tab.term) tab.term.writeln('\x1b[1;31m✗ 自动重连失败，请手动点击重连按钮\x1b[0m')
}

function switchTab(tabId: string) {
  if (tabId === activeTabId.value) return
  activeTabId.value = tabId
  // With per-tab containers, switching is just CSS show/hide.
  // Just fit and focus the new active tab.
  nextTick(() => {
    const tab = tabs.value.find(t => t.id === tabId)
    if (!tab || !tab.term) return
    tab.fitAddon?.fit()
    tab.term.focus()
    currentTerm = tab.term
    // Send resize to backend after switching tabs
    if (tab.sessionId && tab.status === 'connected') {
      const { rows, cols } = tab.term
      if (rows > 0 && cols > 0 && (rows !== lastResizeRows || cols !== lastResizeCols)) {
        lastResizeRows = rows
        lastResizeCols = cols
        getTauriAPI().resizeTerminal(tab.sessionId, rows, cols)
      }
    }
  })
}

function fitActiveTerminal() {
  const tab = tabs.value.find(t => t.id === activeTabId.value)
  if (!tab || !tab.term) return
  // With per-tab containers, each tab has its own wrapper.
  // If the terminal isn't open yet, open it to its dedicated container.
  const container = tabContainers.get(tab.id)
  if (!container) return

  if (!tab.term.element || tab.term.element.parentNode !== container) {
    try {
      tab.term.open(container)
    } catch (e) {
      console.warn('xterm open failed in fitActiveTerminal:', e)
      return
    }
  }

  tab.fitAddon?.fit()
  tab.term.focus()
  currentTerm = tab.term

  // Send resize to backend after fitting
  if (tab.sessionId && tab.status === 'connected') {
    const { rows, cols } = tab.term
    if (rows > 0 && cols > 0 && (rows !== lastResizeRows || cols !== lastResizeCols)) {
      lastResizeRows = rows
      lastResizeCols = cols
      getTauriAPI().resizeTerminal(tab.sessionId, rows, cols)
    }
  }
}

function closeTab(tabId: string) {
  const idx = tabs.value.findIndex(t => t.id === tabId)
  if (idx === -1) return

  cleanupTab(tabs.value[idx])
  tabs.value.splice(idx, 1)

  if (tabs.value.length === 0) {
    emit('close')
    return
  }

  if (activeTabId.value === tabId) {
    const newIdx = Math.min(idx, tabs.value.length - 1)
    activeTabId.value = tabs.value[newIdx].id
    nextTick(() => fitActiveTerminal())
  }
}

function cleanupTab(tab: TerminalTab) {
  if (tab.cleanupData) tab.cleanupData()
  if (tab.cleanupClose) tab.cleanupClose()
  if (tab.sessionId) {
    try { getTauriAPI().closeTerminal(tab.sessionId) } catch {}
  }
  if (tab.term) {
    tab.term.dispose()
    tab.term = null
  }
}

function focusActiveTerminal() {
  currentTerm?.focus()
}

function hideServerSelect() {
  showServerSelect.value = false
  selectedServerId.value = ''
}

async function onAddTab() {
  if (!showServerSelect.value) {
    await loadGroups()
  }
  showServerSelect.value = !showServerSelect.value

  // Close dropdown when clicking outside
  if (showServerSelect.value) {
    setTimeout(() => {
      document.addEventListener('click', hideServerSelect, { once: true })
    }, 0)
  }
}

// ===== 快捷打开 SFTP（获取当前终端路径并打开 SFTP 面板） =====
async function quickOpenSftp() {
  const activeTab = tabs.value.find(t => t.id === activeTabId.value)
  if (!activeTab || !activeTab.sessionId || activeTab.status !== 'connected') return

  logger.info('[Terminal] quickOpenSftp: getting pwd via exec')

  try {
    const result = await getTauriAPI().getServerMonitor(activeTab.server.id, ['pwd'])
    let cwd = result?.results?.['pwd']?.trim() || ''
    logger.info('[Terminal] quickOpenSftp cwd:', JSON.stringify(cwd))
    emit('openSftp', activeTab.server, cwd)
  } catch (e) {
    console.warn('[Terminal] quickOpenSftp: pwd failed, opening with empty path', e)
    emit('openSftp', activeTab.server, '')
  }
}

// ===== 最大化切换（默认全屏，此功能保留但默认已全屏） =====
function maximizeToggle() {
  isMaximized.value = !isMaximized.value
  const panel = document.querySelector('.terminal-panel') as HTMLElement
  if (!panel) return

  if (isMaximized.value) {
    panel.style.top = '0'
    panel.style.left = '0'
    panel.style.width = '100vw'
    panel.style.height = '100vh'
    panel.style.borderRadius = '0'
  } else {
    panel.style.top = '0'
    panel.style.left = '0'
    panel.style.width = '90vw'
    panel.style.height = '80vh'
    panel.style.borderRadius = '12px'
  }

  nextTick(() => fitActiveTerminal())
}

// ===== 拖拽 =====
let isDragging = false
let dragStartX = 0
let dragStartY = 0
let panelStartX = 0
let panelStartY = 0

function startDrag(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('button')) return
  if (isMaximized.value) return

  isDragging = true
  const panel = (e.currentTarget as HTMLElement).closest('.terminal-panel') as HTMLElement
  if (!panel) return

  dragStartX = e.clientX
  dragStartY = e.clientY
  const rect = panel.getBoundingClientRect()
  panelStartX = rect.left
  panelStartY = rect.top

  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

function onDrag(e: MouseEvent) {
  if (!isDragging) return
  const panel = document.querySelector('.terminal-panel') as HTMLElement
  if (!panel) return
  panel.style.left = `${panelStartX + (e.clientX - dragStartX)}px`
  panel.style.top = `${panelStartY + (e.clientY - dragStartY)}px`
  panel.style.right = 'auto'
  panel.style.bottom = 'auto'
}

function stopDrag() {
  isDragging = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

// 暴露给父组件
defineExpose({ addTab })
</script>

<!-- xterm deep styles — retained as-is per migration requirement -->
<style scoped>
:deep(.xterm) {
  padding: 8px;
  height: 100% !important;
}

:deep(.xterm-screen) {
  padding: 4px;
}

:deep(.xterm-viewport) {
  border-radius: 0 !important;
}

:deep(.xterm-viewport::-webkit-scrollbar) {
  width: 8px;
}

:deep(.xterm-viewport::-webkit-scrollbar-track) {
  background: #181825;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb) {
  background: #45475a;
  border-radius: 4px;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb:hover) {
  background: #585b70;
}
</style>

<!-- Minimal supplementary styles that cannot be expressed with Tailwind utilities -->
<style>
/* Tab bar slim scrollbar */
.terminal-tabs::-webkit-scrollbar {
  height: 2px;
}
.terminal-tabs::-webkit-scrollbar-thumb {
  background: #45475a;
  border-radius: 1px;
}
</style>
