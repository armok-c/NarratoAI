import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { AppConfig } from '@/types'

export type WorkMode = 'documentary' | 'sde' | 'sdp'

export interface ModeParams {
  frameInterval: number
  visionBatchSize: number
  dramaName: string
  temperature: number
  clipCount: number
  minDuration: number
  maxDuration: number
}

const defaultParams: ModeParams = {
  frameInterval: 3,
  visionBatchSize: 10,
  dramaName: '',
  temperature: 1.0,
  clipCount: 5,
  minDuration: 2,
  maxDuration: 10,
}

export const useModeStore = defineStore('mode', () => {
  const stored = localStorage.getItem('lastMode') as WorkMode | null
  const currentMode = ref<WorkMode>(
    stored === 'documentary' || stored === 'sde' || stored === 'sdp'
      ? stored
      : 'documentary'
  )

  const params = ref<ModeParams>({ ...defaultParams })
  const dirty = ref(false)

  function setMode(val: WorkMode) {
    currentMode.value = val
    localStorage.setItem('lastMode', val)
  }

  async function loadConfig(config: AppConfig) {
    params.value.frameInterval = Number(config.frames.frame_interval_input)
    params.value.visionBatchSize = config.frames.vision_batch_size
    // dramaName, temperature, clipCount, minDuration, maxDuration 为前端维护参数，保留默认值
    params.value.dramaName = defaultParams.dramaName
    params.value.temperature = defaultParams.temperature
    params.value.clipCount = defaultParams.clipCount
    params.value.minDuration = defaultParams.minDuration
    params.value.maxDuration = defaultParams.maxDuration
  }

  function resetParams() {
    params.value = { ...defaultParams }
  }

  function markClean() {
    dirty.value = false
  }

  watch(params, () => { dirty.value = true }, { deep: true, flush: 'sync' })

  return {
    currentMode,
    params,
    setMode,
    loadConfig,
    resetParams,
    dirty,
    markClean,
  }
})
