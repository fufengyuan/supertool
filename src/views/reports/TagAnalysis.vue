<template>
  <div class="mt-5">
    <h4 class="text-[15px] font-semibold text-base-content mb-3">{{ title || $t('tag.distribution') }}</h4>
    <div class="flex flex-col gap-3">
      <div v-for="tag in Object.keys(stats.byTag)" :key="tag" class="flex items-center gap-3">
        <div class="min-w-[80px] text-[13px] text-base-content">{{ tag }}</div>
        <div class="flex-1 h-6 bg-base-content/10 rounded-full flex items-center p-1 relative">
          <div class="h-full bg-gradient-to-r from-primary to-[#4cc9f0] rounded-full transition-[width] duration-300" :style="{ width: getTagPercentage(tag) + '%' }"></div>
          <span class="absolute right-2 text-xs font-semibold text-base-content">{{ stats.byTag[tag] }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps({
  stats: { type: Object, required: true },
  title: { type: String, default: '标签分布' },
});

const getTagPercentage = (tag: string) => {
  if (props.stats.total === 0) {return 0;}
  return (props.stats.byTag[tag] / props.stats.total) * 100;
};
</script>
