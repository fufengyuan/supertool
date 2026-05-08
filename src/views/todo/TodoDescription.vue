<template>
  <div class="mt-2">
    <h4 class="text-xs font-semibold uppercase text-base-content/60 tracking-wider mb-1.5">详细内容</h4>

    <!-- 编辑模式 -->
    <div v-if="isEditing" class="flex flex-col gap-2">
      <textarea
        :value="content"
        @input="$emit('update:content', ($event.target as HTMLInputElement).value)"
        class="textarea textarea-bordered w-full text-sm font-mono"
        placeholder="输入Markdown格式的内容..."
        rows="6"
      ></textarea>
      <div class="flex gap-2">
        <button @click="$emit('save')" class="btn btn-primary btn-sm">保存</button>
        <button @click="$emit('cancel')" class="btn btn-ghost btn-sm">取消</button>
      </div>
    </div>

    <!-- 显示模式 -->
    <div
      v-else-if="markdown"
      class="p-2 bg-base-200 rounded-lg cursor-pointer hover:bg-base-content/10"
      @dblclick="$emit('start-edit')"
    >
      <div v-html="renderedHtml"></div>
    </div>

    <!-- 占位符 -->
    <div v-else class="p-2 border border-dashed border-base-content/10 rounded-lg text-base-content/60 cursor-pointer text-center hover:border-primary hover:text-primary" @click="$emit('start-edit')">
      <p>点击添加Markdown详情...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import { useErrorHandler } from '../../composables/useErrorHandler';

const { handleError } = useErrorHandler();

const props = defineProps({
  markdown: { type: String, default: '' },
  isEditing: { type: Boolean, default: false },
  content: { type: String, default: '' },
});

defineEmits(['update:content', 'save', 'cancel', 'start-edit']);

const renderedHtml = computed(() => {
  if (!props.markdown) return '';
  try {
    return DOMPurify.sanitize(String(marked(props.markdown)));
  } catch (error) {
    handleError(error, { context: 'Markdown渲染', showToast: false });
    return DOMPurify.sanitize(props.markdown);
  }
});
</script>
