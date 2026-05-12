// TODO Phase 16: replace `any` types with ts-rs generated types from src/types/generated/
import { tauriInvoke } from '@/composables/useTauri'

export function exportJianyingDraft(scriptPath: string, outputDir: string): Promise<string> {
  return tauriInvoke<string>('export_jianying_draft', { scriptPath, outputDir })
}
