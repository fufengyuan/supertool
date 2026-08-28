<template>
  <div class="flex flex-col min-h-0 flex-1">
    <!-- 模型状态条 -->
    <div
      class="shrink-0 flex items-center gap-2 px-3 py-2 border-b border-base-content/10 text-[11px]"
      :class="ready ? 'bg-base-100' : 'bg-warning/10'"
    >
      <template v-if="ready && modelInfo">
        <SvgIcon name="sparkles" size="12" class="text-primary shrink-0" />
        <span class="truncate text-base-content/80">
          {{ modelInfo.provider }} · <b class="font-mono">{{ modelInfo.modelId }}</b>
        </span>
        <span class="text-base-content/45 shrink-0">
          {{ modelInfo.protocol === 'anthropic' ? 'Anthropic 协议' : 'OpenAI 兼容协议' }} ·
          窗口 {{ Math.round(Number(modelInfo.contextWindow || 0) / 1000) }}k
        </span>
      </template>
      <template v-else>
        <SvgIcon name="alert" size="12" class="text-warning shrink-0" />
        <span class="flex-1 min-w-0 truncate text-base-content/80">
          {{ stateError || '还没有配置 AI 模型，助手暂不可用' }}
        </span>
        <button class="btn btn-warning btn-xs shrink-0" @click="gotoSettings">去配置模型</button>
      </template>
      <button
        v-if="entries.length"
        class="btn btn-ghost btn-xs shrink-0 text-base-content/50"
        title="清空当前对话"
        @click="clear"
      >
        <SvgIcon name="trash" size="12" />
      </button>
    </div>

    <!-- 消息区 -->
    <div
      ref="listRef"
      class="flex-1 min-h-0 overflow-y-auto px-3 py-3 flex flex-col gap-3 bg-base-200/60"
      @scroll="onScroll"
    >
      <div v-if="!entries.length" class="m-auto max-w-md text-center py-6">
        <div class="w-11 h-11 mx-auto mb-3 rounded-xl bg-primary/10 text-primary flex items-center justify-center">
          <SvgIcon name="bot" size="22" />
        </div>
        <h3 class="text-sm font-bold text-base-content mb-1">配置助手</h3>
        <p class="text-[11px] text-base-content/60 leading-relaxed mb-4">
          它读得懂你已配的服务器、部署配置和部署日志，能告诉你某个字段该填什么、
          这次部署为什么失败，并把改动写成「待确认的提案」——
          <b>密码与密钥永远由你自己填</b>，改动永远要你点确认才生效。
        </p>
        <div class="flex flex-col gap-1.5">
          <button
            v-for="q in QUICK_PROMPTS"
            :key="q"
            class="text-left text-[11px] px-3 py-2 rounded-lg border border-base-content/10 bg-base-100 hover:border-primary/40 hover:text-primary transition-colors cursor-pointer"
            @click="ask(q)"
          >
            {{ q }}
          </button>
        </div>
      </div>

      <div v-for="entry in entries" :key="entry.id" class="flex flex-col gap-2">
        <!-- 系统提示（停止、裁剪等） -->
        <div v-if="entry.role === 'note'" class="self-center text-[10px] text-base-content/45 px-2 py-0.5 rounded-full bg-base-content/5">
          {{ entry.text }}
        </div>

        <!-- 用户消息 -->
        <div v-else-if="entry.role === 'user'" class="self-end max-w-[85%]">
          <div
            v-if="entry.images && entry.images.length"
            class="flex flex-wrap gap-1.5 mb-1.5 justify-end"
          >
            <img
              v-for="(uri, i) in entry.images"
              :key="i"
              :src="uri"
              class="w-28 h-28 object-cover rounded-lg border border-white/20 bg-base-200"
              alt="用户附图"
            />
          </div>
          <div v-if="entry.text" class="px-3 py-2 rounded-xl rounded-br-sm bg-primary text-primary-content text-xs leading-relaxed whitespace-pre-wrap break-words">
            {{ entry.text }}
          </div>
          <div class="text-[10px] text-base-content/35 text-right mt-0.5">{{ entry.at }}</div>
        </div>

        <!-- 助手消息 -->
        <div v-else class="self-start max-w-[92%] flex flex-col gap-2">
          <!-- 思考过程 -->
          <details v-if="entry.thinking" class="text-[10px] text-base-content/55">
            <summary class="cursor-pointer select-none hover:text-base-content/80">思考过程（{{ entry.thinking.length }} 字）</summary>
            <pre class="whitespace-pre-wrap font-sans leading-relaxed mt-1 pl-2 border-l border-base-content/10">{{ entry.thinking }}</pre>
          </details>

          <!-- 工具调用可视化 -->
          <div v-if="entry.tools.length" class="flex flex-col gap-1">
            <details
              v-for="tool in entry.tools"
              :key="tool.callId"
              class="text-[10px] border border-base-content/10 rounded-lg bg-base-100 overflow-hidden"
            >
              <summary class="flex items-center gap-1.5 px-2 py-1 cursor-pointer select-none hover:bg-base-200">
                <span v-if="tool.state === 'running'" class="loading loading-spinner loading-[10px] shrink-0" />
                <SvgIcon v-else-if="tool.state === 'error'" name="x" size="11" class="text-error shrink-0" />
                <SvgIcon v-else name="check" size="11" class="text-success shrink-0" />
                <span class="font-mono text-base-content/80 shrink-0">{{ tool.name }}</span>
                <span class="text-base-content/40 truncate">{{ summarizeArgs(tool.arguments) }}</span>
              </summary>
              <pre class="px-2 py-1.5 m-0 text-[10px] leading-snug font-mono whitespace-pre-wrap break-all bg-base-200/70 max-h-56 overflow-y-auto">{{ prettyResult(tool.result) }}</pre>
            </details>
          </div>

          <!-- 正文 -->
          <div
            v-if="entry.text || entry.streaming"
            class="px-3 py-2 rounded-xl rounded-bl-sm bg-base-100 border border-base-content/10 text-xs leading-relaxed"
          >
            <div v-if="entry.text" class="markdown-body break-words" v-html="render(entry.text)"></div>
            <span v-if="entry.streaming" class="inline-block w-1.5 h-3.5 bg-primary/70 align-text-bottom animate-pulse" />
          </div>

          <!-- 变更提案 -->
          <ProposalCard
            v-for="p in entry.proposals"
            :key="p.id"
            :proposal="p"
            :initial-secrets="proposalSecrets(p)"
            @apply="(fields, secrets) => applyProposal(p, fields, secrets)"
            @dismiss="dismissProposal(p)"
            @retry="applyProposal(p, p.fields, {})"
          />

          <!-- 信息收集表单 -->
          <FormCard
            v-for="f in entry.forms"
            :key="f.callId"
            :form="f"
            @submit="(values) => { submitForm(f, values); jumpBottom(true) }"
          />

          <!-- 提问答题 -->
          <AskCard
            v-for="q in entry.questions"
            :key="q.callId"
            :ask="q"
            @submit="(answer) => { submitAsk(q, answer); jumpBottom(true) }"
          />

          <div v-if="entry.error" class="px-3 py-2 rounded-xl border border-error/40 bg-error/5 text-[11px] text-error leading-relaxed">
            <div class="font-semibold mb-0.5">这一轮没跑成</div>
            <div class="break-words opacity-90">{{ entry.error }}</div>
            <button v-if="entry.needConfig" class="btn btn-error btn-xs mt-2" @click="gotoSettings">去配置模型</button>
          </div>

          <div class="flex items-center gap-2 text-[10px] text-base-content/35">
            <span>{{ entry.at }}</span>
            <span v-if="entry.usage">tokens {{ entry.usage.input }} → {{ entry.usage.output }}</span>
            <span v-if="entry.actionNote" class="text-primary/70">已带你到对应页面：{{ entry.actionNote }}</span>
          </div>
        </div>
      </div>
    </div>

    <button
      v-if="showJumpBottom"
      class="btn btn-primary btn-xs rounded-full fixed bottom-24 right-8 z-10 shadow-md"
      @click="jumpBottom(true)"
    >
      <SvgIcon name="arrowDown" size="12" /> 回到底部
    </button>

    <!-- 输入区 -->
    <div class="shrink-0 border-t border-base-content/10 bg-base-100 p-2.5">
      <!-- 待发送图片预览（粘贴 / 附带） -->
      <div v-if="pendingImages.length" class="flex flex-wrap gap-1.5 mb-2">
        <div v-for="(uri, i) in pendingImages" :key="i" class="relative">
          <img :src="uri" class="w-16 h-16 object-cover rounded-lg border border-base-content/15 bg-base-200" alt="待发送图片" />
          <button
            class="absolute -top-1.5 -right-1.5 w-4 h-4 rounded-full bg-error text-error-content text-[10px] leading-none flex items-center justify-center"
            title="移除"
            @click="pendingImages.splice(i, 1)"
          >×</button>
        </div>
        <span class="text-[10px] text-base-content/40 self-end">支持粘贴截图/图片（Ctrl/Cmd+V）</span>
      </div>

      <div class="flex items-end gap-2">
        <textarea
          ref="inputRef"
          v-model="input"
          rows="1"
          :disabled="!ready"
          :placeholder="ready ? '说说你要配什么、或哪里报错了（Enter 发送，Shift+Enter 换行；可粘贴截图给助手看）' : '先在设置里配好模型，助手才能工作'"
          class="textarea textarea-bordered flex-1 text-xs leading-relaxed resize-none min-h-[38px] max-h-40 disabled:opacity-60"
          @input="autoGrow"
          @paste="onPaste"
          @keydown.enter.exact.prevent="submit"
        ></textarea>
        <button
          v-if="running"
          class="btn btn-error btn-sm shrink-0 gap-1"
          @click="stop"
        >
          <SvgIcon name="stopSquare" size="13" /> 停止
        </button>
        <button
          v-else
          class="btn btn-primary btn-sm shrink-0 gap-1"
          :disabled="!ready || (!input.trim() && !pendingImages.length)"
          @click="submit"
        >
          <SvgIcon name="send" size="13" /> 发送
        </button>
      </div>
      <div v-if="capabilities.length" class="mt-1.5 text-[10px] text-base-content/40 leading-snug">
        能帮你：{{ capabilitySummary }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import { useRouter } from 'vue-router'
import SvgIcon from '../../../components/ui/SvgIcon.vue'
import { renderMarkdown } from '../../../composables/useMarkdownRenderer'
import { useAssistantChat } from '../../../composables/useAssistantChat'
import { useToast } from '../../../composables/useToast'
import FormCard from './FormCard.vue'
import AskCard from './AskCard.vue'
import ProposalCard from './ProposalCard.vue'

const props = withDefaults(defineProps<{ compact?: boolean }>(), { compact: false })

const router = useRouter()
const toast = useToast()
const {
  entries, running, ready, modelInfo, capabilities, stateError,
  refreshState, start, send, stop, clear, applyProposal, dismissProposal,
  submitForm, submitAsk, proposalSecrets,
} = useAssistantChat((to: RouteLocationRaw) => router.push(to))

const listRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLTextAreaElement | null>(null)
const input = ref('')
const pendingImages = ref<string[]>([])
const pinnedToBottom = ref(true)
const showJumpBottom = ref(false)

/** 单张图片体积上限（原始字节）：base64 会膨胀约 1.33 倍，后端还有兜底校验 */
const MAX_IMAGE_BYTES = 6 * 1024 * 1024

/** 粘贴处理：优先把剪贴板里的图片转成 data URI 加入待发送，文本粘贴照常 */
function onPaste(e: ClipboardEvent) {
  const files = e.clipboardData?.files
  const items = e.clipboardData?.items
  const candidates: File[] = []
  if (files && files.length) {
    for (const f of Array.from(files)) {
      if (f.type.startsWith('image/')) {candidates.push(f)}
    }
  } else if (items) {
    for (const it of Array.from(items)) {
      if (it.kind === 'file' && it.type.startsWith('image/')) {
        const f = it.getAsFile()
        if (f) {candidates.push(f)}
      }
    }
  }
  if (!candidates.length) {return}

  e.preventDefault()
  for (const f of candidates) {
    if (f.size > MAX_IMAGE_BYTES) {
      toast.error(`图片过大（${(f.size / 1024 / 1024).toFixed(1)}MB），请压缩后再粘贴`)
      continue
    }
    const reader = new FileReader()
    reader.onload = () => {
      const uri = String(reader.result || '')
      if (uri.startsWith('data:image/')) {
        pendingImages.value.push(uri)
        jumpBottom(true)
      }
    }
    reader.readAsDataURL(f)
  }
}

function removeImage(i: number) {
  pendingImages.value.splice(i, 1)
}

const QUICK_PROMPTS = [
  '我想新增一台服务器，需要准备哪些信息？',
  '这个部署配置为什么收不到产物？',
  '「构建目录」和「产物目录」到底有什么区别？',
  '帮我看看最近一次部署失败的原因',
  '我想接自己的模型（OpenAI 或 Anthropic 协议）',
]

const capabilitySummary = computed(() => {
  const labels: Record<string, string> = {
    get_app_snapshot: '看全局配置概览',
    list_servers: '查服务器',
    list_server_groups: '查分组',
    test_server_connection: '测 SSH 连通',
    list_db_connections: '查数据库连接',
    list_cicd_configs: '查部署配置',
    get_cicd_config: '看部署配置详情',
    validate_cicd_config: '体检部署配置',
    get_deploy_history: '查部署历史',
    analyze_deploy_error: '分析部署失败原因',
    search_usage_guides: '解答字段含义与用法',
    get_usage_guide: '给完整操作说明',
    propose_config_change: '生成待确认的变更提案',
    open_config_page: '带你到对应页面',
    search_project_guides: '查本项目文档指南',
    get_project_guide: '看项目指南全文',
    search_project_source: '检索本项目源码',
    read_project_source: '读本项目源码文件',
  }
  return capabilities.value
    .map((c: { name: string; description: string }) => labels[c.name] || '')
    .filter(Boolean)
    .join('、')
})

function render(text: string) {
  return renderMarkdown(text) || ''
}

function summarizeArgs(raw: string) {
  try {
    const obj = JSON.parse(raw || '{}')
    const parts = Object.entries(obj).slice(0, 3).map(([k, v]) => `${k}=${typeof v === 'string' ? v.slice(0, 18) : JSON.stringify(v).slice(0, 18)}`)
    return parts.join(' ')
  } catch {
    return (raw || '').slice(0, 60)
  }
}

function prettyResult(result: unknown) {
  if (result === undefined || result === null) {return '（无返回内容）'}
  try {
    return JSON.stringify(result, null, 1)
  } catch {
    return String(result)
  }
}

function jumpBottom(force = false) {
  if (!force && !pinnedToBottom.value) {return}
  nextTick(() => {
    const el = listRef.value
    if (el) {el.scrollTop = el.scrollHeight}
    pinnedToBottom.value = true
    showJumpBottom.value = false
  })
}

function onScroll() {
  const el = listRef.value
  if (!el) {return}
  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40
  pinnedToBottom.value = nearBottom
  showJumpBottom.value = !nearBottom && entries.value.length > 0
}

function autoGrow(e: Event) {
  const el = e.target as HTMLTextAreaElement
  el.style.height = 'auto'
  el.style.height = `${Math.min(el.scrollHeight, 160)}px`
}

function submit() {
  const text = input.value.trim()
  if ((!text && !pendingImages.value.length) || running.value || !ready.value) {return}
  const images = pendingImages.value.slice()
  input.value = ''
  pendingImages.value = []
  if (inputRef.value) {inputRef.value.style.height = 'auto'}
  send(text, images)
  jumpBottom(true)
}

function ask(q: string) {
  input.value = q
  submit()
}

function gotoSettings() {
  router.push({ path: '/settings', query: { tab: 'ai' } })
}

// 流式输出时跟随滚动
watch(
  () => entries.value.map((e: { text: string; thinking: string }) => e.text.length + e.thinking.length).join(','),
  () => jumpBottom(),
)

onMounted(async () => {
  await start()
  await refreshState()
  if (props.compact) {inputRef.value?.focus()}
})
</script>

<style scoped>
.markdown-body :deep(p) {
  margin: 0 0 0.5em;
}
.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}
.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 0.25em 0 0.5em;
  padding-left: 1.25em;
}
.markdown-body :deep(li) {
  margin: 0.1em 0;
}
.markdown-body :deep(code) {
  background: rgb(0 0 0 / 6%);
  border-radius: 4px;
  padding: 0.05em 0.35em;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.95em;
}
.markdown-body :deep(pre) {
  background: rgb(0 0 0 / 6%);
  border-radius: 8px;
  padding: 0.5em 0.6em;
  overflow-x: auto;
  margin: 0.4em 0;
}
.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
}
.markdown-body :deep(strong) {
  font-weight: 700;
}
.markdown-body :deep(a) {
  color: var(--color-primary, #4f7cff);
  text-decoration: underline;
}
.markdown-body :deep(table) {
  border-collapse: collapse;
  font-size: 0.95em;
  margin: 0.4em 0;
}
.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid rgb(0 0 0 / 10%);
  padding: 0.2em 0.5em;
}
</style>
