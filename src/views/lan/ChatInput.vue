<template>
  <div class="flex p-[10px_12px] gap-2 border-t border-base-content/10 bg-base-200 items-end">
    <div class="flex items-end gap-1 shrink-0">
      <!-- 表情按钮 -->
      <div class="relative" ref="emojiWrapperRef">
        <button class="inline-flex items-center justify-center w-[34px] h-[34px] rounded-lg bg-transparent border-none text-base-content/60 cursor-pointer transition-all duration-150 p-0 hover:bg-white/8 hover:text-base-content" @click="toggleEmojiPicker" title="表情">
          <SvgIcon name="smile" size="18" />
        </button>
        <!-- 表情面板 -->
        <Transition name="emoji-fade">
          <div v-if="showEmojiPicker" class="absolute bottom-11 left-0 w-[280px] max-h-[200px] bg-base-100 border border-base-content/10 rounded-xl shadow-lg z-50 p-2 overflow-y-auto">
            <div class="grid grid-cols-8 gap-0.5">
              <button
                v-for="emoji in emojiList"
                :key="emoji"
                class="w-[30px] h-[30px] flex items-center justify-center border-none bg-transparent rounded-md cursor-pointer text-base transition-all duration-100 hover:bg-white/10 hover:scale-110"
                @click="selectEmoji(emoji)"
              >
                {{ emoji }}
              </button>
            </div>
          </div>
        </Transition>
      </div>
      <!-- 截图按钮 -->
      <button class="inline-flex items-center justify-center w-[34px] h-[34px] rounded-lg bg-transparent border-none text-base-content/60 cursor-pointer transition-all duration-150 p-0 hover:bg-white/8 hover:text-base-content" @click="$emit('screenshot')" title="截图">
        <SvgIcon name="camera" size="18" />
      </button>
      <!-- 任务分配按钮 -->
      <button class="inline-flex items-center justify-center w-[34px] h-[34px] rounded-lg bg-transparent border-none text-base-content/60 cursor-pointer transition-all duration-150 p-0 hover:bg-white/8 hover:text-base-content" @click="$emit('assign-task')" title="分配任务">
        <SvgIcon name="checkCircle" size="18" />
      </button>
      <!-- 文件按钮 -->
      <button class="inline-flex items-center justify-center w-[34px] h-[34px] rounded-lg bg-transparent border-none text-base-content/60 cursor-pointer transition-all duration-150 p-0 hover:bg-white/8 hover:text-base-content" @click="$emit('file-select-click')" title="发送文件（最大 500MB）">
        <SvgIcon name="paperclip" size="18" />
      </button>
    </div>
    <textarea
      v-model="inputText"
      @keydown="handleKeydown"
      @input="autoResize"
      :placeholder="placeholder"
      class="flex-1 p-[8px_12px] border border-base-content/10 rounded-xl text-sm bg-white/4 text-base-content resize-none max-h-[120px] min-h-[34px] leading-5 outline-none font-inherit transition-border-color duration-200 focus:border-primary placeholder:text-base-content/60 placeholder:opacity-50"
      rows="1"
      ref="inputRef"
    />
    <button @click="$emit('send')" :class="[inputText.trim() ? '' : 'opacity-30 cursor-default']" class="flex items-center justify-center w-[34px] h-[34px] rounded-lg border-none bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white cursor-pointer transition-all duration-150 shrink-0 hover:scale-105 hover:shadow-[0_4px_12px_rgba(102,126,234,0.4)]">
      <SvgIcon name="send" size="16" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue'

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
  if (!el) {return;}
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

<style>
.emoji-fade-enter-active,
.emoji-fade-leave-active {
  transition: opacity 0.15s, transform 0.15s;
}
.emoji-fade-enter-from,
.emoji-fade-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
</style>
