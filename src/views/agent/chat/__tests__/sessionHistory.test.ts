import { describe, it, expect } from 'vitest'
import {
  dbItemsToChatMessages,
  reconcileStreamedWithDb,
  type DbHistoryItem,
} from '../sessionHistory'
import type { ChatMessage } from '../types'

// ── Fixture helpers ────────────────────────────────────────────────────────

function dbItem(
  overrides: Partial<DbHistoryItem> & { kind: DbHistoryItem['kind'] },
): DbHistoryItem {
  return {
    id: 1,
    content: '',
    text: '',
    callId: '',
    name: '',
    args: '',
    timestamp: 0,
    ...overrides,
  }
}

function streamMsg(overrides: Partial<ChatMessage> & { role: 'user' | 'agent' }): ChatMessage {
  return {
    id: `s-${Date.now()}`,
    content: '',
    ...overrides,
  } as ChatMessage
}

function streamBubble(id: string, role: 'user' | 'agent', content: string): ChatMessage {
  return { id, role, content } as ChatMessage
}

function streamReasoning(id: string, text: string): ChatMessage {
  return { id, kind: 'reasoning', role: 'agent', text } as ChatMessage
}

function streamToolCall(id: string, callId: string, name: string): ChatMessage {
  return { id, kind: 'tool_call', role: 'agent', callId, name, args: '{}' } as ChatMessage
}

function streamToolResult(id: string, callId: string, name: string, content: string): ChatMessage {
  return { id, kind: 'tool_result', role: 'agent', callId, name, content } as ChatMessage
}

// ── dbItemsToChatMessages ──────────────────────────────────────────────────

describe('dbItemsToChatMessages', () => {
  it('should return empty array for empty input', () => {
    expect(dbItemsToChatMessages([])).toEqual([])
  })

  it('should convert user kind to bubble message', () => {
    const result = dbItemsToChatMessages([dbItem({ kind: 'user', id: 42, content: 'hello' })])
    expect(result).toHaveLength(1)
    expect(result[0]).toMatchObject({
      id: 'db-42',
      role: 'user',
      content: 'hello',
    })
  })

  it('should convert assistant kind to agent role bubble', () => {
    const result = dbItemsToChatMessages([
      dbItem({ kind: 'assistant', id: 99, content: 'hi there' }),
    ])
    expect(result).toHaveLength(1)
    expect(result[0]).toMatchObject({
      id: 'db-99',
      role: 'agent',
      content: 'hi there',
    })
  })

  it('should convert reasoning kind', () => {
    const result = dbItemsToChatMessages([
      dbItem({ kind: 'reasoning', id: 10, text: 'thinking...' }),
    ])
    expect(result).toHaveLength(1)
    expect(result[0]).toMatchObject({
      id: 'db-r-10',
      kind: 'reasoning',
      role: 'agent',
      text: 'thinking...',
    })
  })

  it('should convert tool_call kind', () => {
    const result = dbItemsToChatMessages([
      dbItem({ kind: 'tool_call', id: 5, callId: 'call_001', name: 'search', args: '{"q":"test"}' }),
    ])
    expect(result).toHaveLength(1)
    expect(result[0]).toMatchObject({
      id: 'db-tc-5-call_001',
      kind: 'tool_call',
      role: 'agent',
      callId: 'call_001',
      name: 'search',
      args: '{"q":"test"}',
    })
  })

  it('should use "x" as fallback callId in tool_call id', () => {
    const result = dbItemsToChatMessages([
      dbItem({ kind: 'tool_call', id: 5, callId: undefined }),
    ])
    expect(result[0].id).toBe('db-tc-5-x')
  })

  it('should convert tool_result kind', () => {
    const result = dbItemsToChatMessages([
      dbItem({
        kind: 'tool_result',
        id: 8,
        callId: 'call_001',
        name: 'search',
        content: 'found it',
      }),
    ])
    expect(result).toHaveLength(1)
    expect(result[0]).toMatchObject({
      id: 'db-tr-8',
      kind: 'tool_result',
      role: 'agent',
      callId: 'call_001',
      name: 'search',
      content: 'found it',
    })
  })

  it('should supply empty strings when fields are missing', () => {
    const result = dbItemsToChatMessages([dbItem({ kind: 'user', id: 1 })])
    expect(result[0]).toMatchObject({
      id: 'db-1',
      role: 'user',
      content: '',
    })
  })

  it('should filter out unknown kinds as null', () => {
    const result = dbItemsToChatMessages([
      dbItem({ kind: 'user' as const, id: 1, content: 'a' }),
      { kind: 'unknown_kind' as never, id: 2, content: 'b' },
    ])
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('db-1')
  })
})

// ── reconcileStreamedWithDb ─────────────────────────────────────────────────

describe('reconcileStreamedWithDb', () => {
  it('should return empty for both empty', () => {
    expect(reconcileStreamedWithDb([], [])).toEqual([])
  })

  it('should use DB messages when no streamed match exists', () => {
    const db = [dbBubble(1, 'user', 'hello')]
    expect(reconcileStreamedWithDb([], db)).toEqual(db)
  })

  it('should preserve streamed identity when match exists', () => {
    const streamed = [streamBubble('stream-1', 'user', 'hello')]
    const db = [dbBubble(1, 'user', 'hello')]
    const result = reconcileStreamedWithDb(streamed, db)
    expect(result).toHaveLength(1)
    // Streamed id should be preserved
    expect(result[0].id).toBe('stream-1')
  })

  it('should merge tool_result from DB when stream did not deliver them', () => {
    const streamed = [
      streamBubble('s-u1', 'user', 'search for x'),
      streamBubble('s-a1', 'agent', 'result: found'),
    ]
    const db = [
      dbBubble(1, 'user', 'search for x'),
      dbBubble(2, 'assistant', 'result: found'),
      dbToolResult(3, 'tr_call1', 'search', 'found x'),
    ]
    const result = reconcileStreamedWithDb(streamed, db)
    expect(result).toHaveLength(3)
    expect(result[2]).toMatchObject({
      kind: 'tool_result',
      callId: 'tr_call1',
      name: 'search',
    })
  })

  it('should handle multiple consecutive user/agent bubble pairs', () => {
    const streamed = [
      streamBubble('s1', 'user', 'q1'),
      streamBubble('s2', 'agent', 'a1'),
      streamBubble('s3', 'user', 'q2'),
      streamBubble('s4', 'agent', 'a2'),
    ]
    const db = [
      dbBubble(1, 'user', 'q1'),
      dbBubble(2, 'assistant', 'a1'),
      dbBubble(3, 'user', 'q2'),
      dbBubble(4, 'assistant', 'a2'),
    ]
    const result = reconcileStreamedWithDb(streamed, db)
    expect(result).toHaveLength(4)
    // Streamed ids should be preserved
    expect(result[0].id).toBe('s1')
    expect(result[1].id).toBe('s2')
    expect(result[2].id).toBe('s3')
    expect(result[3].id).toBe('s4')
  })

  it('should deduplicate near-duplicate bubbles that slipped past key match', () => {
    const streamed = [streamBubble('s1', 'agent', 'hello')]
    const db = [dbBubble(1, 'assistant', 'hello')]
    const result = reconcileStreamedWithDb(streamed, db)
    // Only one copy
    expect(result).toHaveLength(1)
  })

  it('should preserve streamed-only additions (error bubbles)', () => {
    const streamed = [
      streamBubble('s1', 'user', 'hi'),
      streamBubble('error-1', 'agent', 'Error: something broke'),
    ]
    const db = [dbBubble(1, 'user', 'hi'), dbBubble(2, 'assistant', 'response')]
    const result = reconcileStreamedWithDb(streamed, db)
    // DB has a different assistant message, but the error bubble should still be preserved
    expect(result.length).toBeGreaterThanOrEqual(2)
    const hasError = result.some((m) => m.id === 'error-1')
    expect(hasError).toBe(true)
  })

  it('should match tool_call by callId', () => {
    const streamed = [streamToolCall('s-tc-1', 'call_abc', 'search')]
    const db = [dbToolCall(1, 'call_abc', 'search')]
    const result = reconcileStreamedWithDb(streamed, db)
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('s-tc-1')
  })

  it('should match reasoning by text prefix (200 chars)', () => {
    const streamed = [streamReasoning('s-r-1', 'thinking step by step...')]
    const db = [dbReasoning(1, 'thinking step by step...')]
    const result = reconcileStreamedWithDb(streamed, db)
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('s-r-1')
  })

  it('should handle streamed reasoning being shorter than DB reasoning (streaming in progress)', () => {
    const streamed = [streamReasoning('s-r-1', 'thinking started')]
    const db = [dbReasoning(1, 'thinking started and continued...')]
    const result = reconcileStreamedWithDb(streamed, db)
    // Different text snippets (slice(0,200) is 'thinking started' vs 'thinking started and...')
    expect(result).toHaveLength(2)
  })
})

// ── Internal test helpers ───────────────────────────────────────────────────

function dbBubble(id: number, kind: 'user' | 'assistant', content: string): ChatMessage {
  return {
    id: `db-${id}`,
    role: kind === 'assistant' ? 'agent' : 'user',
    content,
  } as ChatMessage
}

function dbToolResult(id: number, callId: string, name: string, content: string): ChatMessage {
  return {
    id: `db-tr-${id}`,
    kind: 'tool_result',
    role: 'agent',
    callId,
    name,
    content,
  } as ChatMessage
}

function dbToolCall(id: number, callId: string, name: string): ChatMessage {
  return {
    id: `db-tc-${id}`,
    kind: 'tool_call',
    role: 'agent',
    callId,
    name,
    args: '{}',
  } as ChatMessage
}

function dbReasoning(id: number, text: string): ChatMessage {
  return {
    id: `db-r-${id}`,
    kind: 'reasoning',
    role: 'agent',
    text,
  } as ChatMessage
}
