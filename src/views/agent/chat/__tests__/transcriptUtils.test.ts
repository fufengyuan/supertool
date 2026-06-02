import { describe, it, expect } from 'vitest'
import { buildChatTranscript } from '../transcriptUtils'
import type { ChatMessage } from '../types'

function bubble(
  role: 'user' | 'agent',
  content: string,
  overrides: Partial<ChatMessage> = {},
): ChatMessage {
  return { id: `${role}-${Date.now()}`, role, content, ...overrides } as ChatMessage
}

function reasoning(text: string): ChatMessage {
  return {
    id: `reasoning-${Date.now()}`,
    kind: 'reasoning',
    role: 'agent',
    text,
  } as ChatMessage
}

function toolCall(callId: string, name: string): ChatMessage {
  return {
    id: `tc-${callId}`,
    kind: 'tool_call',
    role: 'agent',
    callId,
    name,
    args: '{}',
  } as ChatMessage
}

function toolResult(callId: string, name: string, content: string): ChatMessage {
  return {
    id: `tr-${callId}`,
    kind: 'tool_result',
    role: 'agent',
    callId,
    name,
    content,
  } as ChatMessage
}

describe('buildChatTranscript', () => {
  it('should return empty string for no messages', () => {
    expect(buildChatTranscript([], 'text')).toBe('')
    expect(buildChatTranscript([], 'markdown')).toBe('')
  })

  it('should format plain text with You / Hermes speakers', () => {
    const out = buildChatTranscript(
      [bubble('user', 'hi'), bubble('agent', 'hello there')],
      'text',
    )
    expect(out).toBe('You: hi\n\nHermes: hello there')
  })

  it('should format markdown with bold speaker headers', () => {
    const out = buildChatTranscript(
      [bubble('user', 'hi'), bubble('agent', 'hello there')],
      'markdown',
    )
    expect(out).toBe('**You:**\n\nhi\n\n**Hermes:**\n\nhello there')
  })

  it('should trim surrounding whitespace from message content', () => {
    expect(buildChatTranscript([bubble('user', '  spaced  ')], 'text')).toBe(
      'You: spaced',
    )
  })

  it('should map the agent role to Hermes and user to You', () => {
    expect(buildChatTranscript([bubble('agent', 'x')], 'text')).toBe('Hermes: x')
    expect(buildChatTranscript([bubble('user', 'x')], 'text')).toBe('You: x')
  })

  it('should include any message with a string content field', () => {
    const msgs: ChatMessage[] = [
      bubble('user', 'hello'),
      reasoning('thinking...'),
      toolCall('tc1', 'search'),
      toolResult('tc1', 'search', 'results'),
      bubble('agent', 'done'),
    ]
    const out = buildChatTranscript(msgs, 'text')
    // tool_result has a `content` field so it's included as a Hermes message
    expect(out).toContain('You: hello')
    expect(out).toContain('Hermes: results')
    expect(out).toContain('Hermes: done')
  })

  it('should handle a single user message', () => {
    expect(buildChatTranscript([bubble('user', 'only me')], 'text')).toBe(
      'You: only me',
    )
  })

  it('should handle a single agent message', () => {
    expect(buildChatTranscript([bubble('agent', 'response')], 'text')).toBe(
      'Hermes: response',
    )
  })
})
