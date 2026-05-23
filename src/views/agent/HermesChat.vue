<template>
  <div class="flex h-full">
    <!-- 左侧会话列表 -->
    <SessionSidebar
      :sessions="sessions"
      :currentSessionId="currentSessionId"
      :loadingSessions="loadingSessions"
      :isSearching="isSearching"
      :searchResults="searchResults"
      :formatTime="formatTime"
      :sourceIcon="sourceIcon"
      :highlightSnippet="highlightSnippet"
      @refresh="refreshSessions"
      @newChat="startNewChat"
      @select="selectSession"
      @delete="deleteSession"
      @search="handleSessionSearch"
      @jumpToResult="jumpToSearchResult"
      @clearSearch="clearSessionSearch"
    />

    <!-- 右侧聊天区域 -->
    <div class="flex-1 flex flex-col">
      <!-- 聊天头部 -->
      <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10 bg-base-100">
        <div class="flex items-center gap-2 flex-1">
          <button class="btn btn-ghost btn-xs btn-circle" @click="router.back()" title="返回">
            <SvgIcon name="arrowLeft" size="14" />
          </button>
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
              {{ currentSession?.title || '新对话' }}
            </span>
            <button v-if="currentSession" class="btn btn-ghost btn-xs btn-square" @click="startEditTitle">
              <SvgIcon name="edit" size="12" />
            </button>
          </template>
          <span v-if="currentSession" class="badge badge-ghost badge-xs" :title="currentSession.model">
            {{ parseModelName(currentSession.model).name || currentSession.model }}
          </span>
          <!-- 会话统计 -->
          <span v-if="messages.length > 0" class="text-xs text-base-content/40">
            {{ sessionStats.totalMessages }} 条消息 · {{ sessionStats.totalTokens > 0 ? `${sessionStats.totalTokens} tokens` : '' }}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <!-- 停止按钮 - 流式输出时显示，更醒目 -->
          <button 
            v-if="isStreaming && !isAborting" 
            class="btn btn-error btn-xs gap-1 animate-pulse shadow-error/30 shadow-md" 
            @click="abortChat"
          >
            <SvgIcon name="stop" size="12" />
            停止
          </button>
          <!-- 停止中状态 -->
          <button 
            v-if="isAborting" 
            class="btn btn-xs gap-1 btn-disabled"
            disabled
          >
            <span class="loading loading-spinner loading-xs"></span>
            停止中...
          </button>
          <button v-if="currentSession && messages.length > 0" class="btn btn-ghost btn-xs" @click="exportSession" title="导出 (Cmd+S)">
            <SvgIcon name="download" size="12" />
          </button>
          <button v-if="messages.length > 0" class="btn btn-ghost btn-xs" @click="clearMessages" title="清空消息">
            <SvgIcon name="clear" size="12" />
          </button>
          <!-- 任务面板按钮 -->
          <button 
            v-if="currentTasks.length > 0" 
            class="btn btn-xs"
            :class="showTaskPanel ? 'btn-primary' : 'btn-ghost'"
            @click="showTaskPanel = !showTaskPanel"
            title="显示任务列表"
          >
            <SvgIcon name="checklist" size="12" />
            <span class="text-xs">{{ completedTasksCount }}/{{ currentTasks.length }}</span>
          </button>
          <button v-if="currentSession" class="btn btn-ghost btn-xs" @click="deleteCurrentSession" title="删除">
            <SvgIcon name="trash" size="12" />
          </button>
          <!-- 搜索按钮 -->
          <div v-if="messages.length > 0" class="relative">
            <input
              v-model="searchQuery"
              type="text"
              class="input input-xs input-bordered w-20 focus:w-40 transition-all"
              placeholder="搜索..."
            />
            <button
              v-if="searchQuery"
              class="btn btn-ghost btn-xs btn-square absolute right-0"
              @click="clearSearch"
            >
              <SvgIcon name="close" size="10" />
            </button>
          </div>
        </div>
      </div>

      <!-- 消息列表 -->
      <div ref="messagesContainer" class="flex-1 min-w-0 overflow-y-auto overflow-x-hidden px-4 py-2 space-y-1">
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
              :renderMarkdown="renderMarkdown"
              @toggle="toggleChildSessionExpand"
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
                :renderMarkdown="renderMarkdown"
                :getToolIcon="getToolIcon"
                :formatArgsSummary="formatArgsSummary"
                :formatToolResult="formatToolResult"
                :formatTodoResult="formatTodoResult"
                :isThinkingExpanded="isThinkingExpanded"
                :isToolCallExpanded="isToolCallExpanded"
                :onToggleThinking="toggleThinkingExpand"
                :onToggleToolCall="toggleToolCallExpand"
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

        <!-- 空状态 -->
        <div v-else class="flex flex-col items-center justify-center py-16 text-center">
          <SvgIcon name="chat" size="32" class="text-base-content/30" />
          <p class="mt-2 text-sm text-base-content/50">开始对话</p>
          <p class="text-xs text-base-content/40">输入消息与 Hermes Agent 交流</p>
          <!-- 快捷建议 -->
          <div class="mt-4 flex flex-wrap gap-2 justify-center">
            <button 
              class="btn btn-ghost btn-xs text-xs"
              @click="chatInputRef?.setInputText('帮我分析一下当前项目的结构')"
            >
              分析项目
            </button>
            <button 
              class="btn btn-ghost btn-xs text-xs"
              @click="chatInputRef?.setInputText('帮我写一个测试用例')"
            >
              写测试
            </button>
            <button 
              class="btn btn-ghost btn-xs text-xs"
              @click="chatInputRef?.setInputText('帮我重构这段代码')"
            >
              重构代码
            </button>
          </div>
        </div>
        
        <!-- 回到底部按钮 - 用户向上滚动时显示 -->
        <button 
          v-if="showScrollToBottom && messages.length > 0"
          class="fixed bottom-[140px] right-[30px] btn btn-circle btn-sm btn-primary shadow-lg opacity-90 hover:opacity-100 transition-opacity z-10"
          @click="scrollToBottom"
          title="回到底部"
        >
          <SvgIcon name="chevronDown" size="16" />
        </button>
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

    <!-- 右侧任务专栏 -->
    <div v-if="showTaskPanel && currentTasks.length > 0" class="w-72 border-l border-base-content/10 flex flex-col bg-base-100">
      <!-- 任务面板头部 -->
      <div class="flex items-center justify-between px-3 py-2 border-b border-base-content/10">
        <span class="text-sm font-semibold text-base-content">任务列表</span>
        <button class="btn btn-ghost btn-xs btn-square" @click="showTaskPanel = false" title="关闭">
          <SvgIcon name="close" size="12" />
        </button>
      </div>
      
      <!-- 任务列表 -->
      <div class="flex-1 overflow-y-auto px-3 py-2">
        <div v-for="task in currentTasks" :key="task.id" class="flex items-center gap-2 py-1.5 border-b border-base-content/5 last:border-b-0">
          <!-- 状态图标 -->
          <span :class="task.status === 'completed' ? 'text-success' : task.status === 'cancelled' ? 'text-base-content/30' : 'text-base-content/40'" class="text-xs">
            {{ task.status === 'completed' ? '✓' : task.status === 'cancelled' ? '✕' : '○' }}
          </span>
          <!-- 任务内容 -->
          <div class="flex-1 min-w-0">
            <span class="text-xs text-base-content truncate">{{ task.content }}</span>
          </div>
        </div>
      </div>
      
      <!-- 任务统计 -->
      <div class="px-3 py-2 border-t border-base-content/10 text-xs text-base-content/50">
        {{ completedTasksCount }}/{{ currentTasks.length }} 已完成
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'HermesChat' })
import { ref, reactive, computed, onMounted, onUnmounted, nextTick, watch, type Ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/core';
import { markedHighlight } from 'marked-highlight';
import javascript from 'highlight.js/lib/languages/javascript';
import { getTauriAPI } from '../../utils/tauri-api';
import type { GitRepo } from '../../types';
import python from 'highlight.js/lib/languages/python';
import json from 'highlight.js/lib/languages/json';
import sql from 'highlight.js/lib/languages/sql';
import typescript from 'highlight.js/lib/languages/typescript';
import bash from 'highlight.js/lib/languages/bash';
import xml from 'highlight.js/lib/languages/xml';
import yaml from 'highlight.js/lib/languages/yaml';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import { UserMessage, AssistantMessage, ChildSessionGroup, ToolCallCard, ChatInput, SessionSidebar } from '@/components/chat';

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

interface Session {
  id: string;
  title: string | null;
  model: string;
  source: string;
  startedAt?: number; // 可选，因为 Python bridge 可能不返回
  endedAt?: number | null; // 可选
  messageCount: number;
  preview: string;
  lastActive?: number; // 可选
  parentSessionId?: string | null; // subagent 会话标识
}

// 搜索结果（来自 Hermes FTS5 搜索）
interface SearchResult {
  sessionId: string;
  sessionTitle: string | null;
  messageId: string;
  role: string;
  snippet: string;
  content: string | null;
  timestamp: number | null;
  source: string;
  model: string;
}

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
const sessions = ref<Session[]>([]);
const searchResults = ref<SearchResult[]>([]); // 搜索结果
const isSearching = ref(false); // 搜索中状态
const currentSessionId = ref<string | null>(null);
const currentSession = ref<Session | null>(null);
const messages = ref<Message[]>([]);
const currentTasks = ref<TaskItem[]>([]); // 当前任务列表
const showTaskPanel = ref(true); // 是否显示任务面板
const chatInputRef = ref<InstanceType<typeof ChatInput> | null>(null);
const favoriteFolders = ref<string[]>([]); // 常用文件夹列表
const gitRepos = ref<GitRepo[]>([]); // Git 仓库列表
const FAVORITE_KEY = 'hermes-favorite-folders'; // localStorage key
const loadingSessions = ref(false);
const loadingMessages = ref(false);
const isAborting = ref(false); // 正在停止中
const showScrollToBottom = ref(false); // 是否显示回到底部按钮
const hermesAvailable = ref(false);

// 每个会话独立的状态（支持同时处理多个会话）
const streamingSessions = reactive<Record<string, boolean>>({});  // 各会话是否流式响应中
const thinkingTexts = reactive<Record<string, string>>({});       // 各会话的思考/工具文字
const sessionRoundEnded = reactive<Record<string, boolean>>({});  // 各会话的轮次结束标记
const sessionMessagesCache = reactive<Record<string, Message[]>>({});  // 各会话的消息缓存（流式输出时保存）

// 模板中使用的快捷访问（computed 自动解包）
const isStreaming = computed(() => !!currentSessionId.value && !!streamingSessions[currentSessionId.value]);
const thinkingText = computed({
  get: () => currentSessionId.value ? (thinkingTexts[currentSessionId.value] || '') : '',
  set: (val) => { if (currentSessionId.value) thinkingTexts[currentSessionId.value] = val; },
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

// 工具调用展开状态 (key: `${msgIdx}-${toolIdx}`)
const expandedToolCalls = ref<Set<string>>(new Set());

// 思考过程展开状态 (key: msgIdx)
const expandedThinking = ref<Set<number>>(new Set());

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
    if (msg.isChild) continue; // 子会话消息不单独处理
    
    items.push(msg);
    
    // 检查是否有子会话应该插入到这条消息之后
    // 子会话的开始时间应该 >= 当前消息时间，且 < 下一条主消息时间（如果有的话）
    const msgTime = msg.timestamp || 0;
    for (const [sessionId, group] of childSessionGroups) {
      if (insertedSessions.has(sessionId)) continue;
      
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

// 切换工具调用展开
const toggleToolCallExpand = (key: string) => {
  if (expandedToolCalls.value.has(key)) {
    expandedToolCalls.value.delete(key);
  } else {
    expandedToolCalls.value.add(key);
  }
};

// 检查是否展开
const isToolCallExpanded = (key: string): boolean => {
  return expandedToolCalls.value.has(key);
};

// 切换思考过程展开
const toggleThinkingExpand = (msgIdx: number) => {
  if (expandedThinking.value.has(msgIdx)) {
    expandedThinking.value.delete(msgIdx);
  } else {
    expandedThinking.value.add(msgIdx);
  }
};

// 检查思考过程是否展开
const isThinkingExpanded = (msgIdx: number): boolean => {
  return expandedThinking.value.has(msgIdx);
};

// Refs
const messagesContainer = ref<HTMLElement | null>(null);
const titleInputRef = ref<HTMLInputElement | null>(null);

// 标题编辑状态
const isEditingTitle = ref(false);
const editingTitle = ref('');

// 复制代码功能（全局函数）
const copyCode = (codeId: string) => {
  const codeElement = document.getElementById(codeId);
  if (codeElement) {
    const text = codeElement.textContent || '';
    navigator.clipboard.writeText(text).then(() => {
      // 显示复制成功提示
      const btn = codeElement.closest('.code-block-wrapper')?.querySelector('.copy-btn');
      if (btn) {
        btn.classList.add('copied');
        setTimeout(() => btn.classList.remove('copied'), 2000);
      }
    });
  }
};
// 挂载到 window 以便 onclick 调用
if (typeof window !== 'undefined') {
  (window as any).copyCode = copyCode;
}

// Event listeners
let unlistenDelta: UnlistenFn | null = null;
let unlistenThinking: UnlistenFn | null = null;
let unlistenToolStart: UnlistenFn | null = null;
let unlistenToolComplete: UnlistenFn | null = null;
let unlistenError: UnlistenFn | null = null;
let unlistenDone: UnlistenFn | null = null;

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

// 处理 ChatInput 发送事件
interface PathItem {
  path: string;
  type: 'file' | 'folder';
  name: string;
  previewUrl?: string;
}

const handleSend = async (text: string, paths: PathItem[], model: string) => {
  if (!text.trim()) return;

  // 如果正在处理，先打断当前处理
  if (isStreaming.value) {
    await abortChat();
    await new Promise(resolve => setTimeout(resolve, 200));
    if (isStreaming.value) {
      void agentLog('[handleSend] abort 后状态仍为 streaming，强制重置');
      if (currentSessionId.value) streamingSessions[currentSessionId.value] = false;
    }
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
  if (sid) streamingSessions[sid] = true;
  if (sid) thinkingTexts[sid] = '';
  if (sid) sessionRoundEnded[sid] = false;

  try {
    const modelToUse = model || null;
    
    const result = await invoke<{ response: string; session_id: string; message_count: number }>('agent_chat', {
      message: fullText,
      sessionId: currentSessionId.value,
      model: modelToUse,
    });

    if (result.session_id && !currentSessionId.value) {
      currentSessionId.value = result.session_id;
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
        };
      } catch (e) {
        console.warn('Auto-title failed:', e);
      }
      refreshSessions();
    }

    thinkingText.value = '';
    if (currentSessionId.value) sessionRoundEnded[currentSessionId.value] = false;
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

  if (currentSessionId.value) streamingSessions[currentSessionId.value] = false;
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

// 删除常用文件夹
const removeFavoriteFolder = (folder: string) => {
  favoriteFolders.value = favoriteFolders.value.filter(f => f !== folder);
  saveFavoriteFolders();
};

// 加载常用文件夹
const loadFavoriteFolders = () => {
  try {
    const saved = localStorage.getItem(FAVORITE_KEY);
    if (saved) {
      favoriteFolders.value = JSON.parse(saved);
    }
  } catch {
    favoriteFolders.value = [];
  }
};

// 保存常用文件夹
const saveFavoriteFolders = () => {
  try {
    localStorage.setItem(FAVORITE_KEY, JSON.stringify(favoriteFolders.value));
  } catch {
    // localStorage 不可用
  }
};

// 加载 Git 仓库列表
const loadGitRepos = async () => {
  try {
    const api = getTauriAPI();
    const res = await api.getGitRepos();
    gitRepos.value = res?.data || [];
  } catch (e) {
    console.error('加载 Git 仓库列表失败:', e);
    gitRepos.value = [];
  }
};

// Event listeners
const searchSessions = async (query: string) => {
  const trimmedQuery = query.trim();
  if (!trimmedQuery) {
    searchResults.value = [];
    return;
  }
  
  isSearching.value = true;
  try {
    const result = await invoke<{ results: SearchResult[]; total: number; query: string }>('agent_search_sessions', {
      query: trimmedQuery,
      limit: 20,
    });
    searchResults.value = result.results;
  } catch (e) {
    console.error('Search failed:', e);
    searchResults.value = [];
  } finally {
    isSearching.value = false;
  }
};

// 搜索防抖
let searchDebounceTimer: number | null = null;

// 处理 SessionSidebar 的搜索事件
const handleSessionSearch = (query: string) => {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer);
  }
  if (query.trim()) {
    searchDebounceTimer = window.setTimeout(() => {
      searchSessions(query);
    }, 300);
  } else {
    searchResults.value = [];
  }
};

// 清空搜索
const clearSessionSearch = () => {
  searchResults.value = [];
};

const sourceIcon = (source: string) => {
  const icons: Record<string, string> = {
    cli: 'terminal',
    feishu: 'message',
    telegram: 'message',
    discord: 'message',
    slack: 'message',
    cron: 'clock',
  };
  return icons[source] || 'chat';
};

// 工具图标映射
const toolIconMap: Record<string, { icon: string; color: string }> = {
  // 搜索类
  'search_files': { icon: 'search', color: 'text-info' },
  'web_search': { icon: 'search', color: 'text-info' },
  'browser_*': { icon: 'browser', color: 'text-info' },
  
  // 文件操作
  'read_file': { icon: 'file', color: 'text-success' },
  'write_file': { icon: 'fileEdit', color: 'text-warning' },
  'patch': { icon: 'tool', color: 'text-warning' },
  
  // 终端/代码
  'terminal': { icon: 'terminal', color: 'text-error' },
  'execute_code': { icon: 'code', color: 'text-primary' },
  
  // Agent/技能
  'delegate_task': { icon: 'bot', color: 'text-info' },
  'skill_view': { icon: 'skill', color: 'text-secondary' },
  'skill_manage': { icon: 'skill', color: 'text-secondary' },
  'skills_list': { icon: 'list', color: 'text-secondary' },
  
  // 会话/记忆
  'session_search': { icon: 'history', color: 'text-accent' },
  'memory': { icon: 'brain', color: 'text-accent' },
  
  // 浏览器操作
  'browser_navigate': { icon: 'browser', color: 'text-info' },
  'browser_click': { icon: 'mouse', color: 'text-info' },
  'browser_snapshot': { icon: 'camera', color: 'text-info' },
  'browser_vision': { icon: 'eye', color: 'text-info' },
  
  // Cron
  'cronjob': { icon: 'clock', color: 'text-warning' },
  
  // 其他
  'clarify': { icon: 'question', color: 'text-warning' },
  'todo': { icon: 'checklist', color: 'text-success' },
  'image_generate': { icon: 'image', color: 'text-secondary' },
  'text_to_speech': { icon: 'audio', color: 'text-secondary' },
  'vision_analyze': { icon: 'eye', color: 'text-info' },
  'send_message': { icon: 'send', color: 'text-success' },
};

// 获取工具图标信息
const getToolIcon = (toolName: string): { icon: string; color: string } => {
  // 精确匹配
  if (toolIconMap[toolName]) {
    return toolIconMap[toolName];
  }
  
  // 通配符匹配 (browser_*)
  for (const [pattern, info] of Object.entries(toolIconMap)) {
    if (pattern.endsWith('*') && toolName.startsWith(pattern.slice(0, -1))) {
      return info;
    }
  }
  
  // 默认
  return { icon: 'tool', color: 'text-warning' };
};

// 格式化工具参数摘要（显示关键参数的一行）
const formatArgsSummary = (args: Record<string, unknown>): string => {
  if (!args || typeof args !== 'object') return '';
  
  // 优先显示的关键参数名
  const priorityKeys = ['path', 'url', 'message', 'query', 'command', 'file', 'text', 'pattern', 'name', 'target'];
  
  for (const key of priorityKeys) {
    if (args[key]) {
      const value = String(args[key]);
      return `${key}: ${value.length > 200 ? value.slice(0, 200) + '...' : value}`;
    }
  }
  
  // 没有优先参数，显示第一个参数
  const firstKey = Object.keys(args)[0];
  if (firstKey) {
    const value = String(args[firstKey]);
    return `${firstKey}: ${value.length > 200 ? value.slice(0, 200) + '...' : value}`;
  }
  
  return '';
};

// 格式化 todo 工具返回的任务列表为友好的 HTML
const formatTodoResult = (result: string): string => {
  try {
    // 尝试解析 JSON
    const parsed = JSON.parse(result);
    
    // 提取任务数组：支持两种格式
    // 1. 直接数组 [{id, content, status}]
    // 2. 对象格式 {todos: [...], summary: {...}}
    let tasks: Array<{ id: string; content: string; status?: string }> = [];
    let summary = null;
    
    if (Array.isArray(parsed) && parsed.length > 0 && parsed[0].id && parsed[0].content) {
      // 直接数组格式
      tasks = parsed;
    } else if (parsed.todos && Array.isArray(parsed.todos)) {
      // 对象格式 {todos, summary}
      tasks = parsed.todos;
      summary = parsed.summary;
    }
    
    // 如果成功提取到任务列表
    if (tasks.length > 0) {
      const tasksHtml = tasks.map((task: { id: string; content: string; status?: string }) => {
        const status = task.status || 'pending';
        const isCompleted = status === 'completed';
        const isCancelled = status === 'cancelled';
        const symbol = isCompleted ? '✓' : isCancelled ? '✕' : '○';
        const colorClass = isCompleted ? 'text-success' : isCancelled ? 'text-base-content/30' : 'text-base-content/40';
        return `<div class="flex items-center gap-2 py-0.5">
          <span class="${colorClass} text-xs">${symbol}</span>
          <span class="text-xs flex-1">${task.content}</span>
        </div>`;
      }).join('');
      
      // 如果有汇总信息，显示在底部
      let summaryHtml = '';
      if (summary) {
        summaryHtml = `<div class="flex items-center gap-3 mt-2 pt-1 border-t border-base-content/10 text-xs text-base-content/50">
          <span>总计 ${summary.total}</span>
          <span class="text-warning">进行中 ${summary.in_progress}</span>
          <span class="text-base-content/40">待处理 ${summary.pending}</span>
          <span class="text-success">已完成 ${summary.completed}</span>
          ${summary.cancelled > 0 ? `<span class="text-base-content/30">已取消 ${summary.cancelled}</span>` : ''}
        </div>`;
      }
      
      return `<div class="space-y-0">${tasksHtml}${summaryHtml}</div>`;
    }
    
    // 其他 JSON 格式，美化显示
    return `<pre class="text-xs whitespace-pre-wrap">${JSON.stringify(parsed, null, 2)}</pre>`;
  } catch {
    // 非 JSON，直接显示
    return `<div class="text-xs whitespace-pre-wrap">${result}</div>`;
  }
};

// 格式化子Agent结果（delegate_task）
const formatDelegateResult = (result: string): string => {
  try {
    const parsed = JSON.parse(result);
    
    // delegate_task 返回格式: { results: [...], total_duration_seconds }
    if (parsed.results && Array.isArray(parsed.results)) {
      const htmlParts: string[] = [];
      
      for (const task of parsed.results) {
        const statusIcon = task.status === 'completed' ? '✓' : 
                          task.status === 'error' ? '✕' : 
                          task.status === 'timeout' ? '⏱' : '○';
        const statusColor = task.status === 'completed' ? 'text-success' : 
                           task.status === 'error' ? 'text-error' : 
                           task.status === 'timeout' ? 'text-warning' : 'text-base-content/40';
        
        htmlParts.push(`
          <div class="py-1.5 border-b border-base-content/10 last:border-b-0">
            <div class="flex items-center gap-2 mb-1">
              <span class="${statusColor} text-xs font-bold">${statusIcon}</span>
              <span class="text-xs text-base-content/60">任务 ${task.task_index + 1}</span>
              <span class="text-xs text-base-content/40">${task.duration_seconds?.toFixed(1) || '-'}s</span>
              <span class="text-xs text-base-content/30">${task.model || '-'}</span>
            </div>
            <div class="text-xs text-base-content whitespace-pre-wrap max-h-32 overflow-auto">${task.summary || '（无摘要）'}</div>
          </div>
        `);
        
        // 如果有工具调用记录，显示简要
        if (task.tool_trace && task.tool_trace.length > 0) {
          const toolsBrief = task.tool_trace.slice(0, 5).map((t: { tool: string; status: string }) => 
            `<span class="text-xs text-base-content/40">${t.tool}</span>`
          ).join(' → ');
          htmlParts.push(`
            <div class="px-2 py-0.5 text-xs text-base-content/30">
              调用链: ${toolsBrief}${task.tool_trace.length > 5 ? ' ...' : ''}
            </div>
          `);
        }
      }
      
      // 总耗时
      if (parsed.total_duration_seconds) {
        htmlParts.push(`
          <div class="pt-1 text-xs text-base-content/40">
            总耗时: ${parsed.total_duration_seconds.toFixed(1)}s
          </div>
        `);
      }
      
      return `<div class="space-y-0">${htmlParts.join('')}</div>`;
    }
    
    // 其他 JSON 格式
    return `<pre class="text-xs whitespace-pre-wrap">${JSON.stringify(parsed, null, 2)}</pre>`;
  } catch {
    return `<div class="text-xs whitespace-pre-wrap">${result}</div>`;
  }
};

// 格式化工具结果（根据工具类型选择渲染方式）
const formatToolResult = (toolName: string, result: string): string => {
  // todo 工具特殊渲染
  if (toolName === 'todo') {
    return formatTodoResult(result);
  }
  
  // delegate_task 工具特殊渲染
  if (toolName === 'delegate_task') {
    return formatDelegateResult(result);
  }
  
  // 其他工具，默认显示
  // 尝试解析为 JSON 并美化
  try {
    const parsed = JSON.parse(result);
    return `<pre class="text-xs whitespace-pre-wrap overflow-auto max-h-48">${JSON.stringify(parsed, null, 2)}</pre>`;
  } catch {
    return `<div class="text-xs whitespace-pre-wrap overflow-auto max-h-48">${result}</div>`;
  }
};

// Markdown 渲染缓存 — key 为消息原文，value 为渲染后的 HTML
const markdownCache = new Map<string, string>();
const MAX_CACHE = 500;

// 自定义渲染器单例（不依赖输入，只需创建一次）
const markdownRenderer = new marked.Renderer();
markdownRenderer.code = function({ text: code, lang }: { text: string; lang?: string }): string {
  const language = lang || 'plaintext';
  
  // 检测内容是否已经被 hljs 高亮过（包含 hljs-xxx 类名）
  if (code.includes('class="hljs-') || code.includes('class=\'hljs-')) {
    // 已经高亮过，直接返回，避免双重编码
    const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;
    return `<div class="code-block-wrapper">
      <div class="code-header">
        <span class="code-lang">${language}</span>
        <button class="copy-btn" onclick="copyCode('${codeId}')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
        </button>
      </div>
      <pre><code id="${codeId}" class="hljs">${code}</code></pre>
    </div>`;
  }
  
  const highlighted = language && hljs.getLanguage(language) 
    ? hljs.highlight(code, { language }).value 
    : hljs.highlightAuto(code).value;
  
  const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;
  
  return `<div class="code-block-wrapper">
    <div class="code-header">
      <span class="code-lang">${language}</span>
      <button class="copy-btn" onclick="copyCode('${codeId}')" title="复制代码">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
      </button>
    </div>
    <pre><code id="${codeId}" class="hljs">${highlighted}</code></pre>
  </div>`;
};

// Markdown 渲染函数 - 添加代码块复制按钮和特殊格式处理
const renderMarkdown = (text: string | null): string => {
  if (!text) return '';
  // 缓存命中 — 避免重复解析已渲染过的消息
  const cached = markdownCache.get(text);
  if (cached) return cached;
  try {
    // 预处理：处理特殊格式的警告框
    let processedText = text
      .replace(/^\[IMPORTANT:\s*([^\]]+)\]/gm, '<div class="alert-box alert-important">⚠️ <strong>重要:</strong> $1</div>')
      .replace(/^\[WARNING:\s*([^\]]+)\]/gm, '<div class="alert-box alert-warning">⚠️ <strong>警告:</strong> $1</div>')
      .replace(/^\[NOTE:\s*([^\]]+)\]/gm, '<div class="alert-box alert-note">📝 <strong>注意:</strong> $1</div>')
      .replace(/^\[SILENT\]/gm, '<div class="alert-box alert-silent">🔇 <strong>静默模式</strong></div>')
      .replace(/^\[CONTEXT:/gm, '<div class="alert-box alert-context">📋 <strong>上下文压缩摘要</strong><br>');

    // 直接传 renderer 给 marked.parse，不修改全局 marked 配置
    const html = marked.parse(processedText, {
      renderer: markdownRenderer,
      breaks: true,
      gfm: true,
      async: false,
    }) as string;
    const result = DOMPurify.sanitize(html, {
      ADD_ATTR: ['target', 'onclick', 'id', 'title'],
      ADD_TAGS: ['button', 'svg', 'rect', 'path', 'div'],
    });

    // LRU 缓存：淘汰最旧的条目
    if (markdownCache.size >= MAX_CACHE) {
      const firstKey = markdownCache.keys().next().value;
      if (firstKey) markdownCache.delete(firstKey);
    }
    markdownCache.set(text, result);
    return result;
  } catch (e) {
    console.error('[renderMarkdown] Error:', e);
    return text;
  }
};

const formatTime = (ts: number | null | undefined) => {
  if (!ts) return '';
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
  if (!ts) return '';
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
const refreshSessions = async () => {
  loadingSessions.value = true;
  try {
    const result = await invoke<{ sessions: Session[]; total: number }>('agent_list_sessions', { limit: 50 });
    // 按 lastActive 降序排序（最近活跃的在前）
    sessions.value = result.sessions.sort((a, b) => {
      const aTime = a.lastActive || a.startedAt || 0;
      const bTime = b.lastActive || b.startedAt || 0;
      return bTime - aTime;
    });
  } catch (e) {
    console.error('Failed to list sessions:', e);
  }
  loadingSessions.value = false;
};

// 高亮搜索关键词
const highlightSnippet = (snippet: string, query: string) => {
  if (!query) return snippet;
  // FTS5 already marks matches with >>>...<<<
  // Convert to <mark> tags
  return snippet
    .replace(/>>>/g, '<mark class="bg-warning/30 text-warning px-0.5 rounded">')
    .replace(/<<</g, '</mark>');
};

// 点击搜索结果，跳转到对应会话和消息
const jumpToSearchResult = async (result: SearchResult) => {
  // 清空搜索，回到正常模式
  clearSessionSearch();
  
  // 查找会话是否在列表中
  const session = sessions.value.find(s => s.id === result.sessionId);
  if (session) {
    await selectSession(session);
  } else {
    // 会话不在列表中，需要加载
    try {
      const sessionResult = await invoke<{ sessionId: string; messages: RawMessage[] }>('agent_get_session', {
        sessionId: result.sessionId,
      });
      // 创建临时 Session 对象
      const tempSession: Session = {
        id: result.sessionId,
        title: result.sessionTitle,
        model: result.model,
        source: result.source,
        messageCount: sessionResult.messages.length,
        preview: '',
        lastActive: result.timestamp || Date.now() / 1000,
      };
      sessions.value.unshift(tempSession);
      await selectSession(tempSession);
    } catch (e) {
      console.error('Failed to load session:', e);
    }
  }
};

const selectSession = async (session: Session) => {
  // 切换会话：不打断旧会话的流式处理，只切显示
  // 旧会话继续在后台流式处理，事件更新 per-session 状态
  currentSessionId.value = session.id;
  currentSession.value = session;
  
  // 检查是否有缓存（流式输出中的会话）
  const cached = sessionMessagesCache[session.id];
  const isStreamingSession = !!streamingSessions[session.id];
  
  if (cached && cached.length > 0 && isStreamingSession) {
    // 有缓存且正在流式输出，直接用缓存
    messages.value = [...cached];
    loadingMessages.value = false;
    return;
  }
  
  // 没有缓存或流式已结束，从数据库加载
  loadingMessages.value = true;
  messages.value = [];

  try {
    // 直接从 SQLite 读取（毫秒级），不再通过 Python bridge
    const result = await invoke<{ success: boolean; messages: RawMessage[]; sessionId: string }>('agent_list_messages', {
      sessionId: session.id,
    });
    
    if (!result.success || !result.messages) {
      console.error('Failed to load messages');
      messages.value = [];
      return;
    }
    
    // 处理消息：关联 tool_calls 和 tool 结果
    const processedMessages: Message[] = [];
    const toolResultsMap = new Map<string, string>();
    
    // 先收集所有 tool 消息的结果
    for (const m of result.messages) {
      if (m.role === 'tool' && m.toolCallId) {
        toolResultsMap.set(m.toolCallId, m.content || '');
      }
    }
    
    // 再处理 user 和 assistant 消息
    for (const m of result.messages) {
      if (m.role === 'tool') continue; // tool 消息不单独显示，合并到 assistant
      
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
            
            if (!msg.toolCalls) msg.toolCalls = [];
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
    console.error('Failed to get session:', e);
  }

  loadingMessages.value = false;
  scrollToBottom();
};

const startNewChat = () => {
  currentSessionId.value = null;
  currentSession.value = null;
  messages.value = [];
  chatInputRef.value?.clear();
  thinkingText.value = '';
  if (currentSessionId.value) streamingSessions[currentSessionId.value] = false;
};

// 自动生成会话标题（基于第一条用户消息）
const generateSessionTitle = (firstMessage: string): string => {
  // 截取前30个字符作为标题
  let title = firstMessage.trim().slice(0, 30);
  // 如果截断，添加省略号
  if (firstMessage.trim().length > 30) {
    title += '...';
  }
  return title;
};

// 重试发送消息
const retryMessage = async (retryContent: string) => {
  if (!retryContent.trim()) return;

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
  if (!isStreaming.value || isAborting.value) return;
  
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
    if (currentSessionId.value) streamingSessions[currentSessionId.value] = false;
    isAborting.value = false;
    if (currentSessionId.value) sessionRoundEnded[currentSessionId.value] = false;
    thinkingText.value = '';
  }, 100);
};

// 复制消息内容
const copyMessageContent = async (content: string | null) => {
  if (!content) return;
  try {
    await navigator.clipboard.writeText(content);
    // 可选：显示复制成功提示（用 toast 或临时状态）
  } catch (e) {
    console.error('Copy failed:', e);
  }
};

// 获取用户消息的显示内容（去除已单独展示的文件路径前缀）
const userDisplayContent = (msg: Message): string => {
  if (!msg.content) return '';
  if (!msg.filePaths || msg.filePaths.length === 0) return msg.content;
  // 跳过文件路径行（用户不可见的原始内容）
  const lines = msg.content.split('\n');
  return lines.slice(msg.filePaths.length).join('\n').trimStart();
};

// 高亮搜索匹配文本
const highlightText = (text: string | null, query: string): string => {
  if (!text || !query.trim()) return text || '';
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
  if (messages.value.length === 0) return;
  
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
  if (messages.value.length === 0) return;
  if (!confirm('确定清空所有消息？此操作不可撤销。')) return;
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

const deleteCurrentSession = async () => {
  if (!currentSessionId.value) return;

  // 简单确认对话框
  if (!confirm('确定要删除当前会话吗？此操作不可撤销。')) return;

  try {
    await invoke('agent_delete_session', { sessionId: currentSessionId.value });
    sessions.value = sessions.value.filter(s => s.id !== currentSessionId.value);
    startNewChat();
  } catch (e) {
    console.error('Delete error:', e);
  }
};

// 删除指定会话（从列表直接删除）
const deleteSession = async (sessionId: string) => {
  if (!sessionId) return;

  // 简单确认对话框
  if (!confirm('确定要删除该会话吗？')) return;

  try {
    await invoke('agent_delete_session', { sessionId });
    sessions.value = sessions.value.filter(s => s.id !== sessionId);
    // 如果删除的是当前会话，清空消息
    if (currentSessionId.value === sessionId) {
      startNewChat();
    }
  } catch (e) {
    console.error('Delete error:', e);
  }
};

// 标题编辑功能
const startEditTitle = () => {
  if (!currentSession.value) return;
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
  if (!isEditingTitle.value || !currentSessionId.value) return;
  
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

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
      showScrollToBottom.value = false;
    }
  });
};

// 检测是否需要显示"回到底部"按钮
const checkScrollPosition = () => {
  if (messagesContainer.value) {
    const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value;
    // 当用户向上滚动超过 100px 时显示按钮
    const isNearBottom = scrollHeight - scrollTop - clientHeight < 100;
    showScrollToBottom.value = !isNearBottom && messages.value.length > 0;
  }
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
  
// 监听流式事件
  unlistenDelta = await listen<{ text: string | null; session_id: string | null }>('agent-delta', (event) => {
    const eventSid = event.payload?.session_id;
    void agentLog('[agent-delta] 收到事件: ' + JSON.stringify(event.payload?.text?.slice(0, 50)) + ' session_id: ' + eventSid);
    
    // 必须有 session_id 才处理
    if (!eventSid) return;
    
    // 更新该会话的状态
    if (event.payload?.text) thinkingTexts[eventSid] = '';
    
    // 获取该会话的消息缓存（优先用缓存，支持后台会话）
    let sessionMsgs = sessionMessagesCache[eventSid];
    if (!sessionMsgs) {
      // 如果当前会话没有缓存，从 messages.value 复制（当前视图）
      if (eventSid === currentSessionId.value) {
        sessionMessagesCache[eventSid] = [...messages.value];
        sessionMsgs = sessionMessagesCache[eventSid];
      } else {
        // 非当前会话，初始化空数组
        sessionMessagesCache[eventSid] = [];
        sessionMsgs = sessionMessagesCache[eventSid];
      }
    }
    
    if (event.payload?.text) {
      // 查找最后一个 assistant 消息（在缓存中）
      const messagesCopy = [...sessionMsgs].reverse();
      let currentMsg: Message | undefined = messagesCopy.find((m: Message) => m.role === 'assistant');
      
      // 检查最后一条消息是否是 user（刚发送的），如果是，需要创建新 assistant 消息
      const lastMsg = sessionMsgs[sessionMsgs.length - 1];
      const needsNewMsg = lastMsg?.role === 'user';
      
      // 检查是否已有空内容的 assistant 消息（由 tool_start 创建），避免重复创建
      const hasEmptyAssistant = currentMsg && !currentMsg.content && currentMsg.toolCalls && currentMsg.toolCalls.length > 0;
      
      void agentLog('[agent-delta] session: ' + eventSid + 
        ' 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') +
        ' lastAssistantRoundEnded: ' + !!sessionRoundEnded[eventSid] +
        ' 最后一条: ' + (lastMsg?.role || 'none') +
        ' needsNewMsg: ' + needsNewMsg +
        ' hasEmptyAssistant: ' + hasEmptyAssistant);
      
      // 如果没有 assistant 消息，或上一轮已结束，或最后一条是 user（需要新消息），创建新消息
      // 但如果已有空内容的 assistant 消息（由 tool_start 创建），则复用
      const roundEnded = !!sessionRoundEnded[eventSid];
      if (!currentMsg || (roundEnded && !hasEmptyAssistant) || needsNewMsg) {
        const newMsg: Message = {
          role: 'assistant',
          content: '',
          timestamp: Date.now() / 1000,
          toolName: null,
          toolCalls: [],
        };
        sessionMsgs.push(newMsg);
        // 从缓存获取 Vue 的 reactive proxy
        currentMsg = sessionMsgs[sessionMsgs.length - 1];
        sessionRoundEnded[eventSid] = false;
        void agentLog('[agent-delta] 创建新 assistant 消息, 缓存 length: ' + sessionMsgs.length);
      } else if (hasEmptyAssistant) {
        // 复用已有的空内容 assistant 消息
        sessionRoundEnded[eventSid] = false;
        void agentLog('[agent-delta] 复用已有空 assistant 消息');
      }
      
      // 添加 delta 内容
      if (currentMsg) {
        currentMsg.content = (currentMsg.content || '') + event.payload.text;
      }
      
      // 如果是当前会话，同步更新 messages.value
      if (eventSid === currentSessionId.value) {
        messages.value = [...sessionMsgs];
        scrollToBottom();
      }
    }
  });

  unlistenToolStart = await listen<{ id?: string; name: string; args: unknown; session_id: string | null }>('agent-tool-start', (event) => {
    const eventSid = event.payload?.session_id;
    void agentLog('[agent-tool-start] 收到事件: ' + JSON.stringify(event.payload) + ' session_id: ' + eventSid);
    
    // 必须有 session_id 才处理
    if (!eventSid) return;
    
    // 更新该会话的状态
    if (event.payload.name) {
      const isSubAgent = event.payload.name === 'delegate_task';
      thinkingTexts[eventSid] = isSubAgent ? '🤖 启动子 Agent 处理任务...' : `🔧 调用工具: ${event.payload.name}...`;
    }
    sessionRoundEnded[eventSid] = false;
    
    // 获取该会话的消息缓存
    let sessionMsgs = sessionMessagesCache[eventSid];
    if (!sessionMsgs) {
      if (eventSid === currentSessionId.value) {
        sessionMessagesCache[eventSid] = [...messages.value];
        sessionMsgs = sessionMessagesCache[eventSid];
      } else {
        sessionMessagesCache[eventSid] = [];
        sessionMsgs = sessionMessagesCache[eventSid];
      }
    }
    
    // 工具开始
    const toolId = event.payload.id;
    const toolName = event.payload.name;
    const isSubAgent = toolName === 'delegate_task';
    
    // 获取当前消息（如果没有 assistant 消息，创建一个）
    const messagesCopy = [...sessionMsgs].reverse();
    let currentMsg: Message | undefined = messagesCopy.find((m: Message) => m.role === 'assistant');
    
    // 检查最后一条消息是否是 user（刚发送的），如果是，需要创建新 assistant 消息
    const lastMsg = sessionMsgs[sessionMsgs.length - 1];
    const needsNewMsg = lastMsg?.role === 'user';
    
    void agentLog('[agent-tool-start] session: ' + eventSid +
      ' 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') +
      ' 最后一条: ' + (lastMsg?.role || 'none') +
      ' needsNewMsg: ' + needsNewMsg + ' toolId: ' + (toolId || 'none'));
    
    // 重置轮结束标志（新的工具调用开始）
    sessionRoundEnded[eventSid] = false;
    
    if (!currentMsg || needsNewMsg) {
      const newMsg: Message = {
        role: 'assistant',
        content: '',
        timestamp: Date.now() / 1000,
        toolName: null,
        toolCalls: [],
      };
      sessionMsgs.push(newMsg);
      currentMsg = sessionMsgs[sessionMsgs.length - 1];
      void agentLog('[agent-tool-start] 创建新 assistant 消息, 缓存 length: ' + sessionMsgs.length);
    }
    
    // 确保 toolCalls 数组存在
    if (!currentMsg.toolCalls) {
      currentMsg.toolCalls = [];
    }
    
    // 添加工具调用
    currentMsg.toolCalls.push({
      id: toolId,
      name: toolName,
      args: event.payload.args as Record<string, unknown> || {},
      durationMs: 0,
      isSubAgent,
      status: 'running',
    });
    void agentLog('[agent-tool-start] 添加工具调用: ' + toolName + ' id: ' + (toolId || 'none') + ' toolCalls.length: ' + currentMsg.toolCalls.length);
    
    // 如果是当前会话，同步更新 messages.value
    if (eventSid === currentSessionId.value) {
      messages.value = [...sessionMsgs];
      // 显示提示
      if (isSubAgent) {
        thinkingText.value = '🤖 启动子 Agent 处理任务...';
      } else {
        thinkingText.value = `🔧 调用工具: ${toolName}...`;
      }
      scrollToBottom();
    }
  });

  unlistenToolComplete = await listen<{ id?: string; name: string; result: string | null; duration_ms: number; session_id: string | null }>('agent-tool-complete', (event) => {
    const eventSid = event.payload?.session_id;
    void agentLog('[agent-tool-complete] 收到事件: ' + JSON.stringify({id: event.payload.id, name: event.payload.name, duration_ms: event.payload.duration_ms, session_id: eventSid}));
    
    // 必须有 session_id 才处理
    if (!eventSid) return;
    
    // 更新该会话的状态
    sessionRoundEnded[eventSid] = true;
    
    // 获取该会话的消息缓存
    let sessionMsgs = sessionMessagesCache[eventSid];
    if (!sessionMsgs) {
      if (eventSid === currentSessionId.value) {
        sessionMessagesCache[eventSid] = [...messages.value];
        sessionMsgs = sessionMessagesCache[eventSid];
      } else {
        // 非当前会话，没有缓存就跳过（这种情况不应该发生）
        return;
      }
    }
    
    // 获取当前 assistant 消息
    const messagesCopy = [...sessionMsgs].reverse();
    const currentMsg = messagesCopy.find((m: Message) => m.role === 'assistant');
    void agentLog('[agent-tool-complete] session: ' + eventSid + 
      ' 当前 assistant 消息: ' + (currentMsg ? '存在' : '不存在') + 
      ' toolCalls: ' + (currentMsg?.toolCalls?.length || 0));
    
    if (currentMsg && currentMsg.toolCalls) {
      const toolId = event.payload.id;
      // 优先用 id 精确匹配，如果没有 id 则用 name 匹配（向后兼容）
      const toolCall = toolId
        ? currentMsg.toolCalls.find((t: ToolCall) => t.id === toolId)
        : currentMsg.toolCalls.find((t: ToolCall) => t.name === event.payload.name && t.status === 'running');
      if (toolCall) {
        toolCall.result = event.payload.result ?? '';
        toolCall.durationMs = event.payload.duration_ms || 0;
        toolCall.status = 'completed';
        void agentLog('[agent-tool-complete] 更新工具调用: ' + event.payload.name + ' id: ' + (toolId || 'none') + ' status: completed');
      } else {
        void agentLog('[agent-tool-complete] 未找到匹配的 running 工具调用, id: ' + (toolId || 'none'));
      }
    }
    
    // 标记当前轮次结束
    sessionRoundEnded[eventSid] = true;
    void agentLog('[agent-tool-complete] 设置 sessionRoundEnded = true');
    
    // 如果是当前会话，同步更新 messages.value
    if (eventSid === currentSessionId.value) {
      messages.value = [...sessionMsgs];
    }
    
    // 如果是 todo 工具，更新任务列表（仅当前会话）
    if (eventSid === currentSessionId.value && event.payload.name === 'todo' && event.payload.result) {
      try {
        const parsed = JSON.parse(event.payload.result);
        // 支持两种格式：直接数组 或 {todos: [...], summary: {...}}
        let tasks: Array<{ id: string; content: string; status?: string }> = [];
        
        if (Array.isArray(parsed) && parsed.length > 0 && parsed[0].id && parsed[0].content) {
          // 直接数组格式
          tasks = parsed;
        } else if (parsed.todos && Array.isArray(parsed.todos) && parsed.todos.length > 0) {
          // 对象格式 {todos, summary}
          tasks = parsed.todos;
        }
        
        if (tasks.length > 0) {
          currentTasks.value = tasks.map((t: { id: string; content: string; status?: string }) => ({
            id: t.id,
            content: t.content,
            status: (['pending', 'in_progress', 'completed', 'cancelled'].includes(t.status || '') 
              ? t.status 
              : 'pending') as TaskItem['status'],
          }));
        }
      } catch {
        // 解析失败，忽略
      }
    }
    
    scrollToBottom();
  });

  // 思考动画事件
  unlistenThinking = await listen<{ text: string | null; session_id: string | null }>('agent-thinking', (event) => {
    const eventSid = event.payload?.session_id;
    // 更新该会话的思考文字（不论是否当前会话）
    if (eventSid) {
      thinkingTexts[eventSid] = event.payload?.text || '';
    }
    // 当前会话时同步到视图
    if (eventSid && currentSessionId.value && eventSid === currentSessionId.value) {
      if (event.payload?.text) {
        thinkingText.value = event.payload.text;
      } else {
        thinkingText.value = '';
      }
    }
  });
  
  unlistenError = await listen<{ message: string; session_id: string | null }>('agent-error', (event) => {
    const eventSid = event.payload?.session_id;
    void agentLog('[agent-error] 收到事件: ' + event.payload?.message + ' session_id: ' + eventSid);
    
    // 必须有 session_id 才处理
    if (!eventSid) return;
    
    // 更新该会话的状态
    streamingSessions[eventSid] = false;
    thinkingTexts[eventSid] = '';
    
    // 获取该会话的消息缓存
    let sessionMsgs = sessionMessagesCache[eventSid];
    if (!sessionMsgs) {
      if (eventSid === currentSessionId.value) {
        sessionMessagesCache[eventSid] = [...messages.value];
        sessionMsgs = sessionMessagesCache[eventSid];
      } else {
        sessionMessagesCache[eventSid] = [];
        sessionMsgs = sessionMessagesCache[eventSid];
      }
    }
    
    // 在缓存中添加错误信息
    const messagesCopy = [...sessionMsgs].reverse();
    const currentMsg = messagesCopy.find((m: Message) => m.role === 'assistant');
    if (currentMsg) {
      currentMsg.content = (currentMsg.content || '') + `\n[错误: ${event.payload?.message}]`;
    }
    
    // 如果是当前会话，同步更新 messages.value
    if (eventSid === currentSessionId.value) {
      messages.value = [...sessionMsgs];
      thinkingText.value = '';
    }
  });

  // 流式结束事件
  unlistenDone = await listen<{ response: string | null; session_id: string; message_count: number }>('agent-done', (event) => {
    const eventSid = event.payload?.session_id;
    void agentLog('[agent-done] 收到事件: ' + JSON.stringify(event.payload));
    // 更新该会话的状态（不论是否当前会话）
    if (eventSid) {
      streamingSessions[eventSid] = false;
      thinkingTexts[eventSid] = '';
      sessionRoundEnded[eventSid] = true; // 标记这一轮已结束，下一轮新消息需要创建新 assistant 消息
      // 流式结束后清除缓存（下次切换会话会从数据库加载）
      delete sessionMessagesCache[eventSid];
    }
    // 当前会话时同步到视图
    if (eventSid && currentSessionId.value && eventSid === currentSessionId.value) {
      thinkingText.value = '';
      if (currentSessionId.value) sessionRoundEnded[currentSessionId.value] = true;
      void agentLog('[agent-done] messages.length: ' + messages.value.length + ' 最后一条: ' + (messages.value[messages.value.length - 1]?.role || 'none'));
      if (currentSessionId.value) streamingSessions[currentSessionId.value] = false; // trigger computed update via streamingSessions
    }
    // 恢复 UI 状态（仅当该会话是当前会话时）
    if (!eventSid || (currentSessionId.value && eventSid === currentSessionId.value)) {
      scrollToBottom();
    }
  });

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
      selectSession(session);
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
  // Properly clean up event listeners
  unlistenDelta?.();
  unlistenThinking?.();
  unlistenToolStart?.();
  unlistenToolComplete?.();
  unlistenError?.();
  unlistenDone?.();
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
      if (item.previewUrl) URL.revokeObjectURL(item.previewUrl);
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