<template>
  <div class="sftp-panel" :style="panelStyle">
    <div class="sftp-header" @mousedown="startDrag">
      <div class="sftp-header-left">
        <span class="sftp-title">{{ server.name }} - SFTP</span>
        <span class="connection-badge" :class="connectionStatus">
          <span class="badge-dot" :class="connectionStatus"></span>
          {{ connectionLabel }}
        </span>
      </div>
      <div class="sftp-header-actions">
        <button @click.stop="toggleSize" class="btn-header-icon" :title="isMaximized ? '还原' : '最大化'">
          <svg v-if="!isMaximized" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="5" y="5" width="14" height="14" rx="1"/>
          </svg>
        </button>
        <button @click.stop="$emit('close')" class="btn-close" title="关闭">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="sftp-toolbar">
      <svg class="path-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
        <polyline points="9 22 9 12 15 12 15 22"/>
      </svg>
      <input v-model="currentPath" @keyup.enter="loadDir" placeholder="/home/user" class="path-input" />
      <button @click="loadDir" class="btn-toolbar">刷新</button>
      <button @click="goUp" class="btn-toolbar" :disabled="currentPath === '/'">↑ 上级</button>
      <button @click="uploadFile" class="btn-toolbar">↑ 上传文件</button>
      <button @click="uploadFolder" class="btn-toolbar">📁 上传文件夹</button>
    </div>

    <!-- 文件列表头部 -->
    <div class="file-list-header">
      <span class="col-icon"></span>
      <span class="col-name">名称</span>
      <span class="col-size">大小</span>
      <span class="col-time">修改时间</span>
      <span class="col-actions">操作</span>
    </div>

    <div class="sftp-content">
      <div
        ref="fileListRef"
        class="file-list"
        :class="{ 'drag-over': isDragOver }"
      >
        <!-- 拖拽提示层 -->
        <div v-if="isDragOver" class="drop-overlay">
          <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          <p>释放以上传文件</p>
        </div>

        <div
          v-for="file in files"
          :key="file.name"
          class="file-item"
          :class="{ selected: selectedFile?.name === file.name }"
          @click="selectFile(file)"
          @dblclick="handleDoubleClick(file)"
        >
          <span class="file-icon">
            <svg v-if="file.type === 'directory'" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" class="icon-folder">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" class="icon-file">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
            </svg>
          </span>
          <span class="file-name">{{ file.name }}</span>
          <span class="file-size">{{ formatSize(file.size) }}</span>
          <span class="file-time">{{ formatDate(file.modifyTime) }}</span>
          <div class="file-actions">
            <button v-if="file.type === 'file'" @click.stop="downloadFile(file)" class="btn-action" title="下载">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="7 10 12 15 17 10"/>
                <line x1="12" y1="15" x2="12" y2="3"/>
              </svg>
            </button>
            <button @click.stop="deleteFile(file)" class="btn-action btn-danger" title="删除">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </div>

        <div v-if="files.length === 0" class="empty-state">
          <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
          <p>空目录</p>
        </div>
      </div>
    </div>

    <div class="sftp-footer">
      <span>{{ files.length }} 项</span>
      <span v-if="selectedFile">已选: {{ selectedFile.name }}</span>
    </div>

    <!-- 上传进度 -->
    <div v-if="uploadProgress" class="upload-progress">
      <span class="upload-file">{{ uploadProgress.file }}</span>
      <span class="upload-speed">{{ uploadProgress.speedFormatted || '' }}</span>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: uploadProgress.percent + '%' }"></div>
      </div>
      <span class="upload-percent">{{ uploadProgress.percent }}%</span>
      <!-- 操作按钮 -->
      <div class="upload-actions">
        <button v-if="isUploading" @click="cancelUpload" class="btn-upload-cancel" title="取消上传">✕ 取消</button>
        <button v-if="uploadFailed" @click="retryUpload" class="btn-upload-retry" title="重试上传">↻ 重试</button>
      </div>
    </div>

    <!-- 下载进度 -->
    <div v-if="downloadProgress" class="upload-progress">
      <span class="upload-file">{{ downloadProgress.file }}</span>
      <span class="upload-speed">{{ downloadSpeed }}</span>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: downloadProgress.percent + '%' }"></div>
      </div>
      <span class="upload-percent">{{ downloadProgress.percent }}%</span>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
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
const selectedFile = ref<SftpFile | null>(null);
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
const defaultPos = props.initialPosition || { x: window.innerWidth - 620, y: 80 };
const panelPos = ref({ x: defaultPos.x, y: defaultPos.y });

const panelStyle = computed(() => {
  if (isMaximized.value) {
    return { top: '0', left: '0', width: '100vw', height: '100vh', borderRadius: '0' };
  }
  return {
    left: panelPos.value.x + 'px',
    top: panelPos.value.y + 'px',
    width: '600px',
    height: '500px',
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
      const isConnected = await getTauriAPI().onServerConnected(props.server.id)
      if (!isConnected) {
        const connResult = await getTauriAPI().connectServer(props.server.id)
        if (!connResult?.success) throw new Error(connResult?.error || 'SSH 连接失败')
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
    const result = await getTauriAPI().openSftpFileEditor(props.server.id, remotePath);
    if (!result?.success) {
      toast.error(`打开文件失败: ${result?.error}`);
    }
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
    const resp = await getTauriAPI().uploadFolder(props.server.id, currentPath.value, localDirPath)

    uploadProgress.value = { file: folderName, percent: 100 }

    if (!resp?.success) {
      toast.error(`上传失败: ${resp?.error}`)
    } else {
      toast.success(`文件夹上传成功: ${folderName}`)
      await loadDir()
    }
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

<style scoped>
.sftp-panel {
  position: fixed;
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  z-index: 1000;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.sftp-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--color-base-200);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 12px 12px 0 0;
}

.sftp-header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sftp-title {
  font-weight: 600;
  font-size: 14px;
}

.sftp-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-header-icon {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  transition: all 0.15s ease;
}

.btn-header-icon:hover {
  background: rgba(205, 214, 244, 0.1);
  color: var(--text);
}

/* 连接状态 */
.connection-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
}

.connection-badge.online {
  background: rgba(166, 227, 161, 0.15);
  color: var(--color-success);
}

.connection-badge.offline {
  background: rgba(243, 139, 168, 0.15);
  color: var(--color-error);
}

.connection-badge.connecting {
  background: rgba(249, 168, 37, 0.15);
  color: var(--color-warning);
}

.badge-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.badge-dot.online {
  background: var(--color-success);
}

.badge-dot.offline {
  background: var(--color-error);
}

.badge-dot.connecting {
  background: var(--color-warning);
  animation: blink 0.8s ease-in-out infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.btn-close {
  background: var(--color-error);
  border: none;
  color: white;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-close:hover {
  background: #e04560;
  transform: scale(1.1);
}

.sftp-toolbar {
  display: flex;
  gap: 8px;
  padding: 10px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  align-items: center;
}

.path-icon {
  opacity: 0.5;
  flex-shrink: 0;
}

.path-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
}

.btn-toolbar {
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-100);
  color: var(--color-base-content);
}

.btn-toolbar:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 列表头部 */
.file-list-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: var(--color-base-200);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-weight: 500;
}

.sftp-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  position: relative;
  min-height: 100px;
  transition: background 0.2s ease;
}

.file-list.drag-over {
  background: rgba(137, 180, 250, 0.08);
  border-radius: 8px;
}

/* 拖拽覆盖层 */
.drop-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(30, 30, 46, 0.9);
  border: 2px dashed #89b4fa;
  border-radius: 8px;
  z-index: 10;
  color: #89b4fa;
  gap: 8px;
  pointer-events: none;
}

.drop-overlay p {
  font-size: 14px;
  font-weight: 600;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s ease;
}

.file-item:hover {
  background: var(--color-base-200);
}

.file-item.selected {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.file-icon {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.icon-folder {
  color: #f9a825;
}

.icon-file {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.file-name {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size,
.file-time {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
  flex-shrink: 0;
}

.col-size,
.file-size {
  width: 70px;
  text-align: right;
}

.col-time,
.file-time {
  width: 130px;
}

.file-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s ease;
  flex-shrink: 0;
}

.file-item:hover .file-actions {
  opacity: 1;
}

.btn-action {
  padding: 4px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: var(--color-base-content);
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-action:hover {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.btn-action.btn-danger {
  color: var(--color-error);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  gap: 12px;
}

.sftp-footer {
  padding: 10px 16px;
  background: var(--color-base-200);
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  display: flex;
  gap: 15px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
  border-radius: 0 0 12px 12px;
}

.upload-progress {
  padding: 10px 16px;
  background: var(--color-base-200);
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}

.upload-file {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-base-content);
}

.upload-speed {
  font-size: 11px;
  color: var(--color-primary);
  min-width: 70px;
  text-align: right;
  flex-shrink: 0;
}

.upload-percent {
  font-weight: 600;
  color: var(--color-primary);
  min-width: 40px;
  text-align: right;
}

.upload-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.btn-upload-cancel {
  padding: 2px 8px;
  border: 1px solid var(--color-error);
  background: transparent;
  color: var(--color-error);
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  transition: all 0.15s ease;
}

.btn-upload-cancel:hover {
  background: var(--color-error);
  color: white;
}

.btn-upload-retry {
  padding: 2px 8px;
  border: 1px solid var(--color-primary);
  background: transparent;
  color: var(--color-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  transition: all 0.15s ease;
}

.btn-upload-retry:hover {
  background: var(--color-primary);
  color: white;
}

.progress-bar {
  flex: 1;
  height: 4px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  transition: width 0.2s ease;
}
</style>
