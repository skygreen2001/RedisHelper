import { defineStore } from 'pinia'
import { safeInvoke } from '../utils/tauri'

interface Server {
  id: string
  name: string
  host: string
  port: number
  password?: string
  db: number
  readonly: boolean
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
        const servers = await safeInvoke<Server[]>('get_servers')
        this.servers = servers
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
      readonly?: boolean
    }): Promise<Server[]> {
      try {
        this.servers = await safeInvoke<Server[]>('add_server', {
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
      readonly?: boolean
    }): Promise<Server[]> {
      try {
        this.servers = await safeInvoke<Server[]>('edit_server', {
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
        this.servers = await safeInvoke<Server[]>('delete_server', {
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
        return await safeInvoke<TestConnectionResponse>('test_connection', {
          req: params
        })
      } catch (error) {
        console.error('测试连接失败:', error)
        throw error
      }
    },
    
    async saveServerOrder(servers: Server[]): Promise<Server[]> {
      try {
        this.servers = await safeInvoke<Server[]>('save_server_order', {
          servers
        })
        return this.servers
      } catch (error) {
        console.error('保存服务器顺序失败:', error)
        throw error
      }
    }
  }
})
