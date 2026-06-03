1|<template>
2|  <div class="h-full overflow-y-auto">
3|    <div class="max-w-3xl mx-auto px-4 py-6">
4|      <!-- Header -->
5|      <div class="flex items-center justify-between mb-6">
6|        <h1 class="text-xl font-bold text-base-content">
7|          {{ isClawMode ? 'Claw Sessions' : 'Sessions' }}
8|        </h1>
9|        <button
10|          class="btn btn-primary btn-sm gap-1.5"
11|          @click="onNewChat"
12|        >
13|          <SvgIcon name="plus" :size="14" />
14|          New Chat
15|        </button>
16|      </div>
17|
18|      <!-- ====== Claw Mode ====== -->
19|      <template v-if="isClawMode">
20|        <!-- Loading -->
21|        <div
22|          v-if="clawLoading"
23|          class="flex items-center justify-center py-20"
24|        >
25|          <span class="loading loading-spinner loading-md text-primary" />
26|        </div>
27|
28|        <!-- Not initialized -->
29|        <div
30|          v-else-if="clawError && clawError.includes('not initialized')"
31|          class="flex flex-col items-center justify-center py-20 text-center"
32|        >
33|          <SvgIcon name="terminal" :size="32" class="text-base-content/20 mb-3" />
34|          <p class="text-sm font-medium text-base-content/50">Claw 未初始化</p>
35|          <p class="text-xs text-base-content/30 mt-1">请先在对话页面中切换到 Claw 模式发送消息</p>
36|        </div>
37|
38|        <!-- Error -->
39|        <div
40|          v-else-if="clawError"
41|          class="alert alert-error text-sm py-2 mb-4"
42|        >
43|          <span>{{ clawError }}</span>
44|        </div>
45|
46|        <!-- Empty -->
47|        <div
48|          v-else-if="clawSessions.length === 0"
49|          class="flex flex-col items-center justify-center py-20 text-center"
50|        >
51|          <SvgIcon name="chat" :size="32" class="text-base-content/20 mb-3" />
52|          <p class="text-sm font-medium text-base-content/50">No Claw sessions</p>
53|          <p class="text-xs text-base-content/30 mt-1">Start a conversation in Claw mode</p>
54|        </div>
55|
56|        <!-- Claw session list -->
57|        <div v-else class="flex flex-col gap-1">
58|          <button
59|            v-for="s in clawSessions"
60|            :key="s.sessionId"
61|            class="group w-full text-left rounded-lg p-3 transition-colors hover:bg-base-200 border-l-2 border-l-transparent"
62|          >
63|            <div class="flex items-start justify-between gap-3">
64|              <span class="text-sm font-medium text-base-content truncate leading-snug">
65|                {{ s.cwd || `Session ${s.sessionId.slice(-6)}` }}
66|              </span>
67|              <span class="text-xs text-base-content/40 shrink-0 mt-0.5">
68|                {{ formatDate(s.createdAt) }}
69|              </span>
70|            </div>
71|            <div class="flex items-center gap-1.5 mt-2">
72|              <span class="badge badge-sm badge-outline badge-primary/50 text-[10px]">Claw</span>
73|              <span class="badge badge-sm badge-ghost text-[10px]">active</span>
74|            </div>
75|          </button>
76|        </div>
77|      </template>
78|
79|      <!-- ====== Hermes Mode (existing) ====== -->
80|      <template v-else>
81|        <!-- Search bar -->
82|        <div class="relative mb-6">
83|          <SvgIcon
84|            name="search"
85|            :size="14"
86|            class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40"
87|          />
88|          <input
89|            ref="searchInputRef"
90|            v-model="searchQuery"
91|            type="text"
92|            class="input input-bordered w-full pl-9 pr-9 text-sm"
93|            placeholder="Search sessions..."
94|          />
95|          <button
96|            v-if="searchQuery"
97|            class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
98|            @click="onClearSearch"
99|          >
100|            <SvgIcon name="x" :size="13" />
101|          </button>
102|        </div>
103|
104|        <!-- Loading -->
105|        <div
106|          v-if="loadingSessions"
107|          class="flex items-center justify-center py-20"
108|        >
109|          <span class="loading loading-spinner loading-md text-primary" />
110|        </div>
111|
112|        <!-- Search results -->
113|        <template v-else-if="isSearchMode">
114|          <div
115|            v-if="isSearching"
116|            class="flex items-center justify-center py-20"
117|          >
118|            <span class="loading loading-spinner loading-md text-primary" />
119|          </div>
120|          <div
121|            v-else-if="searchResults.length === 0"
122|            class="flex flex-col items-center justify-center py-20 text-center"
123|          >
124|            <SvgIcon name="search" :size="32" class="text-base-content/20 mb-3" />
125|            <p class="text-sm font-medium text-base-content/50">No results found</p>
126|            <p class="text-xs text-base-content/30 mt-1">Try a different search term</p>
127|          </div>
128|          <div v-else class="flex flex-col gap-1">
129|            <button
130|              v-for="result in searchResults"
131|              :key="result.messageId"
132|              class="group w-full text-left rounded-lg p-3 transition-colors hover:bg-base-200 border-l-2"
133|              :class="
134|                currentSessionId === result.sessionId
135|                  ? 'border-l-primary bg-primary/5'
136|                  : 'border-l-transparent'
137|              "
138|              @click="onResumeSession(result.sessionId)"
139|            >
140|              <div class="flex items-start justify-between gap-3">
141|                <span class="text-sm font-medium text-base-content truncate leading-snug">
142|                  {{ result.sessionTitle || `Session ${result.sessionId.slice(-6)}` }}
143|                </span>
144|                <span class="text-xs text-base-content/40 shrink-0 mt-0.5">
145|                  {{ formatFullDate(result.timestamp) }}
146|                </span>
147|              </div>
148|              <div
149|                v-if="result.snippet"
150|                class="mt-1.5 text-xs text-base-content/60 leading-relaxed line-clamp-3"
151|                v-html="highlightedSnippet(result.snippet)"
152|              />
153|              <div class="flex items-center gap-1.5 mt-2">
154|                <span class="badge badge-sm badge-outline badge-primary/50 text-[10px]">
155|                  {{ result.source }}
156|                </span>
157|                <span v-if="result.model" class="badge badge-sm badge-ghost text-[10px]">
158|                  {{ formatModel(result.model) }}
159|                </span>
160|              </div>
161|            </button>
162|          </div>
163|        </template>
164|
165|        <!-- Normal mode -->
166|        <template v-else>
167|          <div
168|            v-if="sessions.length === 0"
169|            class="flex flex-col items-center justify-center py-20 text-center"
170|          >
171|            <SvgIcon name="chat" :size="32" class="text-base-content/20 mb-3" />
172|            <p class="text-sm font-medium text-base-content/50">No conversations yet</p>
173|            <p class="text-xs text-base-content/30 mt-1">Start a new chat to begin</p>
174|          </div>
175|          <div v-else class="flex flex-col gap-1">
176|            <template v-for="group in groupedSessions" :key="group.label">
177|              <div
178|                class="px-3 pt-4 pb-1.5 text-[10px] font-semibold text-base-content/40 uppercase tracking-wider"
179|              >
180|                {{ group.label }}
181|              </div>
182|              <button
183|                v-for="session in group.sessions"
184|                :key="session.id"
185|                class="group w-full text-left rounded-lg p-3 transition-colors hover:bg-base-200 border-l-2"
186|                :class="
187|                  currentSessionId === session.id
188|                    ? 'border-l-primary bg-primary/5'
189|                    : 'border-l-transparent'
190|                "
191|                @click="onResumeSession(session.id)"
192|              >
193|                <div class="flex items-start justify-between gap-3">
194|                  <span class="text-sm font-medium text-base-content truncate leading-snug">
195|                    {{ session.title || 'New conversation' }}
196|                  </span>
197|                  <span class="text-xs text-base-content/40 shrink-0 mt-0.5">
198|                    {{ group.showFullDate ? formatFullDate(session.startedAt) : formatTime(session.startedAt) }}
199|                  </span>
200|                </div>
201|                <div class="flex items-center gap-1.5 mt-2">
202|                  <span class="badge badge-sm badge-outline badge-primary/50 text-[10px]">
203|                    {{ session.source }}
204|                  </span>
205|                  <span class="badge badge-sm badge-ghost text-[10px]">
206|                    {{ session.messageCount }} msg{{ session.messageCount !== 1 ? 's' : '' }}
207|                  </span>
208|                  <span
209|                    v-if="session.model"
210|                    class="badge badge-sm badge-ghost text-[10px]"
211|                  >
212|                    {{ formatModel(session.model) }}
213|                  </span>
214|                </div>
215|              </button>
216|            </template>
217|          </div>
218|        </template>
219|      </template>
220|    </div>
221|  </div>
222|</template>
223|
224|<script setup lang="ts">
225|import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
226|import { useRouter } from 'vue-router'
227|import { useSessionManager } from '@/composables/useSessionManager'
228|import type { Session } from '@/composables/useSessionManager'
229|import SvgIcon from '@/components/ui/SvgIcon.vue'
230|import { useAgentModeStore } from '@/stores/agentModeStore'
231|import { getTauriAPI } from '@/utils/tauri-api'
232|
233|const router = useRouter()
234|const agentModeStore = useAgentModeStore()
235|
236|const isOmpMode = computed(() => agentModeStore.mode === 'claw')
237|
238|// Hermes session manager
239|const {
240|  sessions,
241|  searchResults,
242|  isSearching,
243|  currentSessionId,
244|  loadingSessions,
245|  refreshSessions,
246|  handleSessionSearch,
247|  highlightSnippet,
248|} = useSessionManager()
249|
250|// Claw session state
251|const ompSessions = ref<{ sessionId: string; cwd: string | null; createdAt: string | null }[]>([])
252|const ompLoading = ref(false)
253|const ompError = ref('')
254|let refreshTimer: ReturnType<typeof setInterval> | null = null
255|
256|async function loadOmpSessions() {
257|  ompLoading.value = true
258|  ompError.value = ''
259|  try {
260|    const api = getTauriAPI()
261|    const raw = await api.ompChatListSessions() as any
262|    // Parse ACP sessions/list response
263|    if (raw?.sessions && Array.isArray(raw.sessions)) {
264|      ompSessions.value = raw.sessions
265|    } else if (Array.isArray(raw)) {
266|      ompSessions.value = raw
267|    } else {
268|      ompSessions.value = []
269|    }
270|  } catch (e: any) {
271|    ompError.value = String(e?.message || e)
272|    ompSessions.value = []
273|  } finally {
274|    ompLoading.value = false
275|  }
276|}
277|
278|// Local state
279|const searchQuery = ref('')
280|const searchInputRef = ref<HTMLInputElement | null>(null)
281|
282|const REFRESH_INTERVAL_MS = 30_000
283|
284|// --- Computed ---
285|
286|const isSearchMode = computed(() => searchQuery.value.trim().length > 0)
287|
288|// --- Date helpers ---
289|
290|type DateGroup = 'today' | 'yesterday' | 'thisWeek' | 'earlier'
291|
292|const DATE_GROUP_LABELS: Record<DateGroup, string> = {
293|  today: 'Today',
294|  yesterday: 'Yesterday',
295|  thisWeek: 'This Week',
296|  earlier: 'Earlier',
297|}
298|
299|function getDateGroup(ts: number | undefined): DateGroup {
300|  if (!ts) {return 'earlier'}
301|  const d = new Date(ts * 1000)
302|  const now = new Date()
303|
304|  const isToday =
305|    d.getDate() === now.getDate() &&
306|    d.getMonth() === now.getMonth() &&
307|    d.getFullYear() === now.getFullYear()
308|  if (isToday) {return 'today'}
309|
310|  const yesterday = new Date(now)
311|  yesterday.setDate(yesterday.getDate() - 1)
312|  const isYesterday =
313|    d.getDate() === yesterday.getDate() &&
314|    d.getMonth() === yesterday.getMonth() &&
315|    d.getFullYear() === yesterday.getFullYear()
316|  if (isYesterday) {return 'yesterday'}
317|
318|  const weekAgo = new Date(now)
319|  weekAgo.setDate(weekAgo.getDate() - 7)
320|  if (d >= weekAgo) {return 'thisWeek'}
321|
322|  return 'earlier'
323|}
324|
325|interface SessionGroup {
326|  label: string
327|  sessions: Session[]
328|  showFullDate: boolean
329|}
330|
331|const groupedSessions = computed<SessionGroup[]>(() => {
332|  const groups = new Map<DateGroup, Session[]>()
333|  for (const s of sessions.value) {
334|    const ts = s.lastActive || s.startedAt
335|    const group = getDateGroup(ts)
336|    if (!groups.has(group)) {groups.set(group, [])}
337|    groups.get(group)!.push(s)
338|  }
339|  const order: DateGroup[] = ['today', 'yesterday', 'thisWeek', 'earlier']
340|  return order
341|    .filter((label) => groups.has(label))
342|    .map((label) => ({
343|      label: DATE_GROUP_LABELS[label],
344|      sessions: groups.get(label)!,
345|      showFullDate: label === 'thisWeek' || label === 'earlier',
346|    }))
347|})
348|
349|function formatDate(dateStr: string | null): string {
350|  if (!dateStr) return ''
351|  try {
352|    const d = new Date(dateStr)
353|    return d.toLocaleDateString([], { month: 'short', day: 'numeric' })
354|  } catch {
355|    return dateStr
356|  }
357|}
358|
359|function formatTime(ts: number | undefined): string {
360|  if (!ts) {return ''}
361|  const d = new Date(ts * 1000)
362|  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
363|}
364|
365|function formatFullDate(ts: number | null | undefined): string {
366|  if (!ts) {return ''}
367|  const d = new Date(ts * 1000)
368|  return (
369|    d.toLocaleDateString([], { month: 'short', day: 'numeric' }) +
370|    ', ' +
371|    d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
372|  )
373|}
374|
375|function formatModel(model: string): string {
376|  const name = model.split('/').pop() || model
377|  return name.split(':')[0]
378|}
379|
380|function highlightedSnippet(snippet: string): string {
381|  return highlightSnippet(snippet)
382|}
383|
384|// --- Search ---
385|
386|watch(searchQuery, (query) => {
387|  handleSessionSearch(query)
388|})
389|
390|function onClearSearch() {
391|  searchQuery.value = ''
392|  searchInputRef.value?.focus()
393|}
394|
395|// --- Navigation ---
396|
397|function onNewChat() {
398|  router.push('/agent/chat')
399|}
400|
401|function onResumeSession(sessionId: string) {
402|  router.push({ path: '/agent/chat', query: { session: sessionId } })
403|}
404|
405|// --- Mode switch ---
406|
407|watch(isOmpMode, (omp) => {
408|  if (omp) {
409|    loadOmpSessions()
410|  } else {
411|    refreshSessions()
412|  }
413|})
414|
415|// --- Auto-refresh ---
416|
417|function onWindowFocus() {
418|  if (isOmpMode.value) {
419|    loadOmpSessions()
420|  } else {
421|    refreshSessions()
422|  }
423|}
424|
425|onMounted(async () => {
426|  if (isOmpMode.value) {
427|    await loadOmpSessions()
428|  } else {
429|    await refreshSessions()
430|  }
431|
432|  refreshTimer = setInterval(() => {
433|    if (isOmpMode.value) {
434|      loadOmpSessions()
435|    } else {
436|      refreshSessions()
437|    }
438|  }, REFRESH_INTERVAL_MS)
439|
440|  window.addEventListener('focus', onWindowFocus)
441|})
442|
443|onUnmounted(() => {
444|  if (refreshTimer) {
445|    clearInterval(refreshTimer)
446|    refreshTimer = null
447|  }
448|  window.removeEventListener('focus', onWindowFocus)
449|})
450|</script>
451|