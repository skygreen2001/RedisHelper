/**
 * auditStore - 审计日志 Pinia Store
 * 负责管理 Redis 操作审计日志
 */
import { defineStore } from 'pinia'
import { safeInvoke } from '../utils/tauri'

export interface AuditEntry {
  id: string
  timestamp: number
  serverId: string
  serverName: string
  db: number
  clientIp: string
  command: string
  args: string[]
  costMs: number
  success: boolean
  errorMessage?: string
}

export interface CommandStats {
  command: string
  count: number
  totalCostMs: number
  avgCostMs: number
  successCount: number
  errorCount: number
  successRate: number
}

export const useAuditStore = defineStore('audit', {
  state: () => ({
    logs: [] as AuditEntry[],
    stats: [] as CommandStats[],
    isLoading: false,
    loadingError: null as string | null,
    
    // 筛选条件
    filters: {
      serverId: null as string | null,
      serverName: null as string | null,
      startTime: null as number | null,
      endTime: null as number | null,
      command: null as string | null,
    },
    
    // 分页
    pagination: {
      limit: 50,
      offset: 0,
      total: 0,
    },
    
    // 选中的服务器信息（用于连接）
    currentServer: {
      host: '',
      port: 6379,
      password: undefined as string | undefined,
    },
  }),

  getters: {
    filteredLogs: (state): AuditEntry[] => {
      return state.logs
    },
    
    totalCommands: (state): number => {
      return state.stats.reduce((sum, s) => sum + s.count, 0)
    },
    
    avgResponseTime: (state): string => {
      const total = state.stats.reduce((sum, s) => sum + s.count * s.avgCostMs, 0)
      const totalCommands = state.stats.reduce((sum, s) => sum + s.count, 0)
      return totalCommands > 0 
        ? (total / totalCommands).toFixed(2) 
        : '0'
    },
    
    successRate: (state): string => {
      const totalSuccess = state.stats.reduce((sum, s) => sum + s.successCount, 0)
      const totalCommands = state.stats.reduce((sum, s) => sum + s.count, 0)
      return totalCommands > 0 
        ? ((totalSuccess / totalCommands) * 100).toFixed(2) 
        : '0'
    },
    
    slowestCommand: (state): string => {
      if (state.stats.length === 0) return '-'
      const max = state.stats.reduce((max, s) => 
        s.avgCostMs > max.avgCostMs ? s : max
      )
      return `${max.command} (${max.avgCostMs.toFixed(2)}ms)`
    },
  },

  actions: {
    /** 设置当前服务器信息 */
    setCurrentServer(host: string, port: number, password?: string) {
      this.currentServer = { host, port, password }
    },

    /** 设置筛选条件 */
    setFilter(key: keyof typeof this.filters, value: any) {
      (this.filters[key] as any) = value
      this.pagination.offset = 0
    },

    /** 获取审计日志 */
    async fetchLogs() {
      this.isLoading = true
      this.loadingError = null
      
      try {
        const params: Record<string, any> = {
          host: this.currentServer.host,
          port: this.currentServer.port,
          password: this.currentServer.password,
          server_id: this.filters.serverId,
          server_name: this.filters.serverName,
          start_time: this.filters.startTime,
          end_time: this.filters.endTime,
          command: this.filters.command,
          limit: this.pagination.limit,
          offset: this.pagination.offset,
        }
        
        console.log('[auditStore] fetching logs with params:', params)
        
        const logs = await safeInvoke<AuditEntry[]>('audit_get_logs', { req: params })
        
        if (Array.isArray(logs)) {
          this.logs = logs
          this.pagination.total = logs.length
        } else {
          this.logs = []
          this.pagination.total = 0
        }
        
        console.log('[auditStore] fetched', this.logs.length, 'logs')
        
      } catch (error) {
        console.error('[auditStore] fetchLogs error:', error)
        this.loadingError = error instanceof Error ? error.message : String(error)
      } finally {
        this.isLoading = false
      }
    },

    /** 获取命令统计 */
    async fetchStats() {
      try {
        const params = {
          host: this.currentServer.host,
          port: this.currentServer.port,
          password: this.currentServer.password,
          server_id: this.filters.serverId,
        }
        
        console.log('[auditStore] fetching stats with params:', params)
        
        const stats = await safeInvoke<CommandStats[]>('audit_get_stats', { req: params })
        
        console.log('[auditStore] raw stats from backend:', JSON.stringify(stats))
        
        if (Array.isArray(stats)) {
          this.stats = stats
          // 计算期望值
          const totalSuccess = stats.reduce((sum, s) => sum + s.successCount, 0)
          const totalCommands = stats.reduce((sum, s) => sum + s.count, 0)
          console.log('[auditStore] expected success rate:', totalCommands > 0 ? ((totalSuccess / totalCommands) * 100).toFixed(2) : '0')
        } else {
          this.stats = []
        }
        
        console.log('[auditStore] fetched', this.stats.length, 'stats')
        
      } catch (error) {
        console.error('[auditStore] fetchStats error:', error)
        this.stats = []
      }
    },

    /** 清空审计日志 */
    async clearLogs() {
      try {
        console.log('[auditStore] clearLogs: before clearing')
        await safeInvoke<boolean>('audit_clear', {
          host: this.currentServer.host,
          port: this.currentServer.port,
          password: this.currentServer.password,
        })
        this.logs = []
        this.pagination.total = 0
        this.stats = []  // 同时清空统计数据
        console.log('[auditStore] clearLogs: after clearing - logs:', this.logs.length, 'stats:', this.stats.length)
      } catch (error) {
        console.error('[auditStore] clearLogs error:', error)
        throw error
      }
    },

    /** 生成测试数据 */
    async generateTestData(count: number = 100) {
      try {
        const result = await safeInvoke<number>('audit_generate_test_data', {
          host: this.currentServer.host,
          port: this.currentServer.port,
          password: this.currentServer.password,
          count,
        })
        console.log('[auditStore] generated', result, 'test entries')
        return result
      } catch (error) {
        console.error('[auditStore] generateTestData error:', error)
        throw error
      }
    },

    /** 分页 - 上一页 */
    prevPage() {
      if (this.pagination.offset > 0) {
        this.pagination.offset = Math.max(0, this.pagination.offset - this.pagination.limit)
        this.fetchLogs()
      }
    },

    /** 分页 - 下一页 */
    nextPage() {
      if (this.pagination.offset + this.pagination.limit < this.pagination.total) {
        this.pagination.offset += this.pagination.limit
        this.fetchLogs()
      }
    },

    /** 跳转到指定页 */
    goToPage(page: number) {
      this.pagination.offset = (page - 1) * this.pagination.limit
      this.fetchLogs()
    },
  },
})
