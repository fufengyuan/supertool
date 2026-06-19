<template>
  <div class="h-full overflow-y-auto border-r border-base-content/10 bg-base-100/50">
    <!-- Header -->
    <div class="flex items-center justify-between p-3 border-b border-base-content/10">
      <h2 class="text-sm font-bold text-base-content">
        {{ isClawMode ? 'Claw Sessions' : 'Sessions' }}
      </h2>
      <button
        class="btn btn-ghost btn-xs gap-1"
        @click="onNewChat"
        title="New Chat"
      >
        <SvgIcon name="plus" :size="14" />
      </button>
    </div>

    <!-- ====== Claw Mode ====== -->
    <template v-if="isClawMode">
      <div v-if="clawLoading" class="flex items-center justify-center py-10">
        <span class="loading loading-spinner loading-sm text-primary" />
      </div>
      <div v-else-if="clawError && clawError.includes('not initialized')" class="flex flex-col items-center justify-center py-10 text-center px-3">
        <p class="text-xs text-base-content/50">Claw 未初始化</p>
      </div>
      <div v-else-if="clawError" class="alert alert-error text-xs py-1 m-2">{{ clawError }}</div>
      <div v-else-if="clawSessions.length === 0" class="flex flex-col items-center justify-center py-10 text-center px-3">
        <p class="text-xs text-base-content/40">暂无会话</p>
      </div>
      <div v-else class="flex flex-col gap-0.5 p-2">
        <button
          v-for="s in clawSessions"
          :key="s.sessionId"
          @click="onResumeClawSession(s.sessionId)"
          class="text-left px-2.5 py-1.5 rounded-lg text-xs cursor-pointer border-l-2 transition-colors"
          :class="currentSessionId === s.sessionId ? 'border-l-primary bg-primary/5 font-medium' : 'border-l-transparent hover:bg-base-200'"
        >
          <div class="truncate">{{ s.title || 'Untitled' }}</div>
          <div class="text-[10px] text-base-content/40 truncate">{{ s.summary || '' }}</div>
        </button>
      </div>
    </template>

    <!-- ====== Hermes Mode ====== -->
    <template v-else>
      <!-- Search bar -->
      <div class="p-2">
        <input
          ref="searchInputRef"
          v-model="searchQuery"
          type="text"
          placeholder="搜索会话..."
          class="input input-xs input-bordered w-full"
          @input="onSearchInput"
        />
      </div>

      <template v-if="searchQuery">
        <!-- Search results -->
        <div v-if="searchResults.length > 0" class="flex flex-col gap-0.5 p-2">
          <div
            v-for="result in searchResults"
            :key="result.messageId"
            @click="onResumeSession(result.sessionId)"
            class="px-2.5 py-1.5 rounded-lg text-xs cursor-pointer border-l-2 transition-colors"
            :class="currentSessionId === result.sessionId ? 'border-l-primary bg-primary/5 font-medium' : 'border-l-transparent hover:bg-base-200'"
          >
            <div class="truncate font-medium" v-html="highlightSnippet(result.snippet || result.content || '')"></div>
            <div class="text-[10px] text-base-content/40 truncate">{{ result.sessionTitle || '' }}</div>
          </div>
        </div>
        <div v-else class="text-center py-6 text-xs text-base-content/40">无结果</div>
      </template>

      <template v-else>
        <!-- Session groups -->
        <div v-if="loadingSessions" class="flex items-center justify-center py-10">
          <span class="loading loading-spinner loading-sm text-primary" />
        </div>
        <div v-else-if="groupedSessions.length === 0" class="flex flex-col items-center justify-center py-10 text-center px-3">
          <p class="text-xs text-base-content/40">暂无会话</p>
        </div>
        <div v-else class="flex flex-col gap-2 p-2">
          <div v-for="group in groupedSessions" :key="group.label">
            <div class="text-[10px] font-semibold text-base-content/50 px-2.5 py-1">{{ group.label }}</div>
            <div class="flex flex-col gap-0.5">
              <button
                v-for="session in group.sessions"
                :key="session.id"
                @click="onResumeSession(session.id)"
                class="text-left px-2.5 py-1.5 rounded-lg text-xs cursor-pointer border-l-2 transition-colors"
                :class="currentSessionId === session.id ? 'border-l-primary bg-primary/5 font-medium' : 'border-l-transparent hover:bg-base-200'"
              >
                <div class="truncate">{{ session.title || 'Untitled' }}</div>
                <div class="text-[10px] text-base-content/40 truncate">
                  <span>{{ formatRelativeTime(session.lastActive) }}</span>
                  <span v-if="session.messageCount"> · {{ session.messageCount }} msgs</span>
                </div>
              </button>
            </div>
          </div>
        </div>
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'SessionsPanel' })
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useSessionManager } from '@/composables/useSessionManager'
import type { Session } from '@/composables/useSessionManager'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useAgentModeStore } from '@/stores/agentModeStore'

const router = useRouter()
const agentModeStore = useAgentModeStore()

const isClawMode = computed(() => agentModeStore.mode === 'claw')

// Session manager from composable
const {
  sessions,
  searchResults,
  isSearching,
  currentSessionId,
  loadingSessions,
  refreshSessions,
  handleSessionSearch,
  highlightSnippet,
} = useSessionManager()

// Local group/date helpers
type DateGroup = 'today' | 'yesterday' | 'thisWeek' | 'earlier'
const DATE_GROUP_LABELS: Record<DateGroup, string> = {
  today: '今天', yesterday: '昨天', thisWeek: '本周', earlier: '更早',
}

function getDateGroup(ts: number | undefined): DateGroup {
  if (!ts) {return 'earlier'}
  const d = new Date(ts * 1000)
  const now = new Date()
  const isToday = d.getDate() === now.getDate() && d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear()
  if (isToday) {return 'today'}
  const yesterday = new Date(now); yesterday.setDate(yesterday.getDate() - 1)
  if (d.getDate() === yesterday.getDate() && d.getMonth() === yesterday.getMonth() && d.getFullYear() === yesterday.getFullYear()) {return 'yesterday'}
  const weekAgo = new Date(now); weekAgo.setDate(weekAgo.getDate() - 7)
  if (d >= weekAgo) {return 'thisWeek'}
  return 'earlier'
}

function formatRelativeTime(dateVal: string | number | null | undefined): string {
  if (dateVal == null) return ''
  try {
    const d = typeof dateVal === 'number' ? new Date(dateVal * 1000) : new Date(dateVal)
    const now = new Date()
    const diffMs = now.getTime() - d.getTime()
    if (diffMs < 60000) return '刚刚'
    if (diffMs < 3600000) return `${Math.floor(diffMs / 60000)}m ago`
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch { return '' }
}

interface SessionGroup { label: string; sessions: Session[] }
const groupedSessions = computed<SessionGroup[]>(() => {
  const groups = new Map<DateGroup, Session[]>()
  for (const s of sessions.value) {
    const ts = s.lastActive || s.startedAt
    const group = getDateGroup(ts)
    if (!groups.has(group)) {groups.set(group, [])}
    groups.get(group)!.push(s)
  }
  const order: DateGroup[] = ['today', 'yesterday', 'thisWeek', 'earlier']
  return order.filter((label) => groups.has(label)).map((label) => ({ label: DATE_GROUP_LABELS[label], sessions: groups.get(label)! }))
})

// Claw session state
const clawSessions = ref<any[]>([])
const clawLoading = ref(false)
const clawError = ref<string | null>(null)

// Search
const searchQuery = ref('')
const searchInputRef = ref<HTMLInputElement | null>(null)

let refreshTimer: ReturnType<typeof setInterval> | null = null

async function refreshClawSessions() {
  if (!isClawMode.value) return
  clawLoading.value = true
  clawError.value = null
  try {
    const result = await invoke<any>('claw_chat_list_sessions')
    if (result?.sessions) {
      clawSessions.value = result.sessions
    } else {
      clawSessions.value = []
    }
  } catch (e: any) {
    clawError.value = typeof e === 'string' ? e : e?.message || 'Failed to load'
  }
  clawLoading.value = false
}

function onNewChat() {
  router.push('/agent/chat')
}

function onResumeSession(sessionId: string) {
  router.push({ path: '/agent/chat', query: { session: sessionId } })
}

function onResumeClawSession(sessionId: string) {
  if (!isClawMode.value) {
    agentModeStore.setMode('claw')
  }
  router.push({ path: '/agent/chat', query: { session: sessionId } })
}

function onSearchInput() {
  handleSessionSearch(searchQuery.value)
}

// Reload sessions when mode switches
watch(isClawMode, () => {
  if (isClawMode.value) {
    refreshClawSessions()
  } else {
    refreshSessions()
  }
})

onMounted(() => {
  if (isClawMode.value) {
    refreshClawSessions()
  }
  refreshTimer = setInterval(() => {
    if (isClawMode.value) {
      refreshClawSessions()
    } else {
      refreshSessions()
    }
  }, 30000)
})

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer)
})
</script>