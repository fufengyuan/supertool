<template>
  <div class="fixed flex flex-col overflow-hidden rounded-xl border border-base-content/10 bg-base-100 shadow-2xl z-[1000]" :style="panelStyle">
    <!-- Header bar -->
    <div class="flex items-center justify-between rounded-t-xl border-b border-base-content/10 bg-base-200 px-4 py-3" @mousedown="startDrag">
      <div class="flex items-center gap-3">
        <span class="text-sm font-semibold">{{ server.name }} - SFTP</span>
        <span :class="[
          'inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs',
          connectionStatus === 'online' ? 'bg-success/15 text-success' : '',
          connectionStatus === 'offline' ? 'bg-error/15 text-error' : '',
          connectionStatus === 'connecting' ? 'bg-warning/15 text-warning' : '',
        ]">
          <span :class="[
            'inline-block h-1.5 w-1.5 rounded-full',
            connectionStatus === 'online' ? 'bg-success' : '',
            connectionStatus === 'offline' ? 'bg-error' : '',
            connectionStatus === 'connecting' ? 'bg-warning animate-pulse' : '',
          ]"></span>
          {{ connectionLabel }}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <button @click.stop="toggleSize" class="btn btn-ghost btn-xs btn-square" :title="isMaximized ? '还原' : '最大化'">
          <SvgIcon v-if="!isMaximized" name="maximize" size="14" />
          <SvgIcon v-else name="minimize" size="14" />
        </button>
        <button @click.stop="$emit('close')" class="btn btn-circle btn-error btn-sm text-white hover:scale-110" title="关闭">
          <SvgIcon name="x" size="20" />
        </button>
      </div>
    </div>

    <!-- VueFinder file manager -->
    <div class="flex-1 overflow-hidden">
      <VueFinder
        v-if="driver"
        :id="`sftp-${server.id}`"
        :driver="driver"
        :locale="locale"
        :features="features"
        :config="config"
        @path-change="onPathChange"
        @select="onSelect"
        @file-dclick="onFileDclick"
        @notify="onNotify"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { VueFinder } from 'vuefinder'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { SftpDriver } from '@/drivers/SftpDriver'
import { useToast } from '@/composables/useToast'

interface Server {
  id: string
  name: string
  username: string
}

const props = defineProps<{
  server: Server
  initialPath?: string
  initialPosition?: { x: number; y: number }
}>()

const emit = defineEmits(['close'])

const toast = useToast()

// Driver instance
const driver = ref<SftpDriver | null>(null)

// Connection status
const connectionStatus = ref<'connecting' | 'online' | 'offline'>('connecting')
const connectionLabel = ref('连接中...')

// VueFinder configuration
const locale = 'zh-CN'
const features = {
  // Enable/disable features
  // Disable features not supported by SFTP or not needed
}
const config = {
  maxFileSize: 1024 * 1024 * 1024, // 1GB
}

// Panel position and size
const defaultPos = props.initialPosition || { x: Math.max(50, (window.innerWidth - 900) / 2), y: 80 }
const panelPos = ref({ x: defaultPos.x, y: defaultPos.y })
const isMaximized = ref(false)

const panelStyle = computed(() => {
  if (isMaximized.value) {
    return { top: '0', left: '0', width: '100vw', height: '100vh', borderRadius: '0' }
  }
  return {
    left: panelPos.value.x + 'px',
    top: panelPos.value.y + 'px',
    width: '900px',
    height: '650px',
  }
})

// Drag handling
let isDragging = false
let dragStartX = 0
let dragStartY = 0
let panelStartX = 0
let panelStartY = 0

function startDrag(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('button, input, .vuefinder')) return
  if (isMaximized.value) return

  isDragging = true
  dragStartX = e.clientX
  dragStartY = e.clientY
  panelStartX = panelPos.value.x
  panelStartY = panelPos.value.y

  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

function onDrag(e: MouseEvent) {
  if (!isDragging) return
  panelPos.value.x = panelStartX + (e.clientX - dragStartX)
  panelPos.value.y = panelStartY + (e.clientY - dragStartY)
}

function stopDrag() {
  isDragging = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

function toggleSize() {
  isMaximized.value = !isMaximized.value
}

// VueFinder event handlers
function onPathChange(path: string) {
  console.log('[SFTP] Path changed:', path)
}

function onSelect(items: any[]) {
  console.log('[SFTP] Selected:', items)
}

function onFileDclick(event: { item: any; defaultPrevented: boolean; preventDefault: () => void }) {
  // Let VueFinder handle the default behavior (open folder or preview file)
  console.log('[SFTP] File double-clicked:', event.item)
}

function onNotify(notification: { type: string; message: string }) {
  if (notification.type === 'success') {
    toast.success(notification.message)
  } else if (notification.type === 'error') {
    toast.error(notification.message)
  } else if (notification.type === 'warning') {
    toast.warning(notification.message)
  } else {
    toast.info(notification.message)
  }
}

// Initialize
onMounted(async () => {
  connectionStatus.value = 'connecting'
  connectionLabel.value = '连接中...'

  const defaultPath = props.server.username === 'root' ? '/root' : `/home/${props.server.username}`
  const initialPath = props.initialPath || defaultPath

  try {
    // Initialize driver
    driver.value = new SftpDriver({
      serverId: props.server.id,
      serverName: props.server.name,
      initialPath: initialPath
    })

    // Test connection by listing
    const result = await driver.value.list({ path: initialPath })
    if (result.files.length >= 0) {
      connectionStatus.value = 'online'
      connectionLabel.value = '已连接'
    }
  } catch (error: any) {
    connectionStatus.value = 'offline'
    connectionLabel.value = '连接失败'
    toast.error(`SFTP 连接失败: ${error.message}`)
  }
})

// Watch initialPath changes
watch(() => props.initialPath, (newPath) => {
  if (newPath && driver.value) {
    // VueFinder will handle path changes internally
    console.log('[SFTP] Initial path updated:', newPath)
  }
})

onUnmounted(() => {
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
})
</script>

<style scoped>
/* Override VueFinder styles to match our theme */
.vuefinder {
  --vf-primary: var(--color-primary);
  --vf-bg-base: var(--color-base-100);
  --vf-bg-base-200: var(--color-base-200);
  --vf-bg-base-300: var(--color-base-300);
  --vf-text-base: var(--color-base-content);
  --vf-border-base: var(--color-base-content);
}

.vuefinder :deep(.vf-toolbar) {
  background: var(--color-base-200);
  border-bottom: 1px solid var(--color-base-content);
}

.vuefinder :deep(.vf-explorer) {
  background: var(--color-base-100);
}

.vuefinder :deep(.vf-item) {
  border-radius: 4px;
}

.vuefinder :deep(.vf-item:hover) {
  background: var(--color-base-200);
}

.vuefinder :deep(.vf-item.selected) {
  background: var(--color-primary);
  color: white;
}
</style>