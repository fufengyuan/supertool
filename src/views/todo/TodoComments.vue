<template>
  <div class="mt-2">
    <h4 class="text-xs font-semibold uppercase text-base-content/60 tracking-wider mb-1.5">{{ $t('todo.comments') }}</h4>
    <div class="flex flex-col gap-1.5 mb-2">
      <textarea
        :value="input"
        @input="$emit('update:input', ($event.target as HTMLInputElement).value)"
        class="textarea textarea-bordered w-full text-sm min-h-[40px]"
        :placeholder="$t('todo.addComment')"
        @keydown.ctrl.enter="$emit('add')"
      ></textarea>
      <button @click="$emit('add')" class="btn btn-primary btn-sm self-end">{{ $t('todo.publish') }}</button>
    </div>
    <div class="flex flex-col gap-1.5">
      <div v-for="comment in comments" :key="comment.id" class="p-1.5 px-2 bg-base-200 rounded-lg">
        <div class="flex justify-between items-center mb-0.5">
          <strong class="text-xs text-base-content">{{ comment.author }}</strong>
          <span class="text-[11px] text-base-content/60">{{ formatTime(comment.timestamp) }}</span>
        </div>
        <div class="text-sm text-base-content leading-normal">{{ comment.content }}</div>
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
  if (!timestamp) {return '';}
  return new Date(timestamp).toLocaleString(i18nLocale.value, {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
  });
};
</script>
