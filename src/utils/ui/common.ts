import { createApp, isProxy, isReactive, isRef, toRaw } from 'vue'
import { getErrorMessage } from '../common'

export function convertProxy<T>(sourceObj: T, deep = false): T {
  // rome-ignore lint/suspicious/noExplicitAny: <explanation>
  const objectIterator = (input: any): any => {
    if (Array.isArray(input)) {
      return input.map((item) => objectIterator(item))
    }

    if (isRef(input) || isReactive(input) || isProxy(input)) {
      return deep ? objectIterator(toRaw(input)) : toRaw(input)
    }

    if (deep && input && typeof input === 'object') {
      return Object.keys(input).reduce((acc, key) => {
        acc[key as keyof typeof acc] = objectIterator(input[key])
        return acc
      }, {} as T)
    }

    return input
  }

  return objectIterator(sourceObj)
}

export function registerLogger(app: ReturnType<typeof createApp>) {
  const preservedConsoleInfo = console.info
  const preservedConsoleError = console.error
  const preservedConsoleWarn = console.warn
  const preservedConsoleDebug = console.debug
  const preservedConsoleTrace = console.trace

  const logCommon = (logger: (...args: unknown[]) => void, ...args: unknown[]) => {
    try {
      logger(...args)
    } catch {
      try {
        logger(...convertProxy(args))
      } catch {
        logger(...convertProxy(args, true))
      }
    }
  }

  console.info = (...args: unknown[]) => {
    preservedConsoleInfo.apply(console, args)
    logCommon(window.LoggerUtils.info, ...args)
  }

  console.error = (...args: unknown[]) => {
    const error = getErrorMessage(...args)
    preservedConsoleError.apply(console, args)
    logCommon(window.LoggerUtils.error, ...args)
  }

  console.warn = (...args: unknown[]) => {
    preservedConsoleWarn.apply(console, args)

    if (!(args[0] as string).startsWith?.('[Vue warn]')) {
      logCommon(window.LoggerUtils.warn, ...args)
    }
  }

  console.debug = (...args: unknown[]) => {
    preservedConsoleDebug.apply(console, args)
    logCommon(window.LoggerUtils.debug, ...args)
  }

  console.trace = (...args: unknown[]) => {
    preservedConsoleTrace.apply(console, args)
    logCommon(window.LoggerUtils.trace, ...args)
  }

  window.onerror = (err) => {
    const error = getErrorMessage(err)
    console.error(...error)
  }

  app.config.errorHandler = (err) => {
    console.error(err)
  }

  window.onunhandledrejection = (ev) => {
    const message = ev.reason.message ?? JSON.stringify(ev.reason) ?? ev.reason
    console.error('Uncaught in promise', message)
  }
}
