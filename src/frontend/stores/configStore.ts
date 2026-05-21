import { defineStore } from 'pinia'
import { safeInvoke } from '../utils/tauri'

export const configStore = defineStore('config', {
  state: () => ({
    debugLogEnabled: false
  }),

  actions: {
    async loadDebugConfig() {
      try {
        const enabled = await safeInvoke<boolean>('get_debug_log_enabled')
        if (enabled !== null) {
          this.debugLogEnabled = enabled
        }
      } catch (error) {
        console.error('加载调试配置失败:', error)
      }
    },

    async setDebugLogEnabled(enabled: boolean) {
      try {
        const result = await safeInvoke<boolean>('set_debug_log_enabled', { enabled })
        if (result !== null) {
          this.debugLogEnabled = result
        }
      } catch (error) {
        console.error('设置调试配置失败:', error)
      }
    }
  }
})
