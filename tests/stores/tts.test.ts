import { describe, it, expect, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useTtsStore } from '../../src/stores/tts'

describe('TTS Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('default engine is edge_tts', () => {
    const store = useTtsStore()
    expect(store.engine).toBe('edge_tts')
    expect(store.dirty).toBe(false)
  })

  it('setEngine switches engine', () => {
    const store = useTtsStore()
    store.setEngine('azure_speech')
    expect(store.engine).toBe('azure_speech')
  })

  it('updateConfig modifies per-engine parameters', () => {
    const store = useTtsStore()
    store.updateConfig('edge_tts', { voiceName: 'zh-CN-Xiaoxiao' })
    expect(store.engineConfigs.edge_tts.voiceName).toBe('zh-CN-Xiaoxiao')
    // Other engine configs are not affected
    expect(store.engineConfigs.azure_speech.voiceName).toBe('')
  })

  it('resetPanel restores defaults and clears engine', () => {
    const store = useTtsStore()
    store.setEngine('azure_speech')
    store.updateConfig('edge_tts', { voiceName: 'test' })
    store.resetPanel()
    expect(store.engine).toBe('edge_tts')
    expect(store.engineConfigs.edge_tts.voiceName).toBe('')
  })
})
