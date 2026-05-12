// TODO Phase 16: replace `any` types with ts-rs generated types from src/types/generated/
import { tauriInvoke } from '@/composables/useTauri'

export interface CommandResponse {
  output_video_path?: string
}

export function runDocumentary(request: any): Promise<CommandResponse> {
  return tauriInvoke<CommandResponse>('run_documentary', { request })
}

export function runSde(request: any): Promise<CommandResponse> {
  return tauriInvoke<CommandResponse>('run_sde', { request })
}

export function runSdp(request: any): Promise<CommandResponse> {
  return tauriInvoke<CommandResponse>('run_sdp', { request })
}

export function stopPipeline(taskId: string): Promise<void> {
  return tauriInvoke<void>('stop_pipeline', { taskId })
}
