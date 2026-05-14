<template>
  <div class="flex flex-col gap-4 p-5 h-full">
    <!-- 页面头部 -->
    <div class="flex items-center justify-between px-4 py-3 bg-base-100 border border-base-content/10 rounded-xl">
      <div class="flex items-center gap-3">
        <SvgIcon name="bot" size="20" class="text-primary" />
        <div class="flex flex-col gap-0.5">
          <h2 class="m-0 text-lg font-bold text-base-content">AI Agent 会话</h2>
          <p class="m-0 text-xs text-base-content/60">Hermes Agent 会话管理</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button class="btn btn-ghost btn-sm" @click="refreshSessions" :disabled="loading">
          <SvgIcon name="refresh" size="14" :class="{ 'animate-spin': loading }" />
        </button>
        <button class="btn btn-primary btn-sm gap-1.5" @click="openNewSession">
          <SvgIcon name="plus" size="14" />
          新会话
        </button>
      </div>
    </div>

    <!-- 统计信息 -->
    <div v-if="stats && !loading" class="flex items-center gap-4 px-4 py-3 bg-base-100 border border-base-content/10 rounded-xl">
      <div class="flex items-center gap-2">
        <span class="text-lg font-bold text-primary">{{ stats.totalSessions }}</span>
        <span class="text-xs text-base-content/60">会话总数</span>
      </div>
      <div class="h-4 w-px bg-base-content/10"></div>
      <div class="flex items-center gap-2">
        <span class="text-lg font-bold text-primary">{{ stats.totalMessages }}</span>
        <span class="text-xs text-base-content/60">消息总数</span>
      </div>
      <div class="h-4 w-px bg-base-content/10"></div>
      <div class="flex items-center gap-2 text-xs text-base-content/60">
        <span v-for="(source, idx) in stats.sources.slice(0, 3)" :key="source[0]" class="badge badge-ghost">
          {{ source[0] }}: {{ source[1] }}
        </span>
      </div>
    </div>

    <!-- 未安装提示 -->
    <div v-if="!installed && !loading" class="flex flex-col items-center justify-center gap-4 py-16 text-center">
      <div class="flex h-16 w-16 items-center justify-center rounded-full bg-base-200">
        <SvgIcon name="bot" size="32" stroke-width="1.5" class="text-base-content/30" />
      </div>
      <p class="m-0 text-base font-semibold text-base-content">Hermes 未安装</p>
      <p class="m-0 text-sm text-base-content/60 max-w-md">
        请先安装 Hermes Agent CLI 才能使用会话管理功能。
        <br />安装方法：<code class="bg-base-200 px-1 rounded">pip install hermes-agent</code>
      </p>
      <button class="btn btn-outline btn-sm" @click="refreshSessions">
        <SvgIcon name="refresh" size="14" />
        重新检测
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="flex flex-col items-center justify-center gap-3 py-16">
      <SvgIcon name="refresh" size="24" class="animate-spin text-primary" />
      <p class="m-0 text-sm text-base-content/60">加载会话列表...</p>
    </div>

    <!-- 空状态 -->
    <div v-if="installed && sessions.length === 0 && !loading" class="flex flex-col items-center justify-center gap-4 py-16 text-center">
      <div class="flex h-16 w-16 items-center justify-center rounded-full bg-base-200">
        <SvgIcon name="chat" size="32" stroke-width="1.5" class="text-base-content/30" />
      </div>
      <p class="m-0 text-base font-semibold text-base-content">暂无会话记录</p>
      <p class="m-0 text-sm text-base-content/60">点击「新会话」开始与 Hermes Agent 对话</p>
    </div>

    <!-- 会话列表 -->
    <div v-if="installed && sessions.length > 0 && !loading" class="flex flex-col gap-2 overflow-y-auto flex-1">
      <div
        v-for="session in sessions"
        :key="session.id"
        class="group flex items-center gap-3 px-4 py-3 bg-base-100 border border-base-content/10 rounded-xl cursor-pointer transition-all hover:border-primary hover:bg-primary/5"
        @click="openSession(session)"
      >
        <!-- Source 图标 -->
        <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary shrink-0">
          <SvgIcon :name="sourceIcon(session.source)" size="18" />
        </div>

        <!-- 会话信息 -->
        <div class="flex flex-col gap-1 min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <span class="truncate text-sm font-semibold text-base-content">
              {{ session.title || session.preview || '新会话' }}
            </span>
            <span class="badge badge-ghost badge-xs shrink-0">{{ session.source }}</span>
          </div>
          <div class="flex items-center gap-2 text-xs text-base-content/60">
            <span>{{ formatTime(session.lastActive) }}</span>
            <span class="text-base-content/30">•</span>
            <span>{{ session.messageCount }} 条消息</span>
            <span v-if="session.model" class="text-base-content/30">•</span>
            <span v-if="session.model" class="truncate max-w-[120px]">{{ session.model }}</span>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
          <button
            class="btn btn-ghost btn-square h-7 w-7 min-h-0 hover:bg-error/10 hover:text-error"
            @click.stop="confirmDelete(session)"
            title="删除"
          >
            <SvgIcon name="trash" size="14" />
          </button>
        </div>
      </div>
    </div>

    <!-- 删除确认弹窗 -->
    <div v-if="showDeleteConfirm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-base-100 rounded-xl p-5 max-w-sm w-full shadow-xl">
        <h3 class="text-lg font-bold mb-2">确认删除</h3>
        <p class="text-sm text-base-content/70 mb-4">
          确定要删除会话「{{ deleteTarget?.title || deleteTarget?.preview || '新会话' }}」吗？
          该操作不可恢复。
        </p>
        <div class="flex gap-2 justify-end">
          <button class="btn btn-ghost btn-sm" @click="showDeleteConfirm = false">取消</button>
          <button class="btn btn-error btn-sm" @click="deleteSession">删除</button>
        </div>
      </div>
    </div>

    <!-- 会话详情弹窗 -->
    <div v-if="showDetail" class="fixed inset-0 bg-black/50 flex z-50">
      <div class="bg-base-100 w-full max-w-3xl h-full shadow-xl flex flex-col">
        <!-- 详情头部 -->
        <div class="flex items-center justify-between px-4 py-3 border-b border-base-content/10">
          <div class="flex items-center gap-2">
            <SvgIcon :name="sourceIcon(currentSession?.source || '')" size="18" class="text-primary" />
            <span class="font-semibold text-base-content">
              {{ currentSession?.title || currentSession?.preview || '会话详情' }}
            </span>
          </div>
          <button class="btn btn-ghost btn-square btn-sm" @click="showDetail = false">
            <SvgIcon name="x" size="14" />
          </button>
        </div>

        <!-- 消息列表 -->
        <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-3">
          <div v-if="messages.length === 0 && !loadingMessages" class="flex flex-col items-center justify-center py-10 text-base-content/60">
            <p>暂无消息</p>
          </div>
          <div
            v-for="msg in messages"
            :key="msg.id"
            class="flex gap-3 px-3 py-2 rounded-lg"
            :class="msg.role === 'user' ? 'bg-primary/5' : 'bg-base-200'"
          >
            <div class="flex h-8 w-8 items-center justify-center rounded-full shrink-0"
                 :class="msg.role === 'user' ? 'bg-primary text-base-100' : 'bg-base-content/20 text-base-content'">
              <SvgIcon :name="msg.role === 'user' ? 'user' : 'bot'" size="14" />
            </div>
            <div class="flex flex-col gap-1 min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="text-xs font-medium"
                      :class="msg.role === 'user' ? 'text-primary' : 'text-base-content/70'">
                  {{ msg.role === 'user' ? '用户' : msg.role }}
                </span>
                <span v-if="msg.toolName" class="badge badge-ghost badge-xs">{{ msg.toolName }}</span>
              </div>
              <div class="text-sm text-base-content whitespace-pre-wrap break-words">
                {{ msg.content || '(空)' }}
              </div>
            </div>
          </div>
        </div>

        <!-- 加载消息 -->
        <div v-if="loadingMessages" class="flex items-center justify-center py-4">
          <SvgIcon name="refresh" size="16" class="animate-spin text-primary" />
        </div>

        <!-- 详情底部：继续对话 -->
        <div class="px-4 py-3 border-t border-base-content/10">
          <button class="btn btn-primary btn-sm w-full gap-2" @click="resumeSession">
            <SvgIcon name="terminal" size="14" />
            在终端继续对话
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';
import SvgIcon from '../../components/SvgIcon.vue';

interface HermesSession {
  id: string;
  source: string;
  model: string;
  title: string | null;
  startedAt: number;
  endedAt: number | null;
  messageCount: number;
  preview: string;
  lastActive: number;
}

interface HermesMessage {
  id: number;
  sessionId: string;
  role: string;
  content: string | null;
  toolName: string | null;
  toolCallId: string | null;
  timestamp: number;
  finishReason: string | null;
}

interface HermesStats {
  totalSessions: number;
  totalMessages: number;
  sources: [string, number][];
}

// State
const installed = ref(false);
const loading = ref(true);
const loadingMessages = ref(false);
const sessions = ref<HermesSession[]>([]);
const stats = ref<HermesStats | null>(null);
const showDeleteConfirm = ref(false);
const deleteTarget = ref<HermesSession | null>(null);
const showDetail = ref(false);
const currentSession = ref<HermesSession | null>(null);
const messages = ref<HermesMessage[]>([]);

// Methods
async function checkInstalled() {
  try {
    const result = await invoke<{ success: boolean; installed: boolean }>('hermes_installed');
    installed.value = result.installed;
  } catch {
    installed.value = false;
  }
}

async function loadSessions() {
  loading.value = true;
  try {
    const result = await invoke<{ success: boolean; sessions: HermesSession[] }>('list_hermes_sessions_cmd', {
      limit: 50,
      offset: 0,
    });
    sessions.value = result.sessions || [];
  } catch (e) {
    console.error('Failed to load sessions:', e);
  }
  loading.value = false;
}

async function loadStats() {
  try {
    const result = await invoke<{ success: boolean; stats: HermesStats }>('get_hermes_stats_cmd');
    stats.value = result.stats;
  } catch {
    stats.value = null;
  }
}

async function refreshSessions() {
  await checkInstalled();
  if (installed.value) {
    await Promise.all([loadSessions(), loadStats()]);
  }
  loading.value = false;
}

function formatTime(ts: number): string {
  const now = Date.now() / 1000;
  const diff = now - ts;

  if (diff < 60) return '刚刚';
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 604800) return `${Math.floor(diff / 86400)} 天前`;

  const date = new Date(ts * 1000);
  return date.toLocaleDateString('zh-CN');
}

function sourceIcon(source: string): string {
  if (source.includes('hermes')) return 'bot';
  if (source.includes('claude')) return 'terminal';
  if (source.includes('chatgpt') || source.includes('openai')) return 'sparkles';
  return 'chat';
}

function openNewSession() {
  // Open Hermes CLI in terminal
  open('hermes chat');
}

async function openSession(session: HermesSession) {
  currentSession.value = session;
  showDetail.value = true;
  loadingMessages.value = true;
  messages.value = [];

  try {
    const result = await invoke<{ success: boolean; messages: HermesMessage[] }>('list_hermes_messages_cmd', {
      sessionId: session.id,
    });
    messages.value = result.messages || [];
  } catch (e) {
    console.error('Failed to load messages:', e);
  }
  loadingMessages.value = false;
}

function resumeSession() {
  if (currentSession.value) {
    open(`hermes chat --resume ${currentSession.value.id}`);
    showDetail.value = false;
  }
}

function confirmDelete(session: HermesSession) {
  deleteTarget.value = session;
  showDeleteConfirm.value = true;
}

async function deleteSession() {
  if (!deleteTarget.value) return;
  const targetId = deleteTarget.value.id;
  try {
    await invoke('delete_hermes_session_cmd', { sessionId: targetId });
    sessions.value = sessions.value.filter(s => s.id !== targetId);
    if (stats.value) {
      stats.value.totalSessions--;
    }
  } catch (e) {
    console.error('Failed to delete session:', e);
  }
  showDeleteConfirm.value = false;
  deleteTarget.value = null;
}

// Lifecycle
onMounted(refreshSessions);
</script>