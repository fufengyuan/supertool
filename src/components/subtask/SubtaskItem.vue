<template>
  <div class="subtask-item" :class="{ completed: subtask.completed }">
    <div class="subtask-checkbox" @click="$emit('toggle', subtask)">
      <div class="checkbox" :class="{ checked: subtask.completed }">
        <span v-if="subtask.completed">✓</span>
      </div>
    </div>
    <div
      v-if="!isEditing"
      class="subtask-text"
      @dblclick="$emit('edit', subtask)"
      @keydown.enter="$emit('edit', subtask)"
      tabindex="0"
    >{{ subtask.text }}</div>
    <input
      v-else
      ref="editInputRef"
      v-model="editText"
      class="subtask-edit-input"
      @keydown.enter="$emit('save', subtask)"
      @keydown.escape="$emit('cancel')"
      @blur="$emit('save', subtask)"
    />
    <div class="subtask-actions">
      <button @click.stop="$emit('delete', subtask)" class="delete-btn">×</button>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, nextTick, watch } from 'vue';

const props = defineProps({ subtask: { type: Object, required: true }, isEditing: { type: Boolean, default: false } });
defineEmits(['toggle', 'edit', 'delete', 'save', 'cancel']);

const editText = ref('');
const editInputRef = ref(null);

watch(() => props.isEditing, async (val) => {
  if (val) {
    editText.value = props.subtask.text;
    await nextTick();
    editInputRef.value?.focus();
  }
});
</script>

<style scoped>
.subtask-item {
  display: flex; align-items: center; gap: 8px; padding: 6px 8px;
  border-radius: 6px; background-color: var(--input-bg); transition: all 0.2s ease;
}
.subtask-item:hover { background-color: var(--completed-bg); }
.subtask-item.completed { opacity: 0.7; }
.subtask-item.completed .subtask-text { text-decoration: line-through; color: var(--main-text-secondary); }
.subtask-checkbox { cursor: pointer; }
.checkbox {
  width: 18px; height: 18px; border: 2px solid var(--border-color);
  border-radius: 4px; display: flex; align-items: center; justify-content: center;
  transition: all 0.2s ease;
}
.checkbox.checked { background-color: #22c55e; border-color: #22c55e; color: white; }
.subtask-text { flex: 1; font-size: 14px; cursor: pointer; }
.subtask-actions { display: flex; gap: 4px; }
.delete-btn {
  width: 24px; height: 24px; border: none; border-radius: 50%;
  background-color: #ef4444; color: white; cursor: pointer; font-size: 12px;
  display: flex; align-items: center; justify-content: center;
}
.delete-btn:hover { opacity: 0.8; }
.subtask-edit-input {
  flex: 1; font-size: 14px; padding: 2px 6px; border: 1px solid var(--primary-color);
  border-radius: 4px; background: var(--card-bg); color: var(--main-text); outline: none;
  font-family: inherit;
}
</style>
