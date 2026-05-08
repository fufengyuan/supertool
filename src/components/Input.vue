<template>
  <div class="input-wrapper" :class="{ 'input-has-label': label }">
    <label v-if="label" class="input-label">
      {{ label }}
      <span v-if="required" class="required">*</span>
    </label>
    <input
      v-if="type !== 'textarea' && type !== 'select'"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      class="input-field"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @focus="$emit('focus', $event)"
      @blur="$emit('blur', $event)"
    />
    <textarea
      v-else-if="type === 'textarea'"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :rows="rows"
      class="input-field input-textarea"
      @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
      @focus="$emit('focus', $event)"
      @blur="$emit('blur', $event)"
    ></textarea>
    <select
      v-else-if="type === 'select'"
      :value="modelValue"
      :disabled="disabled"
      class="input-field input-select"
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <slot />
    </select>
    <p v-if="hint" class="input-hint">{{ hint }}</p>
  </div>
</template>

<script setup lang="ts">
defineProps({
  modelValue: {
    type: [String, Number],
    default: '',
  },
  type: {
    type: String,
    default: 'text',
    validator: (v: string) =>
      ['text', 'textarea', 'select', 'email', 'password', 'number', 'color'].includes(v),
  },
  label: {
    type: String,
    default: '',
  },
  placeholder: {
    type: String,
    default: '',
  },
  hint: {
    type: String,
    default: '',
  },
  required: {
    type: Boolean,
    default: false,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  rows: {
    type: [String, Number],
    default: 3,
  },
});

defineEmits(['update:modelValue', 'focus', 'blur']);
</script>

<style scoped>
.input-wrapper {
  display: flex;
  flex-direction: column;
}

.input-label {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
  color: oklch(var(--bc));
  font-size: 13px;
  font-weight: 500;
}

.required {
  color: oklch(var(--er));
}

.input-field {
  width: 100%;
  padding: 10px 14px;
  border: 1.5px solid oklch(var(--bc) / 0.2);
  border-radius: 10px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  font-size: 14px;
  font-family: inherit;
  transition: all 0.15s ease;
  outline: none;
}

.input-field:focus {
  border-color: oklch(var(--p));
  box-shadow: 0 0 0 3px oklch(var(--p) / 0.1);
}

.input-field::placeholder {
  color: oklch(var(--bc) / 0.6);
  opacity: 0.7;
}

.input-field:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.input-textarea {
  resize: vertical;
  min-height: 60px;
}

.input-select {
  cursor: pointer;
}

.input-hint {
  margin-top: 4px;
  color: oklch(var(--bc) / 0.6);
  font-size: 12px;
}
</style>
