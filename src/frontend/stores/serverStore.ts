import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

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
    async loadServers() {
      try {
        this.servers = await invoke('get_servers')
      } catch (error) {
        console.error('加载服务器失败:', error)
      }
    },
    
    async addServer(server: {
      id: string
      name: string
      host: string
      port: number
      password?: string
      db: number
    }) {
      try {
        this.servers = await invoke('add_server', {
          config: server
        })
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
    }) {
      try {
        this.servers = await invoke('edit_server', {
          config: server
        })
      } catch (error) {
        console.error('编辑服务器失败:', error)
        throw error
      }
    },
    
    async deleteServer(id: string) {
      try {
        this.servers = await invoke('delete_server', {
          id
        })
      } catch (error) {
        console.error('删除服务器失败:', error)
        throw error
      }
    },
    
    async testConnection(params: TestConnectionRequest): Promise<TestConnectionResponse> {
      try {
        return await invoke('test_connection', {
          req: params
        })
      } catch (error) {
        console.error('测试连接失败:', error)
        throw error
      }
    }
  }
})