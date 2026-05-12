import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { LLMConfig } from '@/types'

export const useLlmStore = defineStore('llm', () => {
  const visionConfig = ref<LLMConfig>({ provider: '', model: '', apiKey: '', baseUrl: '' })
  const textConfig = ref<LLMConfig>({ provider: '', model: '', apiKey: '', baseUrl: '' })

  function updateVisionConfig(c: Partial<LLMConfig>) {
    Object.assign(visionConfig.value, c)
  }

  function updateTextConfig(c: Partial<LLMConfig>) {
    Object.assign(textConfig.value, c)
  }

  return {
    visionConfig,
    textConfig,
    updateVisionConfig,
    updateTextConfig,
  }
})
