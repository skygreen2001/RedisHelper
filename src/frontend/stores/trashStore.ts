import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

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
        return await invoke<boolean>('move_to_trash', { req: params })
      } catch (error) {
        console.error('删除废键箱失败:', error)
        throw error
      }
    },

    async batchMoveToTrash(params: BatchMoveToTrashRequest): Promise<number> {
      try {
        return await invoke<number>('batch_move_to_trash', { req: params })
      } catch (error) {
        console.error('批量删除废键箱失败:', error)
        throw error
      }
    },

    async getTrashItems(host: string, port: number): Promise<TrashItemResponse[]> {
      try {
        this.isLoading = true
        const items = await invoke<TrashItemResponse[]>('get_trash_items', { host, port })
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
        return await invoke<boolean>('restore_from_trash', { req: { trash_id: trashId } })
      } catch (error) {
        console.error('恢复键失败:', error)
        throw error
      }
    },

    async batchRestoreFromTrash(trashIds: string[]): Promise<number> {
      try {
        return await invoke<number>('batch_restore_from_trash', { req: { trash_ids: trashIds } })
      } catch (error) {
        console.error('批量恢复键失败:', error)
        throw error
      }
    },

    async permanentDelete(trashIds: string[]): Promise<number> {
      try {
        return await invoke<number>('permanent_delete_trash', { req: { trash_ids: trashIds } })
      } catch (error) {
        console.error('永久删除失败:', error)
        throw error
      }
    },

    async clearExpired(): Promise<number> {
      try {
        return await invoke<number>('clear_expired_trash')
      } catch (error) {
        console.error('清理过期项失败:', error)
        throw error
      }
    },
  },
})
