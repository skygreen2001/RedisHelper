// 检测是否在 Tauri 环境
export function isTauriEnv(): boolean {
  try {
    const win = window as any
    // Tauri 2.0 使用 navigator.userAgent 检测
    const ua = win.navigator?.userAgent || ''
    return ua.includes('Tauri') || 
           !!(win.__TAURI__ || win.__TAURI_INTERNALS__ || win.__TAURI_IPC__) ||
           !!(win.tauri)
  } catch {
    return false
  }
}

// 安全调用 invoke
// - Tauri 环境 → 调用 Rust 后端
// - 浏览器环境 → 调用浏览器适配层（WebSocket 代理 + localStorage）
export async function safeInvoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  const tauri = isTauriEnv()
  if (tauri) {
    console.log(`[safeInvoke] → TAURI IPC: ${cmd}`, args ? Object.keys(args) : '')
    const { invoke } = await import('@tauri-apps/api/core')
    try {
      return await invoke<T>(cmd, args)
    } catch (error: any) {
      // Tauri 2.x 返回的错误可能是一个对象或字符串
      console.error(`[safeInvoke] TAURI IPC 错误 (${cmd}):`, error)
      
      // 提取错误消息
      let errorMsg: string
      if (typeof error === 'string') {
        errorMsg = error
      } else if (error instanceof Error) {
        errorMsg = error.message
      } else if (error && typeof error === 'object') {
        // 尝试获取 Tauri 错误的常见属性
        errorMsg = error.message || error.toString()
      } else {
        errorMsg = String(error)
      }
      
      throw new Error(errorMsg)
    }
  }
  // 浏览器环境：使用适配层
  console.log(`[safeInvoke] → BROWSER adapter: ${cmd}`)
  const { browserExecute } = await import('../adapters/browser-adapter')
  return browserExecute(cmd, args) as Promise<T>
}
