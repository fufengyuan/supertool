<template>
  <div class="flex items-center gap-2 p-1.5 px-2 rounded-lg bg-base-200 transition-all duration-200 hover:bg-success/10" :class="{ 'opacity-70': subtask.completed }">
    <div class="cursor-pointer" @click="$emit('toggle', subtask)">
      <input
        type="checkbox"
        :checked="subtask.completed"
        class="checkbox checkbox-success checkbox-sm"
        @click.stop="$emit('toggle', subtask)"
      />
    </div>
    <div
      v-if="!isEditing"
      class="flex-1 text-sm cursor-pointer select-none"
      :class="{ 'line-through text-base-content/60': subtask.completed }"
      @dblclick="$emit('edit', subtask)"
      @keydown.enter="$emit('edit', subtask)"
      tabindex="0"
    >{{ subtask.text }}</div>
    <input
      v-else
      ref="editInputRef"
      v-model="editText"
      class="input input-bordered input-sm flex-1 text-sm"
      @keydown.enter="$emit('save', subtask)"
      @keydown.escape="$emit('cancel')"
      @blur="$emit('save', subtask)"
    />
    <div class="flex gap-1">
      <button @click.stop="$emit('delete', subtask)" class="btn btn-circle btn-error btn-xs text-xs">×</button>
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
