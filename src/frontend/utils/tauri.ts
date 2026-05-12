// 检测是否在 Tauri 环境
export function isTauriEnv(): boolean {
  try {
    const win = window as any
    return !!(win.__TAURI__ || win.__TAURI_INTERNALS__ || win.__TAURI_IPC__)
  } catch {
    return false
  }
}

// 安全调用 invoke
// - Tauri 环境 → 调用 Rust 后端
// - 浏览器环境 → 调用浏览器适配层（WebSocket 代理 + localStorage）
export async function safeInvoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  if (isTauriEnv()) {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke<T>(cmd, args)
  }
  // 浏览器环境：使用适配层
  const { browserExecute } = await import('../adapters/browser-adapter')
  return browserExecute(cmd, args) as Promise<T>
}
