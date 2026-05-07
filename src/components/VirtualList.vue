<template>
  <div class="virtual-list" ref="containerRef" :style="{ height: containerHeight }" @scroll="onScroll">
    <div :style="{ height: `${totalHeight}px`, position: 'relative' }">
      <div :style="{ transform: `translateY(${offsetY}px)` }">
        <slot
          v-for="item in visibleItems"
          :key="item.key"
          :item="item.data"
          :index="item.index"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">// @ts-nocheck
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'

const props = defineProps({
  items: { type: Array as () => any[], required: true },
  itemHeight: { type: Number, default: 60 },
  height: { type: [Number, String], default: 500 }
})

const containerRef = ref(null)
const scrollTop = ref(0)

const containerHeight = computed(() =>
  typeof props.height === 'number' ? `${props.height}px` : props.height
)

const totalHeight = computed(() => props.items.length * props.itemHeight)

const visibleCount = computed(() => {
  const h = typeof props.height === 'number' ? props.height : parseFloat(props.height) || 500
  return Math.ceil(h / props.itemHeight) + 2
})

const startIndex = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / props.itemHeight) - 1)
)

const endIndex = computed(() =>
  Math.min(props.items.length, startIndex.value + visibleCount.value + 1)
)

const offsetY = computed(() => startIndex.value * props.itemHeight)

const visibleItems = computed(() => {
  const items = []
  for (let i = startIndex.value; i < endIndex.value; i++) {
    items.push({ key: props.items[i].id || i, data: props.items[i], index: i })
  }
  return items
})

function onScroll() {
  if (containerRef.value) {
    scrollTop.value = containerRef.value.scrollTop
  }
}

function scrollTo(index) {
  if (containerRef.value) {
    containerRef.value.scrollTop = index * props.itemHeight
  }
}

// Expose scrollTo for parent components
defineExpose({ scrollTo })

function onResize() {
  // Force recalculation by touching scrollTop (triggers reactive re-render of visibleItems)
  if (containerRef.value) {
    scrollTop.value = containerRef.value.scrollTop
  }
}

onMounted(() => {
  window.addEventListener('resize', onResize)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', onResize)
})
</script>

<style scoped>
.virtual-list {
  overflow-y: auto;
  overflow-x: hidden;
  will-change: transform;
}
</style>
