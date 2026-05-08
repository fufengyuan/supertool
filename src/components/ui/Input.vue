<template>
  <fieldset class="fieldset">
    <legend v-if="label" class="fieldset-label text-sm font-medium">
      {{ label }}
      <span v-if="required" class="text-error">*</span>
    </legend>
    <input
      v-if="type !== 'textarea' && type !== 'select'"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      class="input w-full"
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
      class="textarea w-full"
      @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
      @focus="$emit('focus', $event)"
      @blur="$emit('blur', $event)"
    ></textarea>
    <select
      v-else-if="type === 'select'"
      :value="modelValue"
      :disabled="disabled"
      class="select w-full"
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <slot />
    </select>
    <p v-if="hint" class="fieldset-label text-xs opacity-60 mt-1">{{ hint }}</p>
  </fieldset>
</template>

<script setup lang="ts">
defineProps({
  modelValue: { type: [String, Number], default: '' },
  type: {
    type: String,
    default: 'text',
    validator: (v: string) => ['text', 'textarea', 'select', 'email', 'password', 'number', 'color'].includes(v),
  },
  label: { type: String, default: '' },
  placeholder: { type: String, default: '' },
  hint: { type: String, default: '' },
  required: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
  rows: { type: [String, Number], default: 3 },
})

defineEmits(['update:modelValue', 'focus', 'blur'])
</script>
