/**
 * Chat types — aligned with hermes-desktop's ChatMessage union model.
 *
 * A ChatMessage is either a visible bubble (user / assistant) or a
 * collapsible history row (reasoning / tool_call / tool_result).
 */

export interface Attachment {
  id: string;
  kind: 'image' | 'text-file' | 'path-ref';
  name: string;
  mime: string;
  size: number;
  dataUrl?: string;
  text?: string;
  path?: string;
}

/** Visible chat bubble (user or assistant). */
export interface ChatBubbleMessage {
  id: string;
  kind?: 'user' | 'assistant';
  role: 'user' | 'agent';
  content: string;
  attachments?: Attachment[];
  timestamp?: number;
  isStopped?: boolean;
}

/** Collapsible reasoning / thinking row. */
export interface ReasoningMessage {
  id: string;
  kind: 'reasoning';
  role: 'agent';
  text: string;
  content?: string;
}

/** Collapsible tool-call row. */
export interface ToolCallMessage {
  id: string;
  kind: 'tool_call';
  role: 'agent';
  callId: string;
  name: string;
  args: string;
  toolCallInfo?: {
    name: string;
    args?: Record<string, unknown>;
    status?: string;
    durationMs?: number;
    emoji?: string;
    label?: string;
  };
}

/** Collapsible tool-result row. */
export interface ToolResultMessage {
  id: string;
  kind: 'tool_result';
  role: 'agent';
  callId: string;
  name: string;
  content: string;
  attachments?: Attachment[];
}

/** Discriminated union of all message types. */
export type ChatMessage =
  | ChatBubbleMessage
  | ReasoningMessage
  | ToolCallMessage
  | ToolResultMessage;

/** Model group for the model picker. */
export interface ModelGroup {
  provider: string;
  providerLabel: string;
  models: {
    provider: string;
    model: string;
    label: string;
    baseUrl: string;
  }[];
}

/** Token usage state. */
export interface UsageState {
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
  cost?: number;
}
