import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ProgressPayload } from '@/types'

export type PipelineStatus = 'idle' | 'running' | 'error'

export const usePipelineStore = defineStore('pipeline', () => {
  const status = ref<PipelineStatus>('idle')
  const progress = ref<ProgressPayload | null>(null)

  function start() {
    status.value = 'running'
  }

  function stop() {
    status.value = 'idle'
  }

  function reset() {
    status.value = 'idle'
    progress.value = null
  }

  function updateProgress(p: ProgressPayload) {
    progress.value = p
  }

  return {
    status,
    progress,
    start,
    stop,
    reset,
    updateProgress,
  }
})
