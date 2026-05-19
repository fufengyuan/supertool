<template>
  <div
    :class="[
      'fixed bottom-5 right-5 w-[720px] h-[650px] bg-base-100 rounded-xl shadow-2xl flex flex-col z-[1000] transition-all duration-300 border border-base-content/10 overflow-hidden',
      {
        '!w-[90vw] !h-[85vh] !bottom-[5vh] !right-[5vw] !rounded-2xl': isMaximized,
        'drag-over': isDragOver,
      }
    ]"
    v-if="peer"
    @dragover.prevent="onDragOver"
    @dragleave="onDragLeave"
    @drop.prevent="onDrop"
    @paste="onPaste"
  >
    <!-- 拖拽遮罩 -->
    <div v-if="isDragOver" class="absolute inset-0 bg-[rgba(102,126,234,0.15)] z-[9999] flex items-center justify-center pointer-events-none border-3 border-dashed border-[rgba(102,126,234,0.6)] rounded-xl">
      <div class="flex flex-col items-center gap-3 text-primary text-lg font-semibold">
        <SvgIcon name="upload" size="48" stroke-width="1.5" />
        <p>松开发送文件</p>
      </div>
    </div>

    <div class="flex justify-between items-center px-4 py-[14px] bg-primary text-white rounded-t-xl shrink-0">
      <div class="header-left">
        <h3 class="text-sm font-semibold m-0 whitespace-nowrap overflow-hidden text-ellipsis">{{ $t('lan.chatWith', { name: peer.name }) }}</h3>
      </div>
      <div class="flex items-center gap-1.5">
        <button class="bg-none border-none text-white text-sm cursor-pointer opacity-80 w-7 h-7 flex items-center justify-center rounded transition-all duration-200 hover:opacity-100 hover:bg-white/15" @click="toggleMaximize" :title="isMaximized ? '还原' : '最大化'">
          {{ isMaximized ? '❐' : '⬚' }}
        </button>
        <button class="bg-none border-none text-white text-base cursor-pointer opacity-80 w-7 h-7 flex items-center justify-center rounded transition-all duration-200 hover:opacity-100 hover:bg-white/15" @click="$emit('close')"><SvgIcon name="x" size="14" class="inline-block" /></button>
      </div>
    </div>

    <div class="flex-1 p-3 overflow-y-auto flex flex-col gap-1 bg-base-200" ref="messagesContainerRef" @scroll="handleScroll">
      <!-- 加载更多提示 -->
      <div v-if="isLoadingMore" class="text-center p-2 text-base-content/60 text-xs">
        <span class="inline-block w-3 h-3 border-2 border-base-content/60 border-t-transparent rounded-full animate-spin align-middle me-1.5"></span> 加载历史消息...
      </div>
      <div v-if="noMoreMessages" class="text-center px-2 py-1 text-base-content/60 text-[11px] opacity-60">没有更多消息了</div>

      <template v-for="item in displayMessages" :key="item.key">
        <!-- 时间分隔 -->
        <div v-if="item.type === 'time-sep'" class="text-center py-2 text-[11px] text-base-content/60 opacity-70">
          {{ item.text }}
        </div>
        <!-- 消息气泡 -->
        <ChatMessage
          v-else
          :message="item.message"
          :my-user-id="myUserInfo.id"
          :format-date="formatDate"
          :format-file-size="formatFileSize"
          :get-file-status="getFileStatus"
          :avatar="item.message.fromUserId !== myUserInfo.id ? (props.peer?.avatar || '😀') : (myUserInfo.avatar || '😀')"
          @download="openFileFolder"
          @open-folder="openFileFolder"
          @retry="retryFileTransfer"
        />
      </template>
    </div>

    <ChatInput
      v-model="newMessage"
      @send="sendMessage"
      @file-select-click="pickFileAndSend"
      @emoji-select="onEmojiSelect"
      @screenshot="onScreenshot"
      @assign-task="showTaskAssign = true"
      :placeholder="$t('lan.placeholder')"
    />
  </div>

  <!-- 任务分配弹窗 -->
  <TaskAssign
    v-if="showTaskAssign && peer"
    :peer="peer"
    @close="showTaskAssign = false"
    @assigned="onTaskAssigned"
  />
</template>

<script setup lang="ts">
import * as logger from '../../services/logger'
import { getTauriAPI } from '../../utils/tauri-api'
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import ChatMessage from './ChatMessage.vue';
import ChatInput from './ChatInput.vue';
import TaskAssign from './TaskAssign.vue';
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useErrorHandler } from '../../composables/useErrorHandler';

const { handleError } = useErrorHandler();

// 判断文件是否为图片
function isImageFile(fileName: string): boolean {
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(ext);
}

const props = defineProps<{
  peer?: { id: string; name: string; avatar?: string };
}>();

const emit = defineEmits(['close', 'refresh-unread']);

const messages = ref<any[]>([]);
const newMessage = ref('');
const myUserInfo = ref<{ id: string; name: string; avatar: string }>({ id: '', name: '', avatar: '😀' });
const messagesContainerRef = ref<HTMLElement | null>(null);
const selectedFiles = ref(new Map<string, string>()); // 保存 fileId → 文件路径映射，用于重试

// 拖拽状态
const isDragOver = ref(false);
let dragCounter = 0;

// 分页 & 无限滚动
const PAGE_SIZE = 50;
const currentPage = ref(0);
const isLoadingMore = ref(false);
const noMoreMessages = ref(false);
let previousScrollHeight = 0;

// 窗口最大化
const isMaximized = ref(false);

// 任务分配
const showTaskAssign = ref(false);

// 获取当前用户信息
async function loadUserInfo() {
  try {
    myUserInfo.value = await getTauriAPI().getUserInfo(myUserInfo.value.id);
  } catch (error) {
    handleError(error, { context: '获取用户信息', showToast: true });
    myUserInfo.value = { id: 'unknown', name: 'Unknown', avatar: '😀' };
  }
}

// 消息规范化：解析文件消息的 JSON content
function normalizeMessage(msg: any): any {
  if (msg.type === 'file') {
    // 解析 content 中的 JSON 元数据
    let meta: Record<string, any> = {}
    try {
      meta = typeof msg.content === 'string' ? JSON.parse(msg.content) : (msg.content || {})
    } catch {
      // content 不是 JSON，忽略
    }
    const result = {
      ...msg,
      fileName: meta.fileName || msg.fileName || '未知文件',
      fileSize: meta.fileSize ?? msg.fileSize ?? 0,
      filePath: meta.filePath || msg.filePath || '',
      isImage: meta.isImage ?? isImageFile(meta.fileName || msg.fileName || ''),
      status: meta.status || msg.status || 'pending',
      progress: msg.progress ?? meta.progress ?? 0,
      completedAt: meta.completedAt || msg.completedAt,
    };
    logger.info(`[normalizeMessage] id=${msg.id?.slice(0,8)}, type=${msg.type}, msg.status=${msg.status}, meta.status=${meta.status}, result.status=${result.status}`);
    return result;
  }
  return msg
}

// 加载与当前好友的聊天消息
async function loadMessageHistory(reset = false) {
  if (!props.peer) return;
  if (reset) {
    currentPage.value = 0;
    messages.value = [];
    noMoreMessages.value = false;
  }
  if (isLoadingMore.value) return;

  isLoadingMore.value = true;
  try {
    const history = await getTauriAPI().lanGetMessagesBetween(
      myUserInfo.value.id,
      props.peer.id,
      PAGE_SIZE,
      currentPage.value * PAGE_SIZE
    );
    const newMsgs = (history as any[]) || [];
    if (newMsgs.length < PAGE_SIZE) {
      noMoreMessages.value = true;
    }
    // 数据库返回的是 DESC 排序（最新在前），需要反转为 ASC（最新在后）
    const normalized = [...newMsgs].reverse().map(normalizeMessage);
    if (reset) {
      messages.value = normalized;
    } else {
      // 去重：过滤掉已经存在的消息（通过 id）
      const existingIds = new Set(messages.value.map(m => m.id));
      const deduped = normalized.filter(m => !existingIds.has(m.id));
      messages.value = [...deduped, ...messages.value];
    }
    currentPage.value++;

    // 标记已读
    await getTauriAPI().lanMarkMessagesRead(props.peer.id);
    emit('refresh-unread');
  } catch (error) {
    handleError(error, { context: '加载消息历史', showToast: true });
  } finally {
    isLoadingMore.value = false;
  }
}

// 滚动处理
function handleScroll() {
  const container = messagesContainerRef.value;
  if (!container || isLoadingMore.value || noMoreMessages.value) return;
  if (container.scrollTop < 50) {
    previousScrollHeight = container.scrollHeight;
    loadMessageHistory();
  }
}

function maintainScrollPosition() {
  const container = messagesContainerRef.value;
  if (container && previousScrollHeight > 0) {
    const newScrollHeight = container.scrollHeight;
    container.scrollTop = newScrollHeight - previousScrollHeight;
    previousScrollHeight = 0;
  }
}

// 时间分隔文本生成（提取为函数，消除重复代码）
function formatTimeSeparator(ts: number): string {
  const d = new Date(ts);
  const now = new Date();
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const yesterdayStart = todayStart - 86400000;
  const timeStr = d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  if (ts >= todayStart) return timeStr;
  if (ts >= yesterdayStart) return `昨天 ${timeStr}`;
  return `${d.getMonth() + 1}月${d.getDate()}日 ${timeStr}`;
}

interface DisplayItem {
  key: string;
  type: 'message' | 'time-sep';
  message?: any;
  text?: string;
  showTime: boolean;
}

const displayMessages = computed<DisplayItem[]>(() => {
  const items: DisplayItem[] = [];
  const TIME_GAP_MS = 5 * 60 * 1000; // 5 分钟

  for (let i = 0; i < messages.value.length; i++) {
    const msg = messages.value[i];
    const prevMsg = i > 0 ? messages.value[i - 1] : null;
    const currTime = new Date(msg.createdAt).getTime();
    const prevTime = prevMsg ? new Date(prevMsg.createdAt).getTime() : 0;
    const showTime = !prevMsg || (currTime - prevTime > TIME_GAP_MS);

    // 时间分隔符（第一条消息 + 间隔超过 5 分钟时显示）
    if (showTime) {
      items.push({
        key: `time-${i}-${currTime}`,
        type: 'time-sep',
        text: formatTimeSeparator(currTime),
        showTime: false,
      });
    }

    items.push({
      key: `msg-${msg.id || `${i}-${currTime}`}`,
      type: 'message',
      message: msg,
      showTime,
    });
  }

  return items;
});

// 监听消息加载完成后调整滚动（仅在分页变化时触发，避免每次消息更新都滚动）
let scrollWatchBypass = false;
watch(() => messages.value.length, (newLen, oldLen) => {
  if (scrollWatchBypass) return;
  // 只在批量加载新消息时滚动（增量超过 1 条 = 批量加载）
  if (newLen - oldLen > 1 || currentPage.value > 1) {
    if (currentPage.value > 1) {
      nextTick(() => maintainScrollPosition());
    } else {
      scrollToBottom();
    }
  }
});

function toggleMaximize() {
  isMaximized.value = !isMaximized.value;
}

function formatDate(timestamp: string | number) {
  if (!timestamp) return '';
  const date = new Date(timestamp);
  return date.toLocaleString('zh-CN');
}

function formatFileSize(bytes: number) {
  if (!bytes) return '0 Bytes';
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function getFileStatus(message: any) {
  switch (message.status) {
    case 'pending': return '待传输';
    case 'sending': return '发送中...';
    case 'receiving': return '接收中...';
    case 'completed': return '已完成';
    case 'error': return '传输失败';
    case 'cancelled': return '已取消';
    default: return '未知状态';
  }
}

// 打开文件所在文件夹
function openFileFolder(message: any) {
  if (message.filePath) {
    try {
      getTauriAPI().openFileFolder?.(message.filePath);
    } catch {
      alert(`文件路径: ${message.filePath}`);
    }
  }
}

// 拖拽事件处理
function onDragOver() {
  dragCounter++;
  isDragOver.value = true;
}

function onDragLeave() {
  dragCounter--;
  if (dragCounter <= 0) {
    isDragOver.value = false;
    dragCounter = 0;
  }
}

async function onDrop(event: DragEvent) {
  isDragOver.value = false;
  dragCounter = 0;
  // File paths are handled by the electron-file-drop custom event from preload.
  // The renderer's dataTransfer.files don't have .path with contextIsolation: true.
}

// 拖拽文件处理（通过 preload 的 electron-file-drop 自定义事件获取真实路径）
async function handleDroppedFiles(paths: string[]) {
  if (!props.peer) return;
  for (const filePath of paths) {
    const name = filePath.split('/').pop() || filePath.split('\\\\').pop() || 'file';
    // size=0 — 主进程 sendFile 会用 fs.statSync 获取真实大小
    await sendFile({ path: filePath, name, size: 0 });
  }
}

const onFileDrop = (e: Event) => {
  handleDroppedFiles((e as CustomEvent<string[]>).detail)
};

// 粘贴处理（支持粘贴图片发送）
async function onPaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;

  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item.type.indexOf('image') !== -1) {
      const file = item.getAsFile();
      if (!file || !props.peer) return;

      // Convert clipboard image to base64 and save via IPC
      try {
        const arrayBuffer = await file.arrayBuffer();
        const blob = new Blob([arrayBuffer]);
        const dataUrl = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve(reader.result as string);
          reader.onerror = () => reject(reader.error);
          reader.readAsDataURL(blob);
        });
        const base64 = dataUrl.split(',')[1];
        const fileName = `pasted_image_${Date.now()}.png`;
        const tmpPath = await getTauriAPI().saveTempFile(base64, fileName);
        if (tmpPath) {
          await sendFile({ path: tmpPath, name: fileName, size: file.size });
        }
      } catch (err) {
        console.error('[ChatPanel] Failed to paste image:', err);
      }
      break;
    }
  }
}

  // 发送消息
async function sendMessage() {
  console.error(`[ChatPanel][sendMessage] === ENTRY === peer=${props.peer?.id} (${props.peer?.name}), msgLen=${newMessage.value.trim().length}`)
  if (!newMessage.value.trim() || !props.peer) {
    console.error(`[ChatPanel][sendMessage] ABORT: empty message or no peer`)
    logger.warn(`[ChatPanel][sendMessage] Aborted: newMessage=${newMessage.value.trim()}, hasPeer=${!!props.peer}`)
    return
  }

  const content = newMessage.value.trim()
  console.error(`[ChatPanel][sendMessage] Calling IPC: sendMessage(${props.peer.id}, contentLen=${content.length})`)
  logger.info(`[ChatPanel][sendMessage] === SENDING === to: ${props.peer.id} (${props.peer.name}), contentLen=${content.length}`)

  const result = await getTauriAPI().lanSendMessage(props.peer.id, content)
  console.error(`[ChatPanel][sendMessage] IPC returned: ${JSON.stringify(result)}`)
  logger.info(`[ChatPanel][sendMessage] IPC result: success=${result?.success}, queued=${result?.queued}, messageId=${result?.messageId}, incompatibleVersion=${result?.incompatibleVersion}`)
  logger.info(`[ChatPanel][sendMessage] Full result: ${JSON.stringify(result)}`)

  // 版本不兼容
  if (result?.incompatibleVersion) {
    logger.error(`[ChatPanel][sendMessage] INCOMPATIBLE VERSION: peer v${result.incompatibleVersion} vs local v2.0`)
    handleError(new Error(`对方版本过低 (v${result.incompatibleVersion})，请升级 SuperTool 到最新版`), { context: '发送消息', showToast: true })
    return
  }

  // IPC 失败时不添加到 UI
  if (result?.success) {
    logger.info(`[ChatPanel][sendMessage] SUCCESS — adding to UI`)
  } else {
    const isQueued = result?.queued === true
    if (isQueued) {
      logger.info('[ChatPanel][sendMessage] Message queued for offline delivery')
    } else {
      logger.error(`[ChatPanel][sendMessage] FAILED — showing error toast. result=${JSON.stringify(result)}`)
      handleError(new Error('消息发送失败'), { context: '发送消息', showToast: true })
      return
    }
  }

  // 使用服务端生成的 messageId
  const msgId = result.messageId || crypto.randomUUID();
  const newMsg = {
    id: msgId,
    fromUserId: myUserInfo.value.id,
    fromUserName: myUserInfo.value.name,
    toUserId: props.peer.id,
    toUserName: props.peer.name,
    content: content,
    type: 'text',
    createdAt: new Date().toISOString(),
  };

  messages.value = [...messages.value, newMsg];
  scrollToBottom();
  newMessage.value = '';
}

// 表情选择
function onEmojiSelect(emoji: string) {
  newMessage.value += emoji;
}

// 截图处理
async function onScreenshot() {
  if (!getTauriAPI().lanScreenshot) {
    handleError(new Error('截图功能不可用，请检查应用权限'), { context: '截图', showToast: true });
    return;
  }
  try {
    const result = await getTauriAPI().lanScreenshot();
    if (!result.success) {
      handleError(new Error(result.error || '截图失败'), { context: '截图', showToast: true });
      return;
    }
    if (result.path && props.peer) {
      await sendFile({ path: result.path, name: 'screenshot.png', size: 0 });
    }
  } catch (err) {
    console.error('[ChatPanel] Failed to take screenshot:', err);
    handleError(err, { context: '截图', showToast: true });
  }
}

async function pickFileAndSend() {
  const result = await getTauriAPI().showOpenDialog({
    multiple: false,
    directory: false,
    title: '选择文件发送'
  });
  if (!result.filePaths?.length) return;
  const filePath = result.filePaths[0];
  const name = filePath.split('/').pop() || filePath.split('\\').pop() || 'file';
  logger.info(`[ChatPanel][pickFileAndSend] Selected: path=${filePath}, name=${name}`);
  sendFile({ path: filePath, name, size: 0 });
}

// 任务分配完成回调
function onTaskAssigned(_task: { text: string; priority: string; dueDate: string; note: string }) {
  showTaskAssign.value = false;
}

// 发送文件（带防重入）
const isSendingFile = ref(false);
async function sendFile(file: any, resumeOffset = 0) {
  if (isSendingFile.value) {
    console.warn('[ChatPanel][sendFile] Blocked: isSendingFile is true, skipping');
    return;
  }
  isSendingFile.value = true;

  const fileId = crypto.randomUUID();
  logger.info(`[ChatPanel][sendFile] Starting transfer: file=${file.name}, size=${file.size}, path=${file.path}, resumeOffset=${resumeOffset}, fileId=${fileId}`);

  // 先在 UI 中创建消息，确保回调失败时也能看到
  const fileMsg = {
    id: fileId,
    fromUserId: myUserInfo.value.id,
    fromUserName: myUserInfo.value.name,
    toUserId: props.peer!.id,
    toUserName: props.peer!.name,
    fileName: file.name,
    fileSize: file.size,
    filePath: file.path,
    isImage: isImageFile(file.name),
    status: 'sending',
    progress: resumeOffset > 0 ? Math.round((resumeOffset / file.size) * 100) : 0,
    type: 'file',
    createdAt: new Date().toISOString(),
  };

  try {
    messages.value = [...messages.value, fileMsg];
    selectedFiles.value.set(fileId, file.path);
    scrollToBottom();

    logger.info(`[ChatPanel][sendFile] Calling IPC sendFile: peerId=${props.peer!.id}, path=${file.path}, name=${file.name}, resumeOffset=${resumeOffset}, fileId=${fileId}`);
    const result = await getTauriAPI().lanSendFile(
      props.peer!.id,
      file.path,
      file.name,
      resumeOffset,
      fileId  // 传递 UI 创建的 fileId，确保双方 ID 一致
    );
    logger.info(`[ChatPanel][sendFile] IPC result: success=${result?.success}, error=${result?.error}`);

    if (!result.success) {
      console.error(`[ChatPanel][sendFile] Transfer failed for file=${file.name}, fileId=${fileId}`);
      const msgIndex = messages.value.findIndex(m => m.id === fileId);
      if (msgIndex !== -1) {
        messages.value[msgIndex].status = 'error';
      }
    }
  } catch (error) {
    console.error(`[ChatPanel][sendFile] Exception for file=${file.name}, fileId=${fileId}:`, error);
    handleError(error, { context: '发送文件', showToast: true });
    const msgIndex = messages.value.findIndex(m => m.id === fileId);
    if (msgIndex !== -1) {
      messages.value[msgIndex].status = 'error';
    }
  } finally {
    isSendingFile.value = false;
    logger.info(`[ChatPanel][sendFile] Released isSendingFile lock for fileId=${fileId}`);
  }
}

// 重试文件传输
async function retryFileTransfer(message: any) {
  if (!message || !props.peer) return;

  const filePath = selectedFiles.value.get(message.id) || message.filePath;
  logger.info(`[ChatPanel][retryFileTransfer] Retrying: messageId=${message.id}, fileName=${message.fileName}, progress=${message.progress}%, fileSize=${message.fileSize}, filePath(from selectedFiles)=${!!selectedFiles.value.get(message.id)}, filePath(from message)=${message.filePath}, resolvedPath=${filePath}`);

  if (!filePath) {
    console.error(`[ChatPanel][retryFileTransfer] No file path available for messageId=${message.id}`);
    handleError(new Error('文件路径不可用，无法重试'), { context: '重试文件传输', showToast: true });
    return;
  }

  const alreadySent = message.fileSize ? Math.round((message.progress || 0) / 100 * message.fileSize) : 0;
  logger.info(`[ChatPanel][retryFileTransfer] Resume offset: alreadySent=${alreadySent} bytes (${message.progress}% of ${message.fileSize})`);

  const msgIndex = messages.value.findIndex((m) => m.id === message.id);
  if (msgIndex !== -1) {
    messages.value[msgIndex].status = 'sending';
    messages.value[msgIndex].progress = alreadySent > 0 ? Math.round((alreadySent / message.fileSize) * 100) : 0;
  }

  try {
    logger.info(`[ChatPanel][retryFileTransfer] Calling IPC sendFile: peerId=${props.peer.id}, path=${filePath}, name=${message.fileName}, resumeOffset=${alreadySent}, fileId=${message.id}`);
    const result = await getTauriAPI().lanSendFile(
      props.peer.id,
      filePath,
      message.fileName,
      alreadySent,
      message.id  // 使用已有 messageId
    );
    logger.info(`[ChatPanel][retryFileTransfer] IPC result: success=${result?.success}, error=${result?.error}`);
    if (!result.success) {
      console.error(`[ChatPanel][retryFileTransfer] Retry failed for messageId=${message.id}, fileName=${message.fileName}`);
      if (msgIndex !== -1) {
        messages.value[msgIndex].status = 'error';
      }
    }
  } catch (error) {
    console.error(`[ChatPanel][retryFileTransfer] Exception for messageId=${message.id}, fileName=${message.fileName}:`, error);
    handleError(error, { context: '重试文件传输', showToast: true });
    if (msgIndex !== -1) {
      messages.value[msgIndex].status = 'error';
    }
  }
}

function scrollToBottom() {
  nextTick(() => {
    const container = messagesContainerRef.value;
    if (container) container.scrollTop = container.scrollHeight;
  });
}

// 监听 peer 变化
const previousPeerId = ref<string | null>(null);
watch(() => props.peer?.id, (newId) => {
  if (newId && newId !== previousPeerId.value) {
    previousPeerId.value = newId;
    loadMessageHistory(true);
  }
});

const cleanupFns: (() => void)[] = [];

onMounted(async () => {
  await loadUserInfo();
  await loadMessageHistory(true);
  previousPeerId.value = props.peer?.id || null;

  cleanupFns.push(await getTauriAPI().lanOnMessage((data: any) => {
      if (data.from === props.peer?.id) {
        const msgId = data.messageId || data.id
        if (msgId && messages.value.some(m => m.id === msgId)) return;
        const newMsg = {
          id: msgId || crypto.randomUUID(),
          fromUserId: data.from,
          fromUserName: data.fromName,
          toUserId: myUserInfo.value.id,
          toUserName: myUserInfo.value.name,
          content: data.content,
          type: 'text',
          createdAt: new Date(data.timestamp).toISOString(),
        };
        messages.value = [...messages.value, newMsg];
        scrollToBottom();
      }
    }));
  cleanupFns.push(await getTauriAPI().lanOnFileTransferStarted((data: any) => {
      if (data.fromUserId !== props.peer?.id && data.toUserId !== props.peer?.id) return;
      const exists = messages.value.some(m => m.id === data.fileId);
      if (!exists) {
        const newMsg = {
          id: data.fileId,
          fromUserId: data.fromUserId,
          fromUserName: data.fromUserName,
          toUserId: data.toUserId,
          toUserName: data.toUserName,
          fileName: data.fileName,
          fileSize: data.fileSize,
          filePath: data.filePath || '',
          isImage: isImageFile(data.fileName),
          status: data.status,
          progress: data.progress,
          type: 'file',
          createdAt: data.createdAt,
        };
        messages.value = [...messages.value, newMsg];
        scrollToBottom();
      }
    }));
  cleanupFns.push(await getTauriAPI().lanOnFileTransferProgress((data: any) => {
      const msgIndex = messages.value.findIndex((m) => m.id === data.fileId);
      if (msgIndex !== -1) {
        messages.value[msgIndex].progress = data.progress;
        messages.value[msgIndex].status = data.status;
      }
    }));
  cleanupFns.push(await getTauriAPI().lanOnFileTransferCompleted((data: any) => {
      logger.info(`[ChatPanel][lanOnFileTransferCompleted] Received: fileId=${data.fileId}, status=${data.status}`);
      const msgIndex = messages.value.findIndex((m) => m.id === data.fileId);
      if (msgIndex !== -1) {
        messages.value[msgIndex].status = data.status;
        messages.value[msgIndex].completedAt = data.completedAt;
        messages.value[msgIndex].filePath = data.filePath;
        messages.value[msgIndex].fileName = data.fileName || messages.value[msgIndex].fileName;
        messages.value[msgIndex].fileSize = data.fileSize ?? messages.value[msgIndex].fileSize;
        messages.value[msgIndex].isImage = data.isImage ?? isImageFile(data.fileName || messages.value[msgIndex].fileName);
        logger.info(`[ChatPanel][lanOnFileTransferCompleted] Updated message at index ${msgIndex}`);
      }
      scrollToBottom();
    }));
  cleanupFns.push(await getTauriAPI().lanOnFileTransferError((data: any) => {
      const msgIndex = messages.value.findIndex((m) => m.id === data.fileId);
      if (msgIndex !== -1) {
        messages.value[msgIndex].status = data.status || 'error';
        messages.value[msgIndex].isImage = messages.value[msgIndex].isImage ?? isImageFile(messages.value[msgIndex].fileName);
      }
    }));
  // 收到文件时刷新消息列表（确保 DB 记录已加载）
  cleanupFns.push(await getTauriAPI().lanOnFileReceived((data: any) => {
      if (data.fromUserId !== props.peer?.id) return;
      const exists = messages.value.some(m => m.id === data.fileId);
      if (!exists) {
        const newMsg = {
          id: data.fileId,
          fromUserId: data.fromUserId,
          fromUserName: data.fromUserName,
          toUserId: data.toUserId,
          toUserName: data.toUserName,
          fileName: data.fileName,
          fileSize: data.fileSize,
          isImage: isImageFile(data.fileName),
          status: data.status,
          progress: data.progress,
          type: 'file',
          createdAt: data.createdAt,
          filePath: data.filePath,
        };
        messages.value = [...messages.value, newMsg];
        scrollToBottom();
      }
    }));
  // 收到任务分配消息时添加到聊天列表
  cleanupFns.push(await getTauriAPI().lanOnTaskAssigned((data: any) => {
      const msgId = data.messageId || data.id;
      if (!msgId) return;
      if (data.from !== props.peer?.id && data.to !== props.peer?.id) return;
      const exists = messages.value.some(m => m.id === msgId);
      if (!exists) {
        const taskContent = JSON.stringify({
          taskId: data.task?.id || msgId,
          taskText: data.task?.text || '',
          priority: data.task?.priority || 'medium',
          dueDate: data.task?.dueDate || '',
          note: data.task?.note || ''
        });
        messages.value = [...messages.value, {
          id: msgId,
          fromUserId: data.from,
          fromUserName: data.fromName,
          toUserId: data.to || myUserInfo.value.id,
          toUserName: data.toName || myUserInfo.value.name,
          content: taskContent,
          type: 'task_assigned',
          createdAt: new Date(data.timestamp || Date.now()).toISOString(),
        }];
        scrollToBottom();
      }
    }));
  // Listen for dropped files from preload (has real file paths)
  window.addEventListener('tauri-file-drop', onFileDrop);
});

onUnmounted(() => {
  cleanupFns.forEach(fn => fn());
  selectedFiles.value.clear();
  window.removeEventListener('tauri-file-drop', onFileDrop);
});
</script>
