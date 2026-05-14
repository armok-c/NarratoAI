export interface ProviderPreset {
  recommendedModel: string
  defaultBaseUrl: string
  label: string
}

/**
 * 11 个 LLM Provider 预设的推荐模型和默认 Base URL
 */
export const PROVIDER_PRESETS: Record<string, ProviderPreset> = {
  openai: { label: 'OpenAI', recommendedModel: 'gpt-4o', defaultBaseUrl: 'https://api.openai.com/v1' },
  deepseek: { label: 'DeepSeek', recommendedModel: 'deepseek-chat', defaultBaseUrl: 'https://api.deepseek.com/v1' },
  gemini: { label: 'Gemini', recommendedModel: 'gemini-2.0-flash', defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai' },
  qwen: { label: 'Qwen', recommendedModel: 'qwen-plus', defaultBaseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1' },
  siliconflow: { label: 'SiliconFlow', recommendedModel: 'Qwen/Qwen3.5-122B-A10B', defaultBaseUrl: 'https://api.siliconflow.cn/v1' },
  moonshot: { label: 'Moonshot', recommendedModel: 'moonshot-v1-8k', defaultBaseUrl: 'https://api.moonshot.cn/v1' },
  anthropic: { label: 'Anthropic', recommendedModel: 'claude-sonnet-4-20250514', defaultBaseUrl: 'https://api.anthropic.com/v1' },
  cohere: { label: 'Cohere', recommendedModel: 'command-r-plus', defaultBaseUrl: 'https://api.cohere.com/v1' },
  together: { label: 'Together AI', recommendedModel: 'meta-llama/Llama-4-17B', defaultBaseUrl: 'https://api.together.xyz/v1' },
  openrouter: { label: 'OpenRouter', recommendedModel: 'openai/gpt-4o', defaultBaseUrl: 'https://openrouter.ai/api/v1' },
}

/**
 * v-select 直接使用的 items 数组，12 项（11 presets + 自定义）
 */
export const PROVIDER_OPTIONS: { title: string; value: string }[] = [
  ...Object.entries(PROVIDER_PRESETS).map(([key, p]) => ({
    title: p.label,
    value: key,
  })),
  { title: '自定义', value: 'custom' },
]

/**
 * 根据 Provider 标识获取预设信息，custom 时返回 null
 */
export function getProviderPreset(provider: string): ProviderPreset | null {
  return PROVIDER_PRESETS[provider] ?? null
}
