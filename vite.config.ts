import { defineConfig, Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import { spawn, ChildProcess } from 'child_process'
import path from 'path'

// Vite 插件：开发模式下自动启动 WebSocket 代理
function wsProxyPlugin(): Plugin {
  let proxyProcess: ChildProcess | null = null

  return {
    name: 'ws-proxy',
    configureServer(server) {
      // 启动代理
      const proxyPath = path.resolve(__dirname, 'server/ws-proxy.js')
      proxyProcess = spawn('node', [proxyPath], {
        stdio: ['pipe', 'pipe', 'pipe'],
        env: { ...process.env, WS_PROXY_PORT: '8765' },
      })

      proxyProcess.stdout?.on('data', (data: Buffer) => {
        console.log(`[ws-proxy] ${data.toString().trim()}`)
      })

      proxyProcess.stderr?.on('data', (data: Buffer) => {
        console.error(`[ws-proxy] ${data.toString().trim()}`)
      })

      proxyProcess.on('error', (err) => {
        console.error('[ws-proxy] 启动失败:', err.message)
      })

      proxyProcess.on('exit', (code) => {
        console.log(`[ws-proxy] 已退出 (code: ${code})`)
        proxyProcess = null
      })

      // Vite 关闭时终止代理
      server.httpServer?.on('close', () => {
        if (proxyProcess) {
          proxyProcess.kill()
          proxyProcess = null
        }
      })
    },
  }
}

export default defineConfig({
  plugins: [vue(), wsProxyPlugin()],
  build: {
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          if (id.includes('node_modules')) {
            if (id.includes('vue') || id.includes('pinia')) {
              return 'vue-vendor'
            }
            if (id.includes('element-plus')) {
              return 'element-plus'
            }
            if (id.includes('@tauri-apps')) {
              return 'tauri-api'
            }
          }
        },
      },
    },
  },
})
