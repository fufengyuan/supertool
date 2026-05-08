<template>
  <div class="flex gap-2 items-end">
    <button @click="$emit('generate')" :disabled="generating" class="btn btn-primary">
      {{ generating ? $t('report.generating') : $t('report.generate') }}
    </button>
    <button @click="$emit('export-markdown')" :disabled="!reportData" class="btn btn-ghost">
      {{ $t('report.markdown') }}
    </button>
    <button @click="$emit('export-word')" :disabled="!reportData" class="btn btn-ghost">
      {{ $t('report.word') }}
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps({
  generating: { type: Boolean, default: false },
  reportData: { type: Object, default: null },
});

defineEmits(['generate', 'export-markdown', 'export-word']);
</script>
