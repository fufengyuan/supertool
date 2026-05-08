<template>
  <div class="flex items-center gap-1 shrink-0 ms-1">
    <input
      type="checkbox"
      :checked="isSelected"
      @change="$emit('toggle-selected')"
      class="checkbox checkbox-sm"
      @click.stop
    />
    <button @click.stop="$emit('delete')" class="btn btn-error btn-xs">{{ $t('todo.delete') }}</button>
  </div>
</template>

<script setup lang="ts">
defineProps({
  isSelected: { type: Boolean, default: false },
});
defineEmits(['toggle-selected', 'delete']);
</script>
