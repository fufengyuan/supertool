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
          PID: {{ displayPid }}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <button
          v-if="!isRunning"
          class="px-2 py-0.5 text-[#89b4fa] hover:text-[#89b4fa]/80 transition-colors rounded"
          @click="startProcess"
        >
          启动终端
        </button>
        <button
          v-else
          class="px-2 py-0.5 text-[#f38ba8] hover:text-[#f38ba8]/80 transition-colors rounded"
          @click="killProcess"
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
const processId = ref<string | null>(null)
const isRunning = ref(false)
const displayPid = computed(() => processId.value?.slice(0, 8) || '-')
const generatedId = `omp-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`

let term: Terminal | null = null
let fitAddon: FitAddon | null = null
let unlistenData: UnlistenFn | null = null
let unlistenExit: UnlistenFn | null = null

async function startProcess() {
  const pid = generatedId
  processId.value = pid
  isRunning.value = true

  try {
    await invoke('start_local_process', {
      processId: pid,
      command: '/bin/bash',
      args: ['--login'],
      cwd: null as string | null,
    })
    writeln('\x1b[1;32m终端已启动 (bash)\x1b[0m')
    writeln('\x1b[1;33m提示：输入 omp --help 查看可用命令\x1b[0m')
  } catch (e: any) {
    writeln(`\x1b[1;31m启动失败: ${e?.message || String(e)}\x1b[0m`)
    isRunning.value = false
    processId.value = null
    return
  }

  // 监听输出事件
  try {
    unlistenData = await listen<{
      processId: string
      data: string
      stream: string
    }>('local-process-data', (event) => {
      const payload = event.payload
      if (payload.processId !== pid) return
      const line = payload.data || ''
      if (line) {
        term?.writeln(line)
      }
    })

    unlistenExit = await listen<{
      processId: string
      exitCode: number | null
    }>('local-process-exit', (event) => {
      const payload = event.payload
      if (payload.processId !== pid) return
      writeln(`\x1b[1;33m进程已退出 (exit code: ${payload.exitCode ?? '?'})\x1b[0m`)
      isRunning.value = false
      processId.value = null
    })
  } catch (e: any) {
    writeln(`\x1b[1;31m监听失败: ${e?.message || String(e)}\x1b[0m`)
    isRunning.value = false
    processId.value = null
  }
}

async function writeToProcess(data: string) {
  if (!processId.value || !isRunning.value) return
  try {
    await invoke('write_to_local_process', {
      processId: processId.value,
      data,
    })
  } catch {
    // Process may have exited
  }
}

async function killProcess() {
  if (!processId.value) return
  try {
    await invoke('kill_local_process', { processId: processId.value })
  } catch { /* ignore */ }
  isRunning.value = false
  processId.value = null
  writeln('\x1b[1;31m进程已终止\x1b[0m')
}

function writeln(text: string) {
  term?.writeln(text)
}

function handleTerminalInput(data: string) {
  if (!isRunning.value) {
    // If stopped, offer to restart on Enter
    if (data === '\r') {
      startProcess()
    }
    return
  }
  writeToProcess(data)
}

function fitTerminal() {
  nextTick(() => {
    fitAddon?.fit()
  })
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
  writeln('\x1b[1;36m║      OMP Agent 终端 v1.0            ║')
  writeln('\x1b[1;36m║                                      ║')
  writeln('\x1b[1;36m║  输入 omp launch 启动编码助手         ║')
  writeln('\x1b[1;36m║  输入 omp --help 查看所有命令         ║')
  writeln('\x1b[1;36m╚══════════════════════════════════════╝\x1b[0m')
  writeln('')

  // 自动启动终端
  startProcess()

  // ResizeObserver for terminal fitting
  const observer = new ResizeObserver(() => fitTerminal())
  observer.observe(terminalContainer.value)
})

onUnmounted(() => {
  unlistenData?.()
  unlistenExit?.()
  killProcess()
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
