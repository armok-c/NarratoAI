// TODO:: Phase 14 skeleton — replace with ts-rs generated types from src/types/generated/

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
}

export interface LLMConfig {
  provider: string
  model: string
  apiKey: string
  baseUrl: string
}
