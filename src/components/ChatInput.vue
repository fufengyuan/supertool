<template>
  <div class="chat-input">
    <div class="input-actions">
      <!-- 表情按钮 -->
      <div class="emoji-wrapper" ref="emojiWrapperRef">
        <button class="icon-btn" @click="toggleEmojiPicker" title="表情">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <path d="M8 14s1.5 2 4 2 4-2 4-2"/>
            <line x1="9" y1="9" x2="9.01" y2="9"/>
            <line x1="15" y1="9" x2="15.01" y2="9"/>
          </svg>
        </button>
        <!-- 表情面板 -->
        <Transition name="emoji-fade">
          <div v-if="showEmojiPicker" class="emoji-picker">
            <div class="emoji-grid">
              <button
                v-for="emoji in emojiList"
                :key="emoji"
                class="emoji-btn"
                @click="selectEmoji(emoji)"
              >
                {{ emoji }}
              </button>
            </div>
          </div>
        </Transition>
      </div>
      <!-- 截图按钮 -->
      <button class="icon-btn" @click="$emit('screenshot')" title="截图">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2"/>
          <circle cx="12" cy="13" r="4"/>
        </svg>
      </button>
      <!-- 任务分配按钮 -->
      <button class="icon-btn" @click="$emit('assign-task')" title="分配任务">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
          <polyline points="22 4 12 14.01 9 11.01"/>
        </svg>
      </button>
      <!-- 文件按钮 -->
      <button class="icon-btn" @click="$emit('file-select-click')" title="发送文件（最大 500MB）">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/>
        </svg>
      </button>
    </div>
    <textarea
      v-model="inputText"
      @keydown="handleKeydown"
      @input="autoResize"
      :placeholder="placeholder"
      class="message-input"
      rows="1"
      ref="inputRef"
    />
    <button @click="$emit('send')" class="send-btn" :class="{ disabled: !inputText.trim() }">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 2L11 13"/><path d="M22 2l-7 20-4-9-9-4 20-7z"/>
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  placeholder: { type: String, default: '输入消息...' },
});

const emit = defineEmits<{
  send: [];
  'file-select-click': [];
  'emoji-select': [emoji: string];
  screenshot: [];
  'assign-task': [];
}>();

const inputText = defineModel({ type: String, default: '' });
const inputRef = ref<HTMLTextAreaElement | null>(null);
const emojiWrapperRef = ref<HTMLElement | null>(null);
const showEmojiPicker = ref(false);

// 常用表情列表
const emojiList = [
  '😀','😃','😄','😁','😆','😅','🤣','😂','🙂','🙃',
  '😉','😊','😇','🥰','😍','🤩','😘','😗','😚','😙',
  '😋','😛','😜','🤪','😝','🤑','🤗','🤭','🤫','🤔',
  '🤐','🤨','😐','😑','😶','😏','😒','🙄','😬','😌',
  '😔','😪','🤤','😴','😷','🤒','🤕','🤢','🤮','🥵',
  '🥶','🥴','😵','🤯','🤠','🥳','🥸','😎','🤓','🧐',
  '👍','👎','👊','✊','🤛','🤜','👏','🙌','👐','🤲',
  '🤝','🙏','✌️','🤞','🤟','🤘','👌','🤌','🤏','👈',
  '👉','👆','👇','☝️','✋','🤚','🖐️','🖖','👋','🤙',
  '❤️','🧡','💛','💚','💙','💜','🖤','🤍','🤎','💔',
  '🔥','⭐','🌟','💫','✨','🎉','🎊','🎈','🎁','🏆',
];

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    emit('send');
  }
};

// 自动调整 textarea 高度
const autoResize = () => {
  const el = inputRef.value;
  if (!el) return;
  el.style.height = 'auto';
  el.style.height = Math.min(el.scrollHeight, 120) + 'px';
};

// 表情面板
const toggleEmojiPicker = () => {
  showEmojiPicker.value = !showEmojiPicker.value;
};

const selectEmoji = (emoji: string) => {
  emit('emoji-select', emoji);
  showEmojiPicker.value = false;
  // 重新聚焦输入框并重置高度
  inputRef.value?.focus();
  // 下一帧重置高度（因为内容变化了）
  setTimeout(() => autoResize(), 0);
};

// 点击外部关闭表情面板
const handleClickOutside = (e: MouseEvent) => {
  if (showEmojiPicker.value && emojiWrapperRef.value && !emojiWrapperRef.value.contains(e.target as Node)) {
    showEmojiPicker.value = false;
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});
</script>

<style scoped>
.chat-input {
  display: flex;
  padding: 10px 12px;
  gap: 8px;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  background: oklch(var(--b2));
  align-items: flex-end;
}

.input-actions {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  flex-shrink: 0;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 8px;
  background: transparent;
  border: none;
  color: oklch(var(--bc) / 0.6);
  cursor: pointer;
  transition: all 0.15s;
  padding: 0;
}
.icon-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: oklch(var(--bc));
}

.emoji-wrapper {
  position: relative;
}

/* 表情面板 */
.emoji-picker {
  position: absolute;
  bottom: 44px;
  left: 0;
  width: 280px;
  max-height: 200px;
  background: oklch(var(--b1));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  padding: 8px;
  z-index: 100;
  overflow-y: auto;
}

.emoji-grid {
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 2px;
}

.emoji-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.1s;
}
.emoji-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  transform: scale(1.2);
}

.emoji-fade-enter-active,
.emoji-fade-leave-active {
  transition: opacity 0.15s, transform 0.15s;
}
.emoji-fade-enter-from,
.emoji-fade-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

.message-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 12px;
  font-size: 14px;
  background: rgba(255, 255, 255, 0.04);
  color: oklch(var(--bc));
  resize: none;
  max-height: 120px;
  min-height: 34px;
  line-height: 1.5;
  outline: none;
  font-family: inherit;
  transition: border-color 0.2s;
}
.message-input:focus {
  border-color: oklch(var(--p));
}
.message-input::placeholder {
  color: oklch(var(--bc) / 0.6);
  opacity: 0.5;
}

.send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 8px;
  border: none;
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  cursor: pointer;
  transition: all 0.15s;
  flex-shrink: 0;
}
.send-btn:hover:not(.disabled) {
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}
.send-btn.disabled {
  opacity: 0.3;
  cursor: default;
}
</style>
