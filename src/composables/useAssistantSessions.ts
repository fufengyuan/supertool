/**
 * AI 配置助手 —— 历史会话 store（模块级单例）
 *
 * 主窗口页与悬浮窗是独立的 webview / 组件实例，各自持有独立的 useAssistantChat 状态。
 * 这里把「会话列表、当前会话 id、持久化读写」收拢成模块级单例，让侧边栏与聊天框共享同一份
 * 会话元数据，后端统一存 SQLite，跨界面可继续同一会话。
 *
 * 消息序列化约定：
 * - propose/applyProposal 产生的 entry 可能带工具调用结果（tools）、提案、表单等复杂结构，
 *   整体以 AssistantEntry 的 JSON 数组落库，恢复时原样放回 entries 供渲染。
 * - **敏感字段不落库**：用户确认提案时在卡片里填的密码/密钥只存在于 secretVault（实例级，
 *   本文件不引用），proposal.fields 里不会带真实凭据；tools 的结果默认经后端脱敏（[已隐藏]），
 *   images 是展示用 data URI，恢复时一并丢弃，避免本地白存大图片。
 */
import { ref } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import type { AssistantEntry } from './useAssistantChat'

export interface AssistantSessionMeta {
  id: string
  title: string
  messages: AssistantEntry[]
  createdAt: string
  updatedAt: string
}

const sessions = ref<AssistantSessionMeta[]>([])
/** 当前激活的会话 id；null 表示未进入任何会话（初始空态，首条消息时才落库） */
const currentId = ref<string | null>(null)

const uid = () => Math.random().toString(36).slice(2) + Date.now().toString(36)

/** 当前是否有 id 上的会话（新建会话暂存内存，首次保存时才落库） */
function hasCurrent() {
  return !!currentId.value
}

/** 序列化 entries 用于落库：过滤掉明确不持久化的展示态字段 */
function serializeEntries(entries: AssistantEntry[]): string {
  const clean = entries.map(e => ({
    id: e.id,
    role: e.role,
    text: e.text,
    thinking: e.thinking,
    tools: e.tools,
    proposals: e.proposals,
    forms: e.forms,
    questions: e.questions,
    // images 为展示用 data URI，体积大且随会话无意义，不落库
    streaming: false,
    error: e.error,
    needConfig: e.needConfig,
    usage: e.usage,
    at: e.at,
    actionNote: e.actionNote,
    // textAnchors 仅流式增量用，恢复时归零
    textAnchors: [],
  }))
  return JSON.stringify(clean)
}

/**
 * 反序列化会话消息。
 * 后端 `get_assistant_session` 在 core 层已把 messages 列（JSON 字符串）反序列化成数组下发，
 * 因此这里**数组与字符串两种形态都要接**：早期只按字符串 JSON.parse，传数组时
 * JSON.parse([obj]) 会走 toString 得到 "[object Object]" 并抛错 → 消息被静默清成空数组，
 * 表现为「历史会话加载不出来」，随后 settle 再落库还会把库里的真实消息覆盖成 []。
 */
function deserializeEntries(input: unknown): AssistantEntry[] {
  let raw: unknown
  if (Array.isArray(input)) {
    raw = input
  } else if (typeof input === 'string') {
    try {
      raw = JSON.parse(input || '[]')
    } catch {
      return []
    }
  } else {
    return []
  }
  if (!Array.isArray(raw)) {return []}
  return (raw as Partial<AssistantEntry>[]).map((e: Partial<AssistantEntry>) => ({
      id: e.id || uid(),
      role: e.role || 'assistant',
      text: e.text || '',
      thinking: e.thinking || '',
      tools: e.tools || [],
      proposals: e.proposals || [],
      forms: e.forms || [],
      questions: e.questions || [],
      streaming: false,
      error: e.error,
      needConfig: e.needConfig,
      usage: e.usage,
      at: e.at || '',
      actionNote: e.actionNote,
      textAnchors: [],
    }))
}

/** 由消息序列化出的会话标题：取第一条用户消息截断 */
function deriveTitle(entries: AssistantEntry[]): string {
  const firstUser = entries.find(e => e.role === 'user' && e.text.trim())
  const text = (firstUser?.text || '新会话').trim().replace(/\s+/g, ' ')
  return text.length > 24 ? text.slice(0, 24) + '…' : text || '新会话'
}

let loadingStarted = false

/** 拉取会话列表（只含元信息）。悬浮窗与主窗口各自首次挂载都调用，幂等。 */
async function loadSessions(): Promise<AssistantSessionMeta[]> {
  const api = getTauriAPI() as any
  try {
    const list = (await api.listAssistantSessions?.(false)) || []
    sessions.value = list.map((s: any) => ({
      id: s.id,
      title: s.title || '新会话',
      messages: [],
      createdAt: s.createdAt,
      updatedAt: s.updatedAt,
    }))
    return sessions.value
  } catch {
    sessions.value = []
    return sessions.value
  }
}

/** 新建一个会话并置为当前（不落库，等有消息时 saveCurrent 才建行） */
function newSession() {
  currentId.value = uid()
  return currentId.value
}

/** 把指定 id 置为当前并加载其消息（侧边栏点击切换时用） */
async function switchSession(id: string): Promise<AssistantSessionMeta | null> {
  const api = getTauriAPI() as any
  try {
    const saved = (await api.getAssistantSession?.(id)) || null
    currentId.value = id
    return saved && saved.id
      ? {
          id: saved.id,
          title: saved.title || '新会话',
          messages: deserializeEntries(saved.messages || '[]'),
          createdAt: saved.createdAt,
          updatedAt: saved.updatedAt,
        }
      : null
  } catch {
    return null
  }
}

/** 保存当前会话（entries 由调用方在发送/停止/清空前调用，本函数只负责落库与标题更新） */
async function saveCurrent(entries: AssistantEntry[], idOverride?: string): Promise<void> {
  const id = idOverride || currentId.value
  if (!id) {return}
  // 空会话不落库：历史上曾出现「加载失败→空数组→落库覆盖」把真实消息清成 [] 的情况，
  // 这里挡一道，避免 UPSERT 把已有会话的消息抹掉。
  if (!entries || entries.length === 0) {return}
  const api = getTauriAPI() as any
  const title = deriveTitle(entries)
  try {
    const updated = (await api.saveAssistantSession?.(id, title, serializeEntries(entries))) || {}
    // 更新本地列表元信息
    const idx = sessions.value.findIndex(s => s.id === id)
    if (idx >= 0) {
      sessions.value[idx] = {
        ...sessions.value[idx],
        title,
        updatedAt: updated.updatedAt || sessions.value[idx].updatedAt,
      }
    } else {
      sessions.value.unshift({
        id,
        title,
        messages: entries,
        createdAt: updated.createdAt || new Date().toISOString(),
        updatedAt: updated.updatedAt || new Date().toISOString(),
      })
    }
  } catch {/* 静默：保存失败不阻断聊天 */}
}

/** 重命名会话：保留原消息，只改标题 */
async function renameSession(id: string, title: string): Promise<void> {
  const api = getTauriAPI() as any
  const t = title.trim()
  if (!id || !t) {return}
  try {
    const saved = (await api.getAssistantSession?.(id)) || null
    // 会话不存在（或已删除）时放弃重命名，否则会把该 id 重新插成一条空消息会话
    if (!saved || !saved.id) {return}
    const messages = saved.messages ? JSON.stringify(saved.messages) : '[]'
    await api.saveAssistantSession?.(id, t, messages)
    const idx = sessions.value.findIndex(s => s.id === id)
    if (idx >= 0) {
      sessions.value[idx] = { ...sessions.value[idx], title: t }
    }
  } catch {/* 忽略 */}
}

/** 删除会话；返回是否删的是当前会话 */
async function deleteSession(id: string): Promise<boolean> {
  const api = getTauriAPI() as any
  const wasCurrent = currentId.value === id
  try { await api.deleteAssistantSession?.(id) } catch {/* 忽略 */}
  sessions.value = sessions.value.filter(s => s.id !== id)
  return wasCurrent
}

export function useAssistantSessions() {
  return {
    sessions,
    currentId,
    hasCurrent,
    loadSessions,
    newSession,
    switchSession,
    saveCurrent,
    deleteSession,
    renameSession,
    deriveTitle,
  }
}