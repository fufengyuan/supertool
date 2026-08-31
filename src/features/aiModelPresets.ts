// AI 模型常见预设 —— 首次引导页与设置页共用
// 模型 ID / 上下文窗口 / 输出上限随各家网关迭代，此处仅作起点，可按实际网关调整

export interface AiModelPreset {
  id: string
  label?: string
  contextWindow?: number
  maxOutputTokens?: number
  vision?: boolean
}

export interface AiProviderPreset {
  name: string
  note: string
  protocol: 'openai' | 'anthropic'
  baseUrl: string
  models: AiModelPreset[]
}

export const MODEL_PRESETS: AiProviderPreset[] = [
  {
    name: 'DeepSeek',
    note: '官方 OpenAI 兼容端点；deepseek-v4-flash 1M 上下文、输出最大 384K；识图需用 deepseek-v4-flash-vision-exp',
    protocol: 'openai',
    baseUrl: 'https://api.deepseek.com',
    models: [{ id: 'deepseek-v4-flash', label: 'DeepSeek V4 Flash', contextWindow: 1048576, maxOutputTokens: 32768, vision: false }],
  },
  {
    name: '智谱 GLM',
    note: 'glm-5.3-flash 为 1M 上下文、输出最大 128K、支持识图',
    protocol: 'openai',
    baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    models: [{ id: 'glm-5.3-flash', label: 'GLM 5.3 Flash', contextWindow: 1048576, maxOutputTokens: 32768, vision: true }],
  },
  {
    name: 'Kimi',
    note: 'kimi-k3 为 1M 上下文、默认输出 128K、原生支持识图',
    protocol: 'openai',
    baseUrl: 'https://api.moonshot.cn/v1',
    models: [{ id: 'kimi-k3', label: 'Kimi K3', contextWindow: 1048576, maxOutputTokens: 32768, vision: true }],
  },
  {
    name: '通义千问',
    note: 'qwen3.8-flash 为 1M 上下文（DashScope 国内兼容端点）；识图能力视网关而定，可手动开启',
    protocol: 'openai',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    models: [{ id: 'qwen3.8-flash', label: 'Qwen3.8 Flash', contextWindow: 1048576, maxOutputTokens: 32768, vision: false }],
  },
  {
    name: 'Anthropic Claude',
    note: '1M 上下文为 Sonnet 5 / Opus 5；Haiku 4.5 为 200K；均支持识图',
    protocol: 'anthropic',
    baseUrl: 'https://api.anthropic.com',
    models: [{ id: 'claude-sonnet-5', label: 'Claude Sonnet 5', contextWindow: 1000000, maxOutputTokens: 128000, vision: true }],
  },
  {
    name: 'OpenAI GPT',
    note: 'GPT-5.6 系（Sol/Terra/Luna）为 1M 上下文、支持识图',
    protocol: 'openai',
    baseUrl: 'https://api.openai.com/v1',
    models: [{ id: 'gpt-5.6-sol', label: 'GPT-5.6 Sol', contextWindow: 1050000, maxOutputTokens: 128000, vision: true }],
  },
  {
    name: 'Google Gemini',
    note: 'OpenAI 兼容端点；gemini-2.5-pro / 2.5-flash 均为 1M、支持识图',
    protocol: 'openai',
    baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai',
    models: [{ id: 'gemini-2.5-pro', label: 'Gemini 2.5 Pro', contextWindow: 1048576, maxOutputTokens: 65536, vision: true }],
  },
]

/** 友好显示 token 数：1048576 → 1M、131072 → 128K */
export function formatTokens(n?: number) {
  const v = Number(n) || 0
  if (v >= 1048576) {return `${Math.round(v / 1048576 * 10) / 10}M`}
  if (v >= 1024) {return `${Math.round(v / 1024)}K`}
  return String(v)
}
