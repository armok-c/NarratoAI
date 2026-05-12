import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Mock @tauri-apps/api/core before importing the module under test
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('useTauri', () => {
  let invoke: ReturnType<typeof vi.fn>
  let useTauri: typeof import('../src/composables/useTauri.ts')

  beforeEach(() => {
    vi.resetModules()
    vi.unstubAllEnvs()
  })

  afterEach(() => {
    vi.unstubAllEnvs()
  })

  // Load the module after resetting module cache so import.meta.env is fresh
  async function loadFresh() {
    const core = await import('@tauri-apps/api/core')
    invoke = core.invoke as ReturnType<typeof vi.fn>
    useTauri = await import('../src/composables/useTauri.ts')
    return useTauri
  }

  // ---- isDebugMode ----
  describe('isDebugMode', () => {
    it('returns true when DEV is true', async () => {
      vi.stubEnv('DEV', true)
      vi.stubEnv('VITE_DEBUG', undefined)
      const { isDebugMode } = await loadFresh()
      expect(isDebugMode()).toBe(true)
    })

    it('returns true when VITE_DEBUG is "true"', async () => {
      vi.stubEnv('DEV', false)
      vi.stubEnv('VITE_DEBUG', 'true')
      const { isDebugMode } = await loadFresh()
      expect(isDebugMode()).toBe(true)
    })

    it('returns false in production without VITE_DEBUG', async () => {
      vi.stubEnv('DEV', false)
      vi.stubEnv('VITE_DEBUG', undefined)
      const { isDebugMode } = await loadFresh()
      expect(isDebugMode()).toBe(false)
    })
  })

  // ---- filterSensitiveArgs ----
  describe('filterSensitiveArgs', () => {
    it('removes 6 sensitive fields from objects', async () => {
      vi.stubEnv('DEV', false)
      const { filterSensitiveArgs } = await loadFresh()
      const input = {
        apiKey: 'secret-key',
        password: 'pass123',
        token: 'tok-abc',
        secret: 'sec-xyz',
        api_key: 'ak-123',
        accessToken: 'at-456',
        safeField: 'hello',
      }
      const result = filterSensitiveArgs(input) as Record<string, unknown>
      expect(result.apiKey).toBeUndefined()
      expect(result.password).toBeUndefined()
      expect(result.token).toBeUndefined()
      expect(result.secret).toBeUndefined()
      expect(result.api_key).toBeUndefined()
      expect(result.accessToken).toBeUndefined()
      expect(result.safeField).toBe('hello')
    })

    it('returns arrays as-is', async () => {
      vi.stubEnv('DEV', false)
      const { filterSensitiveArgs } = await loadFresh()
      const arr = [1, 2, 3]
      expect(filterSensitiveArgs(arr)).toBe(arr)
    })

    it('returns primitives as-is', async () => {
      vi.stubEnv('DEV', false)
      const { filterSensitiveArgs } = await loadFresh()
      expect(filterSensitiveArgs('string')).toBe('string')
      expect(filterSensitiveArgs(42)).toBe(42)
      expect(filterSensitiveArgs(true)).toBe(true)
    })

    it('returns null as-is', async () => {
      vi.stubEnv('DEV', false)
      const { filterSensitiveArgs } = await loadFresh()
      expect(filterSensitiveArgs(null)).toBeNull()
    })
  })

  // ---- debugLog ----
  describe('debugLog', () => {
    it('logs when debug mode is on', async () => {
      vi.stubEnv('DEV', true)
      const { debugLog } = await loadFresh()
      const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
      debugLog('test message', { safe: 'data' })
      expect(logSpy).toHaveBeenCalledWith('test message', { safe: 'data' })
      logSpy.mockRestore()
    })

    it('is silent when debug mode is off', async () => {
      vi.stubEnv('DEV', false)
      vi.stubEnv('VITE_DEBUG', undefined)
      const { debugLog } = await loadFresh()
      const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
      debugLog('test message', { data: 'value' })
      expect(logSpy).not.toHaveBeenCalled()
      logSpy.mockRestore()
    })

    it('filters sensitive args in log output', async () => {
      vi.stubEnv('DEV', true)
      const { debugLog } = await loadFresh()
      const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
      debugLog('msg', { apiKey: 'secret', name: 'visible' })
      const loggedArg = logSpy.mock.calls[0][1] as Record<string, unknown>
      expect(loggedArg.apiKey).toBeUndefined()
      expect(loggedArg.name).toBe('visible')
      logSpy.mockRestore()
    })
  })

  // ---- debugError ----
  describe('debugError', () => {
    it('logs error when debug mode is on', async () => {
      vi.stubEnv('DEV', true)
      const { debugError } = await loadFresh()
      const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      const error = new Error('test error')
      debugError('error msg', error)
      expect(errSpy).toHaveBeenCalledWith('error msg', error)
      errSpy.mockRestore()
    })

    it('is silent when debug mode is off', async () => {
      vi.stubEnv('DEV', false)
      vi.stubEnv('VITE_DEBUG', undefined)
      const { debugError } = await loadFresh()
      const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      debugError('error msg', new Error('test'))
      expect(errSpy).not.toHaveBeenCalled()
      errSpy.mockRestore()
    })
  })

  // ---- normalizeTauriArgs ----
  describe('normalizeTauriArgs', () => {
    it('converts BigInt to Number', async () => {
      vi.stubEnv('DEV', false)
      const { normalizeTauriArgs } = await loadFresh()
      expect(normalizeTauriArgs(BigInt(42))).toBe(42)
    })

    it('throws on BigInt overflow', async () => {
      vi.stubEnv('DEV', false)
      const { normalizeTauriArgs } = await loadFresh()
      const huge = BigInt(Number.MAX_SAFE_INTEGER) + BigInt(1)
      expect(() => normalizeTauriArgs(huge)).toThrow(
        /超出 JavaScript 安全整数范围/
      )
    })

    it('normalizes nested objects with BigInt values', async () => {
      vi.stubEnv('DEV', false)
      const { normalizeTauriArgs } = await loadFresh()
      const input = {
        count: BigInt(10),
        nested: { value: BigInt(20) },
      }
      const result = normalizeTauriArgs(input) as Record<string, unknown>
      expect(result.count).toBe(10)
      const nested = result.nested as Record<string, unknown>
      expect(nested.value).toBe(20)
    })

    it('normalizes arrays with BigInt values', async () => {
      vi.stubEnv('DEV', false)
      const { normalizeTauriArgs } = await loadFresh()
      const result = normalizeTauriArgs([BigInt(1), BigInt(2), 3])
      expect(result).toEqual([1, 2, 3])
    })

    it('returns primitives unchanged', async () => {
      vi.stubEnv('DEV', false)
      const { normalizeTauriArgs } = await loadFresh()
      expect(normalizeTauriArgs('hello')).toBe('hello')
      expect(normalizeTauriArgs(42)).toBe(42)
      expect(normalizeTauriArgs(true)).toBe(true)
    })

    it('returns null and undefined unchanged', async () => {
      vi.stubEnv('DEV', false)
      const { normalizeTauriArgs } = await loadFresh()
      expect(normalizeTauriArgs(null)).toBeNull()
      expect(normalizeTauriArgs(undefined)).toBeUndefined()
    })

    it('throws on BigInt underflow (below MIN_SAFE_INTEGER)', async () => {
      vi.stubEnv('DEV', false)
      const { normalizeTauriArgs } = await loadFresh()
      const tiny = BigInt(Number.MIN_SAFE_INTEGER) - BigInt(1)
      expect(() => normalizeTauriArgs(tiny)).toThrow(
        /超出 JavaScript 安全整数范围/
      )
    })
  })

  // ---- tauriInvoke ----
  describe('tauriInvoke', () => {
    it('normalizes args and calls invoke', async () => {
      vi.stubEnv('DEV', false)
      const mod = await loadFresh()
      invoke.mockResolvedValue({ result: 'ok' })
      const result = await mod.tauriInvoke<{ result: string }>('test_cmd', {
        count: BigInt(5),
        name: 'hello',
      })
      expect(result).toEqual({ result: 'ok' })
      expect(invoke).toHaveBeenCalledWith(
        'test_cmd',
        expect.objectContaining({ count: 5, name: 'hello' })
      )
    })

    it('works without args', async () => {
      vi.stubEnv('DEV', false)
      const mod = await loadFresh()
      invoke.mockResolvedValue('no-args-result')
      const result = await mod.tauriInvoke<string>('ping')
      expect(result).toBe('no-args-result')
      expect(invoke).toHaveBeenCalledWith('ping', undefined)
    })

    it('throws on invoke error', async () => {
      vi.stubEnv('DEV', false)
      const mod = await loadFresh()
      invoke.mockRejectedValue(new Error('command failed'))
      await expect(mod.tauriInvoke('failing_cmd')).rejects.toThrow('command failed')
    })
  })
})
