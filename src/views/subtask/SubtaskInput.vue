<template>
  <div class="flex gap-2 mb-3">
    <input
      ref="inputRef"
      v-model="text"
      type="text"
      placeholder="输入子任务..."
      class="input input-bordered flex-1 text-sm"
      @keyup.enter="confirm"
      @blur="cancel"
    />
    <button @click="confirm" class="btn btn-success">✓</button>
    <button @click="cancel" class="btn btn-error"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, onMounted } from 'vue';

const emit = defineEmits(['add', 'cancel']);
const text = ref('');
const inputRef = ref(null);

onMounted(() => inputRef.value?.focus());

const confirm = () => {
  if (text.value.trim()) {
    emit('add', text.value.trim());
    text.value = '';
  }
};

const cancel = () => {
  text.value = '';
  emit('cancel');
};
</script>
