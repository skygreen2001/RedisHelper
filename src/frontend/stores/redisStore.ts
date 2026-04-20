import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

interface ConnectRequest {
  host: string
  port: number
  password?: string
  db: number
}

interface KeyRequest {
  host: string
  port: number
  password?: string
  db: number
  key: string
}

interface KeyValueRequest {
  host: string
  port: number
  password?: string
  db: number
  key: string
  value: string
  key_type: string
}

interface SearchRequest {
  host: string
  port: number
  password?: string
  db: number
  pattern: string
}

interface ExportRequest {
  host: string
  port: number
  password?: string
  db: number
  file_path: string
}

interface KeyValueResponse {
  key: string
  value: string
  key_type: string
}

export const redisStore = defineStore('redis', {
  state: () => ({
    isConnected: false
  }),
  
  actions: {
    async connect(params: ConnectRequest): Promise<boolean> {
      try {
        const result = await invoke<boolean>('connect', { req: params })
        this.isConnected = result
        return result
      } catch (error) {
        console.error('连接Redis失败:', error)
        this.isConnected = false
        throw error
      }
    },
    
    async getDatabases(params: ConnectRequest): Promise<Array<[number, number]>> {
      try {
        return await invoke('get_databases', { req: params })
      } catch (error) {
        console.error('获取数据库失败:', error)
        throw error
      }
    },
    
    async getKeys(params: ConnectRequest): Promise<string[]> {
      try {
        return await invoke('get_keys', { req: params })
      } catch (error) {
        console.error('获取键失败:', error)
        throw error
      }
    },
    
    async getKeyValue(params: KeyRequest): Promise<KeyValueResponse> {
      try {
        return await invoke('get_key_value', { req: params })
      } catch (error) {
        console.error('获取键值失败:', error)
        throw error
      }
    },
    
    async setKeyValue(params: KeyValueRequest): Promise<boolean> {
      try {
        return await invoke('set_key_value', { req: params })
      } catch (error) {
        console.error('设置键值失败:', error)
        throw error
      }
    },
    
    async deleteKey(params: KeyRequest): Promise<boolean> {
      try {
        return await invoke('delete_key', { req: params })
      } catch (error) {
        console.error('删除键失败:', error)
        throw error
      }
    },
    
    async searchKeys(params: SearchRequest): Promise<string[]> {
      try {
        return await invoke('search_keys', { req: params })
      } catch (error) {
        console.error('搜索键失败:', error)
        throw error
      }
    },
    
    async exportData(params: ExportRequest): Promise<boolean> {
      try {
        return await invoke('export_data', { req: params })
      } catch (error) {
        console.error('导出数据失败:', error)
        throw error
      }
    },
    
    async importData(params: ExportRequest): Promise<boolean> {
      try {
        return await invoke('import_data', { req: params })
      } catch (error) {
        console.error('导入数据失败:', error)
        throw error
      }
    }
  }
})