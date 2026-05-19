import { defineStore } from 'pinia'
import { safeInvoke } from '../utils/tauri'

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

export interface KeyMemoryItem {
  key: string
  size: number
  size_human: string
  key_type: string
}

export interface KeyTypeStat {
  key_type: string
  count: number
  memory_bytes: number
  memory_percent: number
}

export interface MemoryInfoResponse {
  used_memory: number
  used_memory_human: string
  used_memory_peak: number
  used_memory_peak_human: string
  mem_fragmentation_ratio: number
  maxmemory: number
  keys_count: number
  expired_keys_ratio: number
  large_keys_count: number
  key_memory_list: KeyMemoryItem[]
  key_type_stats: KeyTypeStat[]
}

export const redisStore = defineStore('redis', {
  state: () => ({
    isConnected: false
  }),
  
  actions: {
    async connect(params: ConnectRequest): Promise<boolean> {
      try {
        const result = await safeInvoke<boolean>('connect', { req: params })
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
        return await safeInvoke<Array<[number, number]>>('get_databases', { req: params })
      } catch (error) {
        console.error('获取数据库失败:', error)
        throw error
      }
    },
    
    async getKeys(params: ConnectRequest): Promise<string[]> {
      try {
        return await safeInvoke<string[]>('get_keys', { req: params })
      } catch (error) {
        console.error('获取键失败:', error)
        throw error
      }
    },
    
    async getKeyValue(params: KeyRequest): Promise<KeyValueResponse> {
      try {
        return await safeInvoke<KeyValueResponse>('get_key_value', { req: params })
      } catch (error) {
        console.error('获取键值失败:', error)
        throw error
      }
    },
    
    async setKeyValue(params: KeyValueRequest): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('set_key_value', { req: params })
      } catch (error) {
        console.error('设置键值失败:', error)
        throw error
      }
    },
    
    async deleteKey(params: KeyRequest): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('delete_key', { req: params })
      } catch (error) {
        console.error('删除键失败:', error)
        throw error
      }
    },
    
    async searchKeys(params: SearchRequest): Promise<string[]> {
      try {
        return await safeInvoke<string[]>('search_keys', { req: params })
      } catch (error) {
        console.error('搜索键失败:', error)
        throw error
      }
    },
    
    async exportData(params: ExportRequest): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('export_data', { req: params })
      } catch (error) {
        console.error('导出数据失败:', error)
        throw error
      }
    },
    
    async importData(params: ExportRequest): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('import_data', { req: params })
      } catch (error) {
        console.error('导入数据失败:', error)
        throw error
      }
    },
    
    async createDatabase(params: ConnectRequest): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('create_database', { req: params })
      } catch (error) {
        console.error('创建数据库失败:', error)
        throw error
      }
    },
    
    async deleteDatabase(params: ConnectRequest): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('delete_database', { req: params })
      } catch (error) {
        console.error('删除数据库失败:', error)
        throw error
      }
    },
    
    async flushDatabase(params: ConnectRequest): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('flush_database', { req: params })
      } catch (error) {
        console.error('清空数据库失败:', error)
        throw error
      }
    },

    async generateTestData(params: ConnectRequest, count: number): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('generate_test_data', { req: params, count })
      } catch (error) {
        console.error('生成测试数据失败:', error)
        throw error
      }
    },

    async getMemoryInfo(params: ConnectRequest): Promise<MemoryInfoResponse> {
      try {
        return await safeInvoke<MemoryInfoResponse>('get_memory_info', { req: params })
      } catch (error) {
        console.error('获取内存信息失败:', error)
        throw error
      }
    }
  }
})