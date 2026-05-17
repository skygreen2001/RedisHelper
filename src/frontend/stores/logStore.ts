/**
 * logStore - 日志 Pinia Store
 * 负责管理各连接的 SLOWLOG 历史日志条目（内存，不持久化）
 * 以及与 ws-proxy 的通信
 */
import { defineStore } from 'pinia'
import { safeInvoke } from '../utils/tauri'

export interface LogEntry {
  id: string
  time: string
  db: string | number
  client: string
  cmd: string
  args: string[]
  costMs: number | null
}

interface MonitorSession {
  serverId: string
  serverName: string
  host: string
  port: number
  password?: string
  logs: LogEntry[]
}


export const useLogStore = defineStore('log', {
  state: () => ({
    sessions: {} as Record<string, MonitorSession>,
    activeServerId: '' as string,
    /** 最后一次加载的错误信息：null=无错误, 'empty'=数据为空, string=错误消息 */
    loadingError: null as string | null,
  }),

  getters: {
    sessionList: (state): MonitorSession[] => Object.values(state.sessions),

    activeLogs: (state): LogEntry[] => {
      return state.sessions[state.activeServerId]?.logs ?? []
    },

    activeSession: (state): MonitorSession | null => {
      return state.sessions[state.activeServerId] ?? null
    },
  },

  actions: {
    /** 注册一个连接（打开日志对话框时调用） */
    registerServer(serverId: string, serverName: string, host: string, port: number, password?: string) {
      if (!this.sessions[serverId]) {
        this.sessions[serverId] = {
          serverId,
          serverName,
          host,
          port,
          password,
          logs: [],
        }
      }
      if (!this.activeServerId) {
        this.activeServerId = serverId
      }
    },

    /** 加载 SLOWLOG 历史命令记录（插入到日志列表头部） */
    async loadSlowLog(serverId: string) {
      const session = this.sessions[serverId]
      if (!session) {
        console.warn('[logStore] loadSlowLog 失败: session 不存在', { serverId, keys: Object.keys(this.sessions) })
        return
      }

      // 暴露加载状态和错误信息供 UI 展示
      this.loadingError = null

      try {
        console.log('[logStore] 开始加载 SLOWLOG 历史...', {
          serverId,
          host: session.host,
          port: session.port,
          hasPassword: !!session.password,
        })
        const rawList = await safeInvoke<any[]>('slowlog_get', {
          req: {
            host: session.host,
            port: session.port,
            password: session.password,
          },
        })

        console.log('[logStore] SLOWLOG 返回:', Array.isArray(rawList) ? `${rawList.length} 条` : typeof rawList, rawList)

        if (!Array.isArray(rawList) || rawList.length === 0) {
          console.log('[logStore] SLOWLOG 为空，无历史数据')
          this.loadingError = 'empty'
          return
        }

        // SLOWLOG 返回的是从新到旧排列，转换为 LogEntry 并标记来源
        const historyLogs: LogEntry[] = rawList
          .map((item: any, index: number) => {
            const timeStr = new Date(item.time * 1000).toLocaleTimeString()
            return {
              id: `slow_${item.id ?? index}`,
              time: timeStr,
              db: '-',   // SLOWLOG 协议不含 db 字段，所有 Redis 版本均如此
              client: item.client || '',
              cmd: (item.cmd || '').toUpperCase(),
              args: item.args ?? [],
              costMs: typeof item.costMs === 'number' ? Math.round(item.costMs * 100) / 100 : null,
            }
          })

        // 脱敏 AUTH
        for (const log of historyLogs) {
          if (log.cmd === 'AUTH') log.args = ['***']
        }

        // 去重：如果已有相同 ID 的慢日志条目则跳过（避免重复加载）
        const existingIds = new Set(session.logs.map(l => l.id))
        const newLogs = historyLogs.filter(l => !existingIds.has(l.id))

        // 插入到现有日志的**前面**（历史在前，实时在后）
        session.logs.unshift(...newLogs)

        // 超过 5000 条时，丢弃尾部最早的条目
        if (session.logs.length > 5000) {
          session.logs.splice(0, session.logs.length - 5000)
        }
      } catch (err: any) {
        console.error('[logStore] loadSlowLog 失败:', err)
        this.loadingError = err?.message || String(err)
      }
    },

    /** 清空指定连接的日志 */
    clearLogs(serverId: string) {
      const session = this.sessions[serverId]
      if (session) session.logs = []
    },

    /** 切换活动 Tab */
    setActiveServer(serverId: string) {
      this.activeServerId = serverId
    },

    /** 重置（关闭对话框时调用，但保留 session 注册状态） */
    reset() {
      this.activeServerId = ''
    },
  },
})
