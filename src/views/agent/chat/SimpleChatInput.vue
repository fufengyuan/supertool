<template>
  <div class="relative">
    <textarea
      ref="textareaRef"
      v-model="localValue"
      :placeholder="placeholder"
      :disabled="isLoading && !isApprovalMode"
      rows="1"
      class="w-full resize-none rounded-xl border border-base-content/15 bg-base-200/30 px-3 py-2.5 pr-20 text-sm text-base-content placeholder:text-base-content/30 focus:outline-none focus:border-primary/40 focus:ring-1 focus:ring-primary/20 disabled:opacity-50 transition-colors"
      style="min-height: 42px; max-height: 200px"
      @input="autoResize"
      @keydown="handleKeydown"
    />

    <!-- Action buttons (right-aligned inside the textarea box) -->
    <div class="absolute right-2 bottom-2 flex items-center gap-1.5">
      <!-- Approve / Deny (approval mode) -->
      <template v-if="isApprovalMode">
        <button
          class="flex items-center gap-1 px-2 py-1 rounded-lg text-[11px] font-medium bg-success/15 text-success border border-success/25 hover:bg-success/25 transition-colors"
          title="Approve (/approve)"
          @click="$emit('approve')"
        >
          <SvgIcon name="check" size="10" />
          Approve
        </button>
        <button
          class="flex items-center gap-1 px-2 py-1 rounded-lg text-[11px] font-medium bg-error/15 text-error border border-error/25 hover:bg-error/25 transition-colors"
          title="Deny (/deny)"
          @click="$emit('deny')"
        >
          <SvgIcon name="x" size="10" />
          Deny
        </button>
      </template>

      <!-- Send / Abort (normal mode) -->
      <template v-else>
        <button
          v-if="isLoading"
          class="flex items-center gap-1 px-2 py-1 rounded-lg text-[11px] font-medium bg-warning/15 text-warning border border-warning/25 hover:bg-warning/25 transition-colors"
          title="Stop generating (Escape)"
          @click="$emit('abort')"
        >
          <SvgIcon name="playerStop" size="10" />
          停止
        </button>
        <button
          v-else
          :disabled="!localValue.trim()"
          class="flex items-center gap-1 px-2 py-1 rounded-lg text-[11px] font-medium bg-primary/15 text-primary border border-primary/25 hover:bg-primary/25 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          title="发送 (⌘ Enter)"
          @click="handleSend"
        >
          <SvgIcon name="send" size="10" />
        </button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';

const props = defineProps<{
  modelValue: string;
  isLoading: boolean;
  isApprovalMode: boolean;
  placeholder?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  send: [];
  abort: [];
  approve: [];
  deny: [];
}>();

const textareaRef = ref<HTMLTextAreaElement | null>(null);
const localValue = ref(props.modelValue);

watch(() => props.modelValue, (v) => {
  localValue.value = v;
  nextTick(autoResize);
});

watch(localValue, (v) => {
  emit('update:modelValue', v);
});

function autoResize() {
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = 'auto';
  el.style.height = Math.min(el.scrollHeight, 200) + 'px';
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
    e.preventDefault();
    handleSend();
    return;
  }
  if (e.key === 'Escape' && props.isLoading) {
    e.preventDefault();
    emit('abort');
  }
}

function handleSend() {
  if (!localValue.value.trim() || props.isLoading) return;
  emit('send');
}

defineExpose({
  clear() {
    localValue.value = '';
    nextTick(autoResize);
  },
  focus() {
    textareaRef.value?.focus();
  },
});
</script>
