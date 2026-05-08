<template>
  <div class="detail-section comment-section">
    <h4>{{ $t('todo.comments') }}</h4>
    <div class="comment-input-area">
      <textarea
        :value="input"
        @input="$emit('update:input', ($event.target as HTMLInputElement).value)"
        class="comment-input"
        :placeholder="$t('todo.addComment')"
        @keydown.ctrl.enter="$emit('add')"
      ></textarea>
      <button @click="$emit('add')" class="add-comment-btn">{{ $t('todo.publish') }}</button>
    </div>
    <div class="comments-list">
      <div v-for="comment in comments" :key="comment.id" class="comment-item">
        <div class="comment-header">
          <strong>{{ comment.author }}</strong>
          <span class="comment-time">{{ formatTime(comment.timestamp) }}</span>
        </div>
        <div class="comment-content">{{ comment.content }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { useI18n } from 'vue-i18n';

const { locale: i18nLocale } = useI18n();

defineProps({
  comments: { type: Array as () => any[], default: () => [] },
  input: { type: String, default: '' },
});
defineEmits(['update:input', 'add']);

const formatTime = (timestamp) => {
  if (!timestamp) return '';
  return new Date(timestamp).toLocaleString(i18nLocale.value, {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
  });
};
</script>

<style scoped>
.comment-section { margin-top: 8px; }
.comment-input-area { display: flex; flex-direction: column; gap: 6px; margin-bottom: 8px; }
.comment-input {
  width: 100%; padding: 6px; border: 1px solid oklch(var(--bc) / 0.2);
  border-radius: 6px; background: oklch(var(--b2)); color: oklch(var(--bc));
  font-size: 13px; resize: vertical; outline: none; min-height: 40px;
}
.comment-input:focus { border-color: oklch(var(--p)); }
.add-comment-btn {
  align-self: flex-end; padding: 4px 12px; background: oklch(var(--p));
  color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;
}
.comments-list { display: flex; flex-direction: column; gap: 6px; }
.comment-item { padding: 6px 8px; background: oklch(var(--b2)); border-radius: 6px; }
.comment-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2px; }
.comment-header strong { font-size: 12px; color: oklch(var(--bc)); }
.comment-time { font-size: 11px; color: oklch(var(--bc) / 0.6); }
.comment-content { font-size: 13px; color: oklch(var(--bc)); line-height: 1.4; }
</style>
