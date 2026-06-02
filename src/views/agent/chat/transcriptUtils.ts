/**
 * Transcript utilities — serialise a conversation for clipboard export.
 * Adapted from hermes-desktop's transcriptUtils.ts.
 */
import type { ChatMessage } from './types'

export type TranscriptFormat = 'text' | 'markdown'

/**
 * Serialise a conversation into a clipboard-ready transcript.
 * - `text`     → plain `You: …` / `Hermes: …` blocks.
 * - `markdown` → `**You:**` / `**Hermes:**` headed blocks.
 */
export function buildChatTranscript(
  messages: ChatMessage[],
  format: TranscriptFormat,
): string {
  return messages
    .filter((m) => 'content' in m && typeof m.content === 'string')
    .map((m) => {
      const msg = m as { role: 'user' | 'agent'; content: string }
      const speaker = msg.role === 'user' ? 'You' : 'Hermes'
      const content = msg.content.trim()
      return format === 'markdown'
        ? `**${speaker}:**\n\n${content}`
        : `${speaker}: ${content}`
    })
    .join('\n\n')
}
