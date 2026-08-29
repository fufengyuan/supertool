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
      <input v-model="currentPath" @keyup.enter="loadDir" placeholder="/home/user"
        class="input input-bordered input-sm flex-1 text-xs transition-colors"
        :class="[loadingDir ? 'border-primary/40' : '']" />
      <button @click="loadDir" class="btn btn-ghost btn-sm gap-1.5" :disabled="loadingDir">
        <span v-if="loadingDir" class="loading loading-spinner loading-xs"></span>
        <span class="transition-opacity" :class="loadingDir ? 'opacity-50' : ''">刷新</span>
      </button>
      <button @click="goUp" class="btn btn-ghost btn-sm" :disabled="currentPath === '/' || loadingDir">↑ 上级</button>
      <button @click="showCreateFolderDialog" class="btn btn-ghost btn-sm gap-1.5" :disabled="loadingDir"><SvgIcon name="folderPlus" size="14" /> 新建文件夹</button>
      <button @click="uploadFile" class="btn btn-ghost btn-sm" :disabled="loadingDir">↑ 上传文件</button>
      <button @click="uploadFolder" class="btn btn-ghost btn-sm gap-1.5" :disabled="loadingDir"><SvgIcon name="folder" size="14" /> 上传文件夹</button>
    </div>

    <!-- 加载中的不定长进度条：切目录时给出「正在取数据」的明确反馈 -->
    <div class="h-0.5 w-full overflow-hidden bg-transparent">
      <div v-if="loadingDir" class="sftp-indet h-full w-1/3 rounded-full bg-primary"></div>
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
        <Transition enter-active-class="transition duration-150 ease-out" enter-from-class="opacity-0 scale-[0.98]"
          leave-active-class="transition duration-100 ease-in" leave-to-class="opacity-0">
          <div v-if="isDragOver" class="absolute inset-0 z-10 flex flex-col items-center justify-center gap-2 border-2 border-dashed border-[#89b4fa] rounded-lg bg-[rgba(30,30,46,0.9)] text-[#89b4fa] pointer-events-none">
            <SvgIcon name="upload" size="48" stroke-width="1.5" class="animate-bounce" />
            <p class="text-sm font-semibold">释放以上传文件</p>
          </div>
        </Transition>

        <!-- 首次进入 / 空列表时拉取中：骨架行 -->
        <div v-if="loadingDir && files.length === 0" class="flex flex-col gap-1">
          <div v-for="i in 8" :key="i" class="flex items-center gap-2.5 rounded-md px-2.5 py-2">
            <span class="w-5 h-5 shrink-0 rounded bg-base-content/10 animate-pulse" :style="{ animationDelay: `${i * 70}ms` }"></span>
            <span class="h-3 rounded bg-base-content/10 animate-pulse flex-1" :style="{ maxWidth: `${34 + ((i * 23) % 52)}%`, animationDelay: `${i * 70}ms` }"></span>
            <span class="w-[70px] h-3 rounded bg-base-content/10 animate-pulse" :style="{ animationDelay: `${i * 70}ms` }"></span>
            <span class="w-[110px] h-3 rounded bg-base-content/10 animate-pulse" :style="{ animationDelay: `${i * 70}ms` }"></span>
          </div>
        </div>

        <!-- 拉取失败：内联错误 + 重试 -->
        <div v-else-if="loadError && files.length === 0" class="flex flex-col items-center justify-center gap-3 py-10 text-base-content/60">
          <SvgIcon name="warning" size="40" stroke-width="1.5" class="text-error/70" />
          <p class="text-sm">{{ loadError }}</p>
          <button class="btn btn-sm btn-primary gap-1.5" :disabled="loadingDir" @click="loadDir">
            <span v-if="loadingDir" class="loading loading-spinner loading-xs"></span>重试
          </button>
        </div>

        <!-- 文件行：进出/换位都有过渡，重拉时旧列表淡出而不是硬切 -->
        <TransitionGroup v-else tag="div" name="sftp-rows" class="flex flex-col gap-1 transition-opacity duration-200"
          :class="loadingDir ? 'opacity-40 pointer-events-none' : ''">
          <div
            v-for="file in filteredFiles"
            :key="file.name"
            class="group relative flex items-center gap-2.5 rounded-md px-2.5 py-2 cursor-pointer transition-all duration-150 hover:bg-base-200 hover:translate-x-0.5"
            :class="[selectedFile?.name === file.name ? 'bg-base-content/10' : '']"
            @click="selectFile(file)"
            @dblclick="handleDoubleClick(file)"
          >
            <span v-if="selectedFile?.name === file.name" class="absolute left-0 top-1.5 bottom-1.5 w-0.5 rounded-full bg-primary"></span>
            <span class="flex items-center shrink-0 transition-transform duration-150 group-hover:scale-110">
              <SvgIcon v-if="file.type === 'directory'" name="folder" size="20" class="text-[#f9a825]" />
              <SvgIcon v-else name="file" size="20" class="text-base-content/60" />
            </span>
            <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-xs">{{ file.name }}</span>
            <span class="w-[70px] shrink-0 text-right text-xs text-base-content/60">{{ formatSize(file.size) }}</span>
            <span class="w-[130px] shrink-0 text-xs text-base-content/60">{{ formatDate(file.modifyTime) }}</span>
            <div class="flex shrink-0 items-center gap-1 transition-opacity duration-150"
              :class="busyRows[file.name] ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'">
              <span v-if="busyRows[file.name]" class="loading loading-dots loading-xs text-primary"></span>
              <template v-else>
                <button v-if="file.type === 'file'" @click.stop="downloadFile(file)" class="btn btn-ghost btn-xs btn-square" title="下载">
                  <SvgIcon name="download" size="14" />
                </button>
                <button @click.stop="deleteFile(file)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
                  <SvgIcon name="trash" size="14" />
                </button>
              </template>
            </div>
          </div>
        </TransitionGroup>

        <!-- 搜索无结果 -->
        <div v-if="!loadingDir && filteredFiles.length === 0 && files.length > 0" class="flex flex-col items-center justify-center gap-3 py-10 text-base-content/60 sftp-fade">
          <SvgIcon name="search" size="48" stroke-width="1.5" />
          <p>未找到匹配文件</p>
          <p class="text-xs">尝试其他关键词</p>
        </div>
        <!-- 真正空目录 -->
        <div v-if="!loadingDir && files.length === 0 && !loadError" class="flex flex-col items-center justify-center gap-3 py-10 text-base-content/60 sftp-fade">
          <SvgIcon name="folder" size="48" stroke-width="1.5" />
          <p>空目录</p>
        </div>
      </div>
    </div>

    <div class="flex gap-4 rounded-b-xl border-t border-base-content/10 bg-base-200 px-4 py-2.5 text-xs text-base-content/60">
      <span v-if="loadingDir" class="flex items-center gap-1.5 text-primary"><span class="loading loading-spinner loading-xs"></span>正在读取目录…</span>
      <template v-else>
        <span v-if="searchQuery">匹配 {{ filteredFiles.length }} / {{ files.length }} 项</span>
        <span v-else>{{ files.length }} 项</span>
      </template>
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

    <!-- 创建文件夹对话框 -->
    <div v-if="showCreateFolder" class="fixed inset-0 z-[1100] flex items-center justify-center bg-black/40">
      <div class="w-[320px] bg-base-100 border border-base-content/10 rounded-xl shadow-2xl p-4 relative">
        <button @click="showCreateFolder = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭">
          <SvgIcon name="x" size="16" />
        </button>
        <h3 class="text-sm font-semibold mb-3">新建文件夹</h3>
        <input
          v-model="newFolderName"
          type="text"
          placeholder="文件夹名称"
          class="input input-bordered input-sm w-full text-xs"
          @keyup.enter="createFolder"
          ref="newFolderInputRef"
        />
        <div class="flex justify-end gap-2 mt-3">
          <button class="btn btn-ghost btn-sm" @click="showCreateFolder = false">取消</button>
          <button class="btn btn-primary btn-sm" @click="createFolder" :disabled="!newFolderName.trim()">创建</button>
        </div>
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <div v-if="showDeleteConfirm" class="fixed inset-0 z-[1100] flex items-center justify-center bg-black/40">
      <div class="w-[320px] bg-base-100 border border-base-content/10 rounded-xl shadow-2xl p-4 relative">
        <button @click="showDeleteConfirm = false" class="absolute top-3 right-3 btn btn-ghost btn-sm btn-square rounded-full" title="关闭">
          <SvgIcon name="x" size="16" />
        </button>
        <h3 class="text-sm font-semibold mb-2">确认删除</h3>
        <p class="text-xs text-base-content/70 mb-1">
          {{ deleteTarget?.type === 'directory' ? '文件夹' : '文件' }}：<span class="font-medium text-base-content">{{ deleteTarget?.name }}</span>
        </p>
        <p class="text-xs text-error mb-3">{{ deleteTarget?.type === 'directory' ? '文件夹内所有内容都将被删除，此操作不可恢复！' : '删除后无法恢复，确定要继续吗？' }}</p>
        <div class="flex justify-end gap-2">
          <button class="btn btn-ghost btn-sm" @click="showDeleteConfirm = false">取消</button>
          <button class="btn btn-error btn-sm" @click="confirmDelete">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import * as logger from '../../services/logger'
import { getTauriAPI } from '../../utils/tauri-api'
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { tempDir } from '@tauri-apps/api/path';
import { writeFile, mkdir, remove, readDir, stat } from '@tauri-apps/plugin-fs';
import { useToast } from '../../composables/useToast';
import { useErrorHandler } from '../../composables/useErrorHandler';

interface SftpFile {
  name: string;
  type: string;
  size?: number;
  modifyTime?: string;
}

import type { Server } from '../../types'

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
  if (!searchQuery.value.trim()) {return files.value;}
  const query = searchQuery.value.toLowerCase();
  return files.value.filter(file => file.name.toLowerCase().includes(query));
});
const uploadProgress = ref<{ file: string; percent: number; speedFormatted?: string } | null>(null);
const connectionStatus = ref('connecting'); // 'online' | 'offline' | 'connecting'
const isDragOver = ref(false);
// 目录读取与行内操作状态：驱动骨架屏、顶部进度条、按钮禁用与行内 spinner
const loadingDir = ref(false);
const loadError = ref('');
const busyRows = ref<Record<string, boolean>>({});
let loadToken = 0;
const isUploading = ref(false);
const uploadMessage = ref(''); // 实时进度消息
const uploadFailed = ref(false); // 上传失败状态
const downloadProgress = ref<{ file: string; percent: number } | null>(null); // 下载进度
const downloadSpeed = ref(''); // 下载速率
let progressCleanup: (() => void) | null = null;
let uploadProgressCleanup: (() => void) | null = null;
let downloadProgressCleanup: (() => void) | null = null;
let uploadDoneCleanup: (() => void) | null = null;
let disconnectCleanup: (() => void) | null = null;
let currentEntries: any[] = []; // 保存拖拽条目用于重试
const fileListRef = ref<HTMLElement | null>(null);

// 创建文件夹
const showCreateFolder = ref(false);
const newFolderName = ref('');
const newFolderInputRef = ref<HTMLInputElement | null>(null);

// 删除确认
const showDeleteConfirm = ref(false);
const deleteTarget = ref<SftpFile | null>(null);

function showCreateFolderDialog() {
  newFolderName.value = '';
  showCreateFolder.value = true;
  setTimeout(() => {
    newFolderInputRef.value?.focus();
  }, 50);
}

async function createFolder() {
  if (!newFolderName.value.trim()) {return;}
  const folderPath = currentPath.value === '/' ? '/' + newFolderName.value : currentPath.value + '/' + newFolderName.value;

  try {
    const result = await getTauriAPI().sftpCreateDir(props.server.id, folderPath);
    if (result?.success) {
      toast.success(`文件夹创建成功: ${newFolderName.value}`);
      showCreateFolder.value = false;
      newFolderName.value = '';
      await loadDir();
    } else {
      toast.error(`创建失败: ${result?.error || '未知错误'}`);
    }
  } catch (error: any) {
    handleError(error, { context: 'SFTP createFolder' });
  }
}

const connectionLabel = ref('连接中...');
const isMaximized = ref(false);
const defaultPos = props.initialPosition || { x: Math.max(50, (window.innerWidth - 900) / 2), y: 50 };
const panelPos = ref({ x: defaultPos.x, y: defaultPos.y });

const panelStyle = computed(() => {
  if (isMaximized.value) {
    return { top: '0', left: '0', width: '100vw', height: '100vh', borderRadius: '0' };
  }
  return {
    left: panelPos.value.x + 'px',
    top: panelPos.value.y + 'px',
    width: '900px',
    height: '700px',
  };
});

// 拖拽
let isDragging = false;
let dragStartX = 0;
let dragStartY = 0;
let panelStartX = 0;
let panelStartY = 0;

function startDrag(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('button, input, select')) {return;}
  if (isMaximized.value) {return;}

  isDragging = true;
  dragStartX = e.clientX;
  dragStartY = e.clientY;
  panelStartX = panelPos.value.x;
  panelStartY = panelPos.value.y;

  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', stopDrag);
}

function onDrag(e: MouseEvent) {
  if (!isDragging) {return;}
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
  const unlistenUpload = await getTauriAPI().onSftpUploadProgress((data) => {
    if (data.serverId === props.server.id) {
      const percent = data.total > 0 ? Math.round((data.uploaded / data.total) * 100) : 0
      // 计算速率
      const speedText = data.total > 0 && data.uploaded > 0
        ? `${formatBytes(data.uploaded)} / ${formatBytes(data.total)}`
        : ''
      uploadProgress.value = {
        file: data.fileName,
        percent,
        speedFormatted: speedText,
      }
    }
  })
  uploadProgressCleanup = unlistenUpload as any

  // 监听下载进度事件
  const unlistenDownload = await getTauriAPI().onSftpDownloadProgress((data) => {
    if (data.serverId === props.server.id) {
      const percent = data.total > 0 ? Math.round((data.downloaded / data.total) * 100) : 0
      const speedText = data.total > 0 && data.downloaded > 0
        ? `${formatBytes(data.downloaded)} / ${formatBytes(data.total)}`
        : ''
      downloadProgress.value = { file: data.fileName, percent }
      downloadSpeed.value = speedText
    }
  })
  downloadProgressCleanup = unlistenDownload as any

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
  if (uploadProgressCleanup) {
    uploadProgressCleanup();
    uploadProgressCleanup = null;
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

function formatBytes(bytes: number): string {
  if (bytes < 1024) {return bytes + ' B'}
  if (bytes < 1024 * 1024) {return (bytes / 1024).toFixed(1) + ' KB'}
  if (bytes < 1024 * 1024 * 1024) {return (bytes / (1024 * 1024)).toFixed(1) + ' MB'}
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
}

let uploadIdCounter = 0

async function loadDir() {
  const token = ++loadToken;          // 连点目录时丢弃过期响应，避免列表闪回旧目录
  loadingDir.value = true;
  loadError.value = '';
  try {
    // 直接列目录，不再调用可能卡死的 connectServer
    // 连接由 onMounted 初始化时完成，Rust 端会自动重连
    const result = await getTauriAPI().listSftpDir(props.server.id, currentPath.value);
    if (token !== loadToken) {return;}
    if (result.success) {
      files.value = result.files.sort((a: any, b: any) => {
        if (a.type === 'directory' && b.type !== 'directory') {return -1;}
        if (a.type !== 'directory' && b.type === 'directory') {return 1;}
        return a.name.localeCompare(b.name);
      });
      connectionStatus.value = 'online';
      connectionLabel.value = '已连接';
    } else {
      connectionStatus.value = 'offline';
      connectionLabel.value = '连接失败';
      loadError.value = result.error || '目录读取失败';
      toast.error('加载失败: ' + result.error);
    }
  } catch (error) {
    if (token !== loadToken) {return;}
    connectionStatus.value = 'offline';
    connectionLabel.value = '连接失败';
    loadError.value = (error as Error)?.message || '目录读取失败';
    handleError(error, { context: 'SFTP loadDir' });
  } finally {
    if (token === loadToken) {loadingDir.value = false;}
  }
}

// SFTP 自动重连
async function autoReconnectSftp() {
  const MAX_RETRIES = 2
  for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
    if (attempt > 1) {await new Promise(r => setTimeout(r, 1500))}
    try {
      const isConnected = await getTauriAPI().isServerConnected(props.server.id)
      if (!isConnected) {
        const connResult = await getTauriAPI().connectServer(props.server.id)
        if (!connResult?.success) {throw new Error(connResult?.error || 'SSH 连接失败')}
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
  if (currentPath.value === '/') {return;}
  const parts = currentPath.value.split('/').filter(Boolean);
  parts.pop();
  currentPath.value = '/' + parts.join('/');
  loadDir();
}

function selectFile(file: any) {
  selectedFile.value = file;
}

function enterDir(file: any) {
  if (file.type === 'directory') {
    const base = currentPath.value === '/' ? '' : currentPath.value;
    currentPath.value = base + '/' + file.name;
    loadDir();
  }
}

async function handleDoubleClick(file: any) {
  if (file.type === 'directory') {
    enterDir(file);
  } else {
    await openFileEditor(file);
  }
}

async function openFileEditor(file: any) {
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

async function downloadFile(file: any) {
  try {
    // 获取用户下载目录
    const downloadsDir = await getTauriAPI().getDownloadsDir()
    const localPath = downloadsDir.endsWith('/') || downloadsDir.endsWith('\\\\')
      ? downloadsDir + file.name
      : downloadsDir + '/' + file.name

    busyRows.value = { ...busyRows.value, [file.name]: true }
    downloadProgress.value = { file: file.name, percent: 0 }
    downloadSpeed.value = ''

    const downloadId = `download-${Date.now()}-${++uploadIdCounter}`
    await getTauriAPI().downloadFileWithProgress(
      downloadId,
      props.server.id,
      props.server.name || props.server.id,
      currentPath.value + '/' + file.name,
      localPath,
      file.name
    )
    downloadProgress.value = null
    downloadSpeed.value = ''
    toast.success(`下载成功: ${localPath}`)
  } catch (error) {
    downloadProgress.value = null
    downloadSpeed.value = ''
    handleError(error, { context: 'SFTP downloadFile' })
  } finally {
    const next = { ...busyRows.value }
    delete next[file.name]
    busyRows.value = next
  }
}

async function uploadFolder() {
  const result = await getTauriAPI().showOpenDialogForDirs()
  if (result.canceled || !result.filePaths?.length) {return}

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
  if (result.canceled || !result.filePaths?.length) {return}
  await doUpload(result.filePaths)
}

async function doUpload(filePaths: string[]) {
  if (isUploading.value || filePaths.length === 0) {return}
  isUploading.value = true

  try {
    // 确保 SSH 已连接
    const isConnected = await getTauriAPI().isServerConnected(props.server.id)
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
        const uploadId = `upload-${Date.now()}-${++uploadIdCounter}`
        await getTauriAPI().uploadFileWithProgress(
          uploadId,
          props.server.id,
          props.server.name || props.server.id,
          remotePath,
          filePath,
          fileName
        )
        successCount++
      } catch (err: any) {
        toast.error(`上传失败 ${fileName}: ${err.message}`)
      }
    }

    uploadProgress.value = null
    if (successCount > 0) {toast.success(`成功上传 ${successCount} 个文件`)}
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
  if (!dt || !dt.items) {return}

  // 收集所有拖拽条目
  const entries = []
  for (let i = 0; i < dt.items.length; i++) {
    const entry = dt.items[i].webkitGetAsEntry?.()
    if (entry) {entries.push(entry)}
  }
  if (entries.length === 0) {return}

  await doDragUpload(entries)
}

async function doDragUpload(entries: any) {
  if (!entries || entries.length === 0) {return}

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

      const fileName = item.path.replace(/^\//, '')
      uploadProgress.value = { file: fileName, percent: 0 }

      // 通过 IPC 上传：先写入 Tauri 临时目录（Tauri v2 API）
      const tempDirPath = await tempDir()
      const safeName = fileName.replace(/[^a-zA-Z0-9._-]/g, '_')
      const tempFilePath = `${tempDirPath}sftp_${Date.now()}_${safeName}`

      const parentDir = tempFilePath.substring(0, tempFilePath.lastIndexOf('/'))
      if (parentDir) {
        await mkdir(parentDir, { recursive: true }).catch(() => {})
      }
      await writeFile(tempFilePath, bytes)

      const uploadId = `upload-${Date.now()}-${++uploadIdCounter}`
      await getTauriAPI().uploadFileWithProgress(
        uploadId,
        props.server.id,
        props.server.name || props.server.id,
        remotePath,
        tempFilePath,
        fileName
      )

      // 清理临时文件
      try { await remove(tempFilePath) } catch {}

      successCount++
    }

    uploadProgress.value = null
    if (successCount > 0) {toast.success(`成功上传 ${successCount} 个文件`)}
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

// 重试上传：按实际来源重试（HTML5 entry 或 Tauri 原生路径）
async function retryUpload() {
  uploadFailed.value = false
  uploadProgress.value = null
  isUploading.value = false
  if (currentEntries.length > 0) {
    await doDragUpload(currentEntries)
  } else if (currentPaths.length > 0) {
    await doDragUploadFromPaths(currentPaths)
  } else {
    toast.info('没有可重试的上传任务')
  }
}

// Tauri 拖拽上传：直接拿到本地绝对路径，无需临时文件
let currentPaths: string[] = []

async function doDragUploadFromPaths(paths: string[]) {
  if (!paths?.length) {return}
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

    // 分类拖入项：目录 → zip 打包上传+远程解压（单文件传输，快且保留结构）；文件 → 单文件上传
    const dirs: { localPath: string; name: string }[] = []
    const files: { localPath: string; relPath: string }[] = []
    for (const p of paths) {
      try {
        const st = await stat(p)
        if (st.isDirectory) {
          const name = (p.split('/').pop() || p.split('\\').pop() || 'folder')
          dirs.push({ localPath: p, name })
        } else if (st.isFile) {
          files.push({ localPath: p, relPath: (p.split('/').pop() || p.split('\\').pop() || 'unknown') })
        }
      } catch (e: any) {
        toast.error(`无法读取 ${p}: ${e?.message || e}`)
      }
    }

    let successCount = 0
    const baseRemote = currentPath.value.endsWith('/') ? currentPath.value : currentPath.value + '/'

    // 1) 目录：zip 打包上传 + 远程解压（每个目录一次请求）
    for (const d of dirs) {
      uploadProgress.value = { file: d.name, percent: 0 }
      uploadMessage.value = `正在打包并上传目录 ${d.name}...`
      try {
        // 上传到 baseRemote/目录名/ 下（zip 内容为目录内文件，解压后平铺进该目录）
        const remoteDir = baseRemote + d.name
        await getTauriAPI().sftpUploadFolderZip(props.server.id, d.localPath, remoteDir)
        successCount++
      } catch (err: any) {
        toast.error(`目录上传失败 ${d.name}: ${err.message}`)
      }
    }

    // 2) 文件：单文件上传（保持原有逻辑）
    for (const f of files) {
      const relPath = f.relPath
      const fileName = relPath.split('/').pop() || 'unknown'

      uploadProgress.value = { file: fileName, percent: 0 }

      try {
        const remotePath = baseRemote + relPath
        const uploadId = `upload-${Date.now()}-${++uploadIdCounter}`
        await getTauriAPI().uploadFileWithProgress(
          uploadId,
          props.server.id,
          props.server.name || props.server.id,
          remotePath,
          f.localPath,
          fileName
        )
        successCount++
      } catch (err: any) {
        toast.error(`上传失败 ${relPath}: ${err.message}`)
      }
    }

    uploadProgress.value = null
    if (successCount > 0) {toast.success(`成功上传 ${successCount} 项`)}
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

async function deleteFile(file: any) {
  deleteTarget.value = file;
  showDeleteConfirm.value = true;
}

async function confirmDelete() {
  if (!deleteTarget.value) {return;}
  const file = deleteTarget.value;
  showDeleteConfirm.value = false;
  deleteTarget.value = null;

  const path = currentPath.value + '/' + file.name;

  busyRows.value = { ...busyRows.value, [file.name]: true };
  try {
    await getTauriAPI().deleteSftpFile(props.server.id, path, file.type === 'directory');
    await loadDir();
    toast.success(`已删除 ${file.name}`);
  } catch (error) {
    handleError(error, { context: 'SFTP deleteFile' });
  } finally {
    const next = { ...busyRows.value }
    delete next[file.name]
    busyRows.value = next
  }
}

function formatSize(size: number | null | undefined) {
  if (size === undefined || size === null) {return '-';}
  if (size === 0) {return '0 B';}
  if (size < 1024) {return size + ' B';}
  if (size < 1024 * 1024) {return (size / 1024).toFixed(1) + ' KB';}
  if (size < 1024 * 1024 * 1024) {return (size / 1024 / 1024).toFixed(1) + ' MB';}
  return (size / 1024 / 1024 / 1024).toFixed(1) + ' GB';
}

function formatDate(dateStr: string | null | undefined) {
  if (!dateStr) {return '-';}
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
/* 不定长进度条：目录读取中 */
.sftp-indet {
  animation: sftpIndet 1.1s ease-in-out infinite;
}
@keyframes sftpIndet {
  0% { transform: translateX(-110%); }
  100% { transform: translateX(330%); }
}

/* 文件行进出/换位动画 */
.sftp-rows-enter-active { transition: opacity .22s ease, transform .22s cubic-bezier(.32,.72,.35,1); }
.sftp-rows-enter-from { opacity: 0; transform: translateY(6px); }
.sftp-rows-leave-active { transition: opacity .16s ease, transform .16s ease; position: absolute; width: calc(100% - 1rem); }
.sftp-rows-leave-to { opacity: 0; transform: translateX(-10px); }
.sftp-rows-move { transition: transform .24s cubic-bezier(.32,.72,.35,1); }

/* 空态淡入 */
.sftp-fade { animation: sftpFade .28s ease both; }
@keyframes sftpFade { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: none; } }

@media (prefers-reduced-motion: reduce) {
  .sftp-indet, .sftp-fade { animation: none; }
  .sftp-rows-enter-active, .sftp-rows-leave-active, .sftp-rows-move { transition: none; }
}
</style>
