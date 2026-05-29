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
            <img v-if="pathItem.previewUrl" :src="pathItem.previewUrl" class="w-6 h-6 rounded object-cover shrink-0 cursor-pointer hover:opacity-80" @click.stop="openPreview(pathItem.previewUrl, pathItem.name)" />
            <SvgIcon v-else :name="pathItem.type === 'folder' ? 'folder' : 'file'" size="11" />
            <span class="max-w-[200px] truncate">{{ pathItem.name }}</span>
          </div>
        </div>
        <!-- Markdown 渲染用户消息 -->
        <VueMarkdown
          :source="displayContent"
          :options="mdOptions"
          class="prose prose-sm max-w-none"
        />
      </div>
      <!-- 时间戳 -->
      <div class="mt-1 flex items-center justify-between">
        <span v-if="message.timestamp" class="text-xs text-base-content/40">
          {{ formatTime(message.timestamp) }}
        </span>
        <button
          v-if="message.content"
          class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
          :title="copied ? '已复制' : '复制消息'"
          @click="copyContent"
        >
          <SvgIcon :name="copied ? 'clipboardCheck' : 'clipboard'" size="12" :class="copied ? 'text-success' : ''" />
        </button>
      </div>
    </div>
    <!-- Image Preview Lightbox -->
    <Teleport to="body">
      <div
        v-if="previewImage"
        class="fixed inset-0 bg-black/70 flex items-center justify-center z-[100] cursor-pointer"
        @click="closePreview"
      >
        <img
          :src="previewImage.src"
          :alt="previewImage.alt"
          class="max-w-[90vw] max-h-[90vh] rounded-lg shadow-2xl object-contain"
          @click.stop
        />
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import VueMarkdown from 'vue-markdown-render';
import hljs from 'highlight.js';
import SvgIcon from '@/components/ui/SvgIcon.vue';

const previewImage = ref<{ src: string; alt: string } | null>(null);

const openPreview = (src: string, alt: string) => {
  previewImage.value = { src, alt };
};

const closePreview = () => {
  previewImage.value = null;
};

const copied = ref(false);

async function copyContent() {
  if (!props.message.content) return;
  try {
    await navigator.clipboard.writeText(displayContent.value);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch {
    const ta = document.createElement('textarea');
    ta.value = displayContent.value;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  }
}

interface FilePath {
  type: 'file' | 'folder';
  path: string;
  name: string;
  previewUrl?: string;
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
