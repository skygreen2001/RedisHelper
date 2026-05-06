import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

// 安全的 invoke 函数，在浏览器环境中返回 mock 结果
async function safeInvoke<T>(command: string, args?: any): Promise<T> {
  try {
    if (typeof window === 'undefined') {
      throw new Error('Not in browser')
    }
    const win = window as any
    if (!(win.__TAURI__ || win.__TAURI_IPC__)) {
      throw new Error('Not in Tauri environment')
    }
    return await invoke<T>(command, args)
  } catch (error) {
    console.warn(`Tauri command '${command}' failed in non-Tauri environment`, error)
    // 根据不同命令返回合适的 mock 结果
    switch (command) {
      case 'get_trash_items':
        return [] as unknown as T
      case 'move_to_trash':
      case 'restore_from_trash':
      case 'permanent_delete_trash':
      case 'clear_expired_trash':
        return false as unknown as T
      case 'batch_move_to_trash':
      case 'batch_restore_from_trash':
        return 0 as unknown as T
      default:
        throw error
    }
  }
}

interface MoveToTrashRequest {
  host: string
  port: number
  password?: string
  db: number
  key: string
}

interface BatchMoveToTrashRequest {
  host: string
  port: number
  password?: string
  db: number
  keys: string[]
}

interface TrashItemResponse {
  id: string
  key: string
  value: string
  key_type: string
  host: string
  port: number
  db: number
  deleted_at: string
  expires_at: string
  is_expired: boolean
}

export const trashStore = defineStore('trash', {
  state: () => ({
    trashItems: [] as TrashItemResponse[],
    isLoading: false,
  }),

  actions: {
    async moveToTrash(params: MoveToTrashRequest): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('move_to_trash', { req: params })
      } catch (error) {
        console.error('移入废键箱失败:', error)
        throw error
      }
    },

    async batchMoveToTrash(params: BatchMoveToTrashRequest): Promise<number> {
      try {
        return await safeInvoke<number>('batch_move_to_trash', { req: params })
      } catch (error) {
        console.error('批量移入废键箱失败:', error)
        throw error
      }
    },

    async getTrashItems(host: string, port: number): Promise<TrashItemResponse[]> {
      try {
        this.isLoading = true
        const items = await safeInvoke<TrashItemResponse[]>('get_trash_items', { host, port })
        this.trashItems = items
        return items
      } catch (error) {
        console.error('获取废键箱列表失败:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async restoreFromTrash(trashId: string): Promise<boolean> {
      try {
        return await safeInvoke<boolean>('restore_from_trash', { trashId })
      } catch (error) {
        console.error('恢复键失败:', error)
        throw error
      }
    },

    async batchRestoreFromTrash(trashIds: string[]): Promise<number> {
      try {
        return await safeInvoke<number>('batch_restore_from_trash', { trashIds })
      } catch (error) {
        console.error('批量恢复键失败:', error)
        throw error
      }
    },

    async permanentDelete(trashIds: string[]): Promise<number> {
      try {
        return await safeInvoke<number>('permanent_delete_trash', { trashIds })
      } catch (error) {
        console.error('永久删除失败:', error)
        throw error
      }
    },

    async clearExpired(): Promise<number> {
      try {
        return await safeInvoke<number>('clear_expired_trash')
      } catch (error) {
        console.error('清理过期项失败:', error)
        throw error
      }
    },
  },
})
