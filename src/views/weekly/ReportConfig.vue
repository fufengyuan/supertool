<template>
  <div class="flex gap-4 items-end flex-wrap">
    <div class="form-field">
      <label>{{ $t('weekly.timeRange') }}</label>
      <select
        v-model="selectedRange"
        @change="$emit('range-change', selectedRange)"
        class="select select-bordered w-full max-w-xs"
      >
        <option value="thisWeek">{{ $t('weekly.thisWeek') }}</option>
        <option value="lastWeek">{{ $t('weekly.lastWeek') }}</option>
        <option value="custom">{{ $t('weekly.custom') }}</option>
      </select>
    </div>

    <div v-if="selectedRange === 'custom'" class="flex items-end gap-2">
      <input v-model="startDate" type="date" class="input input-bordered" @change="onDateChange" />
      <span class="text-base-content/60 pb-[10px] text-sm">{{ $t('report.to') }}</span>
      <input v-model="endDate" type="date" class="input input-bordered" @change="onDateChange" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

const props = defineProps({
  initialRange: { type: String, default: 'thisWeek' },
  initialStartDate: { type: String, default: '' },
  initialEndDate: { type: String, default: '' },
});

const emit = defineEmits(['range-change', 'date-change']);

const selectedRange = ref(props.initialRange);
const startDate = ref(props.initialStartDate);
const endDate = ref(props.initialEndDate);

watch(
  () => props.initialRange,
  (v) => {
    selectedRange.value = v;
  }
);
watch(
  () => props.initialStartDate,
  (v) => {
    startDate.value = v;
  }
);
watch(
  () => props.initialEndDate,
  (v) => {
    endDate.value = v;
  }
);

const onDateChange = () => {
  emit('date-change', { start: startDate.value, end: endDate.value });
};

defineExpose({ selectedRange, startDate, endDate });
</script>
