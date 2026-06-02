<template>
  <div ref="pickerRef" class="relative">
    <button class="flex items-center gap-1 px-2 py-1 rounded text-xs text-base-content/60 hover:text-base-content hover:bg-base-200/60 transition-colors" @click="toggle">
      <span class="max-w-[120px] truncate">{{ displayModel }}</span>
      <SvgIcon name="chevronDown" size="10" />
    </button>

    <div
      v-if="isOpen"
      class="absolute bottom-full left-0 mb-1 w-72 max-h-80 overflow-y-auto bg-base-100 border border-base-content/15 rounded-lg shadow-lg z-50"
    >
      <!-- Model groups -->
      <div v-for="group in modelGroups" :key="group.provider" class="py-1">
        <div class="px-3 py-1 text-[10px] uppercase tracking-wider text-base-content/40 font-medium">
          {{ group.providerLabel }}
        </div>
        <button
          v-for="m in group.models"
          :key="`${m.provider}:${m.model}`"
          :class="[
            'flex flex-col w-full px-3 py-1.5 text-left hover:bg-base-200/60 transition-colors',
            currentModel === m.model && currentProvider === m.provider ? 'bg-primary/10' : '',
          ]"
          @click="select(m.provider, m.model, m.baseUrl)"
        >
          <span class="text-xs text-base-content">{{ m.label }}</span>
          <span class="text-[10px] text-base-content/40 truncate">{{ m.model }}</span>
        </button>
      </div>

      <!-- Custom model input -->
      <div class="border-t border-base-content/10 p-2">
        <div class="text-[10px] uppercase tracking-wider text-base-content/40 font-medium mb-1">
          Custom
        </div>
        <input
          v-model="customInput"
          type="text"
          class="w-full px-2 py-1 text-xs rounded border border-base-content/15 bg-base-200/30 focus:outline-none focus:border-primary/40"
          placeholder="Type model name..."
          @keydown.enter="submitCustom"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import type { ModelGroup } from '../types';

const props = defineProps<{
  currentModel: string;
  currentProvider: string;
  currentBaseUrl: string;
  modelGroups: ModelGroup[];
  displayModel: string;
}>();

const emit = defineEmits<{
  open: [];
  selectModel: [provider: string, model: string, baseUrl: string];
}>();

const isOpen = ref(false);
const customInput = ref('');
const pickerRef = ref<HTMLDivElement | null>(null);

function handleClickOutside(e: MouseEvent): void {
  if (pickerRef.value && !pickerRef.value.contains(e.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});

function toggle(): void {
  if (!isOpen.value) emit('open');
  isOpen.value = !isOpen.value;
}

function select(provider: string, model: string, baseUrl: string): void {
  emit('selectModel', provider, model, baseUrl);
  isOpen.value = false;
  customInput.value = '';
}

function submitCustom(): void {
  const model = customInput.value.trim();
  if (!model) return;
  select(
    props.currentProvider === 'auto' ? 'auto' : props.currentProvider,
    model,
    props.currentBaseUrl,
  );
}
</script>
