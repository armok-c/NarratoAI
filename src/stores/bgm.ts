import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { AppConfig } from '@/types'

export type BgmMode = 'random' | 'specified'

export const useBgmStore = defineStore('bgm', () => {
  const folder = ref<string>('')
  const mode = ref<BgmMode>('random')
  const audioFiles = ref<string[]>([])
  const selectedFile = ref<string>('')
  const loading = ref(false)
  const dirty = ref(false)

  watch([folder, mode, selectedFile], () => { dirty.value = true }, { flush: 'sync' })

  function setFolder(p: string) {
    folder.value = p
    dirty.value = true
  }

  function setMode(m: BgmMode) {
    mode.value = m
    dirty.value = true
  }

  function setSelectedFile(f: string) {
    selectedFile.value = f
    dirty.value = true
  }

  async function loadConfig(_config: AppConfig) {
    // BGM 配置不在 AppConfig 中——纯前端路径选择，设置默认值
    folder.value = ''
    mode.value = 'random'
    audioFiles.value = []
    dirty.value = false
  }

  function markClean() {
    dirty.value = false
  }

  function resetPanel() {
    folder.value = ''
    mode.value = 'random'
    audioFiles.value = []
    selectedFile.value = ''
  }

  return {
    folder,
    mode,
    audioFiles,
    selectedFile,
    loading,
    dirty,
    setFolder,
    setMode,
    setSelectedFile,
    loadConfig,
    markClean,
    resetPanel,
  }
})
