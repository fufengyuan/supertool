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
  width: 100%; padding: 6px; border: 1px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 6px; background: var(--color-base-200); color: var(--color-base-content);
  font-size: 13px; resize: vertical; outline: none; min-height: 40px;
}
.comment-input:focus { border-color: var(--color-primary); }
.add-comment-btn {
  align-self: flex-end; padding: 4px 12px; background: var(--color-primary);
  color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;
}
.comments-list { display: flex; flex-direction: column; gap: 6px; }
.comment-item { padding: 6px 8px; background: var(--color-base-200); border-radius: 6px; }
.comment-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2px; }
.comment-header strong { font-size: 12px; color: var(--color-base-content); }
.comment-time { font-size: 11px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }
.comment-content { font-size: 13px; color: var(--color-base-content); line-height: 1.4; }
</style>
