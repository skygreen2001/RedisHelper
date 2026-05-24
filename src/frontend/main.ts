import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
import App from './App.vue'

// ========== 全局调试日志控制 ==========
const CONFIG_KEY = 'redis-helper-config'

function getDebugEnabled(): boolean {
  try {
    const data = localStorage.getItem(CONFIG_KEY)
    if (data) {
      const config = JSON.parse(data)
      return config.debugLogEnabled ?? false
    }
  } catch (e) {
    console.error('读取调试配置失败:', e)
  }
  return false
}

// 保存原始 console 方法
const originalLog = console.log
const originalError = console.error
const originalWarn = console.warn
const originalInfo = console.info

// 创建受控制的 console 方法
function controlledLog(...args: any[]): void {
  const enabled = getDebugEnabled()
  if (enabled) {
    originalLog('[debug]', ...args)
  }
}

function controlledError(...args: any[]): void {
  if (getDebugEnabled()) {
    originalError('[debug]', ...args)
  }
}

function controlledWarn(...args: any[]): void {
  if (getDebugEnabled()) {
    originalWarn('[debug]', ...args)
  }
}

function controlledInfo(...args: any[]): void {
  if (getDebugEnabled()) {
    originalInfo('[debug]', ...args)
  }
}

// 重写 console 对象
console.log = controlledLog
console.error = controlledError
console.warn = controlledWarn
console.info = controlledInfo

// ========== 应用初始化 ==========
const app = createApp(App)
app.use(createPinia())
app.use(ElementPlus, { locale: zhCn })
app.mount('#app')