<template>
  <div class="subtask-add-form">
    <input
      ref="inputRef"
      v-model="text"
      type="text"
      placeholder="输入子任务..."
      class="subtask-input"
      @keyup.enter="confirm"
      @blur="cancel"
    />
    <button @click="confirm" class="btn-confirm">✓</button>
    <button @click="cancel" class="btn-cancel">✕</button>
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

<style scoped>
.subtask-add-form { display: flex; gap: 8px; margin-bottom: 12px; }
.subtask-input {
  flex: 1; padding: 8px 12px; border: 1px solid var(--border-color);
  border-radius: 6px; background-color: var(--input-bg); color: var(--main-text);
  font-size: 14px;
}
.subtask-input:focus { outline: none; border-color: var(--primary-color); }
.btn-confirm {
  padding: 8px 12px; background-color: #22c55e; color: white;
  border: none; border-radius: 6px; cursor: pointer; font-weight: bold;
}
.btn-cancel {
  padding: 8px 12px; background-color: #ef4444; color: white;
  border: none; border-radius: 6px; cursor: pointer; font-weight: bold;
}
</style>
