import { describe, it, expect, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLlmStore } from '../../src/stores/llm'
import { useTtsStore } from '../../src/stores/tts'
import { useBgmStore } from '../../src/stores/bgm'
import { useExportStore } from '../../src/stores/export'
import { useProxyStore } from '../../src/stores/proxy'
import { useModeStore } from '../../src/stores/mode'

describe('Store Dirty Tracking', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  // Test 1: LLM - direct visionConfig mutation sets dirty
  it('llm store: direct visionConfig ref mutation sets dirty', () => {
    const store = useLlmStore()
    expect(store.dirty).toBe(false)
    // 模拟 v-model 直接赋值，不使用 mutator
    store.visionConfig.provider = 'openai'
    expect(store.dirty).toBe(true)
  })

  // Test 2: LLM - loadConfig resets dirty to false
  it('llm store: loadConfig resets dirty to false', () => {
    const store = useLlmStore()
    store.visionConfig.provider = 'openai'
    expect(store.dirty).toBe(true)
    // 构建一个最小的 mock config（只需要必要的字段避免运行时错误）
    const mockConfig = {
      app: {
        vision_llm_provider: 'openai',
        vision_openai_model_name: 'gpt-4o',
        vision_openai_api_key: 'sk-test',
        vision_openai_base_url: 'https://api.openai.com/v1',
        text_llm_provider: 'deepseek',
        text_openai_model_name: 'deepseek-chat',
        text_openai_api_key: 'sk-test2',
        text_openai_base_url: 'https://api.deepseek.com/v1',
      },
      proxy: { enabled: false, http: '', https: '' },
    } as any
    store.loadConfig(mockConfig as any)
    expect(store.dirty).toBe(false)
  })

  // Test 3: TTS - direct engine mutation sets dirty
  it('tts store: direct engine ref mutation sets dirty', () => {
    const store = useTtsStore()
    expect(store.dirty).toBe(false)
    store.engine = 'azure_speech'
    expect(store.dirty).toBe(true)
  })

  // Test 4: BGM - direct folder mutation sets dirty
  it('bgm store: direct folder ref mutation sets dirty', () => {
    const store = useBgmStore()
    expect(store.dirty).toBe(false)
    store.folder = '/some/music/path'
    expect(store.dirty).toBe(true)
  })

  // Test 5: Export - direct format mutation sets dirty
  it('export store: direct format ref mutation sets dirty', () => {
    const store = useExportStore()
    expect(store.dirty).toBe(false)
    store.format = 'mkv'
    expect(store.dirty).toBe(true)
  })

  // Test 6: Proxy - direct enabled mutation sets dirty
  it('proxy store: direct enabled ref mutation sets dirty', () => {
    const store = useProxyStore()
    expect(store.dirty).toBe(false)
    store.enabled = true
    expect(store.dirty).toBe(true)
  })

  // Test 7: Mode - direct params mutation sets dirty, markClean resets it
  it('mode store: direct params ref mutation sets dirty and markClean resets', () => {
    const store = useModeStore()
    expect(store.dirty).toBe(false)
    store.params.frameInterval = 5
    expect(store.dirty).toBe(true)
    store.markClean()
    expect(store.dirty).toBe(false)
  })
})
