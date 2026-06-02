/**
 * Session history — DB-to-ChatMessage mapping and streamed/DB reconciliation.
 *
 * Adapted from hermes-desktop's sessionHistory.ts for Vue 3.
 * Converts raw DB rows from the Tauri IPC into renderer-ready ChatMessage
 * union types, and merges streamed in-memory messages with DB-loaded
 * equivalents at end-of-stream.
 */
import type { ChatMessage, ChatBubbleMessage } from './types'

export interface DbHistoryItem {
  kind: 'user' | 'assistant' | 'reasoning' | 'tool_call' | 'tool_result'
  id: number
  content?: string
  text?: string
  callId?: string
  name?: string
  args?: string
  timestamp?: number
}

/** Convert DB history items to ChatMessage[] for the renderer. */
export function dbItemsToChatMessages(items: readonly DbHistoryItem[]): ChatMessage[] {
  return items
    .map((it): ChatMessage | null => {
      switch (it.kind) {
        case 'user':
          return {
            id: `db-${it.id}`,
            role: 'user',
            content: it.content || '',
          }
        case 'assistant':
          return {
            id: `db-${it.id}`,
            role: 'agent',
            content: it.content || '',
          }
        case 'reasoning':
          return {
            id: `db-r-${it.id}`,
            kind: 'reasoning',
            role: 'agent',
            text: it.text || '',
          }
        case 'tool_call':
          return {
            id: `db-tc-${it.id}-${it.callId || 'x'}`,
            kind: 'tool_call',
            role: 'agent',
            callId: it.callId || '',
            name: it.name || '',
            args: it.args || '',
          }
        case 'tool_result':
          return {
            id: `db-tr-${it.id}`,
            kind: 'tool_result',
            role: 'agent',
            callId: it.callId || '',
            name: it.name || '',
            content: it.content || '',
          }
        default:
          return null
      }
    })
    .filter((m): m is ChatMessage => m !== null)
}

/** Collapse runs of whitespace into a single space and trim. */
function normalizeWhitespace(s: string): string {
  return s.replace(/\s+/g, ' ').trim()
}

/** Reconciliation key for cross-source matching. */
function reconciliationKey(m: ChatMessage): string | null {
  if ('kind' in m) {
    switch (m.kind) {
      case 'reasoning':
        return `reasoning:${normalizeWhitespace(m.text || '').slice(0, 200)}`
      case 'tool_call':
        return `tool_call:${m.callId || m.id}`
      case 'tool_result':
        return `tool_result:${m.callId || m.id}`
      default:
        return null
    }
  }
  const bubble = m as ChatBubbleMessage
  return `${bubble.role}:${normalizeWhitespace(bubble.content || '').slice(0, 200)}`
}

/** Merge DB-only metadata (e.g. attachments) into a streamed message
 *  while preserving the streamed message's identity (id). */
function mergeDbMetadataIntoStreamed(
  streamed: ChatMessage,
  db: ChatMessage,
): ChatMessage {
  if ('kind' in streamed) return streamed
  const s = streamed as ChatBubbleMessage
  const d = db as ChatBubbleMessage
  if (d.attachments && d.attachments.length > 0 && (!s.attachments || s.attachments.length === 0)) {
    return { ...s, attachments: d.attachments }
  }
  return s
}

/**
 * Merge in-memory streamed transcript with canonical DB transcript
 * at end-of-stream. Preserves streamed identity for already-rendered
 * messages while surfacing DB-only rows (tool_call, tool_result).
 */
export function reconcileStreamedWithDb(
  streamed: readonly ChatMessage[],
  db: readonly ChatMessage[],
): ChatMessage[] {
  const streamedByKey = new Map<string, ChatMessage[]>()
  for (const m of streamed) {
    const key = reconciliationKey(m)
    if (!key) continue
    const bucket = streamedByKey.get(key)
    if (bucket) bucket.push(m)
    else streamedByKey.set(key, [m])
  }

  const result: ChatMessage[] = []
  for (const dbMsg of db) {
    const key = reconciliationKey(dbMsg)
    const bucket = key ? streamedByKey.get(key) : undefined
    const streamedMatch = bucket?.shift()
    if (streamedMatch) {
      result.push(mergeDbMetadataIntoStreamed(streamedMatch, dbMsg))
    } else {
      result.push(dbMsg)
    }
  }

  // Append any streamed messages not matched (e.g. error bubbles)
  const consumedIds = new Set(result.map((m) => m.id))
  const seenBubbleKeys = new Set<string>()
  for (const m of result) {
    if (!('kind' in m)) {
      const bubble = m as ChatBubbleMessage
      seenBubbleKeys.add(
        `${bubble.role}:${normalizeWhitespace(bubble.content || '')}`,
      )
    }
  }

  for (const m of streamed) {
    if (consumedIds.has(m.id)) continue
    if (!('kind' in m)) {
      const bubble = m as ChatBubbleMessage
      const contentKey = `${bubble.role}:${normalizeWhitespace(bubble.content || '')}`
      if (seenBubbleKeys.has(contentKey)) continue
      seenBubbleKeys.add(contentKey)
    }
    result.push(m)
  }

  return result
}
