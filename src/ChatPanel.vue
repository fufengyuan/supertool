<template>
  <div
    class="chat-panel"
    :class="{ maximized: isMaximized, 'drag-over': isDragOver }"
    v-if="peer"
    @dragover.prevent="onDragOver"
    @dragleave="onDragLeave"
    @drop.prevent="onDrop"
    @paste="onPaste"
  >
    <!-- 拖拽遮罩 -->
    <div v-if="isDragOver" class="drag-overlay">
      <div class="drag-content">
        <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
        </svg>
        <p>松开发送文件</p>
      </div>
    </div>

    <div class="chat-header">
      <div class="header-left">
        <h3>与 {{ peer.name }} 的聊天</h3>
        <span v-if="peer.version" class="peer-version" :title="peer.version === '2.0' ? '✅ 兼容' : '⚠️ 版本较低'">v{{ peer.version }}</span>
      </div>
      <div class="header-actions">
        <button class="icon-btn" @click="toggleMaximize" :title="isMaximized ? '还原' : '最大化'">
          {{ isMaximized ? '❐' : '⬚' }}
        </button>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>
    </div>

    <div class="chat-messages" ref="messagesContainerRef" @scroll="handleScroll">
      <!-- 加载更多提示 -->
      <div v-if="isLoadingMore" class="loading-more">
        <span class="spinner"></span> 加载历史消息...
      </div>
      <div v-if="noMoreMessages" class="no-more">没有更多消息了</div>

      <template v-for="item in displayMessages" :key="item.key">
        <!-- 时间分隔 -->
        <div v-if="item.type === 'time-sep'" class="time-separator">
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
          :avatar="item.message.fromUserId !== myUserInfo.id ? (peer?.avatar || '😀') : (myUserInfo.avatar || '😀')"
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
      placeholder="输入消息..."
    />
  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, computed, onMounted, onUnmounted, nextTick, watch, triggerRef } from 'vue';
import ChatMessage from '@/components/lan/ChatMessage.vue';
import ChatInput from '@/components/ChatInput.vue';
import { getTauriAPI } from '@/utils/tauri-api';

// 判断文件是否为图片
function isImageFile(fileName: string): boolean {
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(ext);
}

const props = defineProps<{
  peer?: { id: string; name: string; avatar?: string; version?: string };
}>();

const emit = defineEmits(['close', 'refresh-unread']);

const messages = shallowRef<any[]>([]);
const newMessage = ref('');
const myUserInfo = ref<{ id: string; name: string; avatar: string }>({ id: '', name: '', avatar: '😀' });
const messagesContainerRef = ref<HTMLElement | null>(null);
const selectedFiles = ref(new Map<string, string>()); // fileId → 文件路径映射，用于重试

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

// 获取当前用户信息
async function loadUserInfo() {
  try {
    const api = getTauriAPI();
    const info = await api.getUserInfo('');
    myUserInfo.value = {
      id: info.id || '',
      name: info.name || info.id || 'Unknown',
      avatar: info.avatar || '😀',
    };
  } catch (error) {
    console.error('[ChatPanel] 获取用户信息失败:', error);
    myUserInfo.value = { id: 'unknown', name: 'Unknown', avatar: '😀' };
  }
}

// 消息规范化：解析文件消息的 JSON content
function normalizeMessage(msg: any): any {
  if (msg.type === 'file') {
    let meta: Record<string, any> = {};
    try {
      meta = typeof msg.content === 'string' ? JSON.parse(msg.content) : (msg.content || {});
    } catch {
      // content 不是 JSON，忽略
    }
    return {
      ...msg,
      fileName: meta.fileName || msg.fileName || '未知文件',
      fileSize: meta.fileSize ?? msg.fileSize ?? 0,
      filePath: meta.filePath || msg.filePath || '',
      isImage: meta.isImage ?? isImageFile(meta.fileName || msg.fileName || ''),
      status: meta.status || msg.status || 'pending',
      progress: msg.progress ?? meta.progress ?? 0,
      completedAt: meta.completedAt || msg.completedAt,
    };
  }
  return msg;
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
    const api = getTauriAPI();
    const history = await api.lanGetMessagesBetween(
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
    await api.markMessagesRead(props.peer.id);
    emit('refresh-unread');
  } catch (error) {
    console.error('[ChatPanel] 加载消息历史失败:', error);
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

// 时间分隔文本生成
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

    // 时间分隔符
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

// 监听消息加载完成后调整滚动
let scrollWatchBypass = false;
watch(() => messages.value.length, (newLen, oldLen) => {
  if (scrollWatchBypass) return;
  // 只在批量加载新消息时滚动
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
      getTauriAPI().openFileFolder(message.filePath);
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

  // 处理拖拽的文件
  if (event.dataTransfer?.files) {
    const files = Array.from(event.dataTransfer.files);
    for (const file of files) {
      // 在 Tauri 中，webview 无法获取文件的完整路径
      // 这里提示用户通过文件选择按钮发送
      console.warn('[ChatPanel] Drag-drop of files requires native file path. Please use the file picker button.');
    }
  }
}

// 粘贴处理（支持粘贴图片发送）
async function onPaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;

  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item.type.indexOf('image') !== -1) {
      const file = item.getAsFile();
      if (!file || !props.peer) return;

      try {
        const arrayBuffer = await file.arrayBuffer();
        const base64 = btoa(String.fromCharCode(...new Uint8Array(arrayBuffer)));
        const fileName = `pasted_image_${Date.now()}.png`;

        const api = getTauriAPI();
        const tmpPath = await api.lanSaveTempFile(fileName, base64);
        if (tmpPath) {
          await sendFile({ path: tmpPath, name: fileName, size: file.size });
        }
      } catch (err) {
        console.error('[ChatPanel] 粘贴图片失败:', err);
      }
      break;
    }
  }
}

// 发送消息
async function sendMessage() {
  if (!newMessage.value.trim() || !props.peer) return;

  const content = newMessage.value.trim();
  const api = getTauriAPI();

  try {
    const result = await api.lanSendMessage(props.peer.id, content);

    // 版本不兼容
    if (result?.incompatibleVersion) {
      console.error(`[ChatPanel] 对方版本过低 (v${result.incompatibleVersion})`);
      alert(`对方版本过低 (v${result.incompatibleVersion})，请升级到最新版`);
      return;
    }

    if (result?.success || result?.queued) {
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
    } else {
      console.error('[ChatPanel] 消息发送失败:', result);
      alert('消息发送失败');
    }
  } catch (err) {
    console.error('[ChatPanel] 发送消息异常:', err);
    alert('消息发送失败');
  }
}

// 表情选择
function onEmojiSelect(emoji: string) {
  newMessage.value += emoji;
}

// 截图处理
async function onScreenshot() {
  if (!props.peer) return;
  const api = getTauriAPI();

  try {
    if (api.lanScreenshot) {
      const result = await api.lanScreenshot();
      if (!result.success) {
        alert(result.error || '截图失败');
        return;
      }
      if (result.path) {
        await sendFile({ path: result.path, name: 'screenshot.png', size: 0 });
      }
    } else {
      console.warn('[ChatPanel] 截图功能不可用');
    }
  } catch (err) {
    console.error('[ChatPanel] 截图失败:', err);
    alert('截图失败');
  }
}

// 文件选择并发送
async function pickFileAndSend() {
  if (!props.peer) return;
  const api = getTauriAPI();

  try {
    // 使用系统文件选择对话框
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      multiple: false,
      title: '选择文件',
    });

    if (!selected) return;
    const filePath = selected;
    const name = filePath.split('/').pop() || filePath.split('\\').pop() || 'file';
    sendFile({ path: filePath, name, size: 0 });
  } catch (err) {
    console.error('[ChatPanel] 文件选择失败:', err);
  }
}

// 发送文件（带防重入）
const isSendingFile = ref(false);
async function sendFile(file: any, resumeOffset = 0) {
  if (isSendingFile.value) {
    console.warn('[ChatPanel][sendFile] 正在发送文件中，跳过');
    return;
  }
  isSendingFile.value = true;

  const fileId = crypto.randomUUID();
  console.log(`[ChatPanel][sendFile] 开始传输: file=${file.name}, size=${file.size}, path=${file.path}, fileId=${fileId}`);

  // 先在 UI 中创建消息
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

    const api = getTauriAPI();
    console.log(`[ChatPanel][sendFile] 调用 IPC sendFile: peerId=${props.peer!.id}, path=${file.path}, name=${file.name}`);
    const result = await api.lanSendFile(
      props.peer!.id,
      file.path,
      file.name,
    );
    console.log(`[ChatPanel][sendFile] IPC 返回: success=${result?.success}, error=${result?.error}`);

    if (!result.success) {
      const msgIndex = messages.value.findIndex(m => m.id === fileId);
      if (msgIndex !== -1) {
        messages.value[msgIndex].status = 'error';
        triggerRef(messages);
      }
    }
  } catch (error) {
    console.error(`[ChatPanel][sendFile] 异常:`, error);
    const msgIndex = messages.value.findIndex(m => m.id === fileId);
    if (msgIndex !== -1) {
      messages.value[msgIndex].status = 'error';
      triggerRef(messages);
    }
  } finally {
    isSendingFile.value = false;
  }
}

// 重试文件传输
async function retryFileTransfer(message: any) {
  if (!message || !props.peer) return;

  const filePath = selectedFiles.value.get(message.id) || message.filePath;
  console.log(`[ChatPanel][retryFileTransfer] 重试: messageId=${message.id}, fileName=${message.fileName}, filePath=${filePath}`);

  if (!filePath) {
    alert('文件路径不可用，无法重试');
    return;
  }

  const alreadySent = message.fileSize ? Math.round((message.progress || 0) / 100 * message.fileSize) : 0;

  const msgIndex = messages.value.findIndex((m) => m.id === message.id);
  if (msgIndex !== -1) {
    messages.value[msgIndex].status = 'sending';
    messages.value[msgIndex].progress = alreadySent > 0 ? Math.round((alreadySent / message.fileSize) * 100) : 0;
    triggerRef(messages);
  }

  try {
    const api = getTauriAPI();
    const result = await api.lanSendFile(
      props.peer.id,
      filePath,
      message.fileName,
    );
    if (!result.success) {
      if (msgIndex !== -1) {
        messages.value[msgIndex].status = 'error';
        triggerRef(messages);
      }
    }
  } catch (error) {
    console.error(`[ChatPanel][retryFileTransfer] 异常:`, error);
    if (msgIndex !== -1) {
      messages.value[msgIndex].status = 'error';
      triggerRef(messages);
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

  const api = getTauriAPI();

  // 监听实时消息
  cleanupFns.push(await api.lanOnMessage((data: any) => {
    if (data.from === props.peer?.id) {
      const msgId = data.messageId || data.id;
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

  // 监听文件传输开始
  cleanupFns.push(await api.lanOnFileTransferStarted((data: any) => {
    if (data.fromUserId !== props.peer?.id && data.toUserId !== props.peer?.id) return;
    const exists = messages.value.some(m => m.id === data.id);
    if (!exists) {
      const newMsg = {
        id: data.id,
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

  // 监听文件传输进度
  cleanupFns.push(await api.lanOnFileTransferProgress((data: any) => {
    const msgIndex = messages.value.findIndex((m) => m.id === data.id);
    if (msgIndex !== -1) {
      messages.value[msgIndex].progress = data.progress;
      messages.value[msgIndex].status = data.status;
      triggerRef(messages);
    }
  }));

  // 监听文件传输完成
  cleanupFns.push(await api.lanOnFileTransferCompleted((data: any) => {
    const msgIndex = messages.value.findIndex((m) => m.id === data.id);
    if (msgIndex !== -1) {
      messages.value[msgIndex].status = data.status;
      messages.value[msgIndex].completedAt = data.completedAt;
      messages.value[msgIndex].filePath = data.filePath;
      messages.value[msgIndex].fileName = data.fileName || messages.value[msgIndex].fileName;
      messages.value[msgIndex].fileSize = data.fileSize ?? messages.value[msgIndex].fileSize;
      messages.value[msgIndex].isImage = data.isImage ?? isImageFile(data.fileName || messages.value[msgIndex].fileName);
      triggerRef(messages);
    }
    scrollToBottom();
  }));

  // 监听文件传输错误
  cleanupFns.push(await api.lanOnFileTransferError((data: any) => {
    const msgIndex = messages.value.findIndex((m) => m.id === data.id);
    if (msgIndex !== -1) {
      messages.value[msgIndex].status = data.status || 'error';
      messages.value[msgIndex].isImage = messages.value[msgIndex].isImage ?? isImageFile(messages.value[msgIndex].fileName);
      triggerRef(messages);
    }
  }));

  // 监听文件接收
  cleanupFns.push(await api.lanOnFileReceived((data: any) => {
    if (data.fromUserId !== props.peer?.id) return;
    const exists = messages.value.some(m => m.id === data.id);
    if (!exists) {
      const newMsg = {
        id: data.id,
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

  // 监听任务分配
  cleanupFns.push(await api.lanOnTaskAssigned((data: any) => {
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
});

onUnmounted(() => {
  cleanupFns.forEach(fn => fn());
  selectedFiles.value.clear();
});
</script>

<style scoped>
.chat-panel {
  position: fixed;
  bottom: 20px;
  right: 20px;
  width: 720px;
  height: 650px;
  background: var(--color-base-100);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  z-index: 1000;
  transition: all 0.3s ease;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  overflow: hidden;
}

.chat-panel.maximized {
  width: 90vw !important;
  height: 85vh !important;
  bottom: 5vh !important;
  right: 5vw !important;
  border-radius: 16px;
}

/* 拖拽遮罩 */
.drag-overlay {
  position: absolute;
  inset: 0;
  background: rgba(102, 126, 234, 0.15);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  border: 3px dashed rgba(102, 126, 234, 0.6);
  border-radius: 12px;
}

.drag-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--accent-color, #667eea);
  font-size: 18px;
  font-weight: 600;
}

.chat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.header-left h3 {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.peer-version {
  font-size: 11px;
  opacity: 0.7;
  flex-shrink: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.icon-btn {
  background: none;
  border: none;
  color: white;
  font-size: 14px;
  cursor: pointer;
  opacity: 0.8;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.icon-btn:hover {
  opacity: 1;
  background: rgba(255,255,255,0.15);
}

.close-btn {
  background: none;
  border: none;
  color: white;
  font-size: 16px;
  cursor: pointer;
  opacity: 0.8;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  opacity: 1;
  background: rgba(255,255,255,0.15);
}

.chat-messages {
  flex: 1;
  padding: 12px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: var(--color-base-200);
}

/* 微信式时间分隔 */
.time-separator {
  text-align: center;
  padding: 8px 0;
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.7;
}

.loading-more {
  text-align: center;
  padding: 8px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
}

.loading-more .spinner {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid color-mix(in oklab, var(--color-base-content) 60%, transparent);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-right: 6px;
  vertical-align: middle;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.no-more {
  text-align: center;
  padding: 4px 8px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 11px;
  opacity: 0.6;
}

/* 滚动条 */
.chat-messages::-webkit-scrollbar {
  width: 6px;
}
.chat-messages::-webkit-scrollbar-track {
  background: transparent;
}
.chat-messages::-webkit-scrollbar-thumb {
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 3px;
}
</style>
