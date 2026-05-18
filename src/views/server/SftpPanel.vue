<template>
  <div class="fixed flex flex-col overflow-hidden rounded-xl border border-base-content/10 bg-base-100 shadow-2xl z-[1000]" :style="panelStyle">
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

    <div class="flex items-center gap-2 bg-base-content/10 p-2.5">
      <SvgIcon name="home" size="16" class="shrink-0 opacity-50" />
      <input v-model="currentPath" @keyup.enter="loadDir" placeholder="/home/user" class="input input-bordered input-sm flex-1 text-xs" />
      <button @click="loadDir" class="btn btn-ghost btn-sm">刷新</button>
      <button @click="goUp" class="btn btn-ghost btn-sm" :disabled="currentPath === '/'">↑ 上级</button>
      <button @click="uploadFile" class="btn btn-ghost btn-sm">↑ 上传文件</button>
      <button @click="uploadFolder" class="btn btn-ghost btn-sm gap-1.5"><SvgIcon name="folder" size="14" /> 上传文件夹</button>
    </div>

    <!-- 文件列表头部 + 搜索框 -->
    <div class="flex items-center gap-2.5 border-b border-base-content/10 bg-base-200 px-3 py-2 text-xs font-medium text-base-content/60">
      <span class="shrink-0 w-5"></span>
      <span class="flex-1">名称</span>
      <!-- 搜索框 -->
      <div class="relative flex items-center gap-1">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索..."
          class="input input-bordered input-xs w-[120px] text-xs pr-6"
          :class="[searchQuery ? 'border-primary' : '']"
        />
        <SvgIcon v-if="searchQuery" name="x" size="12" class="absolute right-2 cursor-pointer text-base-content/50 hover:text-base-content" @click="searchQuery = ''" />
      </div>
      <span class="w-[70px] shrink-0 text-right">大小</span>
      <span class="w-[130px] shrink-0">修改时间</span>
      <span class="shrink-0">操作</span>
    </div>

    <div class="flex-1 overflow-y-auto p-2">
      <div
        ref="fileListRef"
        class="flex flex-col gap-1 relative min-h-[100px] transition-colors duration-200"
        :class="[isDragOver ? 'bg-[rgba(137,180,250,0.08)] rounded-lg' : '']"
      >
        <!-- 拖拽提示层 -->
        <div v-if="isDragOver" class="absolute inset-0 z-10 flex flex-col items-center justify-center gap-2 border-2 border-dashed border-[#89b4fa] rounded-lg bg-[rgba(30,30,46,0.9)] text-[#89b4fa] pointer-events-none">
          <SvgIcon name="upload" size="48" stroke-width="1.5" />
          <p class="text-sm font-semibold">释放以上传文件</p>
        </div>

        <div
          v-for="file in filteredFiles"
          :key="file.name"
          class="group flex items-center gap-2.5 rounded-md px-2.5 py-2 cursor-pointer transition-colors duration-100 hover:bg-base-200"
          :class="[selectedFile?.name === file.name ? 'bg-base-content/10' : '']"
          @click="selectFile(file)"
          @dblclick="handleDoubleClick(file)"
        >
          <span class="flex items-center shrink-0">
            <SvgIcon v-if="file.type === 'directory'" name="folder" size="20" class="text-[#f9a825]" />
            <SvgIcon v-else name="file" size="20" class="text-base-content/60" />
          </span>
          <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-xs">{{ file.name }}</span>
          <span class="w-[70px] shrink-0 text-right text-xs text-base-content/60">{{ formatSize(file.size) }}</span>
          <span class="w-[130px] shrink-0 text-xs text-base-content/60">{{ formatDate(file.modifyTime) }}</span>
          <div class="flex shrink-0 gap-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100">
            <button v-if="file.type === 'file'" @click.stop="downloadFile(file)" class="btn btn-ghost btn-xs btn-square" title="下载">
              <SvgIcon name="download" size="14" />
            </button>
            <button @click.stop="deleteFile(file)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
              <SvgIcon name="trash" size="14" />
            </button>
          </div>
        </div>

        <!-- 搜索无结果 -->
        <div v-if="filteredFiles.length === 0 && files.length > 0" class="flex flex-col items-center justify-center gap-3 py-10 text-base-content/60">
          <SvgIcon name="search" size="48" stroke-width="1.5" />
          <p>未找到匹配文件</p>
          <p class="text-xs">尝试其他关键词</p>
        </div>
        <!-- 真正空目录 -->
        <div v-if="files.length === 0" class="flex flex-col items-center justify-center gap-3 py-10 text-base-content/60">
          <SvgIcon name="folder" size="48" stroke-width="1.5" />
          <p>空目录</p>
        </div>
      </div>
    </div>

    <div class="flex gap-4 rounded-b-xl border-t border-base-content/10 bg-base-200 px-4 py-2.5 text-xs text-base-content/60">
      <span v-if="searchQuery">匹配 {{ filteredFiles.length }} / {{ files.length }} 项</span>
      <span v-else>{{ files.length }} 项</span>
      <span v-if="selectedFile">已选: {{ selectedFile.name }}</span>
    </div>

    <!-- 上传进度 -->
    <div v-if="uploadProgress" class="flex items-center gap-2.5 border-t border-base-content/10 bg-base-200 px-4 py-2.5 text-xs">
      <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-base-content">{{ uploadProgress.file }}</span>
      <span class="shrink-0 min-w-[70px] text-right text-[11px] text-primary">{{ uploadProgress.speedFormatted || '' }}</span>
      <progress class="progress progress-primary flex-1 h-1" :value="uploadProgress.percent" max="100"></progress>
      <span class="min-w-[40px] text-right font-semibold text-primary">{{ uploadProgress.percent }}%</span>
      <!-- 操作按钮 -->
      <div class="flex shrink-0 gap-1">
        <button v-if="isUploading" @click="cancelUpload" class="btn btn-outline btn-error btn-xs gap-1" title="取消上传"><SvgIcon name="x" size="12" /> 取消</button>
        <button v-if="uploadFailed" @click="retryUpload" class="btn btn-outline btn-primary btn-xs" title="重试上传">↻ 重试</button>
      </div>
    </div>

    <!-- 下载进度 -->
    <div v-if="downloadProgress" class="flex items-center gap-2.5 border-t border-base-content/10 bg-base-200 px-4 py-2.5 text-xs">
      <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-base-content">{{ downloadProgress.file }}</span>
      <span class="shrink-0 min-w-[70px] text-right text-[11px] text-primary">{{ downloadSpeed }}</span>
      <progress class="progress progress-primary flex-1 h-1" :value="downloadProgress.percent" max="100"></progress>
      <span class="min-w-[40px] text-right font-semibold text-primary">{{ downloadProgress.percent }}%</span>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import SvgIcon from '@/components/ui/SvgIcon.vue'
import * as logger from '../../services/logger'
import { getTauriAPI } from '../../utils/tauri-api'
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useToast } from '../../composables/useToast';
import { useErrorHandler } from '../../composables/useErrorHandler';

interface SftpFile {
  name: string;
  type: string;
  size?: number;
  modifyTime?: string;
}

const props = defineProps<{
  server: Server;
  initialPath?: string;
  initialPosition?: { x: number; y: number };
}>();

const emit = defineEmits(['close']);

const toast = useToast();
const { handleError } = useErrorHandler();

const defaultPath = props.server.username === 'root' ? '/root' : `/home/${props.server.username}`;
const currentPath = ref(defaultPath);
const files = ref<SftpFile[]>([]);
const searchQuery = ref(''); // 搜索关键词
const selectedFile = ref<SftpFile | null>(null);

// 搜索过滤后的文件列表
const filteredFiles = computed(() => {
  if (!searchQuery.value.trim()) return files.value;
  const query = searchQuery.value.toLowerCase();
  return files.value.filter(file => file.name.toLowerCase().includes(query));
});
const uploadProgress = ref<{ file: string; percent: number; speedFormatted?: string } | null>(null);
const connectionStatus = ref('connecting'); // 'online' | 'offline' | 'connecting'
const isDragOver = ref(false);
const isUploading = ref(false);
const uploadMessage = ref(''); // 实时进度消息
const uploadFailed = ref(false); // 上传失败状态
const downloadProgress = ref<{ file: string; percent: number } | null>(null); // 下载进度
const downloadSpeed = ref(''); // 下载速率
let progressCleanup: (() => void) | null = null;
let downloadProgressCleanup: (() => void) | null = null;
let uploadDoneCleanup: (() => void) | null = null;
let disconnectCleanup: (() => void) | null = null;
let currentEntries: any[] = []; // 保存拖拽条目用于重试
const fileListRef = ref<HTMLElement | null>(null);

const connectionLabel = ref('连接中...');
const isMaximized = ref(false);
const defaultPos = props.initialPosition || { x: Math.max(50, (window.innerWidth - 800) / 2), y: 80 };
const panelPos = ref({ x: defaultPos.x, y: defaultPos.y });

const panelStyle = computed(() => {
  if (isMaximized.value) {
    return { top: '0', left: '0', width: '100vw', height: '100vh', borderRadius: '0' };
  }
  return {
    left: panelPos.value.x + 'px',
    top: panelPos.value.y + 'px',
    width: '800px',
    height: '600px',
  };
});

// 拖拽
let isDragging = false;
let dragStartX = 0;
let dragStartY = 0;
let panelStartX = 0;
let panelStartY = 0;

function startDrag(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('button, input, select')) return;
  if (isMaximized.value) return;

  isDragging = true;
  dragStartX = e.clientX;
  dragStartY = e.clientY;
  panelStartX = panelPos.value.x;
  panelStartY = panelPos.value.y;

  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', stopDrag);
}

function onDrag(e: MouseEvent) {
  if (!isDragging) return;
  panelPos.value.x = panelStartX + (e.clientX - dragStartX);
  panelPos.value.y = panelStartY + (e.clientY - dragStartY);
}

function stopDrag() {
  isDragging = false;
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
}

function toggleSize() {
  isMaximized.value = !isMaximized.value;
}

onMounted(async () => {
  // 监听上传进度事件
  /* TODO(tauri-events): progressCleanup = getTauriAPI().onSftpUploadProgress((data) => {
    if (data.serverId === props.server.id) {
      uploadProgress.value = { file: data.message, percent: data.percent, speedFormatted: data.speedFormatted };
      uploadMessage.value = data.message;
    }
  }) || null;
  */
  // 监听下载进度事件
  /* TODO(tauri-events): downloadProgressCleanup = getTauriAPI().onSftpDownloadProgress((data) => {
    if (data.serverId === props.server.id) {
      downloadProgress.value = { file: data.message, percent: data.percent };
      downloadSpeed.value = data.speedFormatted || '';
    }
  }) || null;
  */
  // 监听上传完成事件（清除进度条）
  /* TODO(tauri-events): uploadDoneCleanup = getTauriAPI().onSftpUploadDone((data) => {
    if (data.serverId === props.server.id) {
      uploadProgress.value = null;
    }
  }) || null;
  */
  // 监听 SSH 断开事件，自动重连
  /* TODO(tauri-events): disconnectCleanup = getTauriAPI().onServerDisconnected((data) => {
    if (data.serverId === props.server.id) {
      connectionStatus.value = 'connecting';
      connectionLabel.value = '自动重连中...';
      autoReconnectSftp();
    }
  }) || null;
  */
  // 如果传入了初始路径，使用它；否则使用默认的 /home
  if (props.initialPath && props.initialPath.trim()) {
    currentPath.value = props.initialPath
  }
  await loadDir();

  // Tauri 原生拖拽事件监听
  const win = getCurrentWebviewWindow();
  const unlistenDragDrop = await win.onDragDropEvent(({ payload }) => {
    if (payload.type === 'enter' || payload.type === 'over') {
      // 判断坐标是否在文件列表区域内
      const el = fileListRef.value;
      if (el) {
        const rect = el.getBoundingClientRect();
        const { x, y } = payload.position;
        isDragOver.value = x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
      }
    } else if (payload.type === 'drop' && isDragOver.value && payload.paths?.length > 0) {
      isDragOver.value = false;
      console.log('[SFTP] dropped paths:', payload.paths);
      doDragUploadFromPaths(payload.paths);
    } else if (payload.type === 'leave') {
      isDragOver.value = false;
    } else {
      isDragOver.value = false;
    }
  });
  progressCleanup = unlistenDragDrop as any;
});

// 监听 initialPath 变化（用于从终端快捷打开时动态设置路径）
watch(() => props.initialPath, (newPath) => {
  if (newPath && newPath !== currentPath.value) {
    currentPath.value = newPath
    loadDir()
  }
})

onUnmounted(() => {
  // Clean up any lingering drag listeners
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
  // Clean up progress listeners
  if (progressCleanup) {
    progressCleanup();
    progressCleanup = null;
  }
  if (downloadProgressCleanup) {
    downloadProgressCleanup();
    downloadProgressCleanup = null;
  }
  if (uploadDoneCleanup) {
    uploadDoneCleanup();
    uploadDoneCleanup = null;
  }
  if (disconnectCleanup) {
    disconnectCleanup();
    disconnectCleanup = null;
  }
});

async function loadDir() {
  try {
    // 直接列目录，不再调用可能卡死的 connectServer
    // 连接由 onMounted 初始化时完成，Rust 端会自动重连
    const result = await getTauriAPI().listSftpDir(props.server.id, currentPath.value);
    if (result.success) {
      files.value = result.files.sort((a, b) => {
        if (a.type === 'directory' && b.type !== 'directory') return -1;
        if (a.type !== 'directory' && b.type === 'directory') return 1;
        return a.name.localeCompare(b.name);
      });
      connectionStatus.value = 'online';
      connectionLabel.value = '已连接';
    } else {
      connectionStatus.value = 'offline';
      connectionLabel.value = '连接失败';
      toast.error('加载失败: ' + result.error);
    }
  } catch (error) {
    connectionStatus.value = 'offline';
    connectionLabel.value = '连接失败';
    handleError(error, { context: 'SFTP loadDir' });
  }
}

// SFTP 自动重连
async function autoReconnectSftp() {
  const MAX_RETRIES = 2
  for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
    if (attempt > 1) await new Promise(r => setTimeout(r, 1500))
    try {
      const connStatus = await getTauriAPI().isServerConnected(props.server.id)
      if (!connStatus?.connected) {
        await getTauriAPI().connectServer(props.server.id)
      }
      // 重连成功，刷新目录
      await loadDir()
      logger.info('[SFTP] autoReconnectSftp SUCCESS for', props.server.name)
      return
    } catch (error) {
      console.error(`[SFTP] Auto-reconnect attempt ${attempt} failed:`, error)
    }
  }
  connectionStatus.value = 'offline'
  connectionLabel.value = '重连失败'
  toast.error('SFTP 自动重连失败，请手动刷新')
}

function goUp() {
  if (currentPath.value === '/') return;
  const parts = currentPath.value.split('/').filter(Boolean);
  parts.pop();
  currentPath.value = '/' + parts.join('/');
  loadDir();
}

function selectFile(file) {
  selectedFile.value = file;
}

function enterDir(file) {
  if (file.type === 'directory') {
    const base = currentPath.value === '/' ? '' : currentPath.value;
    currentPath.value = base + '/' + file.name;
    loadDir();
  }
}

async function handleDoubleClick(file) {
  if (file.type === 'directory') {
    enterDir(file);
  } else {
    await openFileEditor(file);
  }
}

async function openFileEditor(file) {
  try {
    const remotePath = currentPath.value + '/' + file.name;
    await getTauriAPI().openSftpFileEditor(props.server.id, remotePath);
  } catch (error: any) {
    handleError(error, { context: 'SFTP openFileEditor' });
  }
}

async function downloadFile(file) {
  try {
    // 获取用户下载目录
    const downloadsDir = await getTauriAPI().getDownloadsDir()
    const localPath = downloadsDir.endsWith('/') || downloadsDir.endsWith('\\\\')
      ? downloadsDir + file.name
      : downloadsDir + '/' + file.name

    downloadProgress.value = { file: file.name, percent: 0 }
    downloadSpeed.value = ''

    await getTauriAPI().downloadFile(
      props.server.id,
      currentPath.value + '/' + file.name,
      localPath
    )
    downloadProgress.value = null
    downloadSpeed.value = ''
    toast.success(`下载成功: ${localPath}`)
  } catch (error) {
    downloadProgress.value = null
    downloadSpeed.value = ''
    handleError(error, { context: 'SFTP downloadFile' })
  }
}

async function uploadFolder() {
  const result = await getTauriAPI().showOpenDialogForDirs()
  if (result.canceled || !result.filePaths?.length) return

  const localDirPath = result.filePaths[0]
  const folderName = localDirPath.split('/').pop() || localDirPath.split('\\').pop() || 'unknown'

  isUploading.value = true
  uploadProgress.value = { file: folderName, percent: 0 }

  try {
    toast.info(`正在压缩 ${folderName}...`)
    await getTauriAPI().uploadFolder(props.server.id, currentPath.value, localDirPath)

    uploadProgress.value = { file: folderName, percent: 100 }
    toast.success(`文件夹上传成功: ${folderName}`)
    await loadDir()
  } catch (error: any) {
    toast.error(`上传失败: ${error.message}`)
  } finally {
    uploadProgress.value = null
    isUploading.value = false
  }
}

async function uploadFile() {
  const result = await getTauriAPI().showOpenDialog()
  if (result.canceled || !result.filePaths?.length) return
  await doUpload(result.filePaths)
}

async function doUpload(filePaths: string[]) {
  if (isUploading.value || filePaths.length === 0) return
  isUploading.value = true

  try {
    // 确保 SSH 已连接
    const isConnected = await getTauriAPI().onServerConnected(props.server.id)
    if (!isConnected) {
      await getTauriAPI().connectServer(props.server.id)
    }

    let successCount = 0
    for (const filePath of filePaths) {
      const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'unknown'
      uploadProgress.value = { file: fileName, percent: 0 }

      const remotePath = currentPath.value.endsWith('/')
        ? currentPath.value + fileName
        : currentPath.value + '/' + fileName

      try {
        await getTauriAPI().uploadFile(props.server.id, remotePath, filePath)
        successCount++
      } catch (err: any) {
        toast.error(`上传失败 ${fileName}: ${err.message}`)
      }
      uploadProgress.value = { file: fileName, percent: 100 }
    }

    uploadProgress.value = null
    if (successCount > 0) toast.success(`成功上传 ${successCount} 个文件`)
    await loadDir()
  } catch (error) {
    uploadProgress.value = null
    handleError(error, { context: 'SFTP upload' })
  } finally {
    isUploading.value = false
  }
}

// 拖拽离开（判断是否真正离开了容器，避免在子元素间移动时闪烁）
function onDragLeave(event: DragEvent) {
  if (!(event.currentTarget as HTMLElement)?.contains(event.relatedTarget as Node)) {
    isDragOver.value = false
  }
}

// 拖拽上传（Tauri 版本：写入临时文件后通过 SFTP 上传）
async function onDrop(event: DragEvent) {
  isDragOver.value = false
  const dt = event.dataTransfer
  if (!dt || !dt.items) return

  // 收集所有拖拽条目
  const entries = []
  for (let i = 0; i < dt.items.length; i++) {
    const entry = dt.items[i].webkitGetAsEntry?.()
    if (entry) entries.push(entry)
  }
  if (entries.length === 0) return

  await doDragUpload(entries)
}

async function doDragUpload(entries) {
  if (!entries || entries.length === 0) return

  isUploading.value = true
  uploadFailed.value = false
  uploadMessage.value = ''
  currentEntries = entries // 保存用于重试

  try {
    // 确保 SSH 已连接
    const isConnected = await getTauriAPI().isServerConnected(props.server.id)
    if (!isConnected) {
      await getTauriAPI().connectServer(props.server.id)
    }

    // 递归收集所有文件
    const allFiles: { path: string; file: File }[] = []
    for (const entry of entries) {
      const files = await readDirRecursive(entry)
      allFiles.push(...files)
    }
    const totalFiles = allFiles.length
    if (totalFiles === 0) {
      toast.info('拖拽内容为空')
      return
    }

    uploadMessage.value = `正在处理 ${totalFiles} 个文件...`
    uploadProgress.value = { file: uploadMessage.value, percent: 0 }

    let successCount = 0
    for (let i = 0; i < allFiles.length; i++) {
      const item = allFiles[i]
      const buffer = await item.file.arrayBuffer()
      const bytes = new Uint8Array(buffer)

      const remotePath = currentPath.value.endsWith('/')
        ? currentPath.value + item.path.replace(/^\//, '')
        : currentPath.value + '/' + item.path.replace(/^\//, '')

      uploadProgress.value = {
        file: item.path.replace(/^\//, ''),
        percent: Math.round(((i + 1) / totalFiles) * 100)
      }

      // 直接通过 IPC 上传：将文件写入 Tauri 临时目录后上传
      const tempDir = await window.__TAURI__.path.tempDir()
      const safeName = item.path.replace(/^\//, '').replace(/[^a-zA-Z0-9._-]/g, '_')
      const tempFilePath = `${tempDir}sftp_${Date.now()}_${safeName}`

      // Tauri v2 fs API: writeFile / mkdir / remove
      const parentDir = tempFilePath.substring(0, tempFilePath.lastIndexOf('/'))
      if (parentDir) {
        await window.__TAURI__.fs.mkdir(parentDir, { recursive: true }).catch(() => {})
      }
      await window.__TAURI__.fs.writeFile(tempFilePath, bytes)

      await getTauriAPI().uploadFile(props.server.id, remotePath, tempFilePath)

      // 清理临时文件
      try { await window.__TAURI__.fs.remove(tempFilePath) } catch {}

      successCount++
    }

    uploadProgress.value = null
    if (successCount > 0) toast.success(`成功上传 ${successCount} 个文件`)
    await loadDir()
  } catch (error: any) {
    uploadProgress.value = null
    uploadFailed.value = true
    toast.error(`上传失败: ${error.message}`)
  } finally {
    isUploading.value = false
  }
}

// 取消上传
async function cancelUpload() {
  uploadProgress.value = null
  uploadMessage.value = ''
  uploadFailed.value = false
  isUploading.value = false
  toast.info('已取消上传')
}

// 重试上传
async function retryUpload() {
  uploadFailed.value = false
  uploadProgress.value = null
  isUploading.value = false
  await doDragUploadFromPaths(currentPaths)
}

// Tauri 拖拽上传：直接拿到本地绝对路径，无需临时文件
let currentPaths: string[] = []

async function doDragUploadFromPaths(paths: string[]) {
  if (!paths?.length) return
  currentPaths = paths

  isUploading.value = true
  uploadFailed.value = false
  uploadMessage.value = ''

  try {
    // 确保 SSH 已连接
    const isConnected = await getTauriAPI().isServerConnected(props.server.id)
    if (!isConnected) {
      await getTauriAPI().connectServer(props.server.id)
    }

    const totalFiles = paths.length
    let successCount = 0

    for (let i = 0; i < paths.length; i++) {
      const localPath = paths[i]
      const fileName = localPath.split('/').pop() || localPath.split('\\').pop() || 'unknown'

      uploadProgress.value = {
        file: fileName,
        percent: Math.round(((i + 1) / totalFiles) * 100)
      }

      const remotePath = currentPath.value.endsWith('/')
        ? currentPath.value + fileName
        : currentPath.value + '/' + fileName

      try {
        await getTauriAPI().uploadFile(props.server.id, remotePath, localPath)
        successCount++
      } catch (err: any) {
        toast.error(`上传失败 ${fileName}: ${err.message}`)
      }
    }

    uploadProgress.value = null
    if (successCount > 0) toast.success(`成功上传 ${successCount} 个文件`)
    await loadDir()
  } catch (error: any) {
    uploadProgress.value = null
    uploadFailed.value = true
    toast.error(`上传失败: ${error.message}`)
  } finally {
    isUploading.value = false
  }
}
// 递归读取目录中的所有文件
function readDirRecursive(entry: any): Promise<{ path: string; file: File }[]> {
  return new Promise((resolve) => {
    if (entry.isFile) {
      entry.file((file: File) => {
        resolve([{ path: entry.fullPath.substring(1), file }]) // 去掉开头的 /
      })
    } else if (entry.isDirectory) {
      const dirReader = entry.createReader()
      const allEntries: any[] = []

      function readBatch() {
        dirReader.readEntries(async (entries: any[]) => {
          if (entries.length === 0) {
            // 所有条目读取完毕，递归处理
            const results: { path: string; file: File }[] = []
            for (const e of allEntries) {
              results.push(...await readDirRecursive(e))
            }
            resolve(results)
          } else {
            allEntries.push(...entries)
            readBatch() // 继续读取下一批（浏览器可能分批次返回）
          }
        })
      }
      readBatch()
    } else {
      resolve([])
    }
  })
}

async function deleteFile(file) {
  if (!confirm(`确定删除 ${file.name}？`)) return;

  const path = currentPath.value + '/' + file.name;

  try {
    await getTauriAPI().deleteSftpFile(props.server.id, path, file.type === 'directory');
    await loadDir();
    toast.success(`已删除 ${file.name}`);
  } catch (error) {
    handleError(error, { context: 'SFTP deleteFile' });
  }
}

function formatSize(size) {
  if (size === undefined || size === null) return '-';
  if (size === 0) return '0 B';
  if (size < 1024) return size + ' B';
  if (size < 1024 * 1024) return (size / 1024).toFixed(1) + ' KB';
  if (size < 1024 * 1024 * 1024) return (size / 1024 / 1024).toFixed(1) + ' MB';
  return (size / 1024 / 1024 / 1024).toFixed(1) + ' GB';
}

function formatDate(dateStr) {
  if (!dateStr) return '-';
  try {
    return new Date(dateStr).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return '-';
  }
}
</script>

