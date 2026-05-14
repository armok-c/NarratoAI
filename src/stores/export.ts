import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppConfig } from '@/types'

export const useExportStore = defineStore('export', () => {
  const outputDir = ref<string>('')
  const format = ref<string>('mp4')
  const loading = ref(false)
  const dirty = ref(false)

  function setOutputDir(p: string) {
    outputDir.value = p
  }

  function setFormat(f: string) {
    format.value = f
  }

  async function loadConfig(_config: AppConfig) {
    // Export 配置不在 AppConfig 中——运行时路径选择，设置默认值
    outputDir.value = ''
    format.value = 'mp4'
    dirty.value = false
  }

  function resetPanel() {
    outputDir.value = ''
    format.value = 'mp4'
  }

  return {
    outputDir,
    format,
    loading,
    dirty,
    setOutputDir,
    setFormat,
    loadConfig,
    resetPanel,
  }
})
