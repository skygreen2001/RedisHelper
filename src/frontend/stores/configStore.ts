import { defineStore } from 'pinia'
import { safeInvoke, isTauriEnv } from '../utils/tauri'

const CONFIG_KEY = 'redis-helper-config'

function updateLocalStorage(enabled: boolean) {
  try {
    const config = JSON.parse(localStorage.getItem(CONFIG_KEY) || '{}')
    config.debugLogEnabled = enabled
    localStorage.setItem(CONFIG_KEY, JSON.stringify(config))
    console.log('[configStore] 已更新 localStorage 中的调试配置:', enabled)
  } catch (e) {
    console.error('[configStore] 更新 localStorage 失败:', e)
  }
}

function loadFromLocalStorage(): boolean {
  try {
    const config = JSON.parse(localStorage.getItem(CONFIG_KEY) || '{}')
    return config.debugLogEnabled ?? false
  } catch (e) {
    console.error('[configStore] 从 localStorage 加载失败:', e)
    return false
  }
}

export const configStore = defineStore('config', {
  state: () => ({
    debugLogEnabled: loadFromLocalStorage()
  }),

  actions: {
    async loadDebugConfig() {
      try {
        const isTauri = isTauriEnv()
        console.log(`[configStore] loadDebugConfig - isTauriEnv: ${isTauri}`)
        const enabled = await safeInvoke<boolean>('get_debug_log_enabled')
        console.log(`[configStore] loadDebugConfig result: ${enabled}`)
        if (enabled !== null) {
          this.debugLogEnabled = enabled
          // 仅在状态确实改变时才更新 localStorage
          updateLocalStorage(enabled)
        }
      } catch (error) {
        console.error('加载调试配置失败:', error)
      }
    },

    async setDebugLogEnabled(enabled: boolean) {
      try {
        const isTauri = isTauriEnv()
        console.log(`[configStore] setDebugLogEnabled - isTauriEnv: ${isTauri}, enabled: ${enabled}`)
        
        // 先立即更新 localStorage 和本地状态，确保响应快速
        this.debugLogEnabled = enabled
        updateLocalStorage(enabled)
        
        const result = await safeInvoke<boolean>('set_debug_log_enabled', { enabled })
        console.log(`[configStore] setDebugLogEnabled result: ${result}`)
        if (result !== null) {
          this.debugLogEnabled = result
          updateLocalStorage(result)
        }
      } catch (error) {
        console.error('设置调试配置失败:', error)
      }
    }
  }
})
