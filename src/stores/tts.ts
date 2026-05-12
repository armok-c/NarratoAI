import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useTtsStore = defineStore('tts', () => {
  const engine = ref<string>('edge_tts')
  const engineConfigs = ref<Record<string, any>>({})

  function setEngine(e: string) {
    engine.value = e
  }

  function updateConfig(engineName: string, config: any) {
    engineConfigs.value[engineName] = config
  }

  return {
    engine,
    engineConfigs,
    setEngine,
    updateConfig,
  }
})
