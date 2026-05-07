<template>
  <div class="color-picker">
    <label class="form-label">项目颜色</label>
    <div class="color-picker-row">
      <input :value="modelValue" type="color" class="color-input" @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)" />
      <span class="color-hex">{{ modelValue }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineModel({ type: String, default: '#6366f1' });
</script>

<style scoped>
.form-label {
  display: flex; align-items: center; gap: 6px; margin-bottom: 8px;
  color: var(--main-text); font-size: 13px; font-weight: 500;
}
.color-picker-row {
  display: flex; align-items: center; gap: 10px; padding: 6px 12px;
  background: var(--input-bg); border-radius: 10px;
  border: 1.5px solid var(--input-border);
}
.color-input {
  width: 28px; height: 28px; border: none; border-radius: 6px;
  cursor: pointer; background: transparent; padding: 0;
}
.color-input::-webkit-color-swatch-wrapper { padding: 0; }
.color-input::-webkit-color-swatch { border: 2px solid var(--border-color); border-radius: 6px; }
.color-hex {
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 13px; color: var(--main-text-secondary);
}
</style>
