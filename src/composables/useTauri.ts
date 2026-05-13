import { invoke } from '@tauri-apps/api/core'

const MAX_SAFE_BIGINT = BigInt(Number.MAX_SAFE_INTEGER)
const MIN_SAFE_BIGINT = BigInt(Number.MIN_SAFE_INTEGER)

function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (Object.prototype.toString.call(value) !== '[object Object]') {
    return false
  }

  const prototype = Object.getPrototypeOf(value)
  return prototype === Object.prototype || prototype === null
}

function serializeBigIntArg(value: bigint, path: string): number {
  if (value > MAX_SAFE_BIGINT || value < MIN_SAFE_BIGINT) {
    throw new Error(`Tauri 参数 ${path} 的 bigint 值超出 JavaScript 安全整数范围，无法安全序列化`)
  }

  return Number(value)
}

export function normalizeTauriArgs(args: unknown, path = 'args'): unknown {
  if (typeof args === 'bigint') {
    return serializeBigIntArg(args, path)
  }

  if (Array.isArray(args)) {
    return args.map((item, index) => normalizeTauriArgs(item, `${path}[${index}]`))
  }

  if (isPlainObject(args)) {
    return Object.fromEntries(
      Object.entries(args).map(([key, value]) => [key, normalizeTauriArgs(value, `${path}.${key}`)])
    )
  }

  return args
}

/**
 * 检查是否处于调试模式
 * 生产环境中禁用调试日志以保护敏感信息
 */
export function isDebugMode(): boolean {
  return import.meta.env.DEV || import.meta.env.VITE_DEBUG === 'true'
}

/**
 * 过滤参数中的敏感字段
 * @param args 原始参数
 * @returns 过滤后的安全参数
 */
export function filterSensitiveArgs(args: unknown): unknown {
  if (isPlainObject(args)) {
    const safe = { ...(args as Record<string, unknown>) }
    const sensitiveFields = ['apiKey', 'password', 'token', 'secret', 'api_key', 'accessToken']
    for (const field of sensitiveFields) {
      delete safe[field]
    }
    for (const key of Object.keys(safe)) {
      safe[key] = filterSensitiveArgs(safe[key])
    }
    return safe
  }
  if (Array.isArray(args)) {
    return args.map(filterSensitiveArgs)
  }
  return args
}

/**
 * 安全地打印日志（仅在调试模式下）
 * 生产环境中不输出可能包含敏感信息的日志
 */
export function debugLog(message: string, ...args: unknown[]): void {
  if (!isDebugMode()) {
    return
  }

  const safeArgs = args.map(filterSensitiveArgs)
  console.log(message, ...safeArgs)
}

/**
 * 安全地打印错误日志（仅在调试模式下）
 */
export function debugError(message: string, error: unknown): void {
  if (!isDebugMode()) {
    return
  }
  console.error(message, error)
}

/**
 * 调用 Tauri 命令的包装函数
 * 自动处理错误并提供安全的日志记录
 *
 * @param command - 要调用的命令名称
 * @param args - 传递给命令的参数（敏感字段会被过滤）
 * @returns 命令执行结果
 * @throws 命令执行失败时抛出错误
 */
export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    debugLog(`[tauriInvoke] 调用命令: ${command}`, args || '')
    const normalizedArgs = normalizeTauriArgs(args)
    const result = await invoke<T>(command, normalizedArgs as Record<string, unknown> | undefined)
    debugLog(`[tauriInvoke] 命令 ${command} 成功`, result)
    return result
  } catch (error) {
    debugError(`[tauriInvoke] 命令 ${command} 失败`, error)
    throw error
  }
}
