export const WEBSOCKET_ENABLED = true

export function isTauriEnv(): boolean {
  try {
    const win = window as any
    const ua = win.navigator?.userAgent || ''
    return ua.includes('Tauri') || 
           !!(win.__TAURI__ || win.__TAURI_INTERNALS__ || win.__TAURI_IPC__) ||
           !!(win.tauri)
  } catch {
    return false
  }
}

export async function safeInvoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  const tauri = isTauriEnv()
  console.log(`[safeInvoke] 环境检测: ${tauri ? 'TAURI' : 'BROWSER'}`)
  console.log(`[safeInvoke] UserAgent: ${window.navigator?.userAgent}`)
  console.log(`[safeInvoke] __TAURI__: ${!!(window as any).__TAURI__}`)
  console.log(`[safeInvoke] __TAURI_INTERNALS__: ${!!(window as any).__TAURI_INTERNALS__}`)
  
  if (tauri) {
    console.log(`[safeInvoke] → TAURI IPC: ${cmd}`, args ? Object.keys(args) : '')
    const { invoke } = await import('@tauri-apps/api/core')
    try {
      return await invoke<T>(cmd, args)
    } catch (error: any) {
      console.error(`[safeInvoke] TAURI IPC 错误 (${cmd}):`, error)
      
      let errorMsg: string
      if (typeof error === 'string') {
        errorMsg = error
      } else if (error instanceof Error) {
        errorMsg = error.message
      } else if (error && typeof error === 'object') {
        errorMsg = error.message || error.toString()
      } else {
        errorMsg = String(error)
      }
      
      throw new Error(errorMsg)
    }
  }
  
  if (!WEBSOCKET_ENABLED) {
    throw new Error(`WebSocket 连接功能已被禁用，请使用 Tauri 环境运行。命令: ${cmd}`)
  }
  
  console.log(`[safeInvoke] → BROWSER adapter: ${cmd}`)
  const { browserExecute } = await import('../adapters/browser-adapter')
  return browserExecute(cmd, args) as Promise<T>
}
