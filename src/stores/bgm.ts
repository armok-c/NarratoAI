import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppConfig } from '@/types'

export type BgmMode = 'random' | 'specified'

export const useBgmStore = defineStore('bgm', () => {
  const folder = ref<string>('')
  const mode = ref<BgmMode>('random')
  const audioFiles = ref<string[]>([])
  const loading = ref(false)
  const dirty = ref(false)

  function setFolder(p: string) {
    folder.value = p
  }

  function setMode(m: BgmMode) {
    mode.value = m
  }

  async function loadConfig(_config: AppConfig) {
    // BGM 配置不在 AppConfig 中——纯前端路径选择，设置默认值
    folder.value = ''
    mode.value = 'random'
    audioFiles.value = []
    dirty.value = false
  }

  function resetPanel() {
    folder.value = ''
    mode.value = 'random'
    audioFiles.value = []
  }

  return {
    folder,
    mode,
    audioFiles,
    loading,
    dirty,
    setFolder,
    setMode,
    loadConfig,
    resetPanel,
  }
})
