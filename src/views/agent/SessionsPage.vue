<template>
  <div class="h-full overflow-y-auto">
    <div class="max-w-3xl mx-auto px-4 py-6">
      <!-- Header: title + New Chat button -->
      <div class="flex items-center justify-between mb-6">
        <h1 class="text-xl font-bold text-base-content">Sessions</h1>
        <button
          class="btn btn-primary btn-sm gap-1.5"
          @click="onNewChat"
        >
          <SvgIcon name="plus" :size="14" />
          New Chat
        </button>
      </div>

      <!-- Integrated search bar -->
      <div class="relative mb-6">
        <SvgIcon
          name="search"
          :size="14"
          class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40"
        />
        <input
          ref="searchInputRef"
          v-model="searchQuery"
          type="text"
          class="input input-bordered w-full pl-9 pr-9 text-sm"
          placeholder="Search sessions..."
        />
        <button
          v-if="searchQuery"
          class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
          @click="onClearSearch"
        >
          <SvgIcon name="x" :size="13" />
        </button>
      </div>

      <!-- Content area -->
      <!-- Loading state -->
      <div
        v-if="loadingSessions"
        class="flex items-center justify-center py-20"
      >
        <span class="loading loading-spinner loading-md text-primary" />
      </div>

      <!-- Search mode -->
      <template v-else-if="isSearchMode">
        <!-- Searching spinner -->
        <div
          v-if="isSearching"
          class="flex items-center justify-center py-20"
        >
          <span class="loading loading-spinner loading-md text-primary" />
        </div>

        <!-- No results -->
        <div
          v-else-if="searchResults.length === 0"
          class="flex flex-col items-center justify-center py-20 text-center"
        >
          <SvgIcon
            name="search"
            :size="32"
            class="text-base-content/20 mb-3"
          />
          <p class="text-sm font-medium text-base-content/50">No results found</p>
          <p class="text-xs text-base-content/30 mt-1">Try a different search term</p>
        </div>

        <!-- Search results list -->
        <div v-else class="flex flex-col gap-1">
          <button
            v-for="result in searchResults"
            :key="result.messageId"
            class="group w-full text-left rounded-lg p-3 transition-colors hover:bg-base-200 border-l-2"
            :class="
              currentSessionId === result.sessionId
                ? 'border-l-primary bg-primary/5'
                : 'border-l-transparent'
            "
            @click="onResumeSession(result.sessionId)"
          >
            <!-- Top row: title + date -->
            <div class="flex items-start justify-between gap-3">
              <span class="text-sm font-medium text-base-content truncate leading-snug">
                {{ result.sessionTitle || `Session ${result.sessionId.slice(-6)}` }}
              </span>
              <span class="text-xs text-base-content/40 shrink-0 mt-0.5">
                {{ formatFullDate(result.timestamp) }}
              </span>
            </div>

            <!-- Snippet with highlights -->
            <div
              v-if="result.snippet"
              class="mt-1.5 text-xs text-base-content/60 leading-relaxed line-clamp-3"
              v-html="highlightedSnippet(result.snippet)"
            />

            <!-- Bottom row: tags -->
            <div class="flex items-center gap-1.5 mt-2">
              <span class="badge badge-sm badge-outline badge-primary/50 text-[10px]">
                {{ result.source }}
              </span>
              <span v-if="result.model" class="badge badge-sm badge-ghost text-[10px]">
                {{ formatModel(result.model) }}
              </span>
            </div>
          </button>
        </div>
      </template>

      <!-- Normal mode: grouped sessions -->
      <template v-else>
        <!-- No sessions -->
        <div
          v-if="sessions.length === 0"
          class="flex flex-col items-center justify-center py-20 text-center"
        >
          <SvgIcon
            name="chat"
            :size="32"
            class="text-base-content/20 mb-3"
          />
          <p class="text-sm font-medium text-base-content/50">No conversations yet</p>
          <p class="text-xs text-base-content/30 mt-1">Start a new chat to begin</p>
        </div>

        <!-- Grouped sessions list -->
        <div v-else class="flex flex-col gap-1">
          <template v-for="group in groupedSessions" :key="group.label">
            <!-- Date group label -->
            <div
              class="px-3 pt-4 pb-1.5 text-[10px] font-semibold text-base-content/40 uppercase tracking-wider"
            >
              {{ group.label }}
            </div>

            <!-- Session cards -->
            <button
              v-for="session in group.sessions"
              :key="session.id"
              class="group w-full text-left rounded-lg p-3 transition-colors hover:bg-base-200 border-l-2"
              :class="
                currentSessionId === session.id
                  ? 'border-l-primary bg-primary/5'
                  : 'border-l-transparent'
              "
              @click="onResumeSession(session.id)"
            >
              <!-- Top row: title + time -->
              <div class="flex items-start justify-between gap-3">
                <span class="text-sm font-medium text-base-content truncate leading-snug">
                  {{ session.title || 'New conversation' }}
                </span>
                <span class="text-xs text-base-content/40 shrink-0 mt-0.5">
                  {{ group.showFullDate ? formatFullDate(session.startedAt) : formatTime(session.startedAt) }}
                </span>
              </div>

              <!-- Bottom row: tags -->
              <div class="flex items-center gap-1.5 mt-2">
                <span class="badge badge-sm badge-outline badge-primary/50 text-[10px]">
                  {{ session.source }}
                </span>
                <span class="badge badge-sm badge-ghost text-[10px]">
                  {{ session.messageCount }} msg{{ session.messageCount !== 1 ? 's' : '' }}
                </span>
                <span
                  v-if="session.model"
                  class="badge badge-sm badge-ghost text-[10px]"
                >
                  {{ formatModel(session.model) }}
                </span>
              </div>
            </button>
          </template>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionManager } from '@/composables/useSessionManager'
import type { Session } from '@/composables/useSessionManager'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const router = useRouter()

// Use session manager composable
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

// Local state
const searchQuery = ref('')
const searchInputRef = ref<HTMLInputElement | null>(null)

// Auto-refresh interval (ms)
const REFRESH_INTERVAL_MS = 30_000

// --- Computed ---

const isSearchMode = computed(() => searchQuery.value.trim().length > 0)

// --- Date helpers ---

type DateGroup = 'today' | 'yesterday' | 'thisWeek' | 'earlier'

const DATE_GROUP_LABELS: Record<DateGroup, string> = {
  today: 'Today',
  yesterday: 'Yesterday',
  thisWeek: 'This Week',
  earlier: 'Earlier',
}

function getDateGroup(ts: number | undefined): DateGroup {
  if (!ts) {return 'earlier'}
  const d = new Date(ts * 1000)
  const now = new Date()

  const isToday =
    d.getDate() === now.getDate() &&
    d.getMonth() === now.getMonth() &&
    d.getFullYear() === now.getFullYear()
  if (isToday) {return 'today'}

  const yesterday = new Date(now)
  yesterday.setDate(yesterday.getDate() - 1)
  const isYesterday =
    d.getDate() === yesterday.getDate() &&
    d.getMonth() === yesterday.getMonth() &&
    d.getFullYear() === yesterday.getFullYear()
  if (isYesterday) {return 'yesterday'}

  const weekAgo = new Date(now)
  weekAgo.setDate(weekAgo.getDate() - 7)
  if (d >= weekAgo) {return 'thisWeek'}

  return 'earlier'
}

interface SessionGroup {
  label: string
  sessions: Session[]
  showFullDate: boolean
}

const groupedSessions = computed<SessionGroup[]>(() => {
  const groups = new Map<DateGroup, Session[]>()
  for (const s of sessions.value) {
    const ts = s.lastActive || s.startedAt
    const group = getDateGroup(ts)
    if (!groups.has(group)) {groups.set(group, [])}
    groups.get(group)!.push(s)
  }
  const order: DateGroup[] = ['today', 'yesterday', 'thisWeek', 'earlier']
  return order
    .filter((label) => groups.has(label))
    .map((label) => ({
      label: DATE_GROUP_LABELS[label],
      sessions: groups.get(label)!,
      showFullDate: label === 'thisWeek' || label === 'earlier',
    }))
})

// --- Formatting ---

function formatTime(ts: number | undefined): string {
  if (!ts) {return ''}
  const d = new Date(ts * 1000)
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function formatFullDate(ts: number | null | undefined): string {
  if (!ts) {return ''}
  const d = new Date(ts * 1000)
  return (
    d.toLocaleDateString([], { month: 'short', day: 'numeric' }) +
    ', ' +
    d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  )
}

function formatModel(model: string): string {
  const name = model.split('/').pop() || model
  return name.split(':')[0]
}

function highlightedSnippet(snippet: string): string {
  return highlightSnippet(snippet)
}

// --- Search ---

watch(searchQuery, (query) => {
  handleSessionSearch(query)
})

function onClearSearch() {
  searchQuery.value = ''
  searchInputRef.value?.focus()
}

// --- Navigation ---

function onNewChat() {
  router.push('/agent/chat')
}

function onResumeSession(sessionId: string) {
  router.push({ path: '/agent/chat', query: { session: sessionId } })
}

// --- Auto-refresh ---

let refreshTimer: ReturnType<typeof setInterval> | null = null

function onWindowFocus() {
  refreshSessions()
}

onMounted(async () => {
  // Initial load
  await refreshSessions()

  // 30s auto-refresh
  refreshTimer = setInterval(() => {
    refreshSessions()
  }, REFRESH_INTERVAL_MS)

  // Window focus refresh
  window.addEventListener('focus', onWindowFocus)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
  window.removeEventListener('focus', onWindowFocus)
})
</script>
