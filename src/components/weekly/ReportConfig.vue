<template>
  <div class="report-config">
    <div class="form-field">
      <label>{{ $t('weekly.timeRange') }}</label>
      <select
        v-model="selectedRange"
        @change="$emit('range-change', selectedRange)"
        class="form-select"
      >
        <option value="thisWeek">{{ $t('weekly.thisWeek') }}</option>
        <option value="lastWeek">{{ $t('weekly.lastWeek') }}</option>
        <option value="custom">{{ $t('weekly.custom') }}</option>
      </select>
    </div>

    <div v-if="selectedRange === 'custom'" class="custom-date-range">
      <input v-model="startDate" type="date" class="form-input" @change="onDateChange" />
      <span class="date-separator">{{ $t('report.to') }}</span>
      <input v-model="endDate" type="date" class="form-input" @change="onDateChange" />
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

<style scoped>
.report-config {
  display: flex;
  gap: 16px;
  align-items: flex-end;
  flex-wrap: wrap;
}

.custom-date-range {
  display: flex;
  align-items: flex-end;
  gap: 8px;
}

.date-separator {
  color: oklch(var(--bc) / 0.6);
  padding-bottom: 10px;
  font-size: 13px;
}
</style>
