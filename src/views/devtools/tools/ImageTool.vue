<template>
  <ToolPage
    icon="image"
    name="图像处理"
    description="压缩, 转格式, 尺寸调整等"
    noScroll
    @back="$emit('back')"
  >
    <ImageProcessor class="h-full" />
  </ToolPage>
</template>

<script setup lang="ts">
defineOptions({ name: 'ImageTool' })
defineEmits<{ back: [] }>()
import ToolPage from '../components/ToolPage.vue'
import ImageProcessor from '@/views/image/ImageProcessor.vue'
</script>
