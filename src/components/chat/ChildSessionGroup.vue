<template>
  <div class="flex gap-2 w-full my-1">
    <!-- 子会话标识线 -->
    <div class="w-0.5 bg-info/40 rounded-full shrink-0 self-stretch"></div>
    <!-- 子会话折叠卡片 -->
    <div class="flex gap-2 w-full">
      <div class="flex h-6 w-6 items-center justify-center rounded-full bg-info/15 shrink-0 mt-0.5">
        <SvgIcon name="bot" size="12" class="text-info/80" />
      </div>
      <div class="max-w-[900px] flex-1">
        <!-- 折叠状态：显示摘要 -->
        <div 
          v-if="!expanded"
          class="bg-info/5 border border-info/15 rounded-lg px-2.5 py-1.5 cursor-pointer hover:bg-info/10 hover:border-info/25 transition-all group"
          @click="toggle"
        >
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <span class="text-[11px] text-info/80 font-medium shrink-0">子 Agent</span>
              <span class="text-[11px] text-base-content/40 shrink-0">{{ group.messageCount }}条</span>
              <span class="text-[11px] text-base-content/50 truncate">{{ previewText }}</span>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <span v-if="group.timestamp" class="text-[11px] text-base-content/35">{{ formatTime(group.timestamp) }}</span>
              <SvgIcon name="chevronRight" size="12" class="text-info/40 group-hover:text-info/60 transition-colors" />
            </div>
          </div>
        </div>
        <!-- 展开状态：显示完整消息 -->
        <div v-else class="animate-expand">
          <!-- 收起按钮 -->
          <div 
            class="bg-info/10 border border-info/20 rounded-lg px-2.5 py-1 cursor-pointer hover:bg-info/15 transition-colors mb-2 inline-flex items-center gap-1.5"
            @click="toggle"
          >
            <SvgIcon name="chevronDown" size="12" class="text-info/60" />
            <span class="text-[11px] text-info/80 font-medium">子 Agent</span>
            <span class="text-[11px] text-base-content/40">{{ group.messageCount }}条消息</span>
            <span v-if="group.timestamp" class="text-[11px] text-base-content/35">{{ formatTime(group.timestamp) }}</span>
          </div>
          <!-- 子会话消息列表 - 紧凑样式 -->
          <div class="space-y-1.5 pl-1 border-l-2 border-info/20">
            <div v-for="(msg, idx) in group.messages" :key="`${group.sessionId}-${idx}`" class="flex gap-2 items-start">
              <div class="flex h-5 w-5 items-center justify-center rounded-full shrink-0" :class="msg.role === 'user' ? 'bg-info/10' : 'bg-success/10'">
                <SvgIcon :name="msg.role === 'user' ? 'user' : 'bot'" size="10" :class="msg.role === 'user' ? 'text-info/70' : 'text-success/70'" />
              </div>
              <div class="flex-1 min-w-0">
                <div v-if="msg.role === 'user'" class="bg-info/5 border border-info/10 rounded-md px-2 py-1">
                  <p class="text-xs text-base-content/80 whitespace-pre-wrap break-words">{{ msg.content }}</p>
                </div>
                <div v-else class="bg-success/5 border border-success/10 rounded-md px-2 py-1">
                  <div v-if="msg.content" class="markdown-content text-xs text-base-content/80" v-html="renderMarkdown(msg.content)"></div>
                  <!-- 工具调用显示 -->
                  <div v-if="msg.toolCalls && msg.toolCalls.length > 0" class="mt-1.5 space-y-1">
                    <ToolCallCard
                      v-for="(tc, tcIdx) in msg.toolCalls"
                      :key="tcIdx"
                      :tool="tc"
                      :expanded="false"
                      :icon="tc.isSubAgent ? 'bot' : 'tool'"
                      :title="tc.name"
                      :summary="tc.isSubAgent ? '执行任务' : ''"
                      :formatPreview="() => ''"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import ToolCallCard from './ToolCallCard.vue';

interface ToolCall {
  name: string;
  args?: Record<string, unknown>;
  result?: string;
  status?: string;
  durationMs?: number;
  isSubAgent?: boolean;
}

interface Message {
  role: string;
  content?: string | null;
  toolCalls?: ToolCall[];
}

interface ChildSessionGroup {
  type: 'childSessionGroup';
  sessionId: string;
  messages: Message[];
  preview: string;
  messageCount: number;
  timestamp: number;
}

const props = defineProps<{
  group: ChildSessionGroup;
  isExpanded?: boolean;
  formatTime: (ts: number) => string;
  renderMarkdown: (content: string) => string;
}>();

const emit = defineEmits<{
  (e: 'toggle', sessionId: string): void;
}>();

const expanded = computed({
  get: () => props.isExpanded || false,
  set: () => emit('toggle', props.group.sessionId)
});

const previewText = computed(() => {
  const text = props.group.preview;
  return text.length > 50 ? text.slice(0, 50) + '...' : text;
});

const toggle = () => {
  expanded.value = !expanded.value;
  emit('toggle', props.group.sessionId);
};
</script>

<style scoped>
.animate-expand {
  animation: expandIn 0.2s ease-out;
}

@keyframes expandIn {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.markdown-content {
  line-height: 1.5;
  word-break: break-word;
}

.markdown-content :deep(p) {
  margin: 0.3em 0;
}
</style>