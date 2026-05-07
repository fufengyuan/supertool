/**
 * helpers.ts — 通用工具函数
 */

/** Safely extract error message from unknown error */
export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return String(error)
}

/**
 * Recursively strip Vue 3 Proxy / reactive wrappers from objects.
 * Tauri IPC uses structured clone which cannot serialize Vue Proxy.
 * Use this at IPC call sites: `ipc(stripVueProxy(obj))`
 */
export function stripVueProxy<T>(obj: T, seen = new WeakSet<object>()): T {
  if (obj === null || typeof obj !== 'object') return obj
  if (seen.has(obj)) return obj as T
  seen.add(obj)

  // Functions, Symbols, DOM nodes — pass through unchanged
  if (typeof obj === 'function' || typeof obj === 'symbol') return obj as unknown as T
  const objAsObj = obj as Record<string, unknown>
  if (typeof objAsObj.nodeType === 'number') return obj

  // Vue Ref unwrap
  if (obj && typeof obj === 'object' && '__v_isRef' in obj) {
    return stripVueProxy((objAsObj as { value: unknown }).value, seen) as T
  }

  // Built-in serializable types
  if (obj instanceof Date) return new Date(obj.getTime()) as unknown as T
  if (obj instanceof RegExp) return new RegExp(obj.source, obj.flags) as unknown as T
  if (obj instanceof Map) return new Map([...obj.entries()].map(([k, v]) => [k, stripVueProxy(v, seen)])) as unknown as T
  if (obj instanceof Set) return new Set([...obj].map(v => stripVueProxy(v, seen))) as unknown as T
  if (obj instanceof Error) return { name: obj.name, message: obj.message, stack: obj.stack } as unknown as T
  if (ArrayBuffer.isView(obj) || obj instanceof ArrayBuffer) return obj

  const result: Record<string, unknown> = {}
  for (const key of Object.keys(obj)) {
    result[key] = stripVueProxy(objAsObj[key], seen)
  }
  return result as T
}

/**
 * Wrap a specific API method to auto-strip Vue Proxy from arguments.
 * Use this at the call site instead of monkey-patching all methods at startup.
 * Example: `const dbConnect = wrapIpc(getTauriAPI().dbConnect)`
 */
export function wrapIpc<T extends (...args: unknown[]) => unknown>(fn: T): T {
  return ((...args: unknown[]) => fn(...args.map(a => stripVueProxy(a)))) as T
}
