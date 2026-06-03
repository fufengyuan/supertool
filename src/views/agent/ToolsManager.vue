<template>
  <div class="h-full flex flex-col">
    <!-- Claw mode -->
    <div v-show="isClawMode" class="flex-1 flex flex-col overflow-y-auto p-4 space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h1 class="text-sm font-medium">Claw 工具管理</h1>
        <button class="btn btn-sm btn-ghost" @click="loadClawTools">
          <SvgIcon name="refresh" size="14" />
        </button>
      </div>

      <!-- Claw MCP Servers -->
      <div v-if="clawMcpServers.length > 0">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">
          MCP 服务器 ({{ clawMcpServers.length }})
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          <div
            v-for="srv in clawMcpServers"
            :key="srv.name"
            class="bg-base-100 rounded-lg border border-base-content/10 p-3"
          >
            <div class="flex items-center gap-2 mb-2">
              <span class="text-sm font-medium">{{ srv.name }}</span>
              <span
                v-if="srv.required"
                class="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded bg-error/20 text-error"
              >Required</span>
            </div>
            <p class="text-xs text-base-content/50 font-mono truncate">{{ srv.command }}</p>
            <p v-if="srv.args && srv.args.length" class="text-xs text-base-content/40 mt-1 font-mono truncate">{{ srv.args.join(' ') }}</p>
          </div>
        </div>
      </div>
      <div v-else-if="!clawLoading">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">
          MCP 服务器
        </h2>
        <p class="text-xs text-base-content/40">未配置 MCP 服务器</p>
      </div>

      <!-- Claw Plugins -->
      <div v-if="clawPlugins.length > 0">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">
          插件 ({{ clawPlugins.length }})
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          <div
            v-for="plugin in clawPlugins"
            :key="plugin.id"
            class="bg-base-100 rounded-lg border border-base-content/10 p-3"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="text-sm font-medium">{{ plugin.name }}</span>
              <span class="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded bg-base-300 text-base-content/60">{{ plugin.kind }}</span>
              <span class="text-[10px] text-base-content/40">v{{ plugin.version }}</span>
            </div>
            <p class="text-xs text-base-content/50 leading-relaxed">{{ plugin.description || '暂无描述' }}</p>
          </div>
        </div>
      </div>
      <div v-else-if="!clawLoading">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">
          插件
        </h2>
        <p class="text-xs text-base-content/40">未安装任何插件</p>
      </div>

      <!-- Loading -->
      <div v-if="clawLoading" class="flex items-center justify-center py-12">
        <span class="loading loading-spinner loading-sm text-base-content/40"></span>
        <span class="text-xs text-base-content/40 ml-2">加载中...</span>
      </div>
    </div>

    <div v-show="!isClawMode">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10">
      <h1 class="text-sm font-medium">工具集管理</h1>
      <div class="flex items-center gap-2">
        <button class="btn btn-sm btn-ghost" @click="refresh">
          <SvgIcon name="refresh" size="14" />
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-4 space-y-6">
      <!-- Toolset cards -->
      <div>
        <h2 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">
          平台工具集 ({{ toolsets.length }})
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          <div
            v-for="tool in toolsets"
            :key="tool.key"
            class="relative bg-base-100 rounded-lg border border-base-content/10 p-3 hover:border-primary/30 transition-colors"
            :class="{ 'opacity-50': !tool.enabled }"
          >
            <div class="flex items-start justify-between">
              <div class="flex items-center gap-2">
                <component :is="toolIcon(tool.key)" size="18" class="shrink-0 text-base-content/70" stroke-width="2" />
                <span class="text-sm font-medium">{{ tool.label }}</span>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  class="sr-only peer"
                  :checked="tool.enabled"
                  @change="toggleTool(tool)"
                />
                <div class="w-9 h-5 bg-base-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-success"></div>
              </label>
            </div>
            <p class="text-xs text-base-content/50 mt-2 leading-relaxed">{{ tool.description }}</p>
            <div v-if="!tool.enabled" class="absolute top-3 right-14">
              <span class="text-[10px] font-bold text-error uppercase">Disabled</span>
            </div>
          </div>
        </div>
      </div>

      <!-- MCP Servers -->
      <div v-if="mcpServers.length > 0">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">
          MCP 服务器 ({{ mcpServers.length }})
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          <div
            v-for="srv in mcpServers"
            :key="srv.name"
            class="bg-base-100 rounded-lg border border-base-content/10 p-3"
          >
            <div class="flex items-center gap-2 mb-2">
              <span class="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded"
                :class="srv.type === 'http' ? 'bg-info/20 text-info' : srv.type === 'stdio' ? 'bg-warning/20 text-warning' : 'bg-base-300 text-base-content/60'"
              >
                {{ srv.type }}
              </span>
              <span class="text-sm font-medium">{{ srv.name }}</span>
            </div>
            <p class="text-xs text-base-content/50 font-mono truncate">{{ srv.detail }}</p>
          </div>
        </div>
      </div>
      <div v-else>
        <h2 class="text-xs font-semibold uppercase tracking-wider text-base-content/60 mb-3">
          MCP 服务器
        </h2>
        <p class="text-xs text-base-content/40">未配置 MCP 服务器</p>
      </div>
    </div>
  </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useAgentModeStore } from '@/stores/agentModeStore'
import {
  IconWifi,
  IconBrowser,
  IconTerminal2,
  IconFile,
  IconCode,
  IconEye,
  IconPhoto,
  IconMicrophone,
  IconTool,
  IconBrain,
  IconSearch,
  IconMessage,
  IconUsers,
  IconClock,
  IconLayoutBottombar,
  IconCheckbox,
  IconServer,
} from '@tabler/icons-vue'
import { getTauriAPI } from '@/utils/tauri-api'
import type { ToolsetInfo, MCPServerInfo } from '@/types'

const agentModeStore = useAgentModeStore()
const isClawMode = computed(() => agentModeStore.mode === 'claw')

const toolsets = ref<ToolsetInfo[]>([])
const mcpServers = ref<MCPServerInfo[]>([])
const loading = ref(false)
const clawMcpServers = ref<Array<{ name: string; command: string; args: string[]; required: boolean; timeoutMs: number | null }>>([])
const clawPlugins = ref<Array<{ id: string; name: string; version: string; description: string; kind: string; installPath: string }>>([])
const clawLoading = ref(false)

const TOOLSET_ICONS: Record<string, any> = {
  web: IconWifi,
  browser: IconBrowser,
  terminal: IconTerminal2,
  file: IconFile,
  code_execution: IconCode,
  vision: IconEye,
  image_gen: IconPhoto,
  tts: IconMicrophone,
  skills: IconTool,
  memory: IconBrain,
  session_search: IconSearch,
  clarify: IconMessage,
  delegation: IconUsers,
  cronjob: IconClock,
  moa: IconLayoutBottombar,
  todo: IconCheckbox,
}

function toolIcon(key: string): any {
  return TOOLSET_ICONS[key] || IconTool
}

async function refresh() {
  loading.value = true
  try {
    const api = getTauriAPI()
    const [ts, mcp] = await Promise.all([
      api.listToolsets(),
      api.listMcpServers(),
    ])
    toolsets.value = ts
    mcpServers.value = mcp
  } catch (e) {
    console.error('[ToolsManager] Failed to load:', e)
  } finally {
    loading.value = false
  }
}

async function loadClawTools() {
  clawLoading.value = true
  try {
    const api = getTauriAPI()
    const [servers, plugins] = await Promise.all([
      api.clawListMcpServers(),
      api.clawListPlugins(),
    ])
    clawMcpServers.value = servers
    clawPlugins.value = plugins
  } catch (e) {
    console.error('[ToolsManager] Failed to load Claw tools:', e)
  } finally {
    clawLoading.value = false
  }
}

async function toggleTool(tool: ToolsetInfo) {
  try {
    const api = getTauriAPI()
    await api.setToolsetEnabled(tool.key, !tool.enabled)
    tool.enabled = !tool.enabled
  } catch (e) {
    console.error('[ToolsManager] Toggle failed:', e)
  }
}

onMounted(() => {
  refresh()
  loadClawTools()
})
</script>
