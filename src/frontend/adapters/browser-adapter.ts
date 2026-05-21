/**
 * 浏览器端命令适配层
 * 在非 Tauri 环境下，通过 WebSocket 代理连接 Redis，
 * 使用 localStorage 存储服务器配置和废键箱数据
 */

// ========== WebSocket 代理客户端 ==========

let ws: WebSocket | null = null
let wsReady = false
let wsConnectPromise: Promise<WebSocket> | null = null  // 添加这个变量
const pendingRequests = new Map<string, { resolve: (v: any) => void; reject: (e: any) => void }>()
let requestId = 0

async function ensureWs(): Promise<WebSocket> {
  if (ws && wsReady) return ws
  
  // 如果已经有一个连接正在进行中，复用同一个 Promise
  if (wsConnectPromise) {
    return wsConnectPromise
  }
  
  wsConnectPromise = new Promise((resolve, reject) => {
    const url = `ws://${window.location.hostname}:8765`
    console.log('[browser-adapter] 连接 WebSocket 代理:', url)
    const socket = new WebSocket(url)
    ws = socket
    
    socket.onopen = () => {
      wsReady = true
      wsConnectPromise = null  // 连接成功后清空 Promise
      console.log('[browser-adapter] WebSocket 代理已连接')
      resolve(socket)
    }
    
    socket.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data)
        const pending = pendingRequests.get(msg.id)
        if (pending) {
          pendingRequests.delete(msg.id)
          if (msg.error) {
            pending.reject(new Error(msg.error))
          } else {
            pending.resolve(msg.result)
          }
        }
      } catch (e) {
        console.error('[browser-adapter] 解析消息失败:', e)
      }
    }
    
    socket.onclose = () => {
      wsReady = false
      wsConnectPromise = null  // 连接关闭后清空 Promise
      console.log('[browser-adapter] WebSocket 代理已断开')
    }
    
    socket.onerror = () => {
      wsReady = false
      wsConnectPromise = null  // 连接错误后清空 Promise
      reject(new Error('WebSocket 代理连接失败，请确保已启动: node server/ws-proxy.js'))
    }
    
    // 超时 5 秒
    setTimeout(() => {
      if (!wsReady) {
        wsConnectPromise = null  // 超时后清空 Promise
        reject(new Error('WebSocket 代理连接超时'))
      }
    }, 5000)
  })
  
  return wsConnectPromise
}

async function wsProxyCall(command: string, args: Record<string, any>, extraArgs?: Record<string, any>): Promise<any> {
  const socket = await ensureWs()
  const id = `req_${++requestId}`
  
  return new Promise((resolve, reject) => {
    pendingRequests.set(id, { resolve, reject })
    socket.send(JSON.stringify({ id, command, args, extraArgs: extraArgs || {} }))
    
    // 超时 30 秒
    setTimeout(() => {
      if (pendingRequests.has(id)) {
        pendingRequests.delete(id)
        reject(new Error(`命令 ${command} 超时`))
      }
    }, 30000)
  })
}

// ========== localStorage 存储工具 ==========

const SERVERS_KEY = 'redis-helper-servers'
const TRASH_KEY = 'redis-helper-trash'

function loadFromStorage<T>(key: string): T[] {
  try {
    const data = localStorage.getItem(key)
    return data ? JSON.parse(data) : []
  } catch {
    return []
  }
}

function saveToStorage<T>(key: string, data: T[]): void {
  localStorage.setItem(key, JSON.stringify(data))
}

// ========== 服务器配置适配 ==========

interface ServerConfig {
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

function handleServerCommands(command: string, args: any): any {
  const servers = loadFromStorage<ServerConfig>(SERVERS_KEY)

  switch (command) {
    case 'get_servers':
      return servers

    case 'add_server': {
      const config = args.config
      const newServer: ServerConfig = {
        id: config.id,
        name: config.name,
        host: config.host,
        port: config.port,
        password: config.password,
        db: config.db || 0,
        readonly: config.readonly || false,
        created: new Date().toISOString(),
        updated: new Date().toISOString(),
      }
      servers.push(newServer)
      saveToStorage(SERVERS_KEY, servers)
      return servers
    }

    case 'edit_server': {
      const config = args.config
      const idx = servers.findIndex(s => s.id === config.id)
      if (idx >= 0) {
        servers[idx] = {
          ...servers[idx],
          name: config.name,
          host: config.host,
          port: config.port,
          password: config.password,
          db: config.db || 0,
          readonly: config.readonly || false,
          updated: new Date().toISOString(),
        }
      }
      saveToStorage(SERVERS_KEY, servers)
      return servers
    }

    case 'delete_server': {
      const idx = servers.findIndex(s => s.id === args.id)
      if (idx >= 0) servers.splice(idx, 1)
      saveToStorage(SERVERS_KEY, servers)
      return servers
    }

    case 'save_server_order': {
      // args.servers 是新的服务器顺序列表
      const newServers = args.servers || []
      // 直接保存新的顺序到 localStorage
      saveToStorage(SERVERS_KEY, newServers)
      return newServers
    }

    default:
      throw new Error(`未知的服务器命令: ${command}`)
  }
}

// ========== 废键箱适配 ==========

interface TrashItem {
  id: string
  server_id: string
  key: string
  value: string
  key_type: string
  host: string
  port: number
  password?: string
  db: number
  deleted_at: string
  expires_at: string
}

async function handleTrashCommands(command: string, args: any): Promise<any> {
  const items = loadFromStorage<TrashItem>(TRASH_KEY)

  switch (command) {
    case 'get_trash_items': {
      const serverId = `${args.host}:${args.port}`
      const now = new Date()
      // 过滤该服务器的未过期项
      return items
        .filter(item => {
          if (item.server_id !== serverId) return false
          const expiresAt = new Date(item.expires_at)
          return expiresAt > now
        })
        .map(item => ({
          id: item.id,
          key: item.key,
          value: item.value,
          key_type: item.key_type,
          host: item.host,
          port: item.port,
          db: item.db,
          deleted_at: item.deleted_at,
          expires_at: item.expires_at,
          is_expired: new Date(item.expires_at) <= now,
        }))
    }

    case 'move_to_trash': {
      const now = new Date()
      const expires = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000)
      
      // 先从 Redis 获取 key 的值和类型
      const keyValue = await wsProxyCall('get_key_value', {
        host: args.host,
        port: args.port,
        password: args.password,
        db: args.db,
        key: args.key
      })
      
      const newItem: TrashItem = {
        id: crypto.randomUUID(),
        server_id: `${args.host}:${args.port}`,
        key: args.key,
        value: keyValue.value,
        key_type: keyValue.key_type,
        host: args.host,
        port: args.port,
        password: args.password,
        db: args.db,
        deleted_at: now.toISOString(),
        expires_at: expires.toISOString(),
      }
      items.push(newItem)
      saveToStorage(TRASH_KEY, items)
      
      // 从 Redis 删除该 key
      await wsProxyCall('delete_key', {
        host: args.host,
        port: args.port,
        password: args.password,
        db: args.db,
        key: args.key
      })
      
      return true
    }

    case 'batch_move_to_trash': {
      const now = new Date()
      const expires = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000)
      let count = 0
      const keysToDelete: string[] = []
      
      for (const key of args.keys) {
        try {
          // 获取每个 key 的值
          const keyValue = await wsProxyCall('get_key_value', {
            host: args.host,
            port: args.port,
            password: args.password,
            db: args.db,
            key
          })
          
          items.push({
            id: crypto.randomUUID(),
            server_id: `${args.host}:${args.port}`,
            key,
            value: keyValue.value,
            key_type: keyValue.key_type,
            host: args.host,
            port: args.port,
            password: args.password,
            db: args.db,
            deleted_at: now.toISOString(),
            expires_at: expires.toISOString(),
          })
          keysToDelete.push(key)
          count++
        } catch {
          // 获取失败的 key 跳过
        }
      }
      
      saveToStorage(TRASH_KEY, items)
      
      // 批量删除 Redis 中的 keys
      if (keysToDelete.length > 0) {
        for (const key of keysToDelete) {
          await wsProxyCall('delete_key', {
            host: args.host,
            port: args.port,
            password: args.password,
            db: args.db,
            key
          }).catch(() => {})
        }
      }
      
      return count
    }

    case 'restore_from_trash': {
      const idx = items.findIndex(i => i.id === args.trash_id)
      if (idx >= 0) {
        const item = items[idx]
        // 恢复到 Redis
        wsProxyCall('set_key_value', {
          host: item.host,
          port: item.port,
          password: item.password,
          db: item.db,
          key: item.key,
          value: item.value,
          key_type: item.key_type,
        }).catch(err => console.error('恢复键失败:', err))
        items.splice(idx, 1)
        saveToStorage(TRASH_KEY, items)
      }
      return true
    }

    case 'batch_restore_from_trash': {
      const ids = args.trash_ids
      let count = 0
      for (const id of ids) {
        const idx = items.findIndex(i => i.id === id)
        if (idx >= 0) {
          const item = items[idx]
          wsProxyCall('set_key_value', {
            host: item.host,
            port: item.port,
            password: item.password,
            db: item.db,
            key: item.key,
            value: item.value,
            key_type: item.key_type,
          }).catch(() => {})
          items.splice(idx, 1)
          count++
        }
      }
      saveToStorage(TRASH_KEY, items)
      return count
    }

    case 'permanent_delete_trash': {
      const ids = args.trash_ids
      const remaining = items.filter(i => !ids.includes(i.id))
      const deletedCount = items.length - remaining.length
      saveToStorage(TRASH_KEY, remaining)
      return deletedCount
    }

    case 'clear_expired_trash': {
      const now = new Date()
      const remaining = items.filter(i => new Date(i.expires_at) > now)
      const clearedCount = items.length - remaining.length
      saveToStorage(TRASH_KEY, remaining)
      return clearedCount
    }

    default:
      throw new Error(`未知的废键箱命令: ${command}`)
  }
}

// ========== 需要通过代理的 Redis 命令列表 ==========

const PROXY_COMMANDS = new Set([
  'connect',
  'get_databases',
  'get_keys',
  'get_key_value',
  'set_key_value',
  'delete_key',
  'search_keys',
  'create_database',
  'delete_database',
  'flush_database',
  'generate_test_data',
  'test_connection',
  'slowlog_get',
  'get_memory_info',
  'get_server_info',  // 添加这个命令
  'get_key_stats',    // 添加这个命令
])

// ========== 需要通过 localStorage 的服务器命令 ==========

const SERVER_COMMANDS = new Set([
  'get_servers',
  'add_server',
  'edit_server',
  'delete_server',
  'save_server_order',
])

// ========== 需要通过 localStorage 的废键箱命令 ==========

const TRASH_COMMANDS = new Set([
  'move_to_trash',
  'batch_move_to_trash',
  'get_trash_items',
  'restore_from_trash',
  'batch_restore_from_trash',
  'permanent_delete_trash',
  'clear_expired_trash',
])

// ========== 主入口 ==========

export async function browserExecute(command: string, args: any): Promise<any> {
  console.log(`[browserExecute] 收到命令: ${command}, args:`, args)
  console.log(`[browserExecute] args?.req:`, args?.req)
  console.log(`[browserExecute] args?.host:`, args?.host)
  
  // args 格式为 { req: { host, port, password, db, ... } }，需要解包
  const params = args?.req || args || {}
  console.log(`[browserExecute] 解包后的 params:`, params)
  
  // Redis 操作 → WebSocket 代理
  if (PROXY_COMMANDS.has(command)) {
    console.log(`[browserExecute] 转发命令 ${command} 到 WebSocket 代理`)
    // generate_test_data 有额外参数 count（与 req 同级）
    if (command === 'generate_test_data') {
      const count = args?.count || 100
      return wsProxyCall(command, params, { count })
    }
    return wsProxyCall(command, params)
  }

  // 服务器配置 → localStorage
  if (SERVER_COMMANDS.has(command)) {
    return handleServerCommands(command, args)
  }

  // 废键箱 → localStorage（部分需要 Redis 代理）
  if (TRASH_COMMANDS.has(command)) {
    return handleTrashCommands(command, params)
  }

  // 导入导出 → 浏览器原生 API（由 MainView.vue 单独处理）
  if (command === 'export_data' || command === 'import_data') {
    throw new Error(`${command} 在浏览器环境下由前端直接处理`)
  }

  console.error(`[browserExecute] 浏览器环境不支持命令: ${command}`)
  throw new Error(`浏览器环境不支持命令: ${command}`)
}
