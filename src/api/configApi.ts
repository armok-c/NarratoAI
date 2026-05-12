// TODO Phase 16: replace `any` types with ts-rs generated AppConfig from src/types/generated/
import { tauriInvoke } from '@/composables/useTauri'

export function getConfig(): Promise<any> {
  return tauriInvoke<any>('get_config')
}

export function setConfig(config: any): Promise<void> {
  return tauriInvoke<void>('set_config', { config })
}
