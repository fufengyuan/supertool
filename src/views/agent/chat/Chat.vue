1|<template>
2|  <div class="flex flex-col h-full bg-base-100">
3|    <!-- Header -->
4|    <ChatHeader
5|      :session-id="hermesSessionId"
6|      :usage="usage"
7|      :fast-mode="fastMode"
8|      :has-messages="messages.length > 0"
9|      :context-folder="contextFolder"
10|      :show-context-folder="true"
11|      @pick-folder="pickContextFolder"
12|      @clear-folder="clearContextFolder"
13|      @toggle-fast="handleToggleFast"
14|      @new-chat="startNewChat"
15|      @clear="clearChat"
16|    />
17|
18|    <!-- Messages area -->
19|    <div ref="containerRef" class="flex-1 overflow-y-auto min-h-0">
20|      <template v-if="messages.length > 0">
21|        <MessageList
22|          ref="messageListRef"
23|          :messages="messages"
24|          :is-loading="isLoading"
25|          :tool-progress="toolProgress"
26|          :on-approve="handleApprove"
27|          :on-deny="handleDeny"
28|        />
29|      </template>
30|      <template v-else>
31|        <ChatEmptyState @select-suggestion="handleSuggestion" />
32|      </template>
33|      <div ref="bottomRef" />
34|    </div>
35|
36|    <!-- Input area -->
37|    <div class="border-t border-base-content/10 bg-base-100/80 backdrop-blur-sm">
38|      <!-- Approval mode indicator -->
39|      <div
40|        v-if="isApprovalMode"
41|        class="px-4 py-1.5 bg-primary/10 border-t border-primary/20 flex items-center gap-2"
42|      >
43|        <SvgIcon name="clock" size="12" class="text-primary animate-pulse" />
44|        <span class="text-xs text-primary/70">等待审批中...</span>
45|      </div>
46|
47|      <div class="px-4 py-3">
48|        <div class="max-w-3xl mx-auto">
49|          <!-- Model picker row -->
50|          <div class="flex items-center justify-between mb-2">
51|            <ModelPicker
52|              :current-model="currentModel"
53|              :current-provider="currentProvider"
54|              :current-base-url="currentBaseUrl"
55|              :model-groups="modelGroups"
56|              :display-model="displayModel"
57|              @open="reload"
58|              @select-model="handleSelectModel"
59|            />
60|            <div class="flex items-center gap-2 text-[10px] text-base-content/30">
61|              <span
62|                v-if="fastMode"
63|                class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-warning/10 text-warning/70"
64|              >
65|                <SvgIcon name="zap" size="8" />
66|                优先模式
67|              </span>
68|              <span>⌘ ↵ 发送</span>
69|            </div>
70|          </div>
71|
72|          <!-- ChatInput -->
73|          <SimpleChatInput
74|            ref="chatInputRef"
75|            :model-value="currentInput"
76|            :is-loading="isLoading"
77|            :is-approval-mode="isApprovalMode"
78|            :placeholder="inputPlaceholder"
79|            @update:model-value="currentInput = $event"
80|            @send="handleSendInput"
81|            @abort="handleAbort"
82|            @approve="handleApprove"
83|            @deny="handleDeny"
84|          />
85|        </div>
86|      </div>
87|    </div>
88|  </div>
89|</template>
90|
91|<script setup lang="ts">
92|import { ref, computed, onMounted, onUnmounted } from 'vue';
93|import { useRoute } from 'vue-router';
94|import { open } from '@tauri-apps/plugin-dialog';
95|import { invoke } from '@tauri-apps/api/core';
96|
97|import SvgIcon from '@/components/ui/SvgIcon.vue';
98|
99|import ChatHeader from './components/ChatHeader.vue';
100|import ChatEmptyState from './components/ChatEmptyState.vue';
101|import MessageList from './components/MessageList.vue';
102|import ModelPicker from './ModelPicker.vue';
103|import SimpleChatInput from './SimpleChatInput.vue';
104|
105|import { useChatIPC } from './composables/useChatIPC';
106|import { useChatActions } from './composables/useChatActions';
107|import { useChatScroll } from './composables/useChatScroll';
108|import { useFastMode } from './composables/useFastMode';
109|import { useInputHistory } from './composables/useInputHistory';
110|import { useModelConfig } from './composables/useModelConfig';
111|import { useLocalCommands } from './composables/useLocalCommands';
112|
113|import type { ChatMessage, UsageState } from './types';
114|import { hermesMessagesToChatMessages } from './sessionHistory';
115|
116|import { useAgentModeStore } from '@/stores/agentModeStore';
117|
118|const route = useRoute();
119|const agentModeStore = useAgentModeStore();
120|
121|// ── Core state ───────────────────────────────────────────────────────────────
122|const messages = ref<ChatMessage[]>([]);
123|const hermesSessionId = ref<string | null>(null);
124|const isLoading = ref(false);
125|const toolProgress = ref<string | null>(null);
126|const usage = ref<UsageState | null>(null);
127|const contextFolder = ref<string | null>(null);
128|const currentInput = ref('');
129|const chatInputRef = ref<{ clear: () => void; focus: () => void } | null>(null);
130|const messageListRef = ref<InstanceType<typeof MessageList> | null>(null);
131|
132|// Claw 模式状态
133|const clawInitialized = ref(false);
134|const isClawMode = computed(() => agentModeStore.mode === 'claw');
135|
136|function setMessages(msgs: ChatMessage[]) {
137|  messages.value = msgs;
138|}
139|
140|// ── Helper: add an agent-side text message ────────────────────────────────────
141|function addAgentMessage(content: string) {
142|  setMessages([
143|    ...messages.value,
144|    { id: `agent-${Date.now()}`, role: 'agent', content },
145|  ]);
146|}
147|
148|function startNewChat() {
149|  if (messages.value.length > 0 && !window.confirm('确认开始新对话？当前对话内容将保留。')) return;
150|  messages.value = [];
151|  hermesSessionId.value = null;
152|  usage.value = null;
153|  toolProgress.value = null;
154|  currentInput.value = '';
155|}
156|
157|function clearChat() {
158|  if (!window.confirm('确认清空当前对话内容？')) return;
159|  messages.value = [];
160|  usage.value = null;
161|  toolProgress.value = null;
162|}
163|
164|// ── Local commands (must be set up before chat actions) ──────────────────────
165|const localCommands = useLocalCommands({
166|  usage,
167|  setFastMode: async (next: boolean) => {
168|    await invoke('hermes_set_config', {
169|      key: 'agent.service_tier',
170|      value: next ? 'fast' : 'normal',
171|    });
172|    fastMode.value = next;
173|  },
174|  onNewChat: startNewChat,
175|  onClear: clearChat,
176|  addAgentMessage,
177|});
178|
179|// ── Scroll management ────────────────────────────────────────────────────────
180|const { containerRef, bottomRef, scrollToBottom } = useChatScroll(messages);
181|
182|// ── Chat IPC listeners ───────────────────────────────────────────────────────
183|useChatIPC({
184|  messages,
185|  setMessages,
186|  hermesSessionId,
187|  toolProgress,
188|  isLoading,
189|  usage,
190|  scrollToBottom,
191|});
192|
193|// ── Fast mode ────────────────────────────────────────────────────────────────
194|const { fastMode, toggle: doToggleFast } = useFastMode();
195|
196|function handleToggleFast() {
197|  doToggleFast();
198|}
199|
200|// ── Input history ────────────────────────────────────────────────────────────
201|const { push: pushHistory, recallPrev, recallNext } = useInputHistory({
202|  currentInput,
203|  applyText: (text: string) => {
204|    currentInput.value = text;
205|    chatInputRef.value?.focus();
206|  },
207|});
208|
209|// ── Model config ─────────────────────────────────────────────────────────────
210|const {
211|  currentModel,
212|  currentProvider,
213|  currentBaseUrl,
214|  modelGroups,
215|  displayModel,
216|  reload,
217|  selectModel,
218|} = useModelConfig();
219|
220|// ── Chat actions (depends on localCommands, scrollToBottom, etc.) ────────────
221|const {
222|  handleSend: doSend,
223|  handleAbort,
224|  handleApprove,
225|  handleDeny,
226|} = useChatActions({
227|  hermesSessionId,
228|  messages,
229|  setMessages,
230|  isLoading,
231|  onSessionStarted: scrollToBottom,
232|  localCommands,
233|  contextFolder,
234|  scrollToBottom,
235|  inputRef: chatInputRef,
236|});
237|
238|// ── Derived ──────────────────────────────────────────────────────────────────
239|const isApprovalMode = computed(() => {
240|  if (messages.value.length === 0) return false;
241|  const last = messages.value[messages.value.length - 1];
242|  return last.kind === 'tool_call' && last.name === 'ask_user_question';
243|});
244|
245|const inputPlaceholder = computed(() => {
246|  if (isApprovalMode.value) return '输入回复内容或审批操作（/approve /deny）...';
247|  if (isLoading.value) return 'Agent 正在处理...';
248|  return '输入消息，Ctrl+Enter 发送...';
249|});
250|
251|// ── Handlers ─────────────────────────────────────────────────────────────────
252|
253|/** 添加用户消息到列表 */
254|function pushUser(content: string) {
255|  setMessages([
256|    ...messages.value,
257|    { id: `user-${Date.now()}`, role: 'user', content },
258|  ])
259|}
260|
261|/** Claw 模式：初始化连接 */
262|async function ensureClawChat() {
263|  if (clawInitialized.value) return
264|  try {
265|    await invoke('claw_chat_init', { cwd: null as string | null })
266|    clawInitialized.value = true
267|    addAgentMessage('Claw 编码助手已就绪')
268|  } catch (e: any) {
269|    addAgentMessage(`Claw 初始化失败: ${e?.message || String(e)}`)
270|    isLoading.value = false
271|  }
272|}
273|
274|/** Claw 模式：发送消息 */
275|async function clawSend(text: string) {
276|  isLoading.value = true
277|  await ensureClawChat()
278|  try {
279|    await invoke('claw_chat_send', { message: text })
280|  } catch (e: any) {
281|    addAgentMessage(`发送失败: ${e?.message || String(e)}`)
282|    isLoading.value = false
283|  }
284|}
285|
286|async function handleSendInput() {
287|  const text = currentInput.value.trim();
288|  if (!text) return;
289|
290|  // Claw 模式 → 走 ACP 协议
291|  if (isClawMode.value) {
292|    chatInputRef.value?.clear()
293|    pushUser(text)
294|    await clawSend(text)
295|    pushHistory(text)
296|    return
297|  }
298|
299|  if (isApprovalMode.value) {
300|    if (/^\/approve$/i.test(text)) {
301|      chatInputRef.value?.clear();
302|      handleApprove();
303|    } else if (/^\/deny$/i.test(text)) {
304|      chatInputRef.value?.clear();
305|      handleDeny();
306|    } else {
307|      await doSend(text);
308|      chatInputRef.value?.clear();
309|    }
310|  } else {
311|    await doSend(text);
312|    chatInputRef.value?.clear();
313|  }
314|
315|  pushHistory(text);
316|}
317|
318|function handleSuggestion(text: string) {
319|  currentInput.value = text;
320|  handleSendInput();
321|}
322|
323|async function handleSelectModel(provider: string, model: string, baseUrl: string) {
324|  await selectModel(provider, model, baseUrl);
325|  try {
326|    await invoke('agent_set_config', {
327|      config: { provider, model, ...(baseUrl ? { baseUrl } : {}) },
328|    });
329|  } catch {
330|    // Best-effort sync to backend
331|  }
332|}
333|
334|async function pickContextFolder() {
335|  try {
336|    const selected = await open({ directory: true });
337|    if (typeof selected === 'string') {
338|      contextFolder.value = selected;
339|    }
340|  } catch {
341|    // User cancelled
342|  }
343|}
344|
345|function clearContextFolder() {
346|  contextFolder.value = null;
347|}
348|
349|// ── Keyboard ─────────────────────────────────────────────────────────────────
350|function handleKeydown(e: KeyboardEvent) {
351|  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
352|    e.preventDefault();
353|    handleSendInput();
354|    return;
355|  }
356|
357|  if (e.key === 'ArrowUp') {
358|    const prev = recallPrev();
359|    if (prev) e.preventDefault();
360|  } else if (e.key === 'ArrowDown') {
361|    const next = recallNext();
362|    if (next) e.preventDefault();
363|  }
364|}
365|
366|async function loadSessionHistory() {
367|  const sessionId = route.query.session as string | undefined;
368|  if (!sessionId) return;
369|  try {
370|    hermesSessionId.value = sessionId;
371|    isLoading.value = true;
372|    const result = await invoke<{
373|      success: boolean;
374|      messages: any[];
375|      sessionId: string;
376|    }>('agent_list_messages', { sessionId });
377|    if (result.success && result.messages?.length) {
378|      const converted = hermesMessagesToChatMessages(result.messages);
379|      setMessages(converted);
380|    }
381|  } catch (e) {
382|    console.error('Failed to load session history:', e);
383|  } finally {
384|    isLoading.value = false;
385|  }
386|}
387|
388|onMounted(async () => {
389|  await loadSessionHistory();
390|  document.addEventListener('keydown', handleKeydown);
391|  chatInputRef.value?.focus();
392|});
393|
394|onUnmounted(() => {
395|  document.removeEventListener('keydown', handleKeydown);
396|  // Claw 清理
397|  if (ompInitialized.value) {
398|    invoke('claw_chat_close').catch(() => {});
399|  }
400|});
401|</script>
402|