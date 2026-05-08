<template>
  <div class="detail-section markdown-section">
    <h4>详细内容</h4>

    <!-- 编辑模式 -->
    <div v-if="isEditing" class="markdown-editor">
      <textarea
        :value="content"
        @input="$emit('update:content', ($event.target as HTMLInputElement).value)"
        class="markdown-textarea"
        placeholder="输入Markdown格式的内容..."
        rows="6"
      ></textarea>
      <div class="markdown-actions">
        <button @click="$emit('save')" class="save-markdown-btn">保存</button>
        <button @click="$emit('cancel')" class="cancel-markdown-btn">取消</button>
      </div>
    </div>

    <!-- 显示模式 -->
    <div
      v-else-if="markdown"
      class="markdown-display"
      @dblclick="$emit('start-edit')"
    >
      <div v-html="renderedHtml"></div>
    </div>

    <!-- 占位符 -->
    <div v-else class="markdown-placeholder" @click="$emit('start-edit')">
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

<style scoped>
.markdown-section { margin-top: 8px; }
.markdown-editor { display: flex; flex-direction: column; gap: 8px; }
.markdown-textarea {
  width: 100%; padding: 8px; border: 1px solid color-mix(in oklab, var(--color-base-content) 20%, transparent);
  border-radius: 6px; background: var(--color-base-200); color: var(--color-base-content);
  font-size: 13px; font-family: monospace; resize: vertical; outline: none;
}
.markdown-textarea:focus { border-color: var(--color-primary); }
.markdown-actions { display: flex; gap: 8px; }
.save-markdown-btn, .cancel-markdown-btn {
  padding: 4px 12px; border-radius: 4px; border: none; cursor: pointer; font-size: 12px;
}
.save-markdown-btn { background: var(--color-primary); color: white; }
.cancel-markdown-btn { background: color-mix(in oklab, var(--color-base-content) 10%, transparent); color: var(--color-base-content); }
.markdown-display {
  padding: 8px; background: var(--color-base-200); border-radius: 6px; cursor: pointer;
}
.markdown-display:hover { background: color-mix(in oklab, var(--color-base-content) 10%, transparent); }
.markdown-placeholder {
  padding: 8px; border: 1px dashed color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent); cursor: pointer; text-align: center;
}
.markdown-placeholder:hover { border-color: var(--color-primary); color: var(--color-primary); }
</style>
