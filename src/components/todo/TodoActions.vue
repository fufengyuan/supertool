<template>
  <div class="todo-actions">
    <input
      type="checkbox"
      :checked="isSelected"
      @change="$emit('toggle-selected')"
      class="select-checkbox"
      @click.stop
    />
    <button @click.stop="$emit('delete')" class="delete-btn">{{ $t('todo.delete') }}</button>
  </div>
</template>

<script setup lang="ts">
defineProps({
  isSelected: { type: Boolean, default: false },
});
defineEmits(['toggle-selected', 'delete']);
</script>

<style scoped>
.todo-actions { display: flex; align-items: center; gap: 4px; flex-shrink: 0; margin-left: 4px; }
.select-checkbox { cursor: pointer; width: 14px; height: 14px; }
.delete-btn {
  padding: 3px 8px; border: none; border-radius: 4px; background: oklch(var(--er));
  color: white; cursor: pointer; font-size: 11px; transition: opacity 0.15s;
}
.delete-btn:hover { opacity: 0.8; }
</style>
