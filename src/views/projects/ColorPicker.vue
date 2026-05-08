<template>
  <div>
    <label class="flex items-center gap-1.5 mb-2 text-xs font-medium text-base-content">项目颜色</label>
    <div class="flex items-center gap-2.5 rounded-xl border border-base-content/20 bg-base-200 px-3 py-1.5">
      <input
        :value="modelValue"
        type="color"
        class="h-7 w-7 cursor-pointer rounded-md border-none bg-transparent p-0 [&::-webkit-color-swatch-wrapper]:p-0 [&::-webkit-color-swatch]:rounded-md [&::-webkit-color-swatch]:border-2 [&::-webkit-color-swatch]:border-base-content/10"
        @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      />
      <span class="font-mono text-xs text-base-content/60">{{ modelValue }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineModel({ type: String, default: '#6366f1' });
</script>
