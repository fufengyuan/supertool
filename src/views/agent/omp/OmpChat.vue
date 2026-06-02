<template>
  <div class="flex flex-col h-full bg-[#1e1e2e]">
    <!-- Terminal container -->
    <div ref="terminalContainer" class="flex-1 min-h-0" />

    <!-- Status bar -->
    <div class="flex items-center justify-between px-4 py-1 bg-[#181825] border-t border-[#313244] text-[11px]">
      <div class="flex items-center gap-2 text-[#6c7086]">
        <span
          class="inline-block w-2 h-2 rounded-full"
          :class="isRunning ? 'bg-[#a6e3a1]' : 'bg-[#f38ba8]'"
        />
        <span>{{ isRunning ? '运行中' : '已停止' }}</span>
        <span v-if="isRunning" class="text-[#585b70]">|</span>
        <span v-if="isRunning" class="text-[#585b70]">
          Session: {{ displaySessionId }}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <button
          v-if="!isRunning"
          class="px-2 py-0.5 text-[#89b4fa] hover:text-[#89b4fa]/80 transition-colors rounded"
          @click="startSession"
        >
          启动终端
        </button>
        <button
          v-else
          class="px-2 py-0.5 text-[#f38ba8] hover:text-[#f38ba8]/80 transition-colors rounded"
          @click="stopSession"
        >
          终止
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import type { UnlistenFn } from '@tauri-apps/api/event'
import '@xterm/xterm/css/xterm.css'

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const terminalContainer = ref<HTMLDivElement | null>(null)
const sessionId = ref<string | null>(null)
const isRunning = ref(false)
const displaySessionId = computed(() => sessionId.value?.slice(0, 8) || '-')

let term: Terminal | null = null
let fitAddon: FitAddon | null = null
let unlistenStdout: UnlistenFn | null = null
let unlistenStderr: UnlistenFn | null = null
let unlistenExit: UnlistenFn | null = null

async function startSession() {
  isRunning.value = true

  try {
    const sid = await invoke<string>('omp_start', {
      sessionId: '',
      args: ['launch'],
      cwd: null as string | null,
    })
    sessionId.value = sid
    writeln('\x1b[1;32mOMP 会话已启动 (omp launch)\x1b[0m')
    writeln('\x1b[1;33m提示：输入 /help 查看可用命令\x1b[0m')
  } catch (e: any) {
    writeln(`\x1b[1;31m启动失败: ${e?.message || String(e)}\x1b[0m`)
    isRunning.value = false
    return
  }

  const sid = sessionId.value
  if (!sid) return

  // 监听 stdout
  try {
    unlistenStdout = await listen<{ sessionId: string; data: string }>('omp:stdout', (event) => {
      if (event.payload.sessionId !== sid) return
      const line = event.payload.data || ''
      if (line) term?.writeln(line)
    })
  } catch { /* ignore */ }

  // 监听 stderr
  try {
    unlistenStderr = await listen<{ sessionId: string; data: string }>('omp:stderr', (event) => {
      if (event.payload.sessionId !== sid) return
      const line = event.payload.data || ''
      if (line) term?.writeln(`\x1b[1;31m${line}\x1b[0m`)
    })
  } catch { /* ignore */ }

  // 监听退出
  try {
    unlistenExit = await listen<{ sessionId: string; exitCode: number | null }>('omp:exit', (event) => {
      if (event.payload.sessionId !== sid) return
      writeln(`\x1b[1;33m进程已退出 (exit: ${event.payload.exitCode ?? '?'})\x1b[0m`)
      isRunning.value = false
      sessionId.value = null
    })
  } catch { /* ignore */ }
}

async function writeToSession(data: string) {
  if (!sessionId.value || !isRunning.value) return
  try {
    await invoke('omp_write', { sessionId: sessionId.value, data })
  } catch {
    // session may have ended
  }
}

async function stopSession() {
  if (!sessionId.value) return
  try {
    await invoke('omp_stop', { sessionId: sessionId.value })
  } catch { /* ignore */ }
  isRunning.value = false
  sessionId.value = null
  writeln('\x1b[1;31m会话已终止\x1b[0m')
}

function writeln(text: string) {
  term?.writeln(text)
}

function handleTerminalInput(data: string) {
  if (!isRunning.value) {
    if (data === '\r') {
      startSession()
    }
    return
  }
  writeToSession(data)
}

function fitTerminal() {
  nextTick(() => fitAddon?.fit())
}

onMounted(() => {
  if (!terminalContainer.value) return

  term = new Terminal({
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

  fitAddon = new FitAddon()
  term.loadAddon(fitAddon)

  term.onData(handleTerminalInput)
  term.open(terminalContainer.value)
  fitAddon.fit()

  writeln('\x1b[1;36m╔══════════════════════════════════════╗')
  writeln('\x1b[1;36m║      OMP 编码助手终端                ║')
  writeln('\x1b[1;36m║                                      ║')
  writeln('\x1b[1;36m║  正在启动 omp launch...               ║')
  writeln('\x1b[1;36m╚══════════════════════════════════════╝\x1b[0m')
  writeln('')

  startSession()

  const observer = new ResizeObserver(() => fitTerminal())
  observer.observe(terminalContainer.value)
})

onUnmounted(() => {
  unlistenStdout?.()
  unlistenStderr?.()
  unlistenExit?.()
  stopSession()
  term?.dispose()
  term = null
  fitAddon = null
})
</script>

<style scoped>
:deep(.xterm) {
  height: 100%;
  padding: 8px;
}
</style>
