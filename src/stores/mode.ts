import { defineStore } from 'pinia'
import { ref } from 'vue'

export type WorkMode = 'documentary' | 'sde' | 'sdp'

export const useModeStore = defineStore('mode', () => {
  const currentMode = ref<WorkMode>('documentary')

  function setMode(val: WorkMode) {
    currentMode.value = val
  }

  function resetPipeline() {
    // TODO Phase 17: full implementation
  }

  return {
    currentMode,
    setMode,
    resetPipeline,
  }
})
