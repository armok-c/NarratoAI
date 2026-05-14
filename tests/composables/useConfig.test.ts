import { describe, it, expect, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLlmStore } from '../../src/stores/llm'
import {
  saveDraft,
  restoreDraft,
  clearDraft,
  hasDraft,
} from '../../src/composables/useConfig'

describe('useConfig Draft Management', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
  })

  it('saveDraft writes SettingsDraft to localStorage', () => {
    const llm = useLlmStore()
    llm.updateVisionConfig({ provider: 'openai', model: 'gpt-4o' })

    saveDraft()

    const raw = localStorage.getItem('settingsDraft')
    expect(raw).not.toBeNull()

    const draft = JSON.parse(raw!)
    expect(draft.llmVision.provider).toBe('openai')
    expect(draft.llmVision.model).toBe('gpt-4o')
    expect(draft.updatedAt).toBeDefined()
    expect(typeof draft.updatedAt).toBe('string')
  })

  it('restoreDraft reads localStorage and populates stores', () => {
    const draft = {
      llmVision: { provider: 'openai', model: 'gpt-4o', apiKey: '', baseUrl: '' },
      llmText: { provider: 'deepseek', model: 'deepseek-chat', apiKey: '', baseUrl: '' },
      updatedAt: new Date().toISOString(),
    }
    localStorage.setItem('settingsDraft', JSON.stringify(draft))

    const restored = restoreDraft()
    expect(restored).toBe(true)

    const llm = useLlmStore()
    expect(llm.visionConfig.provider).toBe('openai')
    expect(llm.textConfig.provider).toBe('deepseek')
    // restoreDraft sets all stores dirty
    expect(llm.dirty).toBe(true)
  })

  it('restoreDraft returns false when no draft exists', () => {
    const result = restoreDraft()
    expect(result).toBe(false)
  })

  it('clearDraft removes localStorage and resets store dirty flags', () => {
    const llm = useLlmStore()
    llm.markDirty()

    localStorage.setItem(
      'settingsDraft',
      JSON.stringify({ updatedAt: new Date().toISOString() })
    )

    clearDraft()

    expect(localStorage.getItem('settingsDraft')).toBeNull()
    expect(llm.dirty).toBe(false)
  })

  it('hasDraft returns correct status', () => {
    expect(hasDraft()).toBe(false)

    localStorage.setItem(
      'settingsDraft',
      JSON.stringify({ updatedAt: new Date().toISOString() })
    )
    expect(hasDraft()).toBe(true)

    localStorage.removeItem('settingsDraft')
    expect(hasDraft()).toBe(false)
  })

  it('restoreDraft silently handles corrupted JSON', () => {
    localStorage.setItem('settingsDraft', '{invalid json}')
    const result = restoreDraft()
    expect(result).toBe(false)
  })
})
