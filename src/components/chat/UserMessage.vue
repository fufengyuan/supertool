<template>
  <div class="flex gap-2 w-full group">
    <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-base-200">
      <SvgIcon name="user" size="14" class="text-base-content/60" />
    </div>
    <div class="max-w-[900px]">
      <!-- 用户消息气泡 -->
      <div class="bg-primary/10 border border-primary/20 rounded-xl px-3 py-2">
        <!-- 文件/文件夹路径徽章 -->
        <div v-if="message.filePaths && message.filePaths.length > 0" class="flex flex-wrap gap-1.5 mb-1.5">
          <div
            v-for="(pathItem, pi) in message.filePaths"
            :key="pi"
            class="flex items-center gap-1 px-2 py-0.5 rounded-md text-xs border cursor-default"
            :class="pathItem.type === 'folder'
              ? 'bg-warning/10 border-warning/25 text-warning'
              : 'bg-info/10 border-info/25 text-info/90'"
            :title="pathItem.path"
          >
            <SvgIcon :name="pathItem.type === 'folder' ? 'folder' : 'file'" size="11" />
            <span class="max-w-[200px] truncate">{{ pathItem.name }}</span>
          </div>
        </div>
        <!-- Markdown 渲染用户消息 -->
        <VueMarkdown
          :source="displayContent"
          :options="mdOptions"
          class="text-sm text-base-content user-message-markdown"
        />
      </div>
      <!-- 时间戳 -->
      <div v-if="message.timestamp" class="mt-1 text-xs text-base-content/40">
        {{ formatTime(message.timestamp) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VueMarkdown from 'vue-markdown-render';
import hljs from 'highlight.js';
import SvgIcon from '@/components/ui/SvgIcon.vue';

interface FilePath {
  type: 'file' | 'folder';
  path: string;
  name: string;
}

interface Message {
  role: string;
  content: string | null;
  filePaths?: FilePath[];
  timestamp: number | null;
  toolName: string | null;
}

const props = defineProps<{
  message: Message;
  searchQuery?: string;
  formatTime: (ts: number) => string;
  highlightText: (text: string, query: string) => string;
  getDisplayContent: (msg: Message) => string;
}>();

const displayContent = computed(() => props.getDisplayContent(props.message));

// Markdown 渲染配置（与 AssistantMessage 一致）
const mdOptions = {
  html: true,
  linkify: true,
  typographer: true,
  breaks: true,
  highlight: (str: string, lang: string) => {
    if (lang && hljs.getLanguage(lang)) {
      return hljs.highlight(str, { language: lang }).value;
    }
    return hljs.highlightAuto(str).value;
  },
};
</script>

<style scoped>
.user-message-markdown {
  line-height: 1.5;
}
.user-message-markdown p {
  margin: 0;
}
.user-message-markdown code {
  background: rgba(0, 0, 0, 0.05);
  padding: 0.1em 0.3em;
  border-radius: 3px;
  font-size: 0.9em;
}
.user-message-markdown pre {
  background: rgba(0, 0, 0, 0.05);
  padding: 0.5em;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5em 0;
}
.user-message-markdown pre code {
  background: transparent;
  padding: 0;
}
</style>