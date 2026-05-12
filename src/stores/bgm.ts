import { defineStore } from 'pinia'
import { ref } from 'vue'

export type BgmMode = 'random' | 'specified'

export const useBgmStore = defineStore('bgm', () => {
  const folder = ref<string>('')
  const mode = ref<BgmMode>('random')

  function setFolder(p: string) {
    folder.value = p
  }

  function setMode(m: BgmMode) {
    mode.value = m
  }

  return {
    folder,
    mode,
    setFolder,
    setMode,
  }
})
