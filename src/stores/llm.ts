import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { LLMConfig } from '@/types'
import type { AppConfig } from '@/types'
import { tauriInvoke } from '@/composables/useTauri'

export const useLlmStore = defineStore('llm', () => {
  const visionConfig = ref<LLMConfig>({ provider: '', model: '', apiKey: '', baseUrl: '' })
  const textConfig = ref<LLMConfig>({ provider: '', model: '', apiKey: '', baseUrl: '' })
  const loading = ref(false)
  const dirty = ref(false)
  const testing = ref(false)

  watch(visionConfig, () => { dirty.value = true }, { deep: true, flush: 'sync' })
  watch(textConfig, () => { dirty.value = true }, { deep: true, flush: 'sync' })

  function updateVisionConfig(c: Partial<LLMConfig>) {
    visionConfig.value = { ...visionConfig.value, ...c }
    dirty.value = true
  }

  function updateTextConfig(c: Partial<LLMConfig>) {
    textConfig.value = { ...textConfig.value, ...c }
    dirty.value = true
  }

  async function loadConfig(config: AppConfig) {
    const app = config.app
    visionConfig.value = {
      provider: app.vision_llm_provider,
      model: app.vision_openai_model_name,
      apiKey: (app as unknown as Record<string, string>).vision_openai_api_key || '',
      baseUrl: app.vision_openai_base_url,
    }
    textConfig.value = {
      provider: app.text_llm_provider,
      model: app.text_openai_model_name,
      apiKey: (app as unknown as Record<string, string>).text_openai_api_key || '',
      baseUrl: app.text_openai_base_url,
    }
    dirty.value = false
  }

  async function testConnection(which: 'vision' | 'text') {
    testing.value = true
    try {
      const cfg = which === 'vision' ? visionConfig.value : textConfig.value
      await tauriInvoke('test_llm_connection', {
        provider: cfg.provider,
        model: cfg.model,
        apiKey: cfg.apiKey,
        baseUrl: cfg.baseUrl,
      })
    } finally {
      testing.value = false
    }
  }

  function resetPanel(which: 'vision' | 'text') {
    const empty: LLMConfig = { provider: '', model: '', apiKey: '', baseUrl: '' }
    if (which === 'vision') {
      visionConfig.value = { ...empty }
    } else {
      textConfig.value = { ...empty }
    }
  }

  function markClean() {
    dirty.value = false
  }

  function markDirty() {
    dirty.value = true
  }

  return {
    visionConfig,
    textConfig,
    loading,
    dirty,
    testing,
    updateVisionConfig,
    updateTextConfig,
    loadConfig,
    testConnection,
    resetPanel,
    markClean,
    markDirty,
  }
})
