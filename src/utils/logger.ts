import log from 'loglevel'

const NAMESPACE = '[NarratoAI]'

if (import.meta.env.DEV) {
  log.setLevel('debug')
} else {
  log.setLevel('warn')
}

function prefix(...args: unknown[]): unknown[] {
  return [NAMESPACE, ...args]
}

export const logger = {
  debug(...args: unknown[]) {
    log.debug(...prefix(...args))
  },
  info(...args: unknown[]) {
    log.info(...prefix(...args))
  },
  warn(...args: unknown[]) {
    log.warn(...prefix(...args))
  },
  error(...args: unknown[]) {
    log.error(...prefix(...args))
  },
}
