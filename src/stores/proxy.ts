import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppConfig } from '@/types'

export const useProxyStore = defineStore('proxy', () => {
  const enabled = ref(false)
  const http = ref('')
  const https = ref('')
  const loading = ref(false)
  const dirty = ref(false)

  function setEnabled(v: boolean) {
    enabled.value = v
    dirty.value = true
  }

  function setHttp(v: string) {
    http.value = v
    dirty.value = true
  }

  function setHttps(v: string) {
    https.value = v
    dirty.value = true
  }

  async function loadConfig(config: AppConfig) {
    enabled.value = config.proxy.enabled
    http.value = config.proxy.http
    https.value = config.proxy.https
    dirty.value = false
  }

  function markClean() {
    dirty.value = false
  }

  function resetPanel() {
    enabled.value = false
    http.value = ''
    https.value = ''
  }

  return {
    enabled,
    http,
    https,
    loading,
    dirty,
    setEnabled,
    setHttp,
    setHttps,
    loadConfig,
    markClean,
    resetPanel,
  }
})
