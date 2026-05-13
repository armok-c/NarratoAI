import { defineStore } from 'pinia'
import { ref } from 'vue'
import { usePipelineStore } from '@/stores/pipeline'

export type WorkMode = 'documentary' | 'sde' | 'sdp'

export const useModeStore = defineStore('mode', () => {
  const stored = localStorage.getItem('lastMode') as WorkMode | null
  const currentMode = ref<WorkMode>(
    stored === 'documentary' || stored === 'sde' || stored === 'sdp'
      ? stored
      : 'documentary'
  )

  function setMode(val: WorkMode) {
    currentMode.value = val
    localStorage.setItem('lastMode', val)
  }

  function resetPipeline() {
    const pipelineStore = usePipelineStore()
    pipelineStore.reset()
  }

  return {
    currentMode,
    setMode,
    resetPipeline,
  }
})
