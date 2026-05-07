<template>
  <div
    :class="[
      'message',
      {
        sent: message.fromUserId === myUserId,
        received: message.fromUserId !== myUserId,
        'file-message': message.type === 'file',
      },
    ]"
  >
    <!-- 接收方头像 -->
    <div v-if="message.fromUserId !== myUserId" class="msg-avatar">
      {{ avatar }}
    </div>

    <div class="message-bubble">
      <!-- 文件消息 -->
      <div v-if="message.type === 'file'" class="file-content">
        <!-- 图片文件预览 -->
        <template v-if="isImageFile">
          <div class="image-preview-container">
            <div
              class="image-preview"
              :class="{ clickable: message.filePath && message.status === 'completed' }"
              @click="handleFileNameClick"
            >
              <img
                v-if="imageUrl && !imageLoadFailed"
                :src="imageUrl"
                :alt="message.fileName"
                class="image-thumb"
                @error="imageLoadFailed = true"
              />
              <!-- 加载失败时显示占位符 -->
              <div v-if="imageLoadFailed" class="image-placeholder">
                <span class="placeholder-name">{{ message.fileName }}</span>
              </div>
              <!-- 加载中占位符 -->
              <div v-if="!imageUrl && !imageLoadFailed" class="image-placeholder">
                <span class="placeholder-icon">🖼️</span>
                <span class="placeholder-name">{{ message.fileName }}</span>
              </div>
              <!-- 传输中遮罩 -->
              <div v-if="message.status === 'sending' || message.status === 'receiving'" class="image-overlay transferring">
                <span class="progress-spinner"></span>
                <span class="overlay-text">{{ message.status === 'sending' ? '发送' : '接收' }}中 {{ message.progress }}%</span>
              </div>
              <!-- 错误遮罩 -->
              <div v-else-if="message.status === 'error'" class="image-overlay error-overlay">
                <span class="error-text">{{ message.fromUserId === myUserId ? '发送失败' : '接收失败' }}</span>
                <button @click.stop="$emit('retry', message)" class="retry-btn-inline" title="重试">
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16"/>
                  </svg>
                </button>
              </div>
              <!-- 已取消遮罩 -->
              <div v-else-if="message.status === 'cancelled'" class="image-overlay cancelled-overlay">
                <span class="cancelled-text">已取消</span>
              </div>
            </div>
            <!-- 状态文字：放在图片下方 -->
            <div v-if="message.status === 'sending'" class="image-status">发送中 {{ message.progress }}%</div>
            <div v-else-if="message.status === 'receiving'" class="image-status">接收中 {{ message.progress }}%</div>
            <div v-else-if="message.status === 'error'" class="image-status error">传输失败</div>
            <div v-else-if="message.status === 'cancelled'" class="image-status">已取消</div>
            <div v-else-if="message.status === 'completed'" class="image-status success">{{ message.fromUserId === myUserId ? '✓ 已发送' : '✓ 已接收' }}</div>
            <!-- 操作按钮：放在图片下方水平排列 -->
            <div class="image-actions-row">
              <button
                v-if="message.toUserId === myUserId && message.status === 'completed'"
                @click="$emit('open-folder', message)"
                class="action-btn open-btn"
                title="打开文件夹"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
              </button>
              <button
                v-if="message.status === 'error' && message.fromUserId === myUserId"
                @click.stop="$emit('retry', message)"
                class="action-btn retry-btn"
                title="重试"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16"/>
                </svg>
              </button>
            </div>
          </div>
        </template>
        <!-- 非图片文件：保持原有文件图标布局 -->
        <template v-else>
          <div class="file-icon">
            {{ fileIcon }}
          </div>
          <div class="file-info">
            <div
              class="file-name"
              :class="{ clickable: message.filePath && message.status === 'completed' }"
              :title="message.filePath ? '点击查看文件' : message.fileName"
              @click="handleFileNameClick"
            >{{ message.fileName }}</div>
            <div class="file-size">{{ formatFileSize(message.fileSize) }}</div>
            <!-- 文件状态 -->
            <div v-if="message.status === 'sending'" class="file-status">
              <span class="progress-spinner"></span> 发送中 {{ message.progress }}%
            </div>
            <div v-else-if="message.status === 'receiving'" class="file-status">
              <span class="progress-spinner"></span> 接收中 {{ message.progress }}%
            </div>
            <div v-else-if="message.status === 'error'" class="file-status error">传输失败</div>
            <div v-else-if="message.status === 'cancelled'" class="file-status">已取消</div>
            <!-- 接收完成：显示保存路径 -->
            <div v-else-if="message.status === 'completed' && message.fromUserId !== myUserId" class="file-status success">
              ✓ 已接收
            </div>
            <div v-else-if="message.status === 'completed'" class="file-status success">
              ✓ 已发送
            </div>
            <!-- 接收方完成时显示文件路径 -->
            <div v-if="message.status === 'completed' && message.filePath" class="file-path">
              <span class="path-text" :title="message.filePath">{{ message.filePath }}</span>
            </div>
          </div>
          <!-- 操作按钮 -->
          <div class="file-actions">
            <button
              v-if="message.toUserId === myUserId && message.status === 'completed'"
              @click="$emit('open-folder', message)"
              class="action-btn open-btn"
              title="打开文件夹"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
            </button>
            <button
              v-else-if="message.status === 'error' && message.fromUserId === myUserId"
              @click="$emit('retry', message)"
              class="action-btn retry-btn"
              title="重试"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16"/>
              </svg>
            </button>
            <div v-else-if="message.status === 'sending' || message.status === 'receiving'" class="action-btn progress-indicator">
              {{ message.progress }}%
            </div>
          </div>
        </template>
      </div>

      <!-- 任务分配消息 -->
      <div v-else-if="message.type === 'task_assigned'" class="task-assigned-content">
        <div class="task-assigned-icon">📋</div>
        <div class="task-assigned-info">
          <div class="task-assigned-text">已分配任务：<strong>{{ parsedTaskContent.taskText }}</strong></div>
          <div v-if="parsedTaskContent.priority" class="task-assigned-meta">
            优先级：{{ priorityLabel }}
          </div>
          <div v-if="parsedTaskContent.dueDate" class="task-assigned-meta">
            截止日期：{{ parsedTaskContent.dueDate }}
          </div>
          <div v-if="parsedTaskContent.note" class="task-assigned-note">
            备注：{{ parsedTaskContent.note }}
          </div>
        </div>
      </div>

      <!-- 文本消息 -->
      <div v-else class="message-text">{{ message.content }}</div>
    </div>

    <!-- 发送方头像 -->
    <div v-if="message.fromUserId === myUserId" class="msg-avatar sender-avatar">
      {{ avatar }}
    </div>
  </div>
  <!-- 图片灯箱（全屏预览） -->
  <div
    v-if="showImageLightbox"
    class="lightbox-overlay"
    @click="closeLightbox"
    @keydown.escape="closeLightbox"
    tabindex="0"
  >
    <div class="lightbox-toolbar">
      <span class="lightbox-filename">{{ message.fileName }}</span>
      <span class="lightbox-size">{{ formatFileSize(message.fileSize) }}</span>
      <div class="lightbox-toolbar-actions">
        <button class="lightbox-btn" @click.stop="openFileInSystem" title="系统打开">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
            <polyline points="15 3 21 3 21 9"/>
            <line x1="10" y1="14" x2="21" y2="3"/>
          </svg>
        </button>
        <button class="lightbox-btn" @click.stop="$emit('open-folder', message)" title="打开文件夹">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
        </button>
        <button class="lightbox-btn close-btn" @click.stop="closeLightbox" title="关闭 (Esc)">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    </div>
    <div class="lightbox-content" @click.stop="stopPropagation">
      <img v-if="imageUrl" :src="imageUrl" :alt="message.fileName" class="lightbox-image" decoding="async" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { getTauriAPI } from '@/utils/tauri-api';

const props = defineProps<{
  message: Record<string, any>;
  myUserId: string;
  formatDate: (ts: string | number) => string;
  formatFileSize: (bytes: number) => string;
  getFileStatus: (msg: Record<string, any>) => string;
  avatar?: string;
}>();

defineEmits<{
  download: [message: Record<string, any>];
  'open-folder': [message: Record<string, any>];
  retry: [message: Record<string, any>];
}>();

const showImageLightbox = ref(false);
const imageLoadFailed = ref(false);

// Tauri 环境下通过 base64 加载图片
const imageUrl = ref<string>('');

onMounted(async () => {
  if (isImageFile.value && props.message.filePath && props.message.status === 'completed') {
    try {
      const api = getTauriAPI();
      if (api.loadLocalFileAsBase64) {
        const base64 = await api.loadLocalFileAsBase64(props.message.filePath);
        if (base64) {
          // 自动检测 MIME 类型
          const ext = props.message.fileName.split('.').pop()?.toLowerCase() || '';
          const mimeMap: Record<string, string> = {
            jpg: 'image/jpeg', jpeg: 'image/jpeg', png: 'image/png',
            gif: 'image/gif', webp: 'image/webp', bmp: 'image/bmp', svg: 'image/svg+xml',
          };
          const mime = mimeMap[ext] || 'image/png';
          imageUrl.value = `data:${mime};base64,${base64}`;
        }
      }
    } catch (e) {
      console.warn('[ChatMessage] Failed to load image:', e);
    }
  }
});

const handleFileNameClick = () => {
  if (!props.message.filePath || props.message.status !== 'completed') return;
  if (isImageFile.value) {
    showImageLightbox.value = true;
  } else {
    try {
      getTauriAPI().openFile(props.message.filePath);
    } catch {
      alert(`文件路径: ${props.message.filePath}`);
    }
  }
};

const closeLightbox = () => {
  showImageLightbox.value = false;
};

const stopPropagation = (e: Event) => {
  e.stopPropagation();
};

const openFileInSystem = () => {
  if (props.message.filePath) {
    getTauriAPI().openFile(props.message.filePath);
  }
};

const isImageFile = computed(() => {
  const fileName = props.message.fileName || '';
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(ext);
});

const fileIcon = computed(() => {
  const fileName = props.message.fileName || '';
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  const iconMap: Record<string, string> = {
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', svg: '🖼️', webp: '🖼️', bmp: '🖼️',
    pdf: '📄', doc: '📝', docx: '📝', xls: '📊', xlsx: '📊', ppt: '📑', pptx: '📑',
    js: '📜', ts: '📜', py: '📜', java: '📜', go: '📜', rs: '📜', cpp: '📜', c: '📜', h: '📜',
    vue: '📜', html: '📜', css: '📜', json: '📜', yaml: '📜', xml: '📜', md: '📜',
    zip: '📦', tar: '📦', gz: '📦', rar: '📦', '7z': '📦',
    mp3: '🎵', wav: '🎵', flac: '🎵', mp4: '🎬', avi: '🎬', mkv: '🎬',
    exe: '⚙️', dmg: '⚙️', deb: '⚙️', rpm: '⚙️', sh: '⚙️', bat: '⚙️',
    jar: '☕',
  };
  return iconMap[ext] || '📁';
});

const parsedTaskContent = computed(() => {
  if (props.message.type === 'task_assigned') {
    try {
      return typeof props.message.content === 'string'
        ? JSON.parse(props.message.content)
        : (props.message.content || {});
    } catch {
      return {};
    }
  }
  return {};
});

const priorityLabel = computed(() => {
  const map: Record<string, string> = {
    high: '🔴 高',
    medium: '🟡 中',
    low: '🟢 低',
  };
  return map[parsedTaskContent.value.priority] || parsedTaskContent.value.priority || '';
});
</script>

<style scoped>
.message {
  display: flex;
  gap: 8px;
  max-width: 85%;
  word-wrap: break-word;
  align-items: flex-start;
}

.message.sent {
  align-self: flex-end;
  flex-direction: row-reverse;
}

.message.received {
  align-self: flex-start;
}

/* 头像 */
.msg-avatar {
  width: 34px;
  height: 34px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 17px;
  flex-shrink: 0;
  margin-top: 2px;
  user-select: none;
}

.sender-avatar {
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.3), rgba(118, 75, 162, 0.3));
}

/* 气泡 */
.message-bubble {
  padding: 10px 14px 8px;
  border-radius: 16px;
  min-width: 60px;
  max-width: 100%;
  position: relative;
  word-break: break-word;
}

.message.sent .message-bubble {
  background: linear-gradient(135deg, #667eea, #764ba2);
  border-bottom-right-radius: 4px;
  color: rgba(255, 255, 255, 0.95);
}

.message.received .message-bubble {
  background: linear-gradient(135deg, #065f46, #047857);
  border-bottom-left-radius: 4px;
  color: rgba(255, 255, 255, 0.92);
}

.message-text {
  font-size: 14px;
  line-height: 1.55;
  white-space: pre-wrap;
  word-break: break-word;
  user-select: text;
}

/* ========== 文件消息 ========== */
.file-content {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 2px 0;
}

.file-icon {
  font-size: 30px;
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-weight: 600;
  font-size: 13px;
  margin-bottom: 3px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
  cursor: default;
}
.file-name.clickable {
  cursor: pointer;
  text-decoration: underline;
  text-decoration-color: rgba(255, 255, 255, 0.3);
  text-underline-offset: 2px;
}
.file-name.clickable:hover {
  text-decoration-color: rgba(255, 255, 255, 0.7);
}

.file-size {
  font-size: 11px;
  opacity: 0.6;
  margin-bottom: 2px;
}

.file-status {
  font-size: 11px;
  opacity: 0.7;
  display: flex;
  align-items: center;
  gap: 4px;
}

.file-status.error {
  color: #fca5a5;
  opacity: 1;
}

.file-status.success {
  color: #86efac;
  opacity: 1;
}

.progress-spinner {
  display: inline-block;
  width: 10px;
  height: 10px;
  border: 1.5px solid rgba(255, 255, 255, 0.3);
  border-top-color: rgba(255, 255, 255, 0.8);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 文件路径 */
.file-path {
  margin-top: 4px;
  padding: 3px 6px;
  background: rgba(0, 0, 0, 0.15);
  border-radius: 6px;
  max-width: 260px;
}

.path-text {
  font-size: 10px;
  opacity: 0.7;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

.file-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.15s;
}
.action-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 1);
}
.action-btn.open-btn:hover {
  background: rgba(96, 165, 250, 0.3);
}
.action-btn.retry-btn:hover {
  background: rgba(252, 165, 165, 0.3);
}
.progress-indicator {
  font-size: 11px;
  font-weight: 600;
  cursor: default;
}

/* ========== 图片预览 ========== */
.image-preview-container {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.image-preview {
  position: relative;
  border-radius: 8px;
  overflow: hidden;
  cursor: default;
}
.image-preview.clickable {
  cursor: pointer;
}

.image-thumb {
  max-width: 240px;
  max-height: 200px;
  display: block;
  object-fit: contain;
  border-radius: 8px;
}

.image-placeholder {
  width: 200px;
  height: 150px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
}
.placeholder-icon {
  font-size: 32px;
  opacity: 0.5;
}
.placeholder-name {
  font-size: 11px;
  opacity: 0.6;
  text-align: center;
  padding: 0 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 180px;
}

.image-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-radius: 8px;
  backdrop-filter: blur(4px);
}

.image-overlay.transferring {
  background: rgba(0, 0, 0, 0.5);
}

.image-overlay.error-overlay {
  background: rgba(220, 38, 38, 0.5);
}

.image-overlay.cancelled-overlay {
  background: rgba(100, 116, 139, 0.5);
}

.overlay-text, .error-text, .cancelled-text {
  font-size: 12px;
  font-weight: 600;
  color: white;
}

.retry-btn-inline {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.3);
  background: rgba(255, 255, 255, 0.1);
  color: white;
  cursor: pointer;
  font-size: 11px;
  transition: all 0.15s;
}
.retry-btn-inline:hover {
  background: rgba(255, 255, 255, 0.2);
}

.image-status {
  font-size: 11px;
  opacity: 0.7;
  text-align: center;
}
.image-status.error {
  color: #fca5a5;
  opacity: 1;
}
.image-status.success {
  color: #86efac;
  opacity: 1;
}

.image-actions-row {
  display: flex;
  gap: 6px;
  justify-content: center;
}

/* ========== 任务分配 ========== */
.task-assigned-content {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.task-assigned-icon {
  font-size: 22px;
  flex-shrink: 0;
}

.task-assigned-info {
  flex: 1;
  min-width: 0;
}

.task-assigned-text {
  font-size: 13px;
  margin-bottom: 4px;
}

.task-assigned-meta {
  font-size: 11px;
  opacity: 0.7;
}

.task-assigned-note {
  font-size: 11px;
  opacity: 0.6;
  margin-top: 2px;
}

/* ========== 图片灯箱 ========== */
.lightbox-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  flex-direction: column;
  z-index: 10000;
  cursor: pointer;
  outline: none;
}

.lightbox-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: rgba(30, 30, 46, 0.9);
  backdrop-filter: blur(10px);
  flex-shrink: 0;
}

.lightbox-filename {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 50%;
}

.lightbox-size {
  font-size: 12px;
  color: #94a3b8;
}

.lightbox-toolbar-actions {
  display: flex;
  gap: 8px;
}

.lightbox-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  color: #e2e8f0;
  cursor: pointer;
  transition: all 0.15s;
}
.lightbox-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}
.lightbox-btn.close-btn:hover {
  background: rgba(239, 68, 68, 0.3);
}

.lightbox-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  cursor: default;
}

.lightbox-image {
  max-width: 90vw;
  max-height: 85vh;
  object-fit: contain;
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}
</style>
