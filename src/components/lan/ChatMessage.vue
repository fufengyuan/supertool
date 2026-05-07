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
                :src="imageUrl"
                :alt="message.fileName"
                class="image-thumb"
                @error="imageLoadFailed = true"
              />
              <!-- 加载失败时显示占位符 -->
              <div v-if="imageLoadFailed" class="image-placeholder">
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
      <img :src="imageUrl || ''" :alt="message.fileName" class="lightbox-image" decoding="async" />
    </div>
  </div>
</template>

<script setup lang="ts">
console.log("[components/lan/ChatMessage.vue] component loaded")
import { computed, ref } from 'vue';
import { getTauriAPI } from '../../utils/tauri-api'

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

// 根据文件扩展名选择图标
const showImageLightbox = ref(false);

const handleFileNameClick = () => {
  if (!props.message.filePath || props.message.status !== 'completed') return;
  if (isImageFile.value) {
    // 图片：打开灯箱查看
    showImageLightbox.value = true;
  } else {
    // 非图片文件：用系统默认应用打开
    try {
      getTauriAPI().openFile?.(props.message.filePath);
    } catch {
      alert(`文件路径: ${props.message.filePath}`);
    }
  }
};

const closeLightbox = () => {
  showImageLightbox.value = false;
};

// 阻止事件冒泡：点击图片时不关闭灯箱
const stopPropagation = (e: Event) => {
  e.stopPropagation();
};

// 灯箱工具栏：系统打开文件
const openFileInSystem = () => {
  if (props.message.filePath) {
    getTauriAPI().openFile?.(props.message.filePath);
  }
};

// Track whether the image failed to load, so we can show the placeholder instead
const imageLoadFailed = ref(false);

// 图片 URL：直接用 file:// 协议加载本地文件（Tauri 渲染进程原生支持）
const imageUrl = computed(() => {
  if (!props.message.filePath) return '';
  return `file://${props.message.filePath.replace(/\\/g, '/')}`;
});

const isImageFile = computed(() => {
  const fileName = props.message.fileName || '';
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(ext);
});

// 将本地文件路径转换为 Tauri 可加载的 URL
// 修复 macOS/Windows 路径格式差异 + 特殊字符编码
function localFileUrl(filePath: string): string {
  // Windows 路径: C:\Users\... → file:///C:/Users/...
  // macOS/Linux 路径: /Users/... → file:///Users/...
  const normalized = filePath.replace(/\\/g, '/');
  return `file://${encodeURIComponent(normalized).replace(/%2F/g, '/')}`;
}

const fileIcon = computed(() => {
  const fileName = props.message.fileName || '';
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  const iconMap: Record<string, string> = {
    // 图片
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', svg: '🖼️', webp: '🖼️', bmp: '🖼️',
    // 文档
    pdf: '📄', doc: '📝', docx: '📝', xls: '📊', xlsx: '📊', ppt: '📑', pptx: '📑',
    // 代码
    js: '📜', ts: '📜', py: '📜', java: '📜', go: '📜', rs: '📜', cpp: '📜', c: '📜', h: '📜',
    vue: '📜', html: '📜', css: '📜', json: '📜', yaml: '📜', xml: '📜', md: '📜',
    // 压缩包
    zip: '📦', tar: '📦', gz: '📦', rar: '📦', '7z': '📦',
    // 音频/视频
    mp3: '🎵', wav: '🎵', flac: '🎵', mp4: '🎬', avi: '🎬', mkv: '🎬',
    // 可执行文件
    exe: '⚙️', dmg: '⚙️', deb: '⚙️', rpm: '⚙️', sh: '⚙️', bat: '⚙️',
    // 默认
    jar: '☕',
  };
  return iconMap[ext] || '📁';
});

// 解析任务分配消息的 content
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

// 优先级标签
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
  max-width: 100%;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

/* 文件操作按钮 */
.file-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
  font-size: 11px;
  font-weight: 500;
  padding: 0;
}

.open-btn {
  background: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.9);
}
.open-btn:hover {
  background: rgba(255, 255, 255, 0.25);
  transform: scale(1.1);
}

.retry-btn {
  background: rgba(245, 158, 11, 0.3);
  color: rgba(255, 255, 255, 0.9);
}
.retry-btn:hover {
  background: rgba(245, 158, 11, 0.5);
  transform: scale(1.1);
}

.progress-indicator {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.5);
  cursor: default;
}

/* ========== 图片预览 ========== */
.image-preview-container {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  width: 100%;
}

.image-preview {
  position: relative;
  display: inline-block;
  border-radius: 8px;
  overflow: hidden;
  cursor: default;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.image-preview.clickable {
  cursor: pointer;
}

.image-preview.clickable:hover {
  transform: scale(1.02);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 点击放大提示图标 */
.image-preview.clickable::after {
  content: '🔍';
  position: absolute;
  top: 8px;
  right: 8px;
  font-size: 16px;
  opacity: 0;
  transition: opacity 0.15s ease;
  pointer-events: none;
  filter: drop-shadow(0 1px 3px rgba(0,0,0,0.5));
}

.image-preview.clickable:hover::after {
  opacity: 1;
}

.image-thumb {
  display: block;
  max-width: 200px;
  max-height: 200px;
  width: auto;
  height: auto;
  object-fit: cover;
  border-radius: 8px;
}

.image-placeholder {
  display: none;
  align-items: center;
  justify-content: center;
  min-width: 120px;
  min-height: 80px;
  max-width: 200px;
  max-height: 200px;
  border-radius: 8px;
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.15), rgba(255, 255, 255, 0.05));
  padding: 12px;
  text-align: center;
}

.placeholder-name {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.7);
  word-break: break-all;
  line-height: 1.3;
}

/* 图片下方状态文字 */
.image-status {
  font-size: 10px;
  opacity: 0.6;
  color: rgba(255, 255, 255, 0.7);
}
.image-status.error { color: #fca5a5; opacity: 1; }
.image-status.success { color: #86efac; opacity: 1; }

/* 图片下方操作按钮行 */
.image-actions-row {
  display: flex;
  gap: 4px;
  align-self: flex-end;
}

/* 图片遮罩层 */
.image-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: 8px;
}

.image-overlay.transferring {
  background: rgba(0, 0, 0, 0.55);
}

.image-overlay .overlay-text {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.9);
  font-weight: 500;
}

.image-overlay.error-overlay {
  background: rgba(180, 30, 30, 0.7);
}

.image-overlay.cancelled-overlay {
  background: rgba(100, 100, 100, 0.7);
}

.image-overlay .error-text {
  font-size: 12px;
  color: #fca5a5;
  font-weight: 600;
}

.image-overlay .cancelled-text {
  font-size: 12px;
  color: #d1d5db;
  font-weight: 600;
}

.retry-btn-inline {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
  cursor: pointer;
  transition: all 0.15s;
}

.retry-btn-inline:hover {
  background: rgba(255, 255, 255, 0.35);
  transform: scale(1.1);
}

/* ========== 图片灯箱 ========== */
.lightbox-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: #000;
  display: flex;
  flex-direction: column;
  cursor: zoom-out;
  outline: none;
}

.lightbox-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  background: rgba(0, 0, 0, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  flex-shrink: 0;
}

.lightbox-filename {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.95);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.lightbox-size {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  flex-shrink: 0;
}

.lightbox-toolbar-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.lightbox-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.85);
  cursor: pointer;
  transition: all 0.15s ease;
}

.lightbox-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
  transform: scale(1.08);
}

.lightbox-btn.close-btn:hover {
  background: rgba(220, 38, 38, 0.5);
}

.lightbox-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  cursor: default;
  overflow: hidden;
}

.lightbox-image {
  max-width: 95vw;
  max-height: 90vh;
  object-fit: contain;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
  user-select: none;
  -webkit-user-drag: none;
}

/* 灯箱过渡动画 — 仅透明度，避免 transform 缩放大图片导致卡顿 */
.lightbox-enter-active,
.lightbox-leave-active {
  transition: opacity 0.15s ease;
}

.lightbox-enter-from,
.lightbox-leave-to {
  opacity: 0;
}

/* ========== 任务分配消息 ========== */
.task-assigned-content {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 4px 0;
}

.task-assigned-icon {
  font-size: 24px;
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
}

.task-assigned-info {
  flex: 1;
  min-width: 0;
}

.task-assigned-text {
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 4px;
  line-height: 1.4;
}

.task-assigned-text strong {
  font-weight: 600;
}

.task-assigned-meta {
  font-size: 11px;
  opacity: 0.7;
  margin-bottom: 2px;
}

.task-assigned-note {
  font-size: 11px;
  opacity: 0.6;
  margin-top: 4px;
  padding: 4px 6px;
  background: rgba(0, 0, 0, 0.15);
  border-radius: 4px;
  line-height: 1.3;
}
</style>
