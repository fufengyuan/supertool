<template>
  <div
    :class="[
      'flex gap-2 max-w-[85%] break-words items-start',
      {
        'self-end flex-row-reverse': message.fromUserId === myUserId,
        'self-start': message.fromUserId !== myUserId,
      },
    ]"
  >
    <!-- 接收方头像 -->
    <div v-if="message.fromUserId !== myUserId" class="w-[34px] h-[34px] rounded-xl bg-white/8 flex items-center justify-center text-[17px] shrink-0 mt-0.5 select-none">
      {{ avatar }}
    </div>

    <div :class="[
      'px-[14px] pt-[10px] pb-2 rounded-2xl min-w-[60px] max-w-full relative break-words',
      message.fromUserId === myUserId
        ? 'bg-gradient-to-br from-[#667eea] to-[#764ba2] rounded-br-[4px] text-white/95'
        : 'bg-gradient-to-br from-emerald-700 to-emerald-600 rounded-bl-[4px] text-white/92'
    ]">
      <!-- 文件消息 -->
      <div v-if="message.type === 'file'" class="flex items-center gap-2.5 py-0.5">
        <!-- 图片文件预览 -->
        <template v-if="isImageFile">
          <div class="flex flex-col items-start gap-1 w-full">
            <div
              class="relative inline-block rounded-lg overflow-hidden cursor-default transition-[transform,box-shadow] duration-150 ease-in-out"
              :class="{ 'cursor-pointer hover:scale-105 hover:shadow-lg': message.filePath && message.status === 'completed' }"
            >
              <img
                v-if="imageUrl && !imageLoadFailed"
                :src="imageUrl"
                :alt="message.fileName"
                class="block max-w-[200px] max-h-[200px] w-auto h-auto object-cover rounded-lg"
                @error="imageLoadFailed = true"
              />
              <!-- 加载失败时显示占位符 -->
              <div v-if="imageLoadFailed" class="flex items-center justify-center min-w-[120px] min-h-[80px] max-w-[200px] max-h-[200px] rounded-lg bg-gradient-to-br from-white/15 to-white/5 p-3 text-center">
                <span class="text-[11px] text-white/70 break-all leading-tight">{{ message.fileName }}</span>
              </div>
              <!-- 加载中占位符 -->
              <div v-if="!imageUrl && !imageLoadFailed" class="flex items-center justify-center min-w-[120px] min-h-[80px] max-w-[200px] max-h-[200px] rounded-lg bg-gradient-to-br from-white/15 to-white/5 p-3 text-center">
                <span class="text-[11px] text-white/70">🖼️</span>
                <span class="text-[11px] text-white/70 break-all leading-tight ml-1">{{ message.fileName }}</span>
              </div>
              <!-- 传输中遮罩 -->
              <div v-if="message.status === 'sending' || message.status === 'receiving'" class="absolute inset-0 flex flex-col items-center justify-center gap-1.5 rounded-lg bg-black/55">
                <span class="inline-block w-[10px] h-[10px] border-[1.5px] border-white/30 border-t-white/80 rounded-full animate-spin"></span>
                <span class="text-[11px] text-white/90 font-medium">{{ message.status === 'sending' ? '发送' : '接收' }}中 {{ message.progress }}%</span>
              </div>
              <!-- 错误遮罩 -->
              <div v-else-if="message.status === 'error'" class="absolute inset-0 flex flex-col items-center justify-center gap-1.5 rounded-lg bg-red-800/70">
                <span class="text-xs text-red-300 font-semibold">{{ message.fromUserId === myUserId ? '发送失败' : '接收失败' }}</span>
                <button @click.stop="$emit('retry', message)" class="inline-flex items-center justify-center w-7 h-7 border-none rounded-full bg-white/20 text-white cursor-pointer transition-all duration-150 hover:bg-white/35 hover:scale-110" title="重试">
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16"/>
                  </svg>
                </button>
              </div>
              <!-- 已取消遮罩 -->
              <div v-else-if="message.status === 'cancelled'" class="absolute inset-0 flex flex-col items-center justify-center gap-1.5 rounded-lg bg-gray-500/70">
                <span class="text-xs text-gray-300 font-semibold">已取消</span>
              </div>
            </div>
            <!-- 状态文字：放在图片下方 -->
            <div v-if="message.status === 'sending'" class="text-[10px] opacity-60 text-white/70">发送中 {{ message.progress }}%</div>
            <div v-else-if="message.status === 'receiving'" class="text-[10px] opacity-60 text-white/70">接收中 {{ message.progress }}%</div>
            <div v-else-if="message.status === 'error'" class="text-[10px] text-red-300 opacity-100">传输失败</div>
            <div v-else-if="message.status === 'cancelled'" class="text-[10px] opacity-60 text-white/70">已取消</div>
            <div v-else-if="message.status === 'completed'" class="text-[10px] text-green-300 opacity-100">{{ message.fromUserId === myUserId ? '✓ 已发送' : '✓ 已接收' }}</div>
            <!-- 操作按钮：放在图片下方水平排列 -->
            <div class="flex gap-1 self-end">
              <button
                v-if="message.toUserId === myUserId && message.status === 'completed'"
                @click="$emit('open-folder', message)"
                class="inline-flex items-center justify-center w-[30px] h-[30px] border-none rounded-lg cursor-pointer transition-all duration-150 text-[11px] font-medium p-0 bg-white/15 text-white/90 hover:bg-white/25 hover:scale-110"
                title="打开文件夹"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
              </button>
              <button
                v-if="message.status === 'error' && message.fromUserId === myUserId"
                @click.stop="$emit('retry', message)"
                class="inline-flex items-center justify-center w-[30px] h-[30px] border-none rounded-lg cursor-pointer transition-all duration-150 text-[11px] font-medium p-0 bg-amber-500/30 text-white/90 hover:bg-amber-500/50 hover:scale-110"
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
          <div class="text-[30px] shrink-0 w-10 h-10 flex items-center justify-center bg-white/10 rounded-xl">
            {{ fileIcon }}
          </div>
          <div class="flex-1 min-w-0">
            <div
              :class="[message.filePath && message.status === 'completed' ? 'cursor-pointer underline underline-offset-2 decoration-white/30 hover:decoration-white/70' : '']"
              class="font-semibold text-[13px] mb-[3px] overflow-hidden text-ellipsis whitespace-nowrap max-w-[200px] cursor-default"
              :title="message.filePath ? '点击查看文件' : message.fileName"
              @click="handleFileNameClick"
            >{{ message.fileName }}</div>
            <div class="text-[11px] opacity-60 mb-0.5">{{ formatFileSize(message.fileSize) }}</div>
            <!-- 文件状态 -->
            <div v-if="message.status === 'sending'" class="text-[11px] opacity-70 flex items-center gap-1">
              <span class="inline-block w-[10px] h-[10px] border-[1.5px] border-white/30 border-t-white/80 rounded-full animate-spin"></span> 发送中 {{ message.progress }}%
            </div>
            <div v-else-if="message.status === 'receiving'" class="text-[11px] opacity-70 flex items-center gap-1">
              <span class="inline-block w-[10px] h-[10px] border-[1.5px] border-white/30 border-t-white/80 rounded-full animate-spin"></span> 接收中 {{ message.progress }}%
            </div>
            <div v-else-if="message.status === 'error'" class="text-[11px] text-red-300 opacity-100">传输失败</div>
            <div v-else-if="message.status === 'cancelled'" class="text-[11px] opacity-70">已取消</div>
            <!-- 接收完成：显示保存路径 -->
            <div v-else-if="message.status === 'completed' && message.fromUserId !== myUserId" class="text-[11px] text-green-300 opacity-100">
              ✓ 已接收
            </div>
            <div v-else-if="message.status === 'completed'" class="text-[11px] text-green-300 opacity-100">
              ✓ 已发送
            </div>
            <!-- 接收方完成时显示文件路径 -->
            <div v-if="message.status === 'completed' && message.filePath" class="mt-1 p-[3px_6px] bg-black/15 rounded-md max-w-[260px]">
              <span class="text-[10px] opacity-70 whitespace-nowrap overflow-hidden text-ellipsis block max-w-full font-mono" :title="message.filePath">{{ message.filePath }}</span>
            </div>
          </div>
          <!-- 操作按钮 -->
          <div class="flex flex-col gap-1 shrink-0">
            <button
              v-if="message.toUserId === myUserId && message.status === 'completed'"
              @click="$emit('open-folder', message)"
              class="inline-flex items-center justify-center w-[30px] h-[30px] border-none rounded-lg cursor-pointer transition-all duration-150 text-[11px] font-medium p-0 bg-white/15 text-white/90 hover:bg-white/25 hover:scale-110"
              title="打开文件夹"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
            </button>
            <button
              v-else-if="message.status === 'error' && message.fromUserId === myUserId"
              @click="$emit('retry', message)"
              class="inline-flex items-center justify-center w-[30px] h-[30px] border-none rounded-lg cursor-pointer transition-all duration-150 text-[11px] font-medium p-0 bg-amber-500/30 text-white/90 hover:bg-amber-500/50 hover:scale-110"
              title="重试"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16"/>
              </svg>
            </button>
            <div v-else-if="message.status === 'sending' || message.status === 'receiving'" class="inline-flex items-center justify-center w-[30px] h-[30px] border-none rounded-lg cursor-default text-[11px] font-medium p-0 bg-white/8 text-white/50">
              {{ message.progress }}%
            </div>
          </div>
        </template>
      </div>

      <!-- 任务分配消息 -->
      <div v-else-if="message.type === 'task_assigned'" class="flex items-start gap-2.5 py-1">
        <div class="text-2xl shrink-0 w-9 h-9 flex items-center justify-center bg-white/10 rounded-lg"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg></div>
        <div class="flex-1 min-w-0">
          <div class="text-[13px] font-medium mb-1 leading-[1.4]">已分配任务：<strong class="font-semibold">{{ parsedTaskContent.taskText }}</strong></div>
          <div v-if="parsedTaskContent.priority" class="text-[11px] opacity-70 mb-0.5">
            优先级：{{ priorityLabel }}
          </div>
          <div v-if="parsedTaskContent.dueDate" class="text-[11px] opacity-70 mb-0.5">
            截止日期：{{ parsedTaskContent.dueDate }}
          </div>
          <div v-if="parsedTaskContent.note" class="text-[11px] opacity-60 mt-1 p-[4px_6px] bg-black/15 rounded leading-tight">
            备注：{{ parsedTaskContent.note }}
          </div>
        </div>
      </div>

      <!-- 文本消息 -->
      <div v-else class="text-sm leading-[1.55] whitespace-pre-wrap break-words select-text">{{ message.content }}</div>
    </div>

    <!-- 发送方头像 -->
    <div v-if="message.fromUserId === myUserId" class="w-[34px] h-[34px] rounded-xl flex items-center justify-center text-[17px] shrink-0 mt-0.5 select-none bg-gradient-to-br from-[#667eea]/30 to-[#764ba2]/30">
      {{ avatar }}
    </div>
  </div>
  <!-- 图片灯箱（全屏预览） -->
  <div
    v-if="showImageLightbox"
    class="fixed inset-0 z-[10000] bg-black flex flex-col cursor-zoom-out outline-none"
    @click="closeLightbox"
    @keydown.escape="closeLightbox"
    tabindex="0"
  >
    <div class="flex items-center gap-3 p-[12px_20px] bg-black/60 border-b border-white/10 shrink-0">
      <span class="text-sm font-semibold text-white/95 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ message.fileName }}</span>
      <span class="text-xs text-white/50 shrink-0">{{ formatFileSize(message.fileSize) }}</span>
      <div class="flex gap-1.5 shrink-0">
        <button class="flex items-center justify-center w-9 h-9 border-none rounded-lg bg-white/10 text-white/85 cursor-pointer transition-all duration-150 hover:bg-white/20 hover:text-white hover:scale-105" @click.stop="openFileInSystem" title="系统打开">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
            <polyline points="15 3 21 3 21 9"/>
            <line x1="10" y1="14" x2="21" y2="3"/>
          </svg>
        </button>
        <button class="flex items-center justify-center w-9 h-9 border-none rounded-lg bg-white/10 text-white/85 cursor-pointer transition-all duration-150 hover:bg-white/20 hover:text-white hover:scale-105" @click.stop="$emit('open-folder', message)" title="打开文件夹">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
        </button>
        <button class="flex items-center justify-center w-9 h-9 border-none rounded-lg bg-white/10 text-white/85 cursor-pointer transition-all duration-150 hover:bg-red-500/50 hover:text-white hover:scale-105" @click.stop="closeLightbox" title="关闭 (Esc)">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    </div>
    <div class="flex-1 flex items-center justify-center p-5 cursor-default overflow-hidden" @click.stop="stopPropagation">
      <img v-if="imageUrl" :src="imageUrl" :alt="message.fileName" class="max-w-[95vw] max-h-[90vh] object-contain shadow-lg select-none" decoding="async" />
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


