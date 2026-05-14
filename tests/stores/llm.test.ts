import { describe, it, expect, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLlmStore } from '../../src/stores/llm'

describe('LLM Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('has empty initial configs', () => {
    const store = useLlmStore()
    expect(store.visionConfig).toEqual({ provider: '', model: '', apiKey: '', baseUrl: '' })
    expect(store.textConfig).toEqual({ provider: '', model: '', apiKey: '', baseUrl: '' })
    expect(store.dirty).toBe(false)
  })

  it('updateVisionConfig merges fields', () => {
    const store = useLlmStore()
    store.updateVisionConfig({ provider: 'openai', model: 'gpt-4o' })
    expect(store.visionConfig.provider).toBe('openai')
    expect(store.visionConfig.model).toBe('gpt-4o')
    expect(store.visionConfig.apiKey).toBe('') // unchanged
  })

  it('updateTextConfig merges fields', () => {
    const store = useLlmStore()
    store.updateTextConfig({ provider: 'deepseek', model: 'deepseek-chat' })
    expect(store.textConfig.provider).toBe('deepseek')
    expect(store.textConfig.model).toBe('deepseek-chat')
  })

  it('resetPanel clears vision config', () => {
    const store = useLlmStore()
    store.updateVisionConfig({ provider: 'openai', model: 'gpt-4o' })
    store.resetPanel('vision')
    expect(store.visionConfig).toEqual({ provider: '', model: '', apiKey: '', baseUrl: '' })
  })

  it('markClean resets dirty flag', () => {
    const store = useLlmStore()
    store.markDirty()
    expect(store.dirty).toBe(true)
    store.markClean()
    expect(store.dirty).toBe(false)
  })
})
