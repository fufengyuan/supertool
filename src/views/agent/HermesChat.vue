<template>
  <div
    class="flex flex-col h-full relative"
    @dragenter="handleDragEnter"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <!-- Chat header - hermes-desktop simplified style -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 bg-base-100 shrink-0">
      <div class="flex items-center gap-2">
        <SvgIcon name="bot" size="16" class="text-primary" />
        <!-- 标题显示/编辑 -->
        <template v-if="isEditingTitle">
          <input
            ref="titleInputRef"
            v-model="editingTitle"
            class="input input-sm input-bordered w-48 text-sm"
            placeholder="输入标题..."
            @keydown.enter.exact="saveTitle"
            @keydown.escape="cancelEditTitle"
            @blur="saveTitle"
          />
        </template>
        <template v-else>
          <span class="text-sm font-semibold text-base-content cursor-pointer hover:opacity-80" @click="startEditTitle">
            {{ currentSession?.title || 'Hermes Chat' }}
          </span>
          <button v-if="currentSession" class="btn btn-ghost btn-xs btn-square" @click="startEditTitle">
            <SvgIcon name="edit" size="12" />
          </button>
        </template>
        <!-- Usage badge -->
        <span
          v-if="lastUsage"
          class="badge badge-ghost badge-xs"
          :title="`Prompt: ${lastUsage.promptTokens} · Completion: ${lastUsage.completionTokens}${lastUsage.cost ? ' · $' + lastUsage.cost.toFixed(4) : ''}`"
        >
          {{ (lastUsage.promptTokens + lastUsage.completionTokens).toLocaleString() }} tokens
          <template v-if="lastUsage.cost"> · ${{ lastUsage.cost.toFixed(4) }}</template>
        </span>
      </div>
      <div class="flex items-center gap-1">
        <!-- Context folder chip -->
        <div v-if="contextFolder" class="flex items-center gap-0.5">
          <button class="btn btn-ghost btn-xs gap-1 text-info" @click="pickContextFolder" :title="'工作目录: ' + contextFolder">
            <SvgIcon name="folder" size="12" />
            <span class="text-xs max-w-[100px] truncate">{{ contextFolder.split('/').pop() || contextFolder }}</span>
          </button>
          <button class="btn btn-ghost btn-xs btn-square" @click="clearContextFolder" title="清除工作目录">
            <SvgIcon name="close" size="10" />
          </button>
        </div>
        <button v-else class="btn btn-ghost btn-xs" @click="pickContextFolder" title="设置工作目录">
          <SvgIcon name="folder" size="12" />
        </button>
        <!-- Fast mode toggle -->
        <button class="btn btn-ghost btn-xs" :class="{ 'text-warning': fastMode }" @click="toggleFastMode" :title="fastMode ? '快速模式: 开启' : '快速模式: 关闭'">
          <SvgIcon name="zap" size="14" />
        </button>
        <!-- New chat -->
        <button class="btn btn-ghost btn-xs" @click="startNewChat" title="新对话 (Cmd+K)">
          <SvgIcon name="plus" size="14" />
        </button>
        <!-- Delete -->
        <button v-if="currentSession" class="btn btn-ghost btn-xs" @click="deleteCurrentSessionLocal" title="删除会话">
          <SvgIcon name="trash" size="14" />
        </button>
      </div>
    </div>

      <!-- 消息列表 -->
      <div ref="messagesContainer" class="flex-1 min-w-0 overflow-y-auto overflow-x-hidden px-4 py-2 space-y-1" @scroll="handleScroll">
        <!-- 加载消息状态 - 骨架屏 -->
        <div v-if="loadingMessages" class="space-y-1">
          <div class="flex gap-2">
            <div class="h-8 w-8 rounded-full bg-base-200 shrink-0 animate-pulse"></div>
            <div class="flex-1 space-y-1">
              <div class="h-4 bg-base-200 rounded w-3/4 animate-pulse"></div>
              <div class="h-4 bg-base-200 rounded w-1/2 animate-pulse"></div>
            </div>
          </div>
          <div class="flex gap-2">
            <div class="h-8 w-8 rounded-full bg-primary/20 shrink-0 animate-pulse"></div>
            <div class="flex-1 space-y-1">
              <div class="h-4 bg-primary/10 rounded w-full animate-pulse"></div>
              <div class="h-4 bg-primary/10 rounded w-2/3 animate-pulse"></div>
            </div>
          </div>
        </div>

        <!-- 消息列表 -->
        <template v-else-if="displayItems.length > 0">
          <template v-for="(item, idx) in (searchQuery ? filteredMessages : displayItems)" :key="idx">
            <!-- 子会话组（折叠展示） -->
            <ChildSessionGroup 
              v-if="'type' in item && item.type === 'childSessionGroup'"
              :group="(item as ChildSessionGroup)"
              :isExpanded="isChildSessionExpanded((item as ChildSessionGroup).sessionId)"
              :formatTime="formatMessageTime"
              @toggle="toggleChildSessionExpand"
              @continue="handleChildSessionContinue"
            />
            
<!-- 普通消息 -->
            <template v-else>
              <!-- 用户消息 -->
              <UserMessage 
                v-if="(item as Message).role === 'user'"
                :message="(item as Message)"
                :searchQuery="searchQuery"
                :formatTime="formatMessageTime"
                :highlightText="highlightText"
                :getDisplayContent="(msg) => userDisplayContent(msg)"
              />
              
              <!-- Assistant 消息 -->
              <AssistantMessage
                v-else
                :message="(item as Message)"
                :messageIndex="idx"
                :formatTime="formatMessageTime"
                :getToolIcon="getToolIcon"
                :formatArgsSummary="formatArgsSummary"
                :isThinkingExpanded="isThinkingExpanded"
                :onToggleThinking="toggleThinkingExpand"
                :onRetry="retryMessage"
              />
            </template>
          </template>

          <!-- 流式响应状态（思考中/工具调用）+ 停止按钮 -->
          <div v-if="isStreaming" class="flex gap-2 w-full">
            <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20 shrink-0">
              <SvgIcon name="bot" size="14" class="text-primary animate-pulse" />
            </div>
            <div class="max-w-[1200px] bg-base-100 border border-base-300 rounded-xl px-3 py-2">
              <!-- 思考文本 -->
              <p v-if="thinkingText" class="text-sm text-base-content/60 animate-pulse">{{ thinkingText }}</p>
              <!-- 工具调用（只显示 running 状态的） -->
              <div v-else-if="currentStreamingMsg?.toolCalls?.some(t => t.status === 'running')" class="mt-0 space-y-1">
                <div v-for="(tool, idx) in currentStreamingMsg.toolCalls.filter(t => t.status === 'running')" :key="idx" class="flex items-center gap-2 text-xs bg-base-200/50 rounded px-2 py-1">
                  <SvgIcon :name="getToolIcon(tool.name).icon" size="12" :class="getToolIcon(tool.name).color + ' animate-pulse'" />
                  <span :class="getToolIcon(tool.name).color" class="font-medium">{{ tool.name }}</span>
                  <span v-if="tool.args" class="text-base-content/70 truncate max-w-[600px]">{{ formatArgsSummary(tool.args) }}</span>
                  <span class="text-base-content/60 ml-auto animate-pulse">执行中...</span>
                </div>
              </div>
              <!-- 思考中（无工具调用时） -->
              <div v-else class="flex items-center gap-2 text-sm text-base-content/80">
                <span class="loading loading-spinner loading-xs text-primary"></span>
                <span>思考中...</span>
              </div>
            </div>
            <!-- 取消按钮 - 更醒目 -->
            <button 
              v-if="!isAborting"
              class="btn btn-error btn-sm btn-square self-center animate-pulse shadow-error/20 shadow-sm"
              @click="abortChat"
              title="取消处理 (Esc)"
            >
              <SvgIcon name="stop" size="16" />
            </button>
            <!-- 停止中状态 -->
            <button 
              v-if="isAborting"
              class="btn btn-sm btn-square self-center btn-disabled"
              disabled
              title="停止中..."
            >
              <span class="loading loading-spinner loading-sm"></span>
            </button>
          </div>
        </template>

        <!-- Agent approval bar -->
        <div v-if="needsApproval" class="flex gap-2 w-full mt-2">
          <div class="flex h-8 w-8 items-center justify-center rounded-full bg-warning/20 shrink-0">
            <SvgIcon name="zap" size="14" class="text-warning" />
          </div>
          <div class="max-w-[600px] bg-warning/10 border border-warning/30 rounded-xl px-3 py-2 flex items-center gap-2">
            <span class="text-sm text-base-content/80">需要你的批准</span>
            <button class="btn btn-success btn-xs gap-1" @click="handleApproval(true)">
              <SvgIcon name="check" size="10" />
              批准
            </button>
            <button class="btn btn-error btn-xs gap-1" @click="handleApproval(false)">
              <SvgIcon name="close" size="10" />
              拒绝
            </button>
          </div>
        </div>

        <!-- Empty state - hermes-desktop style suggestion chips -->
        <div v-else class="flex flex-col items-center justify-center h-full px-6 py-12 text-center">
          <!-- Logo icon -->
          <div class="mb-4">
            <div class="flex items-center justify-center w-16 h-16 rounded-2xl bg-primary/10 border border-primary/20">
              <SvgIcon name="bot" size="32" class="text-primary" />
            </div>
          </div>

          <!-- Title -->
          <h2 class="text-lg font-semibold text-base-content">Hermes Agent</h2>
          <p class="mt-1 text-sm text-base-content/50 max-w-sm">
            我能做什么？试着问我：
          </p>

          <!-- Suggestion chips (6 items matching hermes-desktop pattern) -->
          <div class="mt-5 grid grid-cols-1 sm:grid-cols-2 gap-2 w-full max-w-lg">
            <button
              v-for="suggestion in suggestionChips"
              :key="suggestion.text"
              class="flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl border border-base-content/10 bg-base-200/30 hover:bg-base-200/60 hover:border-primary/20 transition-all text-left group"
              @click="chatInputRef?.setInputText(suggestion.text)"
            >
              <SvgIcon :name="suggestion.icon" size="16" class="text-primary/60 group-hover:text-primary/80 shrink-0" />
              <span class="text-sm text-base-content/70 group-hover:text-base-content/90">{{ suggestion.label }}</span>
            </button>
          </div>

          <!-- Keyboard hint -->
          <div class="mt-6 flex items-center gap-1.5 text-[11px] text-base-content/30">
            <SvgIcon name="keyboard" size="12" />
            <span>输入消息开始对话，<kbd class="px-1 py-0.5 rounded bg-base-200/60 text-base-content/40 font-mono text-[10px]">⌘ ↵</kbd> 发送</span>
          </div>
        </div>
        
        <!-- 回到底部按钮 - 用户向上滚动时显示 -->
        <button 
          v-if="showScrollToBottom && messages.length > 0"
          class="fixed bottom-[140px] right-[30px] btn btn-circle btn-sm btn-primary shadow-lg opacity-90 hover:opacity-100 transition-opacity z-10"
          @click="scrollToBottom(true)"
          title="回到底部"
        >
          <SvgIcon name="chevronDown" size="16" />
        </button>
      </div>

    <!-- Message queue indicator -->
    <div v-if="queuedCount > 0" class="flex items-center justify-center gap-1.5 px-4 py-1.5 bg-warning/10 border-b border-warning/20 shrink-0">
      <SvgIcon name="zap" size="12" class="text-warning" />
      <span class="text-xs text-warning/80">{{ queuedCount }} 条消息排队中</span>
    </div>

      <!-- 拖放文件覆盖层 -->
      <div v-if="dragActive" class="absolute inset-0 bg-primary/5 border-2 border-dashed border-primary/30 rounded-lg flex items-center justify-center z-40 pointer-events-none">
        <div class="text-center">
          <SvgIcon name="paperclip" size="32" class="text-primary/40 mx-auto mb-2" />
          <p class="text-sm text-primary/60">拖放文件到此处</p>
        </div>
      </div>

      <!-- 输入区域 -->
      <ChatInput
        ref="chatInputRef"
        :isStreaming="isStreaming"
        :currentSession="currentSession"
        :favoriteFolders="favoriteFolders"
        :gitRepos="gitRepos"
        :hermesAvailable="hermesAvailable"
        @send="handleSend"
        @paste="onPaste"
        @checkHermes="checkHermes"
        @removeFavoriteFolder="removeFavoriteFolder"
        @modelChanged="onModelChanged"
      />
    </div>

  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'HermesChat' })
import { ref, reactive, computed, onMounted, onUnmounted, nextTick, watch, type Ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { useStreamingHandler } from '@/composables/useStreamingHandler';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/core';
import { markedHighlight } from 'marked-highlight';
import javascript from 'highlight.js/lib/languages/javascript';
import { getTauriAPI } from '../../utils/tauri-api';
import type { GitRepo } from '../../types';
import { useSessionManager, type Session, type SearchResult } from '@/composables/useSessionManager';
import { useToolExpandState, getToolIcon, formatArgsSummary } from '@/composables/useToolFormatter';
import { setupCopyCode } from '@/composables/useMarkdownRenderer';
import { useFavoriteFolders } from '@/composables/useFavoriteFolders';
import python from 'highlight.js/lib/languages/python';
import json from 'highlight.js/lib/languages/json';
import sql from 'highlight.js/lib/languages/sql';
import typescript from 'highlight.js/lib/languages/typescript';
import bash from 'highlight.js/lib/languages/bash';
import xml from 'highlight.js/lib/languages/xml';
import yaml from 'highlight.js/lib/languages/yaml';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import { UserMessage, AssistantMessage, ChildSessionGroup, ToolCallCard, ChatInput } from '@/components/chat';

hljs.registerLanguage('bash', bash);
hljs.registerLanguage('json', json);
hljs.registerLanguage('sql', sql);
hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('xml', xml);
hljs.registerLanguage('yaml', yaml);

// 配置 marked 使用 highlight.js
marked.use(markedHighlight({
  highlight(code: string, lang: string | undefined) {
    // 检测内容是否已经被 hljs 高亮过（包含 hljs-xxx 类名）
    if (code.includes('class="hljs-') || code.includes('class=\'hljs-')) {
      // 已经高亮过，直接返回，避免双重编码
      return code;
    }
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(code, { language: lang }).value;
      } catch {}
    }
    return hljs.highlightAuto(code).value;
  },
}));
marked.setOptions({
  breaks: true, // 支持 GFM 换行
  gfm: true, // GitHub Flavored Markdown
});

// 解析模型名（提取供应商和名称）
function parseModelName(fullName: string): { provider: string | null; name: string } {
  const slashIdx = fullName.indexOf('/');
  if (slashIdx > 0) {
    return { provider: fullName.substring(0, slashIdx), name: fullName.substring(slashIdx + 1) };
  }
  return { provider: null, name: fullName };
}

const route = useRoute();
const router = useRouter();

// ===== 使用会话管理 Composable =====
const {
  sessions,
  searchResults,
  isSearching,
  currentSessionId,
  currentSession,
  loadingSessions,
  gitRepos,
  refreshSessions,
  selectSession: selectSessionBase,
  startNewChat: startNewSession,
  deleteSession: deleteSessionBase,
  deleteCurrentSession: deleteCurrentSessionBase,
  handleSessionSearch,
  clearSessionSearch,
  jumpToSearchResult: jumpToSearchResultBase,
  loadGitRepos,
  sourceIcon,
  highlightSnippet,
  generateSessionTitle,
} = useSessionManager();

// 思考展开状态
const {
  expandedThinking,
  toggleThinkingExpand,
  isThinkingExpanded,
} = useToolExpandState();

// 搜索结果（来自 Hermes FTS5 搜索） - 类型已从 composable 导入

// 工具调用详情
interface ToolCall {
  id?: string; // 工具调用唯一 ID
  name: string;
  args?: Record<string, unknown>; // 工具参数
  result?: string; // 工具返回结果
  durationMs: number;
  isSubAgent?: boolean; // 是否是子 agent
  status?: 'running' | 'completed' | 'error'; // 状态
}

interface Message {
  role: string;
  content: string | null;
  timestamp: number | null;
  toolName: string | null;
  toolCalls?: ToolCall[];
  thinking?: string; // 思考过程
  isError?: boolean; // 是否是错误消息
  isStopped?: boolean; // 是否被用户停止
  retryContent?: string; // 用于重试的原始消息内容
  tokens?: { input: number; output: number }; // token 使用量
  filePaths?: PathItem[]; // 附带的文件/文件夹路径（仅用户消息）
  isChild?: boolean; // 是否来自子会话（subagent）
  sessionId?: string; // 子会话的 session_id
}

// 子会话消息组（用于折叠展示）
interface ChildSessionGroup {
  type: 'childSessionGroup';
  sessionId: string;
  messages: Message[];  // 该子会话的所有消息
  preview: string;      // 预览文本（第一条用户消息摘要）
  messageCount: number; // 消息数量
  timestamp: number;    // 子会话开始时间
}

// 显示项：可以是普通消息或子会话组
type DisplayItem = Message | ChildSessionGroup;

// Raw message from backend (matches HermesMessage in Rust)
interface RawMessage {
  id: number;
  sessionId: string;
  role: string;
  content: string | null;
  timestamp: number;
  toolName: string | null;
  toolCallId: string | null;  // 工具调用 ID（tool 消息才有）
  toolCalls?: string;  // JSON string, parse in frontend
  finishReason: string | null;
  reasoning: string | null;  // 思考内容
  reasoningContent: string | null;
  isChild: boolean;  // 是否来自子会话
}

// Raw tool call from backend
interface RawToolCall {
  id: string;
  function: {
    name: string;
    arguments: string;
  };
}

// Task item from todo tool
interface TaskItem {
  id: string;
  content: string;
  status: 'pending' | 'in_progress' | 'completed' | 'cancelled';
}

// State
// 注：sessions, searchResults, isSearching, currentSessionId, currentSession, loadingSessions, gitRepos
// 已从 useSessionManager composable 导入
const messages = ref<Message[]>([]);
const currentTasks = ref<TaskItem[]>([]); // 当前任务列表
const showTaskPanel = ref(true); // 是否显示任务面板
const chatInputRef = ref<InstanceType<typeof ChatInput> | null>(null);
// 注：gitRepos 已从 useSessionManager comopolitan 导入
const {
  favoriteFolders,
  loadFavoriteFolders,
  removeFavoriteFolder,
} = useFavoriteFolders();
// 注：loadingSessions 已从 useSessionManager comopolitan 导入
const loadingMessages = ref(false);
const isAborting = ref(false); // 正在停止中
const showScrollToBottom = ref(false); // 是否显示回到底部按钮
const hermesAvailable = ref(false);

// Smart auto-scroll: track if user scrolled up
const userScrolledUp = ref(false);

// Fast mode toggle
const fastMode = ref(false);

// Suggestion chips for empty state (matches hermes-desktop pattern)
const suggestionChips = [
  { icon: 'search', label: '搜索今天的科技新闻', text: 'Search the web for today\'s top tech news' },
  { icon: 'bell', label: '设置每日提醒', text: 'Set a reminder to check emails every day at 9 AM' },
  { icon: 'mail', label: '总结最新邮件', text: 'Read my latest emails and summarize them' },
  { icon: 'code', label: '写一个脚本', text: 'Write a Python script to rename all files in a folder' },
  { icon: 'clock', label: '定时备份数据库', text: 'Schedule a cron job to back up my database every night' },
  { icon: 'chart', label: '分析 CSV 数据', text: 'Analyze this CSV file and show key insights' },
];

// Rich usage badge with cost
const lastUsage = ref<{ promptTokens: number; completionTokens: number; cost?: number } | null>(null);
const unlistenUsageFn = ref<(() => void) | null>(null);

// Context folder: 绑定会话到工作目录
const contextFolder = ref<string | null>(null);

// Message queue: agent 忙时排队消息，完成后逐条发送
interface QueuedMessage {
  text: string;
  paths: PathItem[];
  model: string;
}
const messageQueue = ref<QueuedMessage[]>([]);
const queuedCount = ref(0);

// Drag-and-drop file overlay state
const dragActive = ref(false);
const dragCounter = ref(0);

const eventHasFiles = (e: DragEvent): boolean => {
  const types = e.dataTransfer?.types;
  if (!types) {return false;}
  for (let i = 0; i < types.length; i++) {
    if (types[i] === 'Files') {return true;}
  }
  return false;
};

const handleDragEnter = (e: DragEvent) => {
  if (!eventHasFiles(e)) {return;}
  e.preventDefault();
  dragCounter.value += 1;
  if (dragCounter.value === 1) {dragActive.value = true;}
};

const handleDragOver = (e: DragEvent) => {
  if (!eventHasFiles(e)) {return;}
  e.preventDefault();
  if (e.dataTransfer) {e.dataTransfer.dropEffect = 'copy';}
};

const handleDragLeave = (e: DragEvent) => {
  if (!eventHasFiles(e)) {return;}
  dragCounter.value -= 1;
  if (dragCounter.value === 0) {dragActive.value = false;}
};

const handleDrop = async (e: DragEvent) => {
  if (!eventHasFiles(e)) {return;}
  e.preventDefault();
  dragCounter.value = 0;
  dragActive.value = false;
  const files = Array.from(e.dataTransfer?.files || []);
  if (files.length === 0) {return;}
  // Process files through attachment processor
  const { processFiles } = await import('@/composables/useAttachmentProcessor');
  const { attachments, errors } = await processFiles(files);
  if (errors.length > 0) {
    messages.value.push({ role: 'assistant', content: `文件处理错误: ${errors.join(', ')}`, timestamp: Date.now() / 1000, toolName: null, isError: true });
  }
  // Add as path attachments via ChatInput's attachedPaths
  for (const att of attachments) {
    if (att.path) {
      chatInputRef.value?.attachedPaths.push({ path: att.path, type: att.mime?.startsWith('image/') ? 'file' : 'file', name: att.name });
    } else if (att.dataUrl) {
      chatInputRef.value?.attachedPaths.push({ path: att.name, type: 'file', name: att.name, previewUrl: att.dataUrl });
    } else if (att.text) {
      // For text files, we can't directly set ChatInput text, so add as a path ref with the name
      chatInputRef.value?.attachedPaths.push({ path: att.name, type: 'file', name: att.name });
    }
  }
};

// 调试日志函数（写入日志文件）
const agentLog = async (message: string) => {
  // 直接写入 DEBUG 日志，不再调用 console.log（会被 main.ts 拦截写入 INFO，导致双重记录）
  try {
    const api = getTauriAPI();
    await api.writeSystemLog('debug', 'agent-chat', message);
  } catch (e) {
    // 忽略日志写入失败
  }
};

// 滚动到底部
const scrollToBottom = (force = false) => {
  if (!force && userScrolledUp.value) {return;}
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
      showScrollToBottom.value = false;
      userScrolledUp.value = false;
    }
  });
};

// 使用流式响应处理 composable
const {
  streamingSessions,
  thinkingTexts,
  sessionRoundEnded,
  sessionMessagesCache,
  isStreaming,
  thinkingText,
  setupStreamingListeners,
  cleanupStreamingListeners,
  setStreaming,
  resetRoundEnded,
} = useStreamingHandler({
  currentSessionId,
  messages,
  currentTasks,
  agentLog,
  scrollToBottom,
});

// 当前流式响应的 assistant 消息（数组最后一个 assistant 消息）
const currentStreamingMsg = computed(() => {
  const lastMsg = messages.value[messages.value.length - 1];
  return lastMsg?.role === 'assistant' ? lastMsg : null;
});

// 用于渲染的消息列表（流式输出时跳过最后一个 assistant 消息，避免与实时气泡重复）
const displayMessages = computed(() => {
  // 直接返回 messages，流式内容实时显示在消息列表
  // 实时气泡仅用于显示工具调用状态和进度文本
  return messages.value;
});

// 搜索状态
const searchQuery = ref('');
const filteredMessages = ref<Message[]>([]);

// 子会话折叠展开状态 (key: sessionId)
const expandedChildSessions = ref<Set<string>>(new Set());

// 切换子会话展开
const toggleChildSessionExpand = (sessionId: string) => {
  if (expandedChildSessions.value.has(sessionId)) {
    expandedChildSessions.value.delete(sessionId);
  } else {
    expandedChildSessions.value.add(sessionId);
  }
};

// 检查子会话是否展开
const isChildSessionExpanded = (sessionId: string): boolean => {
  return expandedChildSessions.value.has(sessionId);
};

// 处理子会话继续对话
const handleChildSessionContinue = async (sessionId: string, message: string) => {
  console.log('handleChildSessionContinue', sessionId, message);
  
  // 先在 UI 上显示用户发送的消息
  const userMsg: Message = {
    role: 'user',
    content: message,
    timestamp: Date.now() / 1000,
    toolName: null,
    toolCalls: [],
    isChild: true,  // 标记为子会话消息
    sessionId,      // 子会话 ID
  };
  messages.value.push(userMsg);
  scrollToBottom();
  
  // 设置流式状态（让流式事件处理器知道这个子会话正在响应）
  streamingSessions[sessionId] = true;
  sessionRoundEnded[sessionId] = false;
  
  try {
    // 调用 agent_chat 发送消息到子会话
    // 流式事件会自动更新 UI
    const result = await invoke<{ response: string; session_id: string; message_count: number }>('agent_chat', {
      message,
      sessionId,  // 使用子会话的 sessionId
      model: null,
    });
    console.log('agent_chat result:', result);
  } catch (e) {
    console.error('子会话继续对话失败:', e);
    streamingSessions[sessionId] = false;
  }
};

// 将消息列表转换为显示列表（子会话消息分组折叠，插入到调用位置）
const displayItems = computed<DisplayItem[]>(() => {
  const items: DisplayItem[] = [];
  
  // 先分组子会话消息
  const childSessionGroups = new Map<string, { messages: Message[], firstTimestamp: number, preview: string }>();
  for (const msg of messages.value) {
    if (msg.isChild && msg.sessionId) {
      if (!childSessionGroups.has(msg.sessionId)) {
        const preview = msg.role === 'user' ? (msg.content?.slice(0, 100) || '执行子任务') : '执行子任务';
        childSessionGroups.set(msg.sessionId, {
          messages: [],
          firstTimestamp: msg.timestamp || 0,
          preview,
        });
      }
      childSessionGroups.get(msg.sessionId)!.messages.push(msg);
    }
  }
  
  // 遍历主会话消息，在适当位置插入子会话组
  const insertedSessions = new Set<string>();
  for (const msg of messages.value) {
    if (msg.isChild) {continue;} // 子会话消息不单独处理
    
    items.push(msg);
    
    // 检查是否有子会话应该插入到这条消息之后
    // 子会话的开始时间应该 >= 当前消息时间，且 < 下一条主消息时间（如果有的话）
    const msgTime = msg.timestamp || 0;
    for (const [sessionId, group] of childSessionGroups) {
      if (insertedSessions.has(sessionId)) {continue;}
      
      // 子会话开始时间 >= 当前消息时间，插入到这里
      if (group.firstTimestamp >= msgTime) {
        // 检查是否应该插入（没有更早的主消息在后面）
        const nextMainMsg = messages.value.find(m => 
          !m.isChild && m.timestamp && m.timestamp > msgTime && m.timestamp < group.firstTimestamp
        );
        
        if (!nextMainMsg) {
          items.push({
            type: 'childSessionGroup',
            sessionId,
            messages: group.messages,
            preview: group.preview,
            messageCount: group.messages.length,
            timestamp: group.firstTimestamp,
          });
          insertedSessions.add(sessionId);
        }
      }
    }
  }
  
  // 处理未插入的子会话（可能在消息列表末尾）
  for (const [sessionId, group] of childSessionGroups) {
    if (!insertedSessions.has(sessionId)) {
      items.push({
        type: 'childSessionGroup',
        sessionId,
        messages: group.messages,
        preview: group.preview,
        messageCount: group.messages.length,
        timestamp: group.firstTimestamp,
      });
    }
  }
  
  return items;
});

// Refs
const messagesContainer = ref<HTMLElement | null>(null);
const titleInputRef = ref<HTMLInputElement | null>(null);

// 标题编辑状态
const isEditingTitle = ref(false);
const editingTitle = ref('');

// 挂载复制代码功能到 window
setupCopyCode();

// 处理 ChatInput 发送事件
interface PathItem {
  path: string;
  type: 'file' | 'folder';
  name: string;
  previewUrl?: string;
}

const handleSend = async (text: string, paths: PathItem[], model: string) => {
  if (!text.trim()) {return;}

  // Force scroll to bottom when user sends a message
  userScrolledUp.value = false;

  // 如果正在处理，排队消息（而不是打断）
  if (isStreaming.value) {
    messageQueue.value.push({ text, paths, model });
    queuedCount.value = messageQueue.value.length;
    return;
  }

  // 将已选择路径拼入消息头部
  let pathPrefix = '';
  let msgFilePaths: PathItem[] | undefined;
  if (paths.length > 0) {
    pathPrefix = paths.map(p => p.path).join('\n') + '\n';
    msgFilePaths = [...paths];
  }
  const fullText = pathPrefix + text.trim();

  // 添加用户消息
  messages.value.push({
    role: 'user',
    content: fullText,
    timestamp: Date.now() / 1000,
    toolName: null,
    filePaths: msgFilePaths,
  });
  scrollToBottom();

  // 开始流式输出
  const sid = currentSessionId.value || '';
  if (sid) {streamingSessions[sid] = true;}
  if (sid) {thinkingTexts[sid] = '';}
  if (sid) {sessionRoundEnded[sid] = false;}

  try {
    const modelToUse = model || null;
    
    const result = await invoke<{ response: string; session_id: string; message_count: number }>('agent_chat', {
      message: fullText,
      sessionId: currentSessionId.value,
      model: modelToUse,
      contextFolder: contextFolder.value || null,
    });

    // Always update session_id from API response (it may change on continuation)
    if (result.session_id) {
      // Update currentSessionId if it's different (continuation creates new session)
      if (currentSessionId.value !== result.session_id) {
        currentSessionId.value = result.session_id;
      }
      
      // Auto-title for new sessions
      if (!currentSession.value) {
        const autoTitle = generateSessionTitle(fullText);
        try {
          await invoke('agent_rename_session', {
            sessionId: result.session_id,
            newTitle: autoTitle,
          });
          currentSession.value = {
            id: result.session_id,
            title: autoTitle,
            model: modelToUse || 'unknown',
            source: 'unknown',
            startedAt: Date.now() / 1000,
            endedAt: null,
            messageCount: 1,
            preview: fullText.slice(0, 50),
            lastActive: Date.now() / 1000,
            profile: 'default',
          };
        } catch (e) {
          console.warn('Auto-title failed:', e);
        }
        refreshSessions();
      }
    }

    thinkingText.value = '';
    if (currentSessionId.value) {sessionRoundEnded[currentSessionId.value] = false;}
  } catch (e) {
    console.error('Chat error:', e);
    messages.value.push({
      role: 'assistant',
      content: `错误: ${e}`,
      timestamp: Date.now() / 1000,
      toolName: null,
      isError: true,
      retryContent: fullText,
    });
  }

  if (currentSessionId.value) {streamingSessions[currentSessionId.value] = false;}
  scrollToBottom();
  chatInputRef.value?.focus();
};

// 处理粘贴事件（委托给 ChatInput）
const onPaste = async (e: ClipboardEvent) => {
  // ChatInput 内部处理粘贴，这里只是接收事件
};

// 处理模型变更
const onModelChanged = async (model: string) => {
  // ChatInput 已处理模型切换，这里可以更新当前会话信息
  if (currentSession.value && model) {
    currentSession.value.model = model;
  }
};

// 注：favoriteFolders, loadFavoriteFolders, removeFavoriteFolder
// 已从 useFavoriteFolders comopolitan 导入

// 注：loadGitRepos, searchSessions, handleSessionSearch, clearSessionSearch, sourceIcon
// 已从 useSessionManager composable 导入
// Markdown 渲染已从 useMarkdownRenderer composable 导入

const formatTime = (ts: number | null | undefined) => {
  if (!ts) {return '';}
  const date = new Date(ts * 1000);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  if (days === 0) {
    const hours = Math.floor(diff / (1000 * 60 * 60));
    if (hours === 0) {
      const mins = Math.floor(diff / (1000 * 60));
      return mins <= 1 ? '刚刚' : `${mins}分钟前`;
    }
    return `${hours}小时前`;
  } else if (days === 1) {
    return '昨天';
  } else if (days < 7) {
    return `${days}天前`;
  }
  return date.toLocaleDateString();
};

// 格式化消息时间戳（显示具体时间，不是相对时间）
const formatMessageTime = (ts: number | null | undefined) => {
  if (!ts) {return '';}
  const date = new Date(ts * 1000);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();
  
  if (isToday) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  } else {
    return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  }
};

// Methods
// 注：refreshSessions, highlightSnippet 已从 useSessionManager composable 导入

// 选择会话并加载消息（包含消息处理逻辑）
const selectSessionWithMessages = async (session: Session) => {
  // 使用 composable 的 selectSessionBase，并传递消息处理回调
  await selectSessionBase(session, async (params) => {
    // 检查是否有缓存（流式输出中的会话）
    const cached = sessionMessagesCache[params.sessionId];
    const isStreamingSession = !!streamingSessions[params.sessionId];
    
    if (cached && cached.length > 0 && isStreamingSession) {
      // 有缓存且正在流式输出，直接用缓存
      messages.value = [...cached];
      loadingMessages.value = false;
      return;
    }
    
    // 没有缓存或流式已结束，处理消息
    loadingMessages.value = true;
    messages.value = [];
    
    try {
      // 处理消息：关联 tool_calls 和 tool 结果
      const processedMessages: Message[] = [];
      const toolResultsMap = new Map<string, string>();
      
      // 先收集所有 tool 消息的结果
      for (const m of params.messages) {
        if (m.role === 'tool' && m.toolCallId) {
          toolResultsMap.set(m.toolCallId, m.content || '');
        }
      }
      
      // 再处理 user 和 assistant 消息
      for (const m of params.messages) {
        if (m.role === 'tool') {continue;} // tool 消息不单独显示，合并到 assistant
        
        // 历史消息不需要显示思考内容（干扰视线）
        const msg: Message = {
          role: m.role,
          content: m.content,
          timestamp: m.timestamp,
          toolName: m.toolName,
          toolCalls: [],
          isChild: m.isChild, // 保留子会话标识
          sessionId: m.sessionId, // 保留子会话 ID
        };
        
        // 如果是 assistant 消息且有 tool_calls（JSON 字符串），解析并关联结果
        if (m.role === 'assistant' && m.toolCalls) {
          try {
            const toolCallsParsed = JSON.parse(m.toolCalls) as RawToolCall[];
            for (const tc of toolCallsParsed) {
              const toolName = tc.function?.name || 'unknown';
              const toolArgs = tc.function?.arguments ? JSON.parse(tc.function.arguments) : {};
              const toolResult = toolResultsMap.get(tc.id) || '';
              
              if (!msg.toolCalls) {msg.toolCalls = [];}
              msg.toolCalls.push({
                name: toolName,
                args: toolArgs,
                result: toolResult,
                durationMs: 0, // 历史消息没有时长信息
                isSubAgent: toolName === 'delegate_task' || toolName === 'subagent',
                status: 'completed',
              });
            }
          } catch (e) {
            console.warn('Failed to parse tool_calls:', e);
          }
        }
        
        processedMessages.push(msg);
      }
      
      messages.value = processedMessages;
    } catch (e) {
      console.error('Failed to process messages:', e);
    }
    
    loadingMessages.value = false;
    scrollToBottom();
  });
};

// 跳转到搜索结果（使用 composable 的 jumpToSearchResultBase 并传递消息处理回调）
const jumpToSearchResultWithMessages = async (result: SearchResult) => {
  await jumpToSearchResultBase(result, async (params) => {
    // 使用与 selectSessionWithMessages 相同的消息处理逻辑
    loadingMessages.value = true;
    messages.value = [];
    
    try {
      const processedMessages: Message[] = [];
      const toolResultsMap = new Map<string, string>();
      
      for (const m of params.messages) {
        if (m.role === 'tool' && m.toolCallId) {
          toolResultsMap.set(m.toolCallId, m.content || '');
        }
      }
      
      for (const m of params.messages) {
        if (m.role === 'tool') {continue;}
        
        const msg: Message = {
          role: m.role,
          content: m.content,
          timestamp: m.timestamp,
          toolName: m.toolName,
          toolCalls: [],
          isChild: m.isChild,
          sessionId: m.sessionId,
        };
        
        if (m.role === 'assistant' && m.toolCalls) {
          try {
            const toolCallsParsed = JSON.parse(m.toolCalls) as RawToolCall[];
            for (const tc of toolCallsParsed) {
              const toolName = tc.function?.name || 'unknown';
              const toolArgs = tc.function?.arguments ? JSON.parse(tc.function.arguments) : {};
              const toolResult = toolResultsMap.get(tc.id) || '';
              
              if (!msg.toolCalls) {msg.toolCalls = [];}
              msg.toolCalls.push({
                name: toolName,
                args: toolArgs,
                result: toolResult,
                durationMs: 0,
                isSubAgent: toolName === 'delegate_task' || toolName === 'subagent',
                status: 'completed',
              });
            }
          } catch (e) {
            console.warn('Failed to parse tool_calls:', e);
          }
        }
        
        processedMessages.push(msg);
      }
      
      messages.value = processedMessages;
    } catch (e) {
      console.error('Failed to process messages:', e);
    }
    
    loadingMessages.value = false;
    scrollToBottom();
  });
};

const startNewChat = () => {
  // 使用 composable 的 startNewSession，传递清理回调
  startNewSession(() => {
    messages.value = [];
    chatInputRef.value?.clear();
    thinkingText.value = '';
    // 重置 context folder 和消息队列
    contextFolder.value = null;
    messageQueue.value = [];
    queuedCount.value = 0;
  });
};

// 消息队列 drain: agent 完成后自动发送排队的消息
watch(isStreaming, (streaming) => {
  if (!streaming && messageQueue.value.length > 0) {
    const next = messageQueue.value.shift()!;
    queuedCount.value = messageQueue.value.length;
    // 使用 nextTick 确保状态完全更新后发送
    nextTick(() => {
      handleSend(next.text, next.paths, next.model);
    });
  }
});

// 注：generateSessionTitle 已从 useSessionManager composable 导入

// 重试发送消息
const retryMessage = async (retryContent: string) => {
  if (!retryContent.trim()) {return;}

  // 如果正在处理，先打断当前处理
  if (isStreaming.value) {
    await abortChat();
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  // 移除最后一条错误消息
  if (messages.value.length > 0 && messages.value[messages.value.length - 1].isError) {
    messages.value.pop();
  }

  // 使用 handleSend 发送
  await handleSend(retryContent, [], currentSession.value?.model || '');
};

// 取消当前处理
const abortChat = async () => {
  if (!isStreaming.value || isAborting.value) {return;}
  
  isAborting.value = true;
  
  // 标记当前正在输出的消息为"已停止"
  const lastMsg = messages.value[messages.value.length - 1];
  if (lastMsg && lastMsg.role === 'assistant') {
    lastMsg.isStopped = true;
  }
  
  // 不等待后端完成，立即响应用户
  invoke('agent_abort_chat').catch(e => {
    console.error('Abort error:', e);
  });
  
  // 立即重置状态，让用户感知响应
  setTimeout(() => {
    if (currentSessionId.value) {streamingSessions[currentSessionId.value] = false;}
    isAborting.value = false;
    if (currentSessionId.value) {sessionRoundEnded[currentSessionId.value] = false;}
    thinkingText.value = '';
  }, 100);
};

// 复制消息内容
const copyMessageContent = async (content: string | null) => {
  if (!content) {return;}
  try {
    await navigator.clipboard.writeText(content);
    // 可选：显示复制成功提示（用 toast 或临时状态）
  } catch (e) {
    console.error('Copy failed:', e);
  }
};

// 聊天记录导出（纯文本 / Markdown）
const copyChatTranscript = async (format: 'text' | 'markdown') => {
  const textContent = messages.value
    .filter(m => m.content && m.content.trim())
    .map(m => {
      const speaker = m.role === 'user' ? 'You' : 'Hermes';
      const content = m.content!.trim();
      return format === 'markdown'
        ? `**${speaker}:**\n\n${content}`
        : `${speaker}: ${content}`;
    })
    .join('\n\n');
  if (!textContent) {return;}
  try {
    await navigator.clipboard.writeText(textContent);
  } catch (e) {
    console.error('Copy transcript failed:', e);
  }
};

// Context folder: 选择工作目录
const pickContextFolder = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      contextFolder.value = typeof selected === 'string' ? selected : String(selected);
    }
  } catch (e) {
    console.error('Failed to pick folder:', e);
  }
};

const clearContextFolder = () => {
  contextFolder.value = null;
};

// 获取用户消息的显示内容（去除已单独展示的文件路径前缀）
const userDisplayContent = (msg: Message): string => {
  if (!msg.content) {return '';}
  if (!msg.filePaths || msg.filePaths.length === 0) {return msg.content;}
  // 跳过文件路径行（用户不可见的原始内容）
  const lines = msg.content.split('\n');
  return lines.slice(msg.filePaths.length).join('\n').trimStart();
};

// 高亮搜索匹配文本
const highlightText = (text: string | null, query: string): string => {
  if (!text || !query.trim()) {return text || '';}
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(escapedQuery, 'gi');
  return text.replace(regex, '<mark class="bg-warning/30 text-inherit px-0.5 rounded">$&</mark>');
};

// 计算已完成任务数量
const completedTasksCount = computed(() => {
  return currentTasks.value.filter(t => t.status === 'completed').length;
});

// 计算会话统计
const sessionStats = computed(() => {
  const userMessages = messages.value.filter(m => m.role === 'user');
  const assistantMessages = messages.value.filter(m => m.role === 'assistant');
  
  const totalInputTokens = messages.value.reduce((sum, m) => sum + (m.tokens?.input || 0), 0);
  const totalOutputTokens = messages.value.reduce((sum, m) => sum + (m.tokens?.output || 0), 0);
  
  return {
    userCount: userMessages.length,
    assistantCount: assistantMessages.length,
    totalMessages: messages.value.length,
    totalTokens: totalInputTokens + totalOutputTokens,
    inputTokens: totalInputTokens,
    outputTokens: totalOutputTokens,
  };
});

// 搜索消息
const searchMessages = () => {
  if (!searchQuery.value.trim()) {
    filteredMessages.value = messages.value;
    return;
  }
  const query = searchQuery.value.toLowerCase();
  filteredMessages.value = messages.value.filter(msg => 
    msg.content?.toLowerCase().includes(query)
  );
};

// 清除搜索
const clearSearch = () => {
  searchQuery.value = '';
  filteredMessages.value = messages.value;
};

// 全局快捷键处理
const handleGlobalKeydown = (e: KeyboardEvent) => {
  // ESC: 关闭附件菜单或模型下拉菜单
  if (e.key === 'Escape') {
    chatInputRef.value?.closeDropdowns();
    return;
  }
  
  // Cmd/Ctrl + K: 新对话
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault();
    startNewChat();
    return;
  }
  
  // Cmd/Ctrl + S: 保存/导出当前会话
  if ((e.metaKey || e.ctrlKey) && e.key === 's') {
    e.preventDefault();
    exportSession();
    return;
  }
  
  // Home: 滚动到顶部
  if (e.key === 'Home' && messages.value.length > 0) {
    scrollToTop();
    return;
  }
  
  // End: 滚动到底部
  if (e.key === 'End' && messages.value.length > 0) {
    scrollToBottom();
    return;
  }
  
  // Escape: 如果正在编辑标题，取消编辑
  if (e.key === 'Escape' && isEditingTitle.value) {
    cancelEditTitle();
    return;
  }
  
  // Escape: 如果正在流式输出，停止
  if (e.key === 'Escape' && isStreaming.value) {
    abortChat();
    return;
  }
};

// 导出会话为 Markdown
const exportSession = () => {
  if (messages.value.length === 0) {return;}
  
  const title = currentSession.value?.title || '新对话';
  const timestamp = new Date().toISOString().split('T')[0];
  
  let markdown = `# ${title}\n\n`;
  markdown += `> 导出时间: ${timestamp}\n> 模型: ${currentSession.value?.model || 'unknown'}\n\n---\n\n`;
  
  for (const msg of messages.value) {
    if (msg.role === 'user') {
      markdown += `## 用户\n\n${msg.content || ''}\n\n`;
    } else if (msg.role === 'assistant') {
      markdown += `## Hermes\n\n${msg.content || ''}\n\n`;
      if (msg.toolCalls && msg.toolCalls.length > 0) {
        markdown += `**工具调用:**\n`;
        for (const tool of msg.toolCalls) {
          markdown += `- ${tool.name} (${tool.durationMs}ms)\n`;
        }
        markdown += '\n';
      }
    }
  }
  
  // 下载文件
  const blob = new Blob([markdown], { type: 'text/markdown' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${title.replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, '_')}_${timestamp}.md`;
  a.click();
  URL.revokeObjectURL(url);
};

// 清空当前会话消息
const clearMessages = () => {
  if (messages.value.length === 0) {return;}
  if (!confirm('确定清空所有消息？此操作不可撤销。')) {return;}
  messages.value = [];
  currentTasks.value = []; // 清空任务列表
};

// 滚动到顶部
const scrollToTop = () => {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = 0;
  }
};

// 滚动到底部（已有 scrollToBottom 函数）

// 注：deleteSession, deleteCurrentSession 已从 useSessionManager composable 导入
// 但需要包装以支持清理消息的逻辑

// 删除当前会话（包装 composable 的版本）
const deleteCurrentSessionLocal = async () => {
  await deleteCurrentSessionBase(() => {
    messages.value = [];
    thinkingText.value = '';
  });
};

// 删除指定会话（包装 composable 的版本）
const deleteSessionLocal = async (sessionId: string) => {
  await deleteSessionBase(sessionId, (deletedSessionId) => {
    if (currentSessionId.value === deletedSessionId) {
      messages.value = [];
      thinkingText.value = '';
    }
  });
};

// 标题编辑功能
const startEditTitle = () => {
  if (!currentSession.value) {return;}
  isEditingTitle.value = true;
  editingTitle.value = currentSession.value.title || '';
  nextTick(() => {
    titleInputRef.value?.focus();
  });
};

const cancelEditTitle = () => {
  isEditingTitle.value = false;
  editingTitle.value = '';
};

const saveTitle = async () => {
  if (!isEditingTitle.value || !currentSessionId.value) {return;}
  
  const newTitle = editingTitle.value.trim();
  if (!newTitle) {
    cancelEditTitle();
    return;
  }
  
  // 如果标题没有变化，直接取消编辑
  if (newTitle === currentSession.value?.title) {
    cancelEditTitle();
    return;
  }
  
  isEditingTitle.value = false;
  
  try {
    await invoke('agent_rename_session', {
      sessionId: currentSessionId.value,
      title: newTitle,
    });
    
    // 更新本地状态
    if (currentSession.value) {
      currentSession.value.title = newTitle;
    }
    // 更新会话列表中的标题
    const session = sessions.value.find(s => s.id === currentSessionId.value);
    if (session) {
      session.title = newTitle;
    }
  } catch (e) {
    console.error('Rename error:', e);
    // 恢复原标题
    editingTitle.value = currentSession.value?.title || '';
  }
};

const checkHermes = async () => {
  try {
    const result = await invoke<{ available: boolean; error: string | null }>('agent_check_available');
    hermesAvailable.value = result.available;
  } catch (e) {
    hermesAvailable.value = false;
  }
};

// 检测是否需要显示"回到底部"按钮
const checkScrollPosition = () => {
  if (messagesContainer.value) {
    const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value;
    // 当用户向上滚动超过 100px 时显示按钮
    const isNearBottom = scrollHeight - scrollTop - clientHeight < 100;
    showScrollToBottom.value = !isNearBottom && messages.value.length > 0;
    userScrolledUp.value = !isNearBottom;
  }
};

// Handle scroll event for messagesContainer (smart auto-scroll threshold)
const handleScroll = () => {
  const el = messagesContainer.value;
  if (!el) {return;}
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 60;
  userScrolledUp.value = !atBottom;
};

// Fast mode toggle
const toggleFastMode = async () => {
  fastMode.value = !fastMode.value;
  try {
    await invoke('hermes_set_config', { key: 'agent.service_tier', value: fastMode.value ? 'fast' : 'normal' });
  } catch (e) { /* ignore */ }
};

// Agent approval detection
const needsApproval = computed(() => {
  if (isStreaming.value) {return false;}
  const lastMsg = messages.value[messages.value.length - 1];
  if (!lastMsg || lastMsg.role !== 'assistant') {return false;}
  const content = lastMsg.content || '';
  return /⚠️.*dangerous|requires? (your )?approval|do you want (me )?to (proceed|continue|run|execute)/i.test(content);
});

const handleApproval = async (approved: boolean) => {
  await handleSend(approved ? '/approve' : '/deny', [], currentSession.value?.model || '');
};

// Lifecycle
onMounted(async () => {
  // 全局快捷键
  document.addEventListener('keydown', handleGlobalKeydown);
  
  // 全局点击监听 - 关闭下拉菜单
  document.addEventListener('click', (e) => {
    const target = e.target as Element | null;
    // 通过 ChatInput 的方法关闭下拉菜单
    chatInputRef.value?.closeDropdownsOnOutsideClick(target);
  });
  
  // 滚动监听 - 检测是否需要显示"回到底部"按钮
  if (messagesContainer.value) {
    messagesContainer.value.addEventListener('scroll', checkScrollPosition);
  }

  // Listen for agent-usage events to track cost
  unlistenUsageFn.value = await listen<{ prompt_tokens: number; completion_tokens: number; total_tokens: number; cost?: number; session_id: string | null }>('agent-usage', (event) => {
    if (event.payload?.session_id === currentSessionId.value) {
      lastUsage.value = {
        promptTokens: event.payload.prompt_tokens,
        completionTokens: event.payload.completion_tokens,
        cost: event.payload.cost,
      };
    }
  });

  // 使用 composable 设置流式事件监听
  await setupStreamingListeners();

  // 初始化
  await chatInputRef.value?.loadModels(); // 加载模型列表
  await checkHermes();
  await refreshSessions();
  await loadGitRepos(); // 加载 Git 仓库列表
  loadFavoriteFolders(); // 加载常用文件夹

  // 如果URL有sessionId参数，自动选择该会话
  const sessionIdFromQuery = route.query.sessionId as string;
  if (sessionIdFromQuery) {
    const session = sessions.value.find(s => s.id === sessionIdFromQuery);
    if (session) {
      selectSessionWithMessages(session);
    } else {
      // 会话不存在，尝试直接加载
      try {
        const result = await invoke<{ session_id: string; messages: Message[] }>('agent_get_session', {
          sessionId: sessionIdFromQuery,
        });
        currentSessionId.value = sessionIdFromQuery;
        currentSession.value = {
          id: sessionIdFromQuery,
          title: null,
          model: 'unknown',
          source: 'unknown',
          startedAt: Date.now() / 1000,
          endedAt: null,
          messageCount: result.messages.length,
          preview: '',
          profile: 'default',
        };
        messages.value = result.messages;
        scrollToBottom();
      } catch (e) {
        console.error('Failed to load session from query:', e);
      }
    }
  }

  // 自动聚焦输入框，方便立即开始对话
  chatInputRef.value?.focus();
});

onUnmounted(() => {
  // 清理流式事件监听（使用 composable）
  cleanupStreamingListeners();
  // 清理 usage listener
  unlistenUsageFn.value?.();
  unlistenUsageFn.value = null;
  // 移除快捷键监听
  document.removeEventListener('keydown', handleGlobalKeydown);
  // 移除滚动监听
  if (messagesContainer.value) {
    messagesContainer.value.removeEventListener('scroll', checkScrollPosition);
  }
  // 释放残存的图片预览 object URL
  const paths = chatInputRef.value?.attachedPaths;
  if (paths) {
    for (const item of paths) {
      if (item.previewUrl) {URL.revokeObjectURL(item.previewUrl);}
    }
  }
});

// Watch messages to update filteredMessages
watch(messages, () => {
  filteredMessages.value = messages.value;
}, { immediate: true });

// Watch searchQuery to filter messages
watch(searchQuery, () => {
  searchMessages();
});
</script>

<style scoped>
/* Markdown 内容样式 */
.markdown-content {
  line-height: 1.6;
  word-break: break-word;
}

.markdown-content :deep(p) {
  margin: 0.5em 0;
}

/* 行内代码样式 */
.markdown-content :deep(code:not(.hljs)) {
  background: rgba(0, 0, 0, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

/* 代码块包装器 */
.markdown-content :deep(.code-block-wrapper) {
  position: relative;
  margin: 0.8em 0;
  border-radius: 8px;
  overflow-x: auto;
}

/* 代码块头部 */
.markdown-content :deep(.code-header) {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  background: rgba(0, 0, 0, 0.08);
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

/* 语言标签 */
.markdown-content :deep(.code-lang) {
  font-size: 0.75em;
  color: var(--color-base-content, #666);
  opacity: 0.7;
  text-transform: uppercase;
  font-weight: 500;
}

/* 复制按钮 */
.markdown-content :deep(.copy-btn) {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  background: transparent;
  border: 1px solid rgba(0, 0, 0, 0.1);
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-base-content, #666);
}

.markdown-content :deep(.copy-btn:hover) {
  background: rgba(0, 0, 0, 0.05);
}

.markdown-content :deep(.copy-btn.copied) {
  background: rgba(76, 175, 80, 0.2);
  border-color: #4caf50;
  color: #4caf50;
}

/* 代码块内容 */
.markdown-content :deep(.code-block-wrapper pre) {
  margin: 0;
  padding: 12px;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 0;
  overflow-x: auto;
}

.markdown-content :deep(.code-block-wrapper pre code) {
  background: transparent;
  padding: 0;
  font-size: 0.85em;
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

/* 旧版 pre 样式（兼容无包装器的代码块） */
.markdown-content :deep(pre:not(.code-block-wrapper pre)) {
  background: rgba(0, 0, 0, 0.05);
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 0.8em 0;
}

.markdown-content :deep(pre:not(.code-block-wrapper pre) code) {
  background: transparent;
  padding: 0;
  font-size: 0.85em;
}

/* 代码高亮主题 - 适配 daisyUI 主题 */
.markdown-content :deep(.hljs-keyword),
.markdown-content :deep(.hljs-selector-tag) {
  color: #e91e63;
}

.markdown-content :deep(.hljs-string),
.markdown-content :deep(.hljs-attr) {
  color: #4caf50;
}

.markdown-content :deep(.hljs-number),
.markdown-content :deep(.hljs-literal) {
  color: #2196f3;
}

.markdown-content :deep(.hljs-comment) {
  color: #9e9e9e;
}

.markdown-content :deep(.hljs-function),
.markdown-content :deep(.hljs-title) {
  color: #ff9800;
}

.markdown-content :deep(.hljs-variable),
.markdown-content :deep(.hljs-params) {
  color: #673ab7;
}

/* 链接样式 */
.markdown-content :deep(a) {
  color: var(--color-primary);
  text-decoration: underline;
}

.markdown-content :deep(a:hover) {
  opacity: 0.8;
}

/* 列表样式 */
.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}

.markdown-content :deep(li) {
  margin: 0.3em 0;
}

/* 表格样式 */
.markdown-content :deep(table) {
  border-collapse: collapse;
  margin: 0.8em 0;
  width: 100%;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid var(--color-base-content, #ccc);
  padding: 6px 12px;
  text-align: left;
}

.markdown-content :deep(th) {
  background: rgba(0, 0, 0, 0.05);
  font-weight: 600;
}

/* 引用块样式 */
.markdown-content :deep(blockquote) {
  border-left: 3px solid var(--color-primary);
  padding-left: 1em;
  margin: 0.8em 0;
  color: var(--color-base-content);
  opacity: 0.8;
}

/* 标题样式 */
.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3),
.markdown-content :deep(h4) {
  margin: 1em 0 0.5em;
  font-weight: 600;
  line-height: 1.3;
}

.markdown-content :deep(h1) { font-size: 1.4em; border-bottom: 1px solid rgba(0,0,0,0.1); padding-bottom: 0.3em; }
.markdown-content :deep(h2) { font-size: 1.2em; }
.markdown-content :deep(h3) { font-size: 1.1em; }
.markdown-content :deep(h4) { font-size: 1em; }

/* 加粗和斜体 */
.markdown-content :deep(strong) {
  font-weight: 600;
}

.markdown-content :deep(em) {
  font-style: italic;
}

/* 分隔线 */
.markdown-content :deep(hr) {
  border: none;
  border-top: 1px solid rgba(0,0,0,0.1);
  margin: 1em 0;
}

/* 特殊警告框样式 */
.markdown-content :deep(.alert-box) {
  padding: 8px 12px;
  border-radius: 6px;
  margin: 0.8em 0;
  font-size: 0.9em;
}

.markdown-content :deep(.alert-important) {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #dc2626;
}

.markdown-content :deep(.alert-warning) {
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: #d97706;
}

.markdown-content :deep(.alert-note) {
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: #2563eb;
}

.markdown-content :deep(.alert-silent) {
  background: rgba(107, 114, 128, 0.1);
  border: 1px solid rgba(107, 114, 128, 0.3);
  color: #4b5563;
}

.markdown-content :deep(.alert-context) {
  background: rgba(168, 85, 247, 0.1);
  border: 1px solid rgba(168, 85, 247, 0.3);
  color: #7c3aed;
}

/* 思考中打字点动画 */
.typing-dot {
  animation: typingDot 1.4s infinite both;
  opacity: 0;
}

@keyframes typingDot {
  0% { opacity: 0; }
  50% { opacity: 1; }
  100% { opacity: 0; }
}
</style>