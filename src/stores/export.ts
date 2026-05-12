import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useExportStore = defineStore('export', () => {
  const outputDir = ref<string>('')
  const format = ref<string>('mp4')

  function setOutputDir(p: string) {
    outputDir.value = p
  }

  function setFormat(f: string) {
    format.value = f
  }

  return {
    outputDir,
    format,
    setOutputDir,
    setFormat,
  }
})
