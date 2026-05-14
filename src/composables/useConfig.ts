import type { AppConfig } from '@/types'
import { tauriInvoke } from '@/composables/useTauri'
import { useLlmStore } from '@/stores/llm'
import { useTtsStore } from '@/stores/tts'
import { useBgmStore } from '@/stores/bgm'
import { useExportStore } from '@/stores/export'
import { useModeStore } from '@/stores/mode'
import { useProxyStore } from '@/stores/proxy'
import { ref } from 'vue'

// ============================================================
// Types
// ============================================================

export interface SettingsDraft {
  llmVision?: { provider: string; model: string; apiKey: string; baseUrl: string }
  llmText?: { provider: string; model: string; apiKey: string; baseUrl: string }
  tts?: { engine: string; engineConfigs: Record<string, unknown> }
  bgm?: { folder: string; mode: string; selectedFile: string }
  export?: { outputDir: string; format: string }
  modeParams?: Record<string, unknown>
  networkProxy?: { enabled: boolean; http: string; https: string }
  updatedAt: string
}

// ============================================================
// Module-level state
// ============================================================

let _lastConfig: AppConfig | null = null

export const draftStatus = ref<{ hasDraft: boolean; updatedAt: string | null }>({
  hasDraft: false,
  updatedAt: null,
})

// ============================================================
// Draft Management (D-20 ~ D-24)
// ============================================================

/**
 * 将当前所有 store 值暂存到 localStorage('settingsDraft')
 * 触发时机：SettingsDrawer 关闭时
 * 失败时静默降级（D-24）
 */
export function saveDraft(): void {
  try {
    const llm = useLlmStore()
    const tts = useTtsStore()
    const bgm = useBgmStore()
    const exp = useExportStore()
    const mode = useModeStore()
    const proxy = useProxyStore()

    const draft: SettingsDraft = {
      llmVision: { ...llm.visionConfig },
      llmText: { ...llm.textConfig },
      tts: {
        engine: tts.engine,
        engineConfigs: JSON.parse(JSON.stringify(tts.engineConfigs)),
      },
      bgm: { folder: bgm.folder, mode: bgm.mode, selectedFile: bgm.selectedFile },
      export: { outputDir: exp.outputDir, format: exp.format },
      modeParams: { ...mode.params },
      networkProxy: { enabled: proxy.enabled, http: proxy.http, https: proxy.https },
      updatedAt: new Date().toISOString(),
    }

    localStorage.setItem('settingsDraft', JSON.stringify(draft))
    draftStatus.value = { hasDraft: true, updatedAt: draft.updatedAt }
  } catch {
    // D-24: 静默降级（localStorage 满/禁用时不弹窗）
  }
}

/**
 * 从 localStorage 恢复草稿到 Pinia stores
 * 覆盖显示（D-22 乐观策略）
 * 恢复失败时静默跳过（D-24）
 * @returns 是否成功恢复了草稿
 */
export function restoreDraft(): boolean {
  try {
    const raw = localStorage.getItem('settingsDraft')
    if (!raw) return false

    const draft: SettingsDraft = JSON.parse(raw)
    if (!draft || !draft.updatedAt) return false

    const llm = useLlmStore()
    const tts = useTtsStore()
    const bgm = useBgmStore()
    const exp = useExportStore()
    const mode = useModeStore()
    const proxy = useProxyStore()

    if (draft.llmVision) {
      llm.visionConfig = { ...llm.visionConfig, ...draft.llmVision }
    }
    if (draft.llmText) {
      llm.textConfig = { ...llm.textConfig, ...draft.llmText }
    }
    if (draft.tts) {
      tts.engine = draft.tts.engine
      if (draft.tts.engineConfigs) {
        tts.engineConfigs = JSON.parse(
          JSON.stringify(draft.tts.engineConfigs)
        ) as typeof tts.engineConfigs
      }
    }
    if (draft.bgm) {
      bgm.folder = draft.bgm.folder
      bgm.mode = draft.bgm.mode as 'random' | 'specified'
      if (draft.bgm.selectedFile) {
        bgm.selectedFile = draft.bgm.selectedFile
      }
    }
    if (draft.export) {
      exp.outputDir = draft.export.outputDir
      exp.format = draft.export.format
    }
    if (draft.modeParams) {
      mode.params = { ...mode.params, ...draft.modeParams } as typeof mode.params
    }
    if (draft.networkProxy) {
      proxy.enabled = draft.networkProxy.enabled
      proxy.http = draft.networkProxy.http
      proxy.https = draft.networkProxy.https
    }

    // 标记所有 stores 为 dirty
    llm.dirty = true
    tts.dirty = true
    bgm.dirty = true
    exp.dirty = true
    proxy.dirty = true

    draftStatus.value = { hasDraft: true, updatedAt: draft.updatedAt }
    return true
  } catch {
    // D-24: JSON 解析失败等 → 静默跳过
    return false
  }
}

/**
 * 清除 localStorage 草稿 + 重置所有 dirty 标志
 * 调用时机：全局保存成功后（D-23）
 */
export function clearDraft(): void {
  try {
    localStorage.removeItem('settingsDraft')
  } catch {
    // 静默
  }
  const llm = useLlmStore()
  const tts = useTtsStore()
  const bgm = useBgmStore()
  const exp = useExportStore()
  const proxy = useProxyStore()
  llm.dirty = false
  tts.dirty = false
  bgm.dirty = false
  exp.dirty = false
  proxy.dirty = false
  draftStatus.value = { hasDraft: false, updatedAt: null }
}

/**
 * 检查是否存在 localStorage 草稿
 */
export function hasDraft(): boolean {
  try {
    return localStorage.getItem('settingsDraft') !== null
  } catch {
    return false
  }
}

// ============================================================
// Config Load / Save
// ============================================================

/**
 * 从后端加载完整配置并分发到各 Pinia stores
 * 失败时不影响 stores 现有状态
 */
export async function loadFromBackend(): Promise<AppConfig | null> {
  try {
    const config = await tauriInvoke<AppConfig>('get_config')
    _lastConfig = config

    const llmStore = useLlmStore()
    const ttsStore = useTtsStore()
    const bgmStore = useBgmStore()
    const exportStore = useExportStore()
    const modeStore = useModeStore()
    const proxyStore = useProxyStore()

    await Promise.all([
      llmStore.loadConfig(config),
      ttsStore.loadConfig(config),
      bgmStore.loadConfig(config),
      exportStore.loadConfig(config),
      modeStore.loadConfig(config),
      proxyStore.loadConfig(config),
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
 * 收集所有 store 脏字段 + 引擎配置 → 扁平 JSON
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

    const activeEngine = ttsStore.engine
    const cfg = ttsStore.engineConfigs

    // 只发送当前选中引擎的配置字段，避免覆盖其他引擎的凭据
    switch (activeEngine) {
      case 'edge_tts':
        changes.edge_voice_name = cfg.edge_tts.voiceName
        changes.edge_volume = cfg.edge_tts.volume
        changes.edge_rate = cfg.edge_tts.rate
        changes.edge_pitch = cfg.edge_tts.pitch
        break
      case 'azure_speech':
        changes.azure_voice_name = cfg.azure_speech.voiceName
        changes.azure_volume = cfg.azure_speech.volume
        changes.azure_rate = cfg.azure_speech.rate
        changes.azure_pitch = cfg.azure_speech.pitch
        changes.azure_speech_key = cfg.azure_speech.speechKey
        changes.azure_speech_region = cfg.azure_speech.speechRegion
        break
      case 'tencent_tts':
        changes.tencent_secret_id = cfg.tencent_tts.secretId
        changes.tencent_secret_key = cfg.tencent_tts.secretKey
        changes.tencent_region = cfg.tencent_tts.region
        break
      case 'soulvoice':
        changes.soulvoice_api_key = cfg.soulvoice.apiKey
        changes.soulvoice_voice_uri = cfg.soulvoice.voiceUri
        changes.soulvoice_api_url = cfg.soulvoice.apiUrl
        changes.soulvoice_model = cfg.soulvoice.model
        break
      case 'tts_qwen':
        changes.tts_qwen_api_key = cfg.tts_qwen.apiKey
        changes.tts_qwen_api_url = cfg.tts_qwen.apiUrl
        changes.tts_qwen_model_name = cfg.tts_qwen.modelName
        break
      case 'indextts2':
        changes.indextts2_api_url = cfg.indextts2.apiUrl
        changes.indextts2_reference_audio = cfg.indextts2.referenceAudio
        changes.indextts2_infer_mode = cfg.indextts2.inferMode
        changes.indextts2_temperature = cfg.indextts2.temperature
        changes.indextts2_top_p = cfg.indextts2.topP
        changes.indextts2_top_k = cfg.indextts2.topK
        changes.indextts2_do_sample = cfg.indextts2.doSample
        changes.indextts2_num_beams = cfg.indextts2.numBeams
        changes.indextts2_repetition_penalty = cfg.indextts2.repetition_penalty
        break
      case 'doubaotts':
        changes.doubaotts_ak = cfg.doubaotts.ak
        changes.doubaotts_sk = cfg.doubaotts.sk
        changes.doubaotts_appid = cfg.doubaotts.appid
        changes.doubaotts_token = cfg.doubaotts.token
        changes.doubaotts_cluster = cfg.doubaotts.cluster
        changes.doubaotts_api_url = cfg.doubaotts.apiUrl
        changes.doubaotts_volume = cfg.doubaotts.volume
        changes.doubaotts_pitch = cfg.doubaotts.pitch
        break
    }
  }

  if (bgmStore.dirty) {
    changes.bgm_folder = bgmStore.folder
    changes.bgm_mode = bgmStore.mode
  }

  if (exportStore.dirty) {
    changes.output_dir = exportStore.outputDir
    changes.output_format = exportStore.format
  }

  const proxyStore = useProxyStore()
  if (proxyStore.dirty) {
    changes.proxy_enabled = proxyStore.enabled
    changes.proxy_http = proxyStore.http
    changes.proxy_https = proxyStore.https
  }

  return changes
}

/**
 * 聚合所有配置字段为扁平 JSON
 * 包含 dirty 字段 + 模式参数
 * 用于全局保存时发送完整的增量更新
 */
export function collectAllConfig(): Record<string, unknown> {
  const changes = collectChangedFields()
  const modeStore = useModeStore()

  // 始终包含模式参数
  changes.mode_frame_interval = modeStore.params.frameInterval
  changes.mode_vision_batch_size = modeStore.params.visionBatchSize
  changes.mode_drama_name = modeStore.params.dramaName
  changes.mode_temperature = modeStore.params.temperature
  changes.mode_clip_count = modeStore.params.clipCount
  changes.mode_min_duration = modeStore.params.minDuration
  changes.mode_max_duration = modeStore.params.maxDuration

  return changes
}
