// TODO Phase 16: replace `any` types with ts-rs generated types from src/types/generated/
import { tauriInvoke } from '@/composables/useTauri'

export function loadScript(path: string): Promise<string> {
  return tauriInvoke<string>('load_script', { path })
}

export function saveScript(path: string, content: string): Promise<void> {
  return tauriInvoke<void>('save_script', { path, content })
}

export function validateScript(path: string): Promise<{ valid: boolean; errors: string[] }> {
  return tauriInvoke<{ valid: boolean; errors: string[] }>('validate_script', { path })
}

export function updateNarration(path: string, clipIndex: number, narration: string): Promise<void> {
  return tauriInvoke<void>('update_narration', { path, clipIndex, narration })
}

export function setOst(path: string, clipIndex: number, ostType: number): Promise<void> {
  return tauriInvoke<void>('set_ost', { path, clipIndex, ostType })
}
