1|<template>
2|  <div class="h-full flex flex-col">
3|    <!-- Claw mode overlay -->
4|    <div v-show="isOmpMode" class="flex-1 flex items-center justify-center">
5|      <div class="text-center max-w-md px-6">
6|        <SvgIcon name="terminal" :size="40" class="mx-auto text-base-content/20 mb-4" />
7|        <p class="text-sm font-medium text-base-content/50">Claw 定时任务</p>
8|        <p class="text-xs text-base-content/30 mt-2 leading-relaxed">
9|          Claw 不内置定时任务系统。如需自动化，请使用系统 cron 或 Hermes 的定时任务。
10|        </p>
11|      </div>
12|    </div>
13|
14|    <div v-show="!isOmpMode">
15|    <!-- Header -->
16|    <div class="flex items-center justify-between px-4 py-2 border-b border-base-content/10">
17|      <h1 class="text-sm font-medium">定时任务</h1>
18|      <div class="flex items-center gap-2">
19|        <button class="btn btn-sm btn-ghost" @click="refresh" :disabled="loading">
20|          <SvgIcon name="refresh" size="14" />
21|        </button>
22|        <button class="btn btn-sm btn-primary" @click="openCreateModal">
23|          <IconPlus size="14" class="mr-0.5" />
24|          新建任务
25|        </button>
26|      </div>
27|    </div>
28|
29|    <!-- Error message -->
30|    <div v-if="errorMsg" class="px-4 py-2 text-xs text-error bg-error/5 border-b border-error/10 flex items-center gap-2">
31|      <IconAlertCircle size="14" class="shrink-0" />
32|      <span class="flex-1">{{ errorMsg }}</span>
33|      <button class="btn btn-ghost btn-xs px-1" @click="errorMsg = ''"><SvgIcon name="x" size="12" /></button>
34|    </div>
35|
36|    <!-- Success message -->
37|    <div v-if="successMsg" class="px-4 py-2 text-xs text-success bg-success/5 border-b border-success/10 flex items-center gap-2">
38|      <IconCircleCheck size="14" class="shrink-0" />
39|      <span class="flex-1">{{ successMsg }}</span>
40|      <button class="btn btn-ghost btn-xs px-1" @click="successMsg = ''"><SvgIcon name="x" size="12" /></button>
41|    </div>
42|
43|    <!-- Loading state -->
44|    <div v-if="loading && jobs.length === 0" class="flex-1 flex items-center justify-center">
45|      <span class="text-xs text-base-content/40">加载中...</span>
46|    </div>
47|
48|    <!-- Empty state -->
49|    <div v-else-if="jobs.length === 0" class="flex-1 flex flex-col items-center justify-center gap-2">
50|      <IconClock size="32" class="text-base-content/20" stroke-width="1.5" />
51|      <p class="text-sm text-base-content/40">暂无定时任务</p>
52|      <button class="btn btn-sm btn-primary mt-2" @click="openCreateModal">新建定时任务</button>
53|    </div>
54|
55|    <!-- Job cards -->
56|    <div v-else class="flex-1 overflow-y-auto p-4">
57|      <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3">
58|        <div
59|          v-for="job in jobs"
60|          :key="job.id"
61|          class="bg-base-100 rounded-lg border border-base-content/10 p-3 hover:border-primary/30 transition-colors flex flex-col gap-2"
62|          :class="{ 'opacity-60': job.state === 'paused' || job.state === 'completed' }"
63|        >
64|          <!-- Row 1: name + status badge -->
65|          <div class="flex items-start justify-between gap-2">
66|            <span class="text-sm font-medium truncate" :title="job.name || job.id">{{ job.name || job.id }}</span>
67|            <span
68|              class="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded shrink-0"
69|              :class="statusBadgeClass(job)"
70|            >
71|              {{ job.state === 'paused' ? '已暂停' : job.state === 'active' ? '运行中' : job.state }}
72|            </span>
73|          </div>
74|
75|          <!-- Row 2: schedule -->
76|          <div class="flex items-center gap-1.5 text-xs text-base-content/60">
77|            <IconCalendarEvent size="12" class="shrink-0" />
78|            <span class="font-mono">{{ job.schedule }}</span>
79|          </div>
80|
81|          <!-- Row 3: prompt preview -->
82|          <p v-if="job.prompt" class="text-xs text-base-content/50 leading-relaxed line-clamp-2">{{ job.prompt }}</p>
83|
84|          <!-- Row 4: timestamps -->
85|          <div class="flex items-center justify-between text-[11px] text-base-content/40">
86|            <span v-if="job.last_run_at" title="上次运行">
87|              <IconPlayerPlay size="10" class="inline mr-0.5" />{{ formatTime(job.last_run_at) }}
88|            </span>
89|            <span v-else-if="job.last_error" title="上次错误" class="text-error/60">
90|              <IconAlertCircle size="10" class="inline mr-0.5" />{{ formatTime(job.last_error) }}
91|            </span>
92|            <span v-else>-</span>
93|            <span v-if="job.next_run_at" title="下次运行">
94|              <IconClock size="10" class="inline mr-0.5" />{{ formatTime(job.next_run_at) }}
95|            </span>
96|            <span v-else class="text-base-content/30">-</span>
97|          </div>
98|
99|          <!-- Last error -->
100|          <div v-if="job.last_error" class="text-[11px] text-error/70 truncate" :title="job.last_error">
101|            <IconAlertCircle size="10" class="inline mr-0.5 shrink-0" />{{ job.last_error }}
102|          </div>
103|
104|          <!-- Row 5: actions -->
105|          <div class="flex items-center gap-1 pt-1 border-t border-base-content/5 mt-auto">
106|            <button
107|              class="btn btn-ghost btn-xs px-1.5 text-base-content/50 hover:text-primary"
108|              title="立即执行"
109|              :disabled="actionLoading === job.id"
110|              @click="triggerJob(job)"
111|            >
112|              <IconPlayerPlay size="13" />
113|            </button>
114|            <button
115|              v-if="job.state === 'paused' || job.state === 'completed'"
116|              class="btn btn-ghost btn-xs px-1.5 text-base-content/50 hover:text-success"
117|              title="恢复"
118|              :disabled="actionLoading === job.id"
119|              @click="resumeJob(job)"
120|            >
121|              <IconPlayerPlay size="13" />
122|            </button>
123|            <button
124|              v-else-if="job.state === 'active'"
125|              class="btn btn-ghost btn-xs px-1.5 text-base-content/50 hover:text-warning"
126|              title="暂停"
127|              :disabled="actionLoading === job.id"
128|              @click="pauseJob(job)"
129|            >
130|              <IconPlayerPause size="13" />
131|            </button>
132|            <button
133|              class="btn btn-ghost btn-xs px-1.5 text-base-content/50 hover:text-error"
134|              title="删除"
135|              :disabled="actionLoading === job.id"
136|              @click="removeJob(job)"
137|            >
138|              <IconTrash size="13" />
139|            </button>
140|          </div>
141|        </div>
142|      </div>
143|    </div>
144|
145|    <!-- Create Modal -->
146|    <Modal v-model="showCreateModal" title="新建定时任务" width="540px">
147|      <div class="space-y-4">
148|        <!-- Name -->
149|        <div>
150|          <label class="text-xs font-medium text-base-content/70 block mb-1">任务名称</label>
151|          <input
152|            v-model="form.name"
153|            type="text"
154|            class="input input-bordered input-sm w-full text-sm"
155|            placeholder="可选，留空自动生成"
156|          />
157|        </div>
158|
159|        <!-- Schedule -->
160|        <div>
161|          <label class="text-xs font-medium text-base-content/70 block mb-1">
162|            调度规则 <span class="text-error">*</span>
163|          </label>
164|          <input
165|            v-model="form.schedule"
166|            type="text"
167|            class="input input-bordered input-sm w-full text-sm"
168|            placeholder="e.g. every 2h, 0 9 * * *, once in 30m"
169|          />
170|          <p class="text-[11px] text-base-content/40 mt-1">支持自然语言描述、cron 表达式或 "once in 30m" 格式</p>
171|        </div>
172|
173|        <!-- Prompt -->
174|        <div>
175|          <label class="text-xs font-medium text-base-content/70 block mb-1">任务内容 (Prompt)</label>
176|          <textarea
177|            v-model="form.prompt"
178|            class="textarea textarea-bordered textarea-sm w-full text-sm"
179|            rows="4"
180|            placeholder="可选，描述任务的执行内容"
181|          ></textarea>
182|        </div>
183|
184|        <!-- Deliver -->
185|        <div>
186|          <label class="text-xs font-medium text-base-content/70 block mb-1">送达方式</label>
187|          <input
188|            v-model="form.deliver"
189|            type="text"
190|            class="input input-bordered input-sm w-full text-sm"
191|            placeholder="可选，如 telegram, discord:#channel"
192|          />
193|          <p class="text-[11px] text-base-content/40 mt-1">留空使用默认通知方式</p>
194|        </div>
195|      </div>
196|
197|      <template #footer>
198|        <button class="btn btn-sm btn-ghost" @click="showCreateModal = false">取消</button>
199|        <button
200|          class="btn btn-sm btn-primary"
201|          :disabled="!form.schedule.trim() || creating"
202|          @click="createJob"
203|        >
204|          {{ creating ? '创建中...' : '创建' }}
205|        </button>
206|      </template>
207|    </Modal>
208|  </div>
209|  </div>
210|</template>
211|
212|<script setup lang="ts">
213|import { ref, computed, onMounted } from 'vue'
214|import { useAgentModeStore } from '@/stores/agentModeStore'
215|import SvgIcon from '@/components/ui/SvgIcon.vue'
216|import Modal from '@/components/ui/Modal.vue'
217|import {
218|  IconClock,
219|  IconPlus,
220|  IconTrash,
221|  IconPlayerPlay,
222|  IconPlayerPause,
223|  IconCalendarEvent,
224|  IconAlertCircle,
225|  IconCircleCheck,
226|} from '@tabler/icons-vue'
227|import { getTauriAPI } from '@/utils/tauri-api'
228|import type { CronJob } from '@/types'
229|
230|const agentModeStore = useAgentModeStore()
231|const isOmpMode = computed(() => agentModeStore.mode === 'claw')
232|
233|const loading = ref(false)
234|const jobs = ref<CronJob[]>([])
235|const errorMsg = ref('')
236|const successMsg = ref('')
237|const actionLoading = ref<string | null>(null)
238|const creating = ref(false)
239|const showCreateModal = ref(false)
240|const form = ref({
241|  name: '',
242|  schedule: '',
243|  prompt: '',
244|  deliver: '',
245|})
246|
247|function statusBadgeClass(job: CronJob): string {
248|  switch (job.state) {
249|    case 'active':
250|      return 'bg-success/15 text-success'
251|    case 'paused':
252|      return 'bg-warning/15 text-warning'
253|    case 'completed':
254|      return 'bg-info/15 text-info'
255|    default:
256|      return 'bg-base-300 text-base-content/60'
257|  }
258|}
259|
260|function formatTime(t: string | null): string {
261|  if (!t) {return ''}
262|  try {
263|    const d = new Date(t)
264|    return d.toLocaleString('zh-CN', {
265|      month: '2-digit',
266|      day: '2-digit',
267|      hour: '2-digit',
268|      minute: '2-digit',
269|    })
270|  } catch {
271|    return t
272|  }
273|}
274|
275|function showError(msg: string) {
276|  errorMsg.value = msg
277|  successMsg.value = ''
278|  setTimeout(() => { errorMsg.value = '' }, 6000)
279|}
280|
281|function showSuccess(msg: string) {
282|  successMsg.value = msg
283|  errorMsg.value = ''
284|  setTimeout(() => { successMsg.value = '' }, 3000)
285|}
286|
287|async function refresh() {
288|  loading.value = true
289|  errorMsg.value = ''
290|  try {
291|    const api = getTauriAPI()
292|    jobs.value = await api.listCronJobs()
293|  } catch (e: any) {
294|    showError(e?.message || '加载失败')
295|  } finally {
296|    loading.value = false
297|  }
298|}
299|
300|function openCreateModal() {
301|  form.value = { name: '', schedule: '', prompt: '', deliver: '' }
302|  showCreateModal.value = true
303|}
304|
305|async function createJob() {
306|  if (!form.value.schedule.trim()) {return}
307|  creating.value = true
308|  try {
309|    const api = getTauriAPI()
310|    await api.createCronJob(
311|      form.value.schedule.trim(),
312|      form.value.prompt.trim() || undefined,
313|      form.value.name.trim() || undefined,
314|      form.value.deliver.trim() || undefined,
315|    )
316|    showCreateModal.value = false
317|    showSuccess('任务创建成功')
318|    await refresh()
319|  } catch (e: any) {
320|    showError(e?.message || '创建失败')
321|  } finally {
322|    creating.value = false
323|  }
324|}
325|
326|async function removeJob(job: CronJob) {
327|  if (!confirm(`确定要删除定时任务「${job.name || job.id}」吗？`)) {return}
328|  actionLoading.value = job.id
329|  try {
330|    const api = getTauriAPI()
331|    await api.removeCronJob(job.id)
332|    showSuccess('任务已删除')
333|    await refresh()
334|  } catch (e: any) {
335|    showError(e?.message || '删除失败')
336|  } finally {
337|    actionLoading.value = null
338|  }
339|}
340|
341|async function pauseJob(job: CronJob) {
342|  actionLoading.value = job.id
343|  try {
344|    const api = getTauriAPI()
345|    await api.pauseCronJob(job.id)
346|    showSuccess('任务已暂停')
347|    await refresh()
348|  } catch (e: any) {
349|    showError(e?.message || '暂停失败')
350|  } finally {
351|    actionLoading.value = null
352|  }
353|}
354|
355|async function resumeJob(job: CronJob) {
356|  actionLoading.value = job.id
357|  try {
358|    const api = getTauriAPI()
359|    await api.resumeCronJob(job.id)
360|    showSuccess('任务已恢复')
361|    await refresh()
362|  } catch (e: any) {
363|    showError(e?.message || '恢复失败')
364|  } finally {
365|    actionLoading.value = null
366|  }
367|}
368|
369|async function triggerJob(job: CronJob) {
370|  actionLoading.value = job.id
371|  try {
372|    const api = getTauriAPI()
373|    await api.triggerCronJob(job.id)
374|    showSuccess('任务已触发执行')
375|    await refresh()
376|  } catch (e: any) {
377|    showError(e?.message || '触发失败')
378|  } finally {
379|    actionLoading.value = null
380|  }
381|}
382|
383|onMounted(refresh)
384|</script>
385|