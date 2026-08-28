/**
 * AI 配置助手 —— 会话状态机
 *
 * 主窗口页与悬浮窗共用同一套逻辑（各自持有独立会话，后端每轮无状态）。
 * 后端只发一种事件 `assistant-event`，用 payload.type 区分：
 *   start / delta / thinking-start / thinking / tool-start / tool-running /
 *   tool-result / proposal / action / usage / notice / error / done
 *
 * 关键约定：助手自己没有任何写入能力，proposal 只是「待确认的意图」；
 * 真正写库发生在这里（applyProposal），调用的是各功能页一直在用的既有命令，
 * 密码类字段只可能来自用户在确认卡片里亲手输入的值，永远不会来自模型。
 */
import { computed, onUnmounted, ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import { getTauriAPI } from '../utils/tauri-api'
import { useToast } from './useToast'

export interface ToolRun {
  callId: string
  name: string
  arguments: string
  state: 'running' | 'done' | 'error'
  result?: unknown
}

export interface Proposal {
  id: string
  targetType: 'server' | 'cicd' | 'dbConnection' | 'aiProvider' | string
  operation: 'create' | 'update'
  targetId?: string
  displayName: string
  fields: Record<string, unknown>
  rationale: string
  needUserInput: string[]
  applyRoute?: string
  allowedFields?: string[]
  status: 'pending' | 'applied' | 'dismissed' | 'failed'
  error?: string
}

/** request_form 弹出来的表单字段（后端 sanitize_form_schema 净化后下发） */
export interface FormField {
  name: string
  label: string
  type: 'text' | 'number' | 'select' | 'boolean' | 'textarea' | 'password'
  required?: boolean
  placeholder?: string
  default?: string | number | boolean
  options?: string[]
  description?: string
}

export interface AssistantForm {
  callId: string
  title: string
  description?: string
  fields: FormField[]
  status: 'pending' | 'submitted'
}

/** ask 弹出来的问题卡片（单选/多选/自由输入） */
export interface AssistantAsk {
  callId: string
  question: string
  type: 'single' | 'multiple' | 'text'
  options?: string[]
  description?: string
  status: 'pending' | 'submitted'
}

export interface AssistantEntry {
  id: string
  role: 'user' | 'assistant' | 'note'
  text: string
  thinking: string
  tools: ToolRun[]
  proposals: Proposal[]
  forms: AssistantForm[]
  questions: AssistantAsk[]
  streaming: boolean
  error?: string
  /** 模型未配置，需要先去设置页 */
  needConfig?: boolean
  usage?: { input: number; output: number }
  at: string
  /** 界面动作提示（跳转等） */
  actionNote?: string
}

const nowTime = () => new Date().toLocaleTimeString('zh-CN', { hour12: false })
const uid = () => Math.random().toString(36).slice(2) + Date.now().toString(36)

/** 表单里收集的敏感值：只在本组件实例内暂存，绝不进对话文本/模型上下文；
 *  按字段名（password/sshKeyPath/apiKey/token/secret/privateKey）与提案凭据槽位匹配自动带入 */
const SECRET_KEYS = ['password', 'sshKeyPath', 'apiKey', 'token', 'secret', 'privateKey']
const SECRET_PLACEHOLDER = '已填写（保存在本地，确认提案时自动带入）'

/** 送给后端的历史：只要 user/assistant 正文，最多 20 条（后端还会再裁一次） */
const HISTORY_LIMIT = 20

export function useAssistantChat(navigate?: (to: RouteLocationRaw) => void) {
  const toast = useToast()
  const entries = ref<AssistantEntry[]>([])
  const running = ref(false)
  const ready = ref(false)
  const modelInfo = ref<Record<string, unknown> | null>(null)
  const capabilities = ref<{ name: string; description: string }[]>([])
  const stateError = ref('')
  let turnId = ''
  let unlisten: (() => void) | undefined

  const pendingProposals = computed(() =>
    entries.value.flatMap(e => e.proposals.filter(p => p.status === 'pending')))
  /** 表单里收集过的敏感值暂存区（实例级，不持久化、不进模型上下文） */
  const secretVault = ref<Record<string, string>>({})
  const lastAssistant = () => {
    for (let i = entries.value.length - 1; i >= 0; i--) {
      if (entries.value[i].role === 'assistant') {return entries.value[i]}
    }
    return null
  }

  function newAssistantEntry(): AssistantEntry {
    const entry: AssistantEntry = {
      id: uid(), role: 'assistant', text: '', thinking: '', tools: [],
      proposals: [], forms: [], questions: [], streaming: true, at: nowTime(),
    }
    entries.value.push(entry)
    return entry
  }

  /** 找到事件所属的助手气泡：一轮内所有事件都属于同一个 turnId，按当前轮次归位 */
  function targetEntry(create = false): AssistantEntry | null {
    const current = lastAssistant()
    if (current && current.streaming) {return current}
    return create ? newAssistantEntry() : current
  }

  async function refreshState() {
    const api = getTauriAPI() as any
    try {
      const state = await api.assistantGetState?.()
      ready.value = !!state?.configured
      modelInfo.value = state?.active || null
      capabilities.value = state?.capabilities || []
      stateError.value = state?.error || ''
    } catch (e) {
      ready.value = false
      stateError.value = String((e as Error)?.message || e)
    }
  }

  function onEvent(data: any) {
    if (!data || !data.turnId || data.turnId !== turnId) {return}
    const entry = targetEntry(data.type !== 'usage')
    if (!entry) {return}
    switch (data.type) {
      case 'start':
        entry.streaming = true
        break
      case 'delta':
        entry.text += data.text || ''
        break
      case 'thinking-start':
        break
      case 'thinking':
        entry.thinking += data.text || ''
        break
      case 'tool-start':
      case 'tool-running': {
        const existing = entry.tools.find(t => t.callId === data.callId)
        if (!existing) {
          entry.tools.push({
            callId: data.callId, name: data.name, arguments: data.arguments || '{}',
            state: 'running',
          })
        } else if (data.arguments) {
          existing.arguments = data.arguments
        }
        break
      }
      case 'tool-result': {
        const tool = entry.tools.find(t => t.callId === data.callId)
        if (tool) {
          tool.state = data.isError ? 'error' : 'done'
          tool.result = data.result
        } else {
          entry.tools.push({
            callId: data.callId, name: data.name, arguments: '{}',
            state: data.isError ? 'error' : 'done', result: data.result,
          })
        }
        break
      }
      case 'proposal':
        if (data.proposal) {
          entry.proposals.push({ id: uid(), status: 'pending', ...data.proposal })
        }
        break
      case 'form':
        if (data.form) {
          entry.forms.push({
            callId: data.callId || uid(),
            status: 'pending',
            title: data.form.title || '请填写以下信息',
            description: data.form.description,
            fields: data.form.fields || [],
          })
        }
        break
      case 'question':
        if (data.question) {
          entry.questions.push({
            callId: data.callId || uid(),
            status: 'pending',
            ...data.question,
          })
        }
        break
      case 'action':
        handleAction(data.action, entry)
        break
      case 'usage':
        entry.usage = { input: data.inputTokens, output: data.outputTokens }
        break
      case 'notice':
        entries.value.push({
          id: uid(), role: 'note', text: data.message || '', thinking: '', tools: [],
          proposals: [], forms: [], questions: [], streaming: false, at: nowTime(),
        })
        break
      case 'error':
        entry.error = data.message || '模型调用失败'
        entry.needConfig = !!data.needConfig
        entry.streaming = false
        break
      case 'done':
        if (!entry.text && data.text) {entry.text = data.text}
        entry.streaming = false
        break
    }
  }

  function handleAction(action: any, entry: AssistantEntry) {
    if (!action || action.type !== 'navigate') {return}
    entry.actionNote = action.note || ''
    if (navigate && action.route) {
      navigate({ path: action.route })
    }
  }

  async function send(message: string) {
    const text = message.trim()
    if (!text || running.value) {return}
    const api = getTauriAPI() as any

    const history = entries.value
      .filter(e => (e.role === 'user' || e.role === 'assistant') && e.text.trim() && !e.error)
      .slice(-HISTORY_LIMIT)
      .map(e => ({ role: e.role, content: e.text }))

    entries.value.push({
      id: uid(), role: 'user', text, thinking: '', tools: [], proposals: [],
      forms: [], questions: [], streaming: false, at: nowTime(),
    })
    running.value = true
    turnId = uid()
    newAssistantEntry()

    try {
      await api.assistantChat(turnId, text, history)
    } catch (e) {
      const entry = targetEntry(true)
      if (entry) {
        entry.error = String((e as Error)?.message || e)
        entry.needConfig = /尚未配置|未就绪|提供商|模型/.test(entry.error)
        entry.streaming = false
      }
      running.value = false
    }
  }

  /** done/error 之后收尾：关掉光标动画并释放输入 */
  function settle() {
    const entry = lastAssistant()
    if (entry) {entry.streaming = false}
    running.value = false
  }

  async function stop() {
    const api = getTauriAPI() as any
    if (!turnId) {return}
    await api.assistantAbort?.(turnId)
    settle()
    entries.value.push({
      id: uid(), role: 'note', text: '已停止本次回答', thinking: '', tools: [],
      proposals: [], forms: [], questions: [], streaming: false, at: nowTime(),
    })
  }

  async function start() {
    const api = getTauriAPI() as any
    await refreshState()
    unlisten = await api.onAssistantEvent?.((data: any) => {
      onEvent(data)
      if (data?.type === 'done' || data?.type === 'error') {settle()}
    }) as (() => void) | undefined
  }

  /** 把表单填写的值作为新消息回给模型继续处理。敏感字段值只进本地暂存区，不进对话文本。 */
  function submitForm(form: AssistantForm, values: Record<string, unknown>) {
    if (running.value || form.status === 'submitted') {return}
    for (const f of form.fields) {
      if (f.type !== 'password') {continue}
      const v = values[f.name]
      if (typeof v === 'string' && v.trim()) {secretVault.value[f.name] = v.trim()}
    }
    form.status = 'submitted'
    const lines: string[] = [`【表单提交】${form.title}`]
    for (const f of form.fields) {
      if (f.type === 'password') {
        lines.push(`${f.label}：${SECRET_PLACEHOLDER}`)
        continue
      }
      const raw = values[f.name]
      if (typeof raw === 'boolean') {lines.push(`${f.label}：${raw ? '是' : '否'}`)}
      else if (raw === undefined || raw === null || raw === '') {lines.push(`${f.label}：（未填写）`)}
      else {lines.push(`${f.label}：${String(raw)}`)}
    }
    send(lines.join('\n'))
  }

  /** 把答案作为新消息回给模型继续处理（勾选结果 + 自定义输入合并） */
  function submitAsk(ask: AssistantAsk, answer: string | string[]) {
    if (running.value || ask.status === 'submitted') {return}
    ask.status = 'submitted'
    const text = Array.isArray(answer) ? answer.join('、') : answer
    send(`【回答】${ask.question}\n${text}`)
  }

  /** 提案卡片凭据槽位的初始值：从表单收集的敏感值按字段名匹配自动带入 */
  function proposalSecrets(proposal: Proposal): Record<string, string> {
    const out: Record<string, string> = {}
    const candidates = new Set([
      ...(proposal.needUserInput || []),
      ...Object.keys(proposal.fields || {}).filter(k => SECRET_KEYS.includes(k)),
    ])
    for (const name of candidates) {
      const v = secretVault.value[name]
      if (v) {out[name] = v}
    }
    return out
  }

  function clear() {
    entries.value = []
    secretVault.value = {}
    settle()
  }

  onUnmounted(() => {
    unlisten?.()
    unlisten = undefined
  })

  // ─── 提案落库（用户在卡片上确认后才走到这里） ───
  const cicdDefaults = {
    mavenProfile: '', deployPath: '', restartScript: '', healthCheckTimeout: 30,
    buildMode: 'single', parentBuildMode: false, parentBuildPath: '', requiresApproval: false,
    groupName: '默认', libSeparate: false, incrementalUpload: true, healthCheckRetries: 3,
  }

  /**
   * 应用提案：卡片上用户可能改过字段值（fields 由卡片回传），
   * 凭据字段只可能来自用户在卡片里的输入（userSecrets）——模型给的 fields 里不会有它们。
   * 写入走的都是各功能页一直在用的既有命令，助手本身没有写库通道。
   */
  async function applyProposal(
    proposal: Proposal,
    fields?: Record<string, unknown>,
    userSecrets: Record<string, string> = {},
  ) {
    const api = getTauriAPI() as any
    if (proposal.status === 'applied') {return}
    proposal.status = 'pending'
    proposal.error = undefined
    const merged: Record<string, unknown> = { ...(fields || proposal.fields || {}), ...userSecrets }
    try {
      if (proposal.targetType === 'server') {
        if (proposal.operation === 'update' && proposal.targetId) {
          const base = await api.getServerById(proposal.targetId)
          if (!base) {throw new Error('原服务器记录不存在，可能已被删除')}
          await api.updateServer({ ...base, ...merged })
        } else {
          if (!merged.host || !merged.name) {throw new Error('新建服务器至少需要名称与 IP/主机')}
          await api.addServer({ id: crypto.randomUUID(), ...merged })
        }
      } else if (proposal.targetType === 'cicd') {
        if (proposal.operation === 'update' && proposal.targetId) {
          const base = await api.getCicdConfigById(proposal.targetId)
          if (!base) {throw new Error('原部署配置不存在，可能已被删除')}
          await api.updateCicdConfig({ ...base, ...merged })
        } else {
          const missing = ['name', 'deployBranch'].filter(k => !merged[k])
          if (missing.length) {throw new Error(`新建部署配置缺少必填字段：${missing.join('、')}`)}
          await api.addCicdConfig({
            id: crypto.randomUUID(),
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            ...cicdDefaults,
            ...merged,
          })
        }
      } else if (proposal.targetType === 'dbConnection') {
        const list: any[] = (await api.getDbConnections()) || []
        if (proposal.operation === 'update' && proposal.targetId) {
          const index = list.findIndex(c => c.id === proposal.targetId)
          if (index < 0) {throw new Error('原数据库连接不存在，可能已被删除')}
          list[index] = { ...list[index], ...merged }
        } else {
          // password 只可能来自用户在卡片里的输入；没填就是空串，由用户后续自行补
          list.push({ id: crypto.randomUUID(), password: '', ...merged })
        }
        await api.setDbConnections(list)
      } else if (proposal.targetType === 'aiProvider') {
        const list: any[] = (await api.listAiProviders()) || []
        const base =
          proposal.operation === 'update' && proposal.targetId
            ? list.find(p => p.id === proposal.targetId)
            : null
        if (proposal.operation === 'update' && !base) {throw new Error('原模型提供商不存在，可能已被删除')}
        // apiKey 传掩码即代表沿用已存密钥（后端约定），掩码值不会被写回去
        await api.saveAiProvider({ ...(base || {}), ...merged })
      } else {
        throw new Error(`暂不支持直接应用 ${proposal.targetType} 类型的提案`)
      }
      proposal.status = 'applied'
      toast.success(`已应用：${proposal.displayName}`)
      await refreshState()
    } catch (e) {
      proposal.status = 'failed'
      proposal.error = String((e as Error)?.message || e)
      toast.error(`应用失败：${proposal.error}`)
    }
  }

  function dismissProposal(proposal: Proposal) {
    proposal.status = 'dismissed'
  }

  return {
    entries, running, ready, modelInfo, capabilities, stateError,
    pendingProposals, refreshState, start, send, stop, clear,
    applyProposal, dismissProposal, submitForm, submitAsk, proposalSecrets,
  }
}
