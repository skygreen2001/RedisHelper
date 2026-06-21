/**
 * Session - 独立会话类
 * 每个标签/窗口拥有一个 Session 实例，实现会话隔离
 */
import { safeInvoke } from '../utils/tauri'

let sessionCounter = 0

export interface ServerConfig {
  id: string
  name: string
  host: string
  port: number
  password?: string
  db: number
  readonly: boolean
}

export class Session {
  readonly id: string
  title: string

  // 连接状态
  selectedServer: ServerConfig | null = null
  selectedDb: number | null = null
  databases: Array<[number, number]> = []

  // 键列表
  keys: string[] = []
  keysCursor = 0
  keysTotal = 0
  isLoadingMore = false
  isLoadingAll = false

  // 键值详情
  selectedKey = ''
  keyValue = ''
  keyType = ''

  // 搜索
  searchPattern = ''
  sortOrder: 'none' | 'asc' | 'desc' = 'none'
  caseSensitive = false
  searchAll = true

  // 数据库跟踪
  newlyCreatedDbs = new Set<number>()
  visitedDbs = new Set<number>()

  // 废键箱
  isTrashView = false
  trashItems: any[] = []
  trashSelectedIds: string[] = []

  // 多选
  isMultiSelectMode = false
  selectedKeys: string[] = []

  // 消息
  message = ''
  messageType: 'success' | 'error' | 'warning' = 'error'

  // 对话框
  showAddKeyDialog = false
  showEditKeyDialog = false
  showServerConfig = false
  showAddDbDialog = false
  showDeleteDbDialog = false
  showExportDialog = false
  showFlushDialog = false

  // 新建标签页时等待用户选择连接
  isSelectingServer = false

  // 是否必须添加连接才能关闭设置页面
  requireServerConnection = false

  // 表单
  newKeyForm = { key: '', value: '', type: 'string' }
  editKeyForm = { key: '', value: '', type: 'string' }

  // 编辑模式：'overwrite'（覆盖模式） | 'element'（元素编辑模式）
  editMode: 'overwrite' | 'element' = 'overwrite'

  // 导出
  exportFileName = 'redis-export'
  exportFolderPath = ''
  isFolderLoading = false

  // 数据库操作
  selectedDbsForDelete: number[] = []
  newDbNumber = 0

  constructor(title: string = '新标签') {
    this.id = `session_${++sessionCounter}_${Date.now()}`
    this.title = title
  }

  /** 更新标签标题 */
  updateTitle() {
    if (this.selectedServer && this.selectedDb !== null) {
      this.title = `${this.selectedServer.name} - DB${this.selectedDb}`
    } else if (this.selectedServer) {
      this.title = this.selectedServer.name
    } else {
      this.title = '新标签'
    }
  }

  /** 获取连接参数 */
  get connParams() {
    if (!this.selectedServer) return null
    return {
      host: this.selectedServer.host,
      port: this.selectedServer.port,
      password: this.selectedServer.password,
      db: this.selectedDb ?? 0,
    }
  }

  /** 是否只读 */
  get isReadonly(): boolean {
    return this.selectedServer?.readonly ?? false
  }

  /** 设置消息 */
  setMessage(msg: string, type: 'success' | 'error' | 'warning' = 'error') {
    this.message = msg
    this.messageType = type
  }

  /** 清空消息 */
  clearMessage() {
    this.message = ''
  }

  /** 重置键值详情 */
  clearKeyDetail() {
    this.selectedKey = ''
    this.keyValue = ''
    this.keyType = ''
    this.editKeyForm = { key: '', value: '', type: 'string' }
    this.editMode = 'overwrite'
  }

  /** 重置多选 */
  clearSelection() {
    this.selectedKeys = []
    this.isMultiSelectMode = false
  }

  // ========== Redis 操作（通过 safeInvoke） ==========

  async getDatabases() {
    if (!this.connParams) return []
    const result = await safeInvoke<Array<[number, number]>>('get_databases', { req: this.connParams })
    return result || []
  }

  async getKeys() {
    if (!this.connParams) return { keys: [], total: 0 }
    const result = await safeInvoke<{ keys: string[]; total: number }>('get_keys', { req: this.connParams })
    return result || { keys: [], total: 0 }
  }

  async getKeyValue(key: string) {
    if (!this.connParams) return null
    return safeInvoke<{ key: string; value: string; key_type: string }>('get_key_value', {
      req: { ...this.connParams, key }
    })
  }

  async setKeyValue(key: string, value: string, keyType: string) {
    if (!this.connParams) return
    await safeInvoke('set_key_value', {
      req: { ...this.connParams, key, value, key_type: keyType }
    })
  }

  async deleteKey(key: string) {
    if (!this.connParams) return
    await safeInvoke('delete_key', { req: { ...this.connParams, key } })
  }

  async searchKeys(pattern: string) {
    if (!this.connParams) return []
    return safeInvoke<string[]>('search_keys', {
      req: { ...this.connParams, pattern }
    })
  }

  async flushDatabase() {
    if (!this.connParams) return
    await safeInvoke('flush_database', { req: this.connParams })
  }

  async deleteDatabase(db: number) {
    if (!this.connParams) return
    await safeInvoke('delete_database', { req: { ...this.connParams, db } })
  }

  async createDatabase(db: number) {
    if (!this.connParams) return
    await safeInvoke('create_database', { req: { ...this.connParams, db } })
  }

  async generateTestData(count: number) {
    if (!this.connParams) return
    await safeInvoke('generate_test_data', { req: this.connParams, count })
  }

  async exportData(file_path: string) {
    if (!this.connParams) return
    await safeInvoke('export_data', { req: { ...this.connParams, file_path } })
  }

  async importData(file_path: string) {
    if (!this.connParams) return
    await safeInvoke('import_data', { req: { ...this.connParams, file_path } })
  }

  // ========== 废键箱操作 ==========

  async moveToTrash(key: string) {
    if (!this.connParams) return
    await safeInvoke('move_to_trash', { req: { ...this.connParams, key } })
  }

  async batchMoveToTrash(keys: string[]) {
    if (!this.connParams) return 0
    return safeInvoke<number>('batch_move_to_trash', { req: { ...this.connParams, keys } })
  }

  async getTrashItems() {
    if (!this.selectedServer) return []
    return safeInvoke<any[]>('get_trash_items', {
      host: this.selectedServer.host,
      port: this.selectedServer.port,
    })
  }

  async restoreFromTrash(trashId: string) {
    await safeInvoke('restore_from_trash', { req: { trash_id: trashId } })
  }

  async batchRestoreFromTrash(trashIds: string[]) {
    await safeInvoke('batch_restore_from_trash', { req: { trash_ids: trashIds } })
  }

  async permanentDeleteTrash(trashIds: string[]) {
    await safeInvoke('permanent_delete_trash', { req: { trash_ids: trashIds } })
  }
}
