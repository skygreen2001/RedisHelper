import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

// 简化的 Tauri 环境检测
function isTauri(): boolean {
  try {
    if (typeof window === 'undefined') return false
    const win = window as any
    // 只要有 Tauri 相关对象就认为是 Tauri 环境
    return !!(win.__TAURI__ || win.__TAURI_IPC__)
  } catch (e) {
    console.warn('Tauri检测失败:', e)
    return false
  }
}

interface Server {
  id: string
  name: string
  host: string
  port: number
  password?: string
  db: number
  created: string
  updated: string
}

interface TestConnectionRequest {
  host: string
  port: number
  password?: string
}

interface TestConnectionResponse {
  success: boolean
  message: string
}

export const serverStore = defineStore('server', {
  state: () => ({
    servers: [] as Server[]
  }),
  
  actions: {
    async loadServers(): Promise<Server[]> {
      try {
        const servers = await invoke<Server[]>('get_servers')
        this.servers = servers
        
        // 检查是否已存在本地服务器配置
        const hasLocalServer = this.servers.some((server) => 
          server.host === '127.0.0.1' && server.port === 6379
        )
        
        // 如果不存在，添加本地服务器配置
        if (!hasLocalServer) {
          const localServer = {
            id: 'local',
            name: '本地Redis服务器',
            host: '127.0.0.1',
            port: 6379,
            password: '',
            db: 0,
            created: new Date().toISOString(),
            updated: new Date().toISOString()
          }
          await this.addServer(localServer)
        }
        return this.servers
      } catch (error) {
        console.error('加载服务器失败:', error)
        throw error
      }
    },
    
    async addServer(server: {
      id: string
      name: string
      host: string
      port: number
      password?: string
      db: number
    }): Promise<Server[]> {
      try {
        this.servers = await invoke<Server[]>('add_server', {
          config: server
        })
        return this.servers
      } catch (error) {
        console.error('添加服务器失败:', error)
        throw error
      }
    },
    
    async editServer(server: {
      id: string
      name: string
      host: string
      port: number
      password?: string
      db: number
    }): Promise<Server[]> {
      try {
        this.servers = await invoke<Server[]>('edit_server', {
          config: server
        })
        return this.servers
      } catch (error) {
        console.error('编辑服务器失败:', error)
        throw error
      }
    },
    
    async deleteServer(id: string): Promise<Server[]> {
      try {
        this.servers = await invoke<Server[]>('delete_server', {
          id
        })
        return this.servers
      } catch (error) {
        console.error('删除服务器失败:', error)
        throw error
      }
    },
    
    async testConnection(params: TestConnectionRequest): Promise<TestConnectionResponse> {
      try {
        return await invoke<TestConnectionResponse>('test_connection', {
          req: params
        })
      } catch (error) {
        console.error('测试连接失败:', error)
        throw error
      }
    }
  }
})