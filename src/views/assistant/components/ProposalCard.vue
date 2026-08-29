<template>
  <div
    class="border rounded-xl overflow-hidden"
    :class="statusClass"
  >
    <!-- 标题条 -->
    <div class="flex items-center gap-2 px-3 py-2 bg-base-200 border-b" :class="borderClass">
      <SvgIcon name="clipboard" size="14" class="shrink-0" />
      <span class="text-[10px] font-bold px-1.5 py-0.5 rounded" :class="opClass">{{ opLabel }}</span>
      <span class="text-xs font-semibold text-base-content truncate flex-1">{{ proposal.displayName }}</span>
      <span class="text-[10px] px-1.5 py-0.5 rounded-full shrink-0" :class="badgeClass">{{ statusLabel }}</span>
    </div>

    <div class="p-3 flex flex-col gap-2.5">
      <p v-if="proposal.rationale" class="text-[11px] text-base-content/70 m-0 leading-relaxed">
        {{ proposal.rationale }}
      </p>

      <!-- 目标对象 -->
      <div class="text-[11px] text-base-content/50 flex flex-wrap gap-x-3 gap-y-1">
        <span>类型：<b class="text-base-content/80">{{ targetLabel }}</b></span>
        <span v-if="proposal.operation === 'update'">
          目标：<b class="text-base-content/80 font-mono">{{ shortId }}</b>
        </span>
        <span v-else class="text-info">新建记录</span>
      </div>

      <!-- 字段清单：待确认时可编辑，让用户能改完再落库 -->
      <div class="border border-base-content/10 rounded-lg overflow-hidden">
        <div
          v-for="(row, key) in rows"
          :key="key"
          class="flex items-start gap-2 px-2.5 py-1.5 border-b border-base-content/5 last:border-b-0 text-[11px]"
        >
          <span class="w-[132px] shrink-0 font-mono text-base-content/60 pt-0.5 break-all">{{ key }}</span>
          <div class="flex-1 min-w-0">
            <template v-if="editable">
              <input
                v-if="row.kind === 'string'"
                v-model="draft[key]"
                class="input input-bordered input-xs w-full font-mono text-[11px]"
              />
              <input
                v-else-if="row.kind === 'number'"
                v-model.number="draft[key]"
                type="number"
                class="input input-bordered input-xs w-32 font-mono text-[11px]"
              />
              <label v-else-if="row.kind === 'boolean'" class="flex items-center gap-1.5 cursor-pointer">
                <input v-model="draft[key]" type="checkbox" class="toggle toggle-xs toggle-primary" />
                <span class="font-mono">{{ String(draft[key]) }}</span>
              </label>
              <textarea
                v-else
                v-model="draft[key]"
                rows="2"
                class="textarea textarea-bordered w-full font-mono text-[11px] leading-snug"
              ></textarea>
            </template>
            <span v-else class="font-mono break-all text-base-content/90">{{ row.display }}</span>
          </div>
        </div>
      </div>

      <!-- 凭据：只有用户亲手输入，永远不会经过助手 -->
      <div
        v-if="proposal.operation === 'create' && secretFields.length"
        class="border border-warning/40 bg-warning/5 rounded-lg p-2.5"
      >
        <div class="flex items-center gap-1.5 text-[11px] font-semibold text-warning mb-1.5">
          <SvgIcon name="lock" size="12" /> 还需要你本人填写（助手看不到这些值）
        </div>
        <div v-for="field in secretFields" :key="field" class="flex items-center gap-2 mb-1.5 last:mb-0">
          <span class="w-[132px] shrink-0 text-[11px] font-mono text-base-content/60">{{ field }}</span>
          <input
            v-model="secrets[field]"
            type="password"
            autocomplete="new-password"
            class="input input-bordered input-xs flex-1 font-mono text-[11px]"
            :placeholder="`填完直接写入本地加密存储，不会显示在对话里`"
          />
        </div>
      </div>
      <p v-else-if="secretHint" class="text-[11px] text-base-content/50 m-0 leading-relaxed">
        {{ secretHint }}
      </p>

      <p v-if="proposal.error || localError" class="text-[11px] text-error m-0 break-all">
        {{ proposal.error || localError }}
      </p>

      <!-- 操作 -->
      <div class="flex items-center gap-2 pt-0.5">
        <template v-if="proposal.status === 'pending'">
          <button class="btn btn-primary btn-xs" :disabled="applying" @click="apply">
            <span v-if="applying" class="loading loading-spinner loading-xs" />
            <SvgIcon v-else name="check" size="12" /> 确认应用
          </button>
          <button class="btn btn-ghost btn-xs border border-base-content/10" @click="$emit('dismiss')">
            不用了
          </button>
          <button
            v-if="proposal.applyRoute"
            class="btn btn-ghost btn-xs text-base-content/60 ml-auto"
            @click="goThere"
          >
            我自己去改
          </button>
        </template>
        <span v-else-if="proposal.status === 'applied'" class="text-[11px] text-success flex items-center gap-1">
          <SvgIcon name="check" size="12" /> 已写入，可回对应页面复核
        </span>
        <span v-else-if="proposal.status === 'dismissed'" class="text-[11px] text-base-content/40">已忽略</span>
        <button v-else-if="proposal.status === 'failed'" class="btn btn-error btn-xs btn-outline" @click="retry">重试</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import SvgIcon from '../../../components/ui/SvgIcon.vue'
import type { Proposal } from '../../../composables/useAssistantChat'

const props = defineProps<{
  proposal: Proposal
  /** 从表单收集的敏感值按字段名预填，用户可直接确认（值仍只来自本地暂存） */
  initialSecrets?: Record<string, string>
}>()
const emit = defineEmits<{
  (e: 'apply', fields: Record<string, unknown>, secrets: Record<string, string>): void
  (e: 'dismiss'): void
  (e: 'retry'): void
}>()

const router = useRouter()
const applying = ref(false)
const localError = ref('')

/** 助手给出的字段先复制到本地，用户可以在卡片上直接改 */
const draft = reactive<Record<string, any>>(JSON.parse(JSON.stringify(props.proposal.fields || {})))
// 凭据初始值只在「新建」时从表单暂存带入；更新走「保留原有密码」语义，不预填也不覆盖
const secrets = reactive<Record<string, string>>(
  props.proposal.operation === 'create' ? { ...(props.initialSecrets || {}) } : {},
)

const TARGET_LABELS: Record<string, string> = {
  server: '服务器',
  cicd: 'CI/CD 部署配置',
  dbConnection: '数据库连接',
  aiProvider: 'AI 模型提供商',
  logPreset: '日志聚合预设',
  gitRepo: 'Git 仓库',
}
/** 界面可识别的「需要凭据」字段名（与后端脱敏清单一致） */
const SECRET_FIELDS = ['password', 'sshKeyPath', 'apiKey', 'token', 'secret', 'privateKey']

const targetLabel = computed(() => TARGET_LABELS[props.proposal.targetType] || props.proposal.targetType)
const shortId = computed(() => (props.proposal.targetId || '').slice(0, 8) || '—')
const editable = computed(() => props.proposal.status === 'pending')
watch(
  () => props.proposal.status,
  next => {
    if (next !== 'pending') {applying.value = false}
  },
)
const opLabel = computed(() => (props.proposal.operation === 'update' ? '修改' : '新建'))
const opClass = computed(() =>
  props.proposal.operation === 'update'
    ? 'bg-warning/15 text-warning'
    : 'bg-success/15 text-success',
)
const secretFields = computed(() => {
  const declared = (props.proposal.needUserInput || []) as string[]
  const fromKeys = Object.keys(draft).filter(k => SECRET_FIELDS.includes(k))
  // 表单里收集过并已预填的凭据（即使提案没声明，也要显示槽位让用户核对/确认）
  const fromVault = Object.keys(props.initialSecrets || {}).filter(k => SECRET_FIELDS.includes(k))
  return Array.from(new Set([...declared.filter(f => SECRET_FIELDS.includes(f)), ...fromKeys, ...fromVault]))
})
const secretHint = computed(() => {
  if (props.proposal.operation === 'update') {
    return '更新会保留这条记录原有的密码/密钥，不需要重新输入。'
  }
  return secretFields.value.length
    ? ''
    : '这条提案不含凭据字段，确认后即可写入。'
})

const rows = computed(() => {
  const out: Record<string, { kind: string; display: string }> = {}
  for (const [k, v] of Object.entries(draft)) {
    if (secretFields.value.includes(k)) {continue}
    if (typeof v === 'boolean') {out[k] = { kind: 'boolean', display: String(v) }}
    else if (typeof v === 'number') {out[k] = { kind: 'number', display: String(v) }}
    else if (typeof v === 'string') {out[k] = { kind: 'string', display: v }}
    else {out[k] = { kind: 'json', display: JSON.stringify(v) }}
  }
  return out
})

const STATUS_LABELS: Record<string, string> = {
  pending: '待确认',
  applied: '已应用',
  dismissed: '已忽略',
  failed: '应用失败',
}
const statusLabel = computed(() => STATUS_LABELS[props.proposal.status] || props.proposal.status)
const borderClass = computed(() =>
  props.proposal.status === 'pending' ? 'border-primary/40' : 'border-base-content/10',
)
const statusClass = computed(
  () => `bg-base-100 ${props.proposal.status === 'failed' ? 'ring-1 ring-error/30' : ''}`,
)
const BADGE_CLASSES: Record<string, string> = {
  pending: 'bg-primary/10 text-primary',
  applied: 'bg-success/10 text-success',
  dismissed: 'bg-base-content/5 text-base-content/40',
  failed: 'bg-error/10 text-error',
}
const badgeClass = computed(() => BADGE_CLASSES[props.proposal.status] || 'bg-base-content/5')

/** 结构化字段按原类型送回去；JSON 文本解析失败就地报错，不静默写坏 */
function buildPayload(): Record<string, unknown> | null {
  const out: Record<string, unknown> = {}
  for (const [k, v] of Object.entries(draft)) {
    if (secretFields.value.includes(k)) {continue}
    const kind = rows.value[k]?.kind
    if (kind === 'json' && typeof v === 'string') {
      try {
        out[k] = JSON.parse(v)
      } catch {
        localError.value = `字段「${k}」不是合法 JSON，改对再应用`
        return null
      }
    } else {
      out[k] = v
    }
  }
  localError.value = ''
  return out
}

function apply() {
  const payload = buildPayload()
  if (!payload) {return}
  applying.value = true
  emit('apply', payload, { ...secrets })
}

/** 状态由会话层拥有，卡片只负责清掉本地错误再请求重试 */
function retry() {
  localError.value = ''
  emit('retry')
}

function goThere() {
  if (props.proposal.applyRoute) {router.push({ path: props.proposal.applyRoute })}
}
</script>
