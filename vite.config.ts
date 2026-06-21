import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig(({ mode }) => {
  const isTauri = process.env.TAURI === 'true'
  return {
    plugins: [vue()],
    server: isTauri ? {} : {  // Tauri 模式不配置代理
      proxy: {
        '/ws': {
          target: 'ws://localhost:10000',
          ws: true,
          changeOrigin: true
        }
      }
    },
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
          }
        }
      }
    }
  }
})
