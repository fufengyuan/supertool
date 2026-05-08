<template>
  <div class="flex flex-col items-center justify-center py-10 px-5 gap-3 text-center text-base-content/60">
    <svg
      v-if="!$slots.icon"
      viewBox="0 0 24 24"
      width="48"
      height="48"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      class="opacity-30"
    >
      <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
    </svg>
    <slot name="icon" />
    <p class="text-sm m-0">{{ text }}</p>
    <p v-if="subtext" class="text-xs opacity-70 m-0">{{ subtext }}</p>
    <slot name="action" />
  </div>
</template>

<script setup lang="ts">
defineProps({
  text: { type: String, default: '暂无内容' },
  subtext: { type: String, default: '' },
})
</script>
