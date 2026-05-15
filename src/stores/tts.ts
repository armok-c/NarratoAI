import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { AppConfig } from '@/types'
import { tauriInvoke } from '@/composables/useTauri'

export interface EdgeEngineConfig {
  voiceName: string
  volume: number
  rate: number
  pitch: number
}

export interface AzureEngineConfig {
  voiceName: string
  volume: number
  rate: number
  pitch: number
  speechKey: string
  speechRegion: string
}

export interface TencentEngineConfig {
  secretId: string
  secretKey: string
  region: string
}

export interface SoulvoiceEngineConfig {
  apiKey: string
  voiceUri: string
  apiUrl: string
  model: string
}

export interface QwenEngineConfig {
  apiKey: string
  apiUrl: string
  modelName: string
}

export interface IndexTTS2EngineConfig {
  apiUrl: string
  referenceAudio: string
  inferMode: string
  temperature: number
  topP: number
  topK: number
  doSample: boolean
  numBeams: number
  repetition_penalty: number
}

export interface DoubaoEngineConfig {
  ak: string
  sk: string
  appid: string
  token: string
  cluster: string
  apiUrl: string
  volume: number
  pitch: number
}

export type EngineConfigs = {
  edge_tts: EdgeEngineConfig
  azure_speech: AzureEngineConfig
  tencent_tts: TencentEngineConfig
  soulvoice: SoulvoiceEngineConfig
  tts_qwen: QwenEngineConfig
  indextts2: IndexTTS2EngineConfig
  doubaotts: DoubaoEngineConfig
}

const defaultEngineConfigs: EngineConfigs = {
  edge_tts: { voiceName: '', volume: 100, rate: 0, pitch: 0 },
  azure_speech: { voiceName: '', volume: 100, rate: 0, pitch: 0, speechKey: '', speechRegion: '' },
  tencent_tts: { secretId: '', secretKey: '', region: '' },
  soulvoice: { apiKey: '', voiceUri: '', apiUrl: '', model: '' },
  tts_qwen: { apiKey: '', apiUrl: '', modelName: '' },
  indextts2: { apiUrl: '', referenceAudio: '', inferMode: '', temperature: 0.7, topP: 0.9, topK: 50, doSample: true, numBeams: 1, repetition_penalty: 1.0 },
  doubaotts: { ak: '', sk: '', appid: '', token: '', cluster: '', apiUrl: '', volume: 100, pitch: 0 },
}

function cloneDefaults(): EngineConfigs {
  return JSON.parse(JSON.stringify(defaultEngineConfigs))
}

export const useTtsStore = defineStore('tts', () => {
  const engine = ref<string>('edge_tts')
  const engineConfigs = ref<EngineConfigs>(cloneDefaults())
  const loading = ref(false)
  const voiceList = ref<string[]>([])
  const voicesLoading = ref(false)
  const dirty = ref(false)

  watch(engine, () => { dirty.value = true }, { flush: 'sync' })
  watch(engineConfigs, () => { dirty.value = true }, { deep: true, flush: 'sync' })

  function setEngine(e: string) {
    engine.value = e
    dirty.value = true
  }

  function updateConfig(engineName: string, config: Record<string, unknown>) {
    const map = engineConfigs.value as unknown as Record<string, Record<string, unknown>>
    const current = map[engineName]
    if (current) {
      map[engineName] = { ...current, ...config }
      dirty.value = true
    }
  }

  async function loadConfig(config: AppConfig) {
    const ui = config.ui
    engine.value = ui.tts_engine

    // 注意：以下部分凭证字段（如 speech_key、secret_id、api_key 等）
    // 在 TypeScript 生成类型（如 AzureSection）中未声明，但 Rust 后端
    // 的 TOML 配置段实际包含这些字段。使用 as unknown as Record 绕过
    // 类型检查以访问运行时存在的字段。

    engineConfigs.value.edge_tts = {
      voiceName: ui.edge_voice_name,
      volume: ui.edge_volume,
      rate: ui.edge_rate,
      pitch: ui.edge_pitch,
    }

    engineConfigs.value.azure_speech = {
      voiceName: ui.azure_voice_name,
      volume: ui.azure_volume,
      rate: ui.azure_rate,
      pitch: ui.azure_pitch,
      speechKey: (config.azure as unknown as Record<string, string>).speech_key || '',
      speechRegion: config.azure.speech_region,
    }

    engineConfigs.value.tencent_tts = {
      secretId: (config.tencent as unknown as Record<string, string>).secret_id || '',
      secretKey: (config.tencent as unknown as Record<string, string>).secret_key || '',
      region: config.tencent.region,
    }

    engineConfigs.value.soulvoice = {
      apiKey: (config.soulvoice as unknown as Record<string, string>).api_key || '',
      voiceUri: config.soulvoice.voice_uri,
      apiUrl: config.soulvoice.api_url,
      model: config.soulvoice.model,
    }

    engineConfigs.value.tts_qwen = {
      apiKey: (config.tts_qwen as unknown as Record<string, string>).api_key || '',
      apiUrl: config.tts_qwen.api_url,
      modelName: config.tts_qwen.model_name,
    }

    engineConfigs.value.indextts2 = {
      apiUrl: config.indextts2.api_url,
      referenceAudio: config.indextts2.reference_audio,
      inferMode: config.indextts2.infer_mode,
      temperature: config.indextts2.temperature,
      topP: config.indextts2.top_p,
      topK: config.indextts2.top_k,
      doSample: config.indextts2.do_sample,
      numBeams: config.indextts2.num_beams,
      repetition_penalty: config.indextts2.repetition_penalty,
    }

    engineConfigs.value.doubaotts = {
      ak: (config.doubaotts as unknown as Record<string, string>).ak || '',
      sk: (config.doubaotts as unknown as Record<string, string>).sk || '',
      appid: config.doubaotts.appid,
      token: (config.doubaotts as unknown as Record<string, string>).token || '',
      cluster: config.doubaotts.cluster,
      apiUrl: config.doubaotts.api_url,
      volume: config.doubaotts.volume,
      pitch: config.doubaotts.pitch,
    }

    dirty.value = false
  }

  async function loadVoices() {
    voicesLoading.value = true
    try {
      const voices = await tauriInvoke<string[]>('get_edge_tts_voices')
      voiceList.value = voices
    } catch (err: unknown) {
      console.warn('[tts] 获取音色列表失败:', err)
      voiceList.value = []
    } finally {
      voicesLoading.value = false
    }
  }

  function markClean() {
    dirty.value = false
  }

  function resetPanel() {
    engine.value = 'edge_tts'
    engineConfigs.value = cloneDefaults()
    voiceList.value = []
  }

  return {
    engine,
    engineConfigs,
    loading,
    voiceList,
    voicesLoading,
    dirty,
    setEngine,
    updateConfig,
    loadConfig,
    loadVoices,
    markClean,
    resetPanel,
  }
})
