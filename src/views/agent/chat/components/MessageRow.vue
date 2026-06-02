<template>
  <div :class="['flex gap-2 w-full group', msg.role === 'user' ? 'justify-end' : '']">
    <!-- Agent avatar -->
    <div
      v-if="msg.role === 'agent'"
      class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-primary/20"
    >
      <SvgIcon name="bot" size="14" class="text-primary" />
    </div>

    <div :class="['max-w-[80%]', msg.role === 'user' ? 'order-1' : '']">
      <!-- Attachments -->
      <div
        v-if="msg.attachments && msg.attachments.length > 0"
        class="flex flex-wrap gap-1 mb-1"
      >
        <div
          v-for="att in msg.attachments"
          :key="att.id"
          class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-base-200/60 border border-base-content/10"
        >
          <SvgIcon :name="att.kind === 'image' ? 'image' : 'file'" size="10" />
          <span class="truncate max-w-[120px]">{{ att.name }}</span>
        </div>
      </div>

      <!-- Message bubble -->
      <div
        v-if="hasContent"
        :class="[
          'rounded-xl px-3 py-2 border border-base-content/10',
          msg.role === 'user'
            ? 'bg-primary/10 rounded-tr-sm'
            : 'bg-base-200/60 rounded-tl-sm',
        ]"
      >
        <!-- Markdown rendered content -->
        <VueMarkdown
          v-if="msg.role === 'agent' && contentText"
          :source="contentText"
          class="prose prose-sm max-w-none"
          :options="mdOptions"
        />
        <!-- User content (plain text) -->
        <div v-else-if="contentText" class="text-sm whitespace-pre-wrap">{{ contentText }}</div>
      </div>

      <!-- Timestamp & actions -->
      <div class="mt-1 flex items-center justify-between px-1">
        <span class="text-xs text-base-content/30" />
        <button
          v-if="contentText"
          class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
          :title="copied ? 'Copied' : 'Copy'"
          @click="copyContent"
        >
          <SvgIcon :name="copied ? 'clipboardCheck' : 'clipboard'" size="12" :class="copied ? 'text-success' : ''" />
        </button>
      </div>
    </div>

    <!-- User avatar -->
    <div
      v-if="msg.role === 'user'"
      class="flex h-8 w-8 items-center justify-center rounded-full shrink-0 bg-base-200 text-xs font-medium text-base-content/60"
    >
      U
    </div>

    <!-- Approval bar -->
    <div
      v-if="showApproval"
      class="fixed bottom-24 left-1/2 -translate-x-1/2 flex gap-2 z-50"
    >
      <button
        class="btn btn-success btn-sm gap-1"
        @click="onApprove"
      >
        <SvgIcon name="check" size="14" />
        Approve
      </button>
      <button
        class="btn btn-error btn-sm gap-1"
        @click="onDeny"
      >
        <SvgIcon name="x" size="14" />
        Deny
      </button>
    </div>

    <!-- Image preview modal -->
    <div
      v-if="previewAttachment"
      class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
      @click="previewAttachment = null"
    >
      <img
        :src="previewAttachment.dataUrl"
        :alt="previewAttachment.name"
        class="max-w-[80vw] max-h-[80vh] rounded-lg"
        @click.stop
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import VueMarkdown from 'vue-markdown-render';
import hljs from 'highlight.js';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import type { ChatBubbleMessage, Attachment } from '../types';

const APPROVAL_RE =
  /⚠️.*dangerous|requires? (your )?approval|\/approve.*\/deny|do you want (me )?to (proceed|continue|run|execute)/i;

const props = defineProps<{
  msg: ChatBubbleMessage;
  isLast: boolean;
  isLoading: boolean;
  onApprove: () => void;
  onDeny: () => void;
}>();

const copied = ref(false);
const previewAttachment = ref<Attachment | null>(null);

const contentText = computed(() => props.msg.content || '');

const hasContent = computed(() => (props.msg.content || '').trim().length > 0);

const showApproval = computed(
  () =>
    props.msg.role === 'agent' &&
    !props.isLoading &&
    props.isLast &&
    APPROVAL_RE.test(props.msg.content || ''),
);

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

async function copyContent() {
  if (!props.msg.content) return;
  try {
    await navigator.clipboard.writeText(props.msg.content);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch {
    const ta = document.createElement('textarea');
    ta.value = props.msg.content;
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
</script>
