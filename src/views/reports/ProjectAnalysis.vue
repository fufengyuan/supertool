<template>
  <div class="mb-8 p-5 bg-gradient-to-br from-success/10 to-base-200 rounded-xl border-2 border-base-content/10">
    <h3 class="text-lg font-semibold text-primary mb-4">{{ title || $t('comparison.lastWeekTitle') }}</h3>
    <div class="grid grid-cols-3 gap-4 mb-5">
      <div class="bg-base-100 p-4 rounded-xl text-center border-2 border-base-content/10">
        <div class="text-[13px] text-base-content/40 mb-2">{{ currentLabel || $t('comparison.thisWeek') }}</div>
        <div class="text-2xl font-bold text-base-content">{{ currentStats.total }}</div>
      </div>
      <div class="bg-base-100 p-4 rounded-xl text-center border-2 border-base-content/10">
        <div class="text-[13px] text-base-content/40 mb-2">{{ previousLabel || $t('comparison.lastWeek') }}</div>
        <div class="text-2xl font-bold text-base-content">{{ previousStats.total }}</div>
      </div>
      <div class="bg-primary p-4 rounded-xl text-center border-2 border-primary text-white">
        <div class="text-[13px] text-white/80 mb-2">{{ $t('comparison.change') }}</div>
        <div class="text-2xl font-bold" :class="{ 'text-success': changeValue > 0, 'text-error': changeValue < 0, 'text-base-content/40': changeValue === 0 }">
          {{ changeText }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps({
  currentStats: { type: Object, required: true },
  previousStats: { type: Object, required: true },
  title: { type: String, default: '与上周对比' },
  currentLabel: { type: String, default: '本周完成' },
  previousLabel: { type: String, default: '上周完成' },
});

const changeValue = computed(() => props.currentStats.total - props.previousStats.total);

const changeText = computed(() => {
  const change = changeValue.value;
  if (change > 0) {return `+${change}`;}
  if (change < 0) {return `${change}`;}
  return '0';
});
</script>
