<template>
  <div class="w-64 border-r border-base-content/10 flex flex-col bg-base-100">
    <!-- 会话列表头部 -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-base-content/10">
      <span class="text-sm font-semibold text-base-content">会话</span>
      <button class="btn btn-ghost btn-xs" @click="emit('refresh')" :disabled="loadingSessions">
        <SvgIcon name="refresh" size="12" :class="{ 'animate-spin': loadingSessions }" />
      </button>
    </div>

    <!-- 新会话按钮 -->
    <div class="px-2 py-2">
      <button class="btn btn-primary btn-sm w-full gap-1.5" @click="emit('newChat')" title="快捷键: Cmd+K">
        <SvgIcon name="plus" size="14" />
        新对话
      </button>
    </div>

    <!-- 会话搜索框 -->
    <div class="px-2 py-1">
      <div class="relative">
        <input
          v-model="sessionSearchQuery"
          type="text"
          class="input input-sm input-bordered w-full pl-7 text-xs"
          placeholder="搜索会话..."
        />
        <SvgIcon name="search" size="12" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/40" />
        <button
          v-if="sessionSearchQuery"
          class="absolute right-2 top-1/2 -translate-y-1/2 text-base-content/40 hover:text-base-content"
          @click="handleClearSearch"
        >
          <SvgIcon name="close" size="12" />
        </button>
      </div>
    </div>

    <!-- 会话列表/搜索结果 -->
    <div class="flex-1 overflow-y-auto">
      <!-- 搜索结果 -->
      <template v-if="isSearchMode">
        <div v-if="isSearching" class="flex items-center justify-center py-8">
          <SvgIcon name="refresh" size="16" class="animate-spin text-base-content/40" />
        </div>
        <div v-else-if="searchResults.length === 0" class="flex flex-col items-center justify-center py-8 text-center">
          <SvgIcon name="search" size="24" class="text-base-content/30" />
          <p class="mt-2 text-xs text-base-content/50">未找到匹配的内容</p>
        </div>
        <div v-else class="flex flex-col gap-1 px-2 py-1">
          <div
            v-for="result in searchResults"
            :key="result.messageId"
            class="group flex flex-col gap-1 px-2 py-1.5 rounded-lg cursor-pointer transition-colors hover:bg-base-200"
            @click="emit('jumpToResult', result)"
          >
            <div class="flex items-center gap-2">
              <SvgIcon :name="sourceIcon(result.source)" size="12" class="shrink-0 text-base-content/50" />
              <span class="text-xs text-base-content/60">{{ result.sessionTitle || '新会话' }}</span>
              <span v-if="result.profile !== 'default'" class="badge badge-xs badge-info shrink-0">{{ result.profile }}</span>
              <span class="text-xs text-base-content/40">•</span>
              <span class="text-xs text-base-content/50">{{ result.role }}</span>
            </div>
            <div class="text-xs text-base-content line-clamp-2" v-html="highlightSnippet(result.snippet, sessionSearchQuery)"></div>
          </div>
        </div>
      </template>
      
      <!-- 正常会话列表 -->
      <template v-else>
        <div v-if="loadingSessions" class="flex items-center justify-center py-8">
          <SvgIcon name="refresh" size="16" class="animate-spin text-base-content/40" />
        </div>
        <div v-else-if="groupedSessions.length === 0" class="flex flex-col items-center justify-center py-8 text-center">
          <SvgIcon name="chat" size="24" class="text-base-content/30" />
          <p class="mt-2 text-xs text-base-content/50">暂无会话</p>
        </div>
        <div v-else class="flex flex-col gap-1 px-2 py-1">
          <template v-for="group in groupedSessions" :key="group.label">
            <div class="px-2 pt-3 pb-1 text-[10px] font-semibold text-base-content/40 uppercase tracking-wider">
              {{ group.label }}
            </div>
            <div
              v-for="session in group.sessions"
              :key="session.id"
              class="group flex items-center gap-2 px-2 py-1.5 rounded-lg cursor-pointer transition-colors"
              :class="currentSessionId === session.id ? 'bg-primary/10 text-primary' : 'hover:bg-base-200'"
              @click="emit('select', session)"
            >
              <SvgIcon :name="sourceIcon(session.source)" size="14" class="shrink-0" />
              <SvgIcon v-if="session.parentSessionId" name="gitBranch" size="12" class="shrink-0 text-warning" title="Subagent 会话" />
              <div class="flex flex-col min-w-0 flex-1">
                <div class="flex items-center gap-1">
                  <span class="truncate text-xs font-medium">{{ session.title || session.preview || '新会话' }}</span>
                  <span v-if="session.profile !== 'default'" class="badge badge-xs badge-info shrink-0">{{ session.profile }}</span>
                </div>
                <span class="truncate text-xs text-base-content/50">{{ formatTime(session.lastActive || session.startedAt) }}</span>
              </div>
              <span class="text-xs text-base-content/40 shrink-0">{{ session.messageCount }}</span>
              <!-- hover 显示删除按钮 -->
              <button 
                class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                @click.stop="emit('delete', session.id)"
                title="删除会话"
              >
                <SvgIcon name="trash" size="10" class="text-error" />
              </button>
            </div>
          </template>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import SvgIcon from '@/components/ui/SvgIcon.vue';

// Types
export interface Session {
  id: string;
  title: string | null;
  model: string;
  source: string;
  startedAt?: number;
  endedAt?: number | null;
  messageCount: number;
  preview: string;
  lastActive?: number;
  parentSessionId?: string | null;
  profile: string;
}

export interface SearchResult {
  sessionId: string;
  sessionTitle: string | null;
  messageId: string;
  role: string;
  snippet: string;
  content: string | null;
  timestamp: number | null;
  source: string;
  model: string;
  profile: string;
}

// Props
const props = defineProps<{
  sessions: Session[];
  currentSessionId: string | null;
  loadingSessions: boolean;
  isSearching: boolean;
  searchResults: SearchResult[];
  formatTime: (ts: number | null | undefined) => string;
  sourceIcon: (source: string) => string;
  highlightSnippet: (snippet: string, query: string) => string;
}>();

// Events
const emit = defineEmits<{
  refresh: [];
  newChat: [];
  select: [session: Session];
  delete: [sessionId: string];
  search: [query: string];
  jumpToResult: [result: SearchResult];
  clearSearch: [];
}>();

// 内部状态
const sessionSearchQuery = ref('');

// 计算属性
const isSearchMode = computed(() => sessionSearchQuery.value.trim().length > 0);

// Date grouping
type DateGroup = 'today' | 'yesterday' | 'thisWeek' | 'earlier'

const DATE_GROUP_LABELS: Record<DateGroup, string> = {
  today: '今天',
  yesterday: '昨天',
  thisWeek: '本周',
  earlier: '更早',
};

function getDateGroup(ts: number): DateGroup {
  const d = new Date(ts * 1000);
  const now = new Date();
  const isToday = d.getDate() === now.getDate() &&
    d.getMonth() === now.getMonth() &&
    d.getFullYear() === now.getFullYear();
  if (isToday) {return 'today';}
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  const isYesterday = d.getDate() === yesterday.getDate() &&
    d.getMonth() === yesterday.getMonth() &&
    d.getFullYear() === yesterday.getFullYear();
  if (isYesterday) {return 'yesterday';}
  const weekAgo = new Date(now);
  weekAgo.setDate(weekAgo.getDate() - 7);
  if (d >= weekAgo) {return 'thisWeek';}
  return 'earlier';
}

const groupedSessions = computed(() => {
  const groups = new Map<DateGroup, Session[]>();
  for (const s of props.sessions) {
    const group = getDateGroup(s.lastActive || s.startedAt || 0);
    if (!groups.has(group)) {groups.set(group, []);}
    groups.get(group)!.push(s);
  }
  const order: DateGroup[] = ['today', 'yesterday', 'thisWeek', 'earlier'];
  return order
    .filter(label => groups.has(label))
    .map(label => ({ label: DATE_GROUP_LABELS[label], sessions: groups.get(label)! }));
});


// 清空搜索
const handleClearSearch = () => {
  sessionSearchQuery.value = '';
  emit('clearSearch');
};

// 监听搜索输入变化
watch(sessionSearchQuery, (query) => {
  emit('search', query);
});
</script>