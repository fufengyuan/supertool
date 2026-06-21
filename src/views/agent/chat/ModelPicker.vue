<template>
  <div ref="pickerRef" class="relative">
    <button
      class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs border border-base-content/10 bg-base-200/30 hover:bg-base-200/60 transition-colors cursor-pointer"
      @click="toggle"
    >
      <span class="text-base-content/70 max-w-[140px] truncate">{{ displayModel }}</span>
      <SvgIcon
        :name="isOpen ? 'chevronUp' : 'chevronDown'"
        size="10"
        class="text-base-content/40 shrink-0"
      />
    </button>

    <div
      v-if="isOpen"
      class="absolute bottom-full left-0 mb-1 z-50 w-[320px] max-h-[340px] overflow-y-auto rounded-xl border border-base-content/10 bg-base-100 shadow-xl"
    >
      <!-- Model groups -->
      <div v-for="group in modelGroups" :key="group.provider" class="p-1.5">
        <div class="px-2 py-1 text-[10px] font-semibold text-base-content/40 uppercase tracking-wider">
          {{ group.providerLabel }}
        </div>
        <button
          v-for="m in group.models"
          :key="`${m.provider}:${m.model}`"
          class="w-full flex flex-col gap-0.5 px-2 py-1.5 rounded-lg text-left transition-colors"
          :class="
            currentModel === m.model && currentProvider === m.provider
              ? 'bg-primary/15 text-primary'
              : 'text-base-content/70 hover:bg-base-200/60'
          "
          @click="select(m.provider, m.model, m.baseUrl)"
        >
          <span class="text-xs font-medium">{{ m.label }}</span>
          <span class="text-[10px] opacity-50 font-mono truncate">{{ m.model }}</span>
        </button>
      </div>

      <!-- Custom model input -->
      <div class="p-1.5 border-t border-base-content/10">
        <div class="px-2 py-1 text-[10px] font-semibold text-base-content/40 uppercase tracking-wider">
          自定义模型
        </div>
        <div class="flex items-center gap-1.5 px-2">
          <input
            v-model="customInput"
            type="text"
            class="flex-1 px-2 py-1.5 text-xs bg-base-200/40 border border-base-content/10 rounded-lg focus:outline-none focus:border-primary/40"
            placeholder="输入模型名称，按 Enter 选择"
            @keydown.enter="submitCustom"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import type { ModelGroup } from './types';

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

function toggle() {
  if (!isOpen.value) {
    emit('open');
  }
  isOpen.value = !isOpen.value;
}

function select(provider: string, model: string, baseUrl: string) {
  emit('selectModel', provider, model, baseUrl);
  isOpen.value = false;
  customInput.value = '';
}

function submitCustom() {
  const model = customInput.value.trim();
  if (!model) return;
  select(
    props.currentProvider === 'auto' ? 'auto' : props.currentProvider,
    model,
    props.currentBaseUrl,
  );
}

function handleClickOutside(e: MouseEvent) {
  if (pickerRef.value && !pickerRef.value.contains(e.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => document.addEventListener('mousedown', handleClickOutside));
onUnmounted(() => document.removeEventListener('mousedown', handleClickOutside));
</script>
