// Re-export ts-rs generated types from narratoai-core
export type { AppConfig } from './generated/AppConfig'
export type { AppSection } from './generated/AppSection'
export type { UiSection } from './generated/UiSection'
export type { ProxySection } from './generated/ProxySection'
export type { FramesSection } from './generated/FramesSection'
export type { AzureSection } from './generated/AzureSection'
export type { TencentSection } from './generated/TencentSection'
export type { SoulVoiceSection } from './generated/SoulVoiceSection'
export type { TtsQwenSection } from './generated/TtsQwenSection'
export type { IndexTTS2Section } from './generated/IndexTTS2Section'
export type { DoubaoTTSSection } from './generated/DoubaoTTSSection'
export type { AudioSection } from './generated/AudioSection'

// Remaining manual types
export interface VideoMeta {
  id: string
  name: string
  path: string
  status: string
  duration?: number
}

export interface ProgressPayload {
  pipeline_type: string
  task_id: string
  step_name: string
  percent: number
  message: string
  step_index: number
  total_steps: number
  status: string
  error_code?: string | null
  error_message?: string | null
}

export interface LLMConfig {
  provider: string
  model: string
  apiKey: string
  baseUrl: string
}
