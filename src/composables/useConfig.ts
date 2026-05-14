import type { AppConfig } from '@/types'
import { tauriInvoke } from '@/composables/useTauri'
import { useLlmStore } from '@/stores/llm'
import { useTtsStore } from '@/stores/tts'
import { useBgmStore } from '@/stores/bgm'
import { useExportStore } from '@/stores/export'
import { useModeStore } from '@/stores/mode'

/**
 * 从后端加载完整配置并分发到各 Pinia stores
 * 失败时不影响 stores 现有状态
 */
export async function loadFromBackend(): Promise<AppConfig | null> {
  try {
    const config = await tauriInvoke<AppConfig>('get_config')
    const llmStore = useLlmStore()
    const ttsStore = useTtsStore()
    const bgmStore = useBgmStore()
    const exportStore = useExportStore()
    const modeStore = useModeStore()

    await Promise.all([
      llmStore.loadConfig(config),
      ttsStore.loadConfig(config),
      bgmStore.loadConfig(config),
      exportStore.loadConfig(config),
      modeStore.loadConfig(config),
    ])

    return config
  } catch (err) {
    console.warn('[useConfig] 配置加载失败:', err)
    return null
  }
}

/**
 * 保存增量配置变更到后端
 * @param changes 仅包含用户修改字段的扁平 JSON 对象
 */
export async function saveToBackend(changes: Record<string, unknown>): Promise<void> {
  await tauriInvoke<void>('save_config', changes)
}

/**
 * 收集所有 store 的脏字段，返回扁平 JSON 对象
 * 用于 save_config 的增量更新参数
 */
export function collectChangedFields(): Record<string, unknown> {
  const llmStore = useLlmStore()
  const ttsStore = useTtsStore()
  const bgmStore = useBgmStore()
  const exportStore = useExportStore()

  const changes: Record<string, unknown> = {}

  if (llmStore.dirty) {
    changes.vision_llm_provider = llmStore.visionConfig.provider
    changes.vision_openai_model_name = llmStore.visionConfig.model
    changes.vision_openai_api_key = llmStore.visionConfig.apiKey
    changes.vision_openai_base_url = llmStore.visionConfig.baseUrl
    changes.text_llm_provider = llmStore.textConfig.provider
    changes.text_openai_model_name = llmStore.textConfig.model
    changes.text_openai_api_key = llmStore.textConfig.apiKey
    changes.text_openai_base_url = llmStore.textConfig.baseUrl
  }

  if (ttsStore.dirty) {
    changes.tts_engine = ttsStore.engine
  }

  if (bgmStore.dirty) {
    changes.bgm_folder = bgmStore.folder
    changes.bgm_mode = bgmStore.mode
  }

  if (exportStore.dirty) {
    changes.output_dir = exportStore.outputDir
    changes.output_format = exportStore.format
  }

  return changes
}
