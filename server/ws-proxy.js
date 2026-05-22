/**
 * WebSocket Redis 代理服务器
 * 用于浏览器开发环境下转发 Redis 操作命令
 * 
 * 启动方式: node server/ws-proxy.js
 * 默认端口: 8765
 */

import { WebSocketServer } from 'ws'
import Redis from 'ioredis'

const PORT = process.env.WS_PROXY_PORT || 8765

// 统一 Redis 命令执行（自动管理连接生命周期）
async function executeRedisCommand(host, port, password, db, commandFn) {
  console.log(`[ws-proxy] 正在连接 Redis ${host}:${port} db=${db}`)
  
  const conn = new Redis({
    host,
    port,
    password: password || undefined,
    db,
    retryStrategy: (times) => {
      // 最多重连一次
      if (times > 1) return null
      return 100
    },
    enableReadyCheck: true,
    connectTimeout: 10000,
    commandTimeout: 10000,
  })
  
  // 添加错误监听器，避免未处理的错误事件
  conn.on('error', (err) => {
    console.error(`[ws-proxy] Redis 连接错误:`, err.message)
  })
  
  conn.on('connect', () => {
    console.log(`[ws-proxy] Redis 已连接 ${host}:${port}`)
  })
  
  conn.on('ready', () => {
    console.log(`[ws-proxy] Redis 已就绪 ${host}:${port}`)
  })
  
  try {
    // 等待连接就绪
    await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Redis 连接超时'))
      }, 10000)
      
      // 检查是否已经 ready
      if (conn.status === 'ready') {
        clearTimeout(timeout)
        resolve()
        return
      }
      
      conn.on('ready', () => {
        clearTimeout(timeout)
        resolve()
      })
      
      conn.on('error', (err) => {
        clearTimeout(timeout)
        reject(err)
      })
    })
    
    // 执行命令
    const result = await commandFn(conn)
    console.log(`[ws-proxy] Redis 命令执行成功`)
    
    // 先返回结果，再关闭连接
    await conn.quit().catch(err => {
      console.error(`[ws-proxy] 关闭连接失败:`, err.message)
    })
    
    return result
  } catch (err) {
    console.error(`[ws-proxy] Redis 命令执行失败:`, err.message)
    // 确保关闭连接
    try {
      await conn.quit().catch(() => {})
    } catch {}
    throw err
  }
}

// 命令处理器
const handlers = {
  // 测试连接
  async connect({ host, port, password, db }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      await conn.ping()
      return true
    })
  },

  // 获取数据库列表
  async get_databases({ host, port, password }) {
    return executeRedisCommand(host, port, password, 0, async (conn) => {
      try {
        const info = await conn.info('keyspace')
        console.log(`[ws-proxy] INFO keyspace 返回:`, info, typeof info)
        
        // 确保 info 是字符串
        if (typeof info !== 'string') {
          console.error(`[ws-proxy] INFO 返回类型错误:`, typeof info)
          return [[0, 0]]
        }
        
        const dbs = []
        for (const line of info.split('\n')) {
          const match = line.match(/^db(\d+):keys=(\d+)/)
          if (match) {
            dbs.push([parseInt(match[1]), parseInt(match[2])])
          }
        }
        
        // 确保至少返回 db0
        if (dbs.length === 0) {
          console.log(`[ws-proxy] 未找到数据库，返回默认 db0`)
          dbs.push([0, 0])
        }
        
        console.log(`[ws-proxy] 返回数据库列表:`, dbs)
        return dbs
      } catch (err) {
        console.error(`[ws-proxy] 获取数据库列表失败:`, err.message)
        // 返回默认数据库
        return [[0, 0]]
      }
    })
  },

  // 获取服务器信息
  async get_server_info({ host, port, password, db }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      // 使用 INFO 命令获取服务器信息
      const info = await conn.info()
      console.log(`[ws-proxy] 获取服务器信息成功, 类型:`, typeof info)
      
      // 如果已经是对象，直接返回
      if (typeof info === 'object' && info !== null && !Array.isArray(info)) {
        return info
      }
      
      // 如果是字符串，解析成对象
      if (typeof info === 'string') {
        const result = {}
        for (const line of info.split('\n')) {
          if (!line || line.startsWith('#')) continue
          if (line.includes(':')) {
            const [key, ...valueParts] = line.split(':')
            result[key.trim()] = valueParts.join(':').trim()
          }
        }
        return result
      }
      
      // 如果是其他类型，尝试转换为字符串再解析
      return String(info)
    })
  },

  // 获取键统计信息
  async get_key_stats({ host, port, password }) {
    return executeRedisCommand(host, port, password, 0, async (conn) => {
      const info = await conn.info('keyspace')
      console.log(`[ws-proxy] get_key_stats INFO keyspace 返回:`, info, typeof info)
      
      // 解析 keyspace 信息
      const stats = []
      
      // INFO 命令可能返回字符串或对象
      let lines = []
      if (typeof info === 'string') {
        lines = info.split('\n')
      } else if (typeof info === 'object' && info !== null) {
        // 如果是对象，尝试转换为字符串处理
        lines = String(info).split('\n')
      }
      
      for (const line of lines) {
        if (!line || line.startsWith('#')) continue
        if (line.includes(':')) {
          const [dbKey, value] = line.split(':')
          const dbNum = parseInt(dbKey.replace('db', ''), 10)
          
          let keys = 0, expires = 0, avgTtl = 0
          for (const part of value.split(',')) {
            const [k, v] = part.split('=')
            if (k === 'keys') keys = parseInt(v, 10)
            if (k === 'expires') expires = parseInt(v, 10)
            if (k === 'avg_ttl') avgTtl = parseInt(v, 10)
          }
          
          stats.push({
            db: dbNum,
            keys,
            expires,
            avg_ttl: avgTtl
          })
        }
      }
      
      return stats
    })
  },

  // 获取所有键
  async get_keys({ host, port, password, db, limit }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      const allKeys = await conn.keys('*')
      const total = allKeys.length
      
      // 如果有 limit 参数，只返回前 limit 个
      if (limit && limit > 0) {
        return {
          keys: allKeys.slice(0, limit),
          total
        }
      }
      
      return {
        keys: allKeys,
        total
      }
    })
  },

  // 获取键值
  async get_key_value({ host, port, password, db, key }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      const type = await conn.type(key)
      let value = ''
      
      switch (type) {
        case 'string':
          value = await conn.get(key)
          break
        case 'hash':
          value = JSON.stringify(await conn.hgetall(key))
          break
        case 'list':
          value = JSON.stringify(await conn.lrange(key, 0, -1))
          break
        case 'set':
          value = JSON.stringify(await conn.smembers(key))
          break
        case 'zset':
          value = JSON.stringify(await conn.zrange(key, 0, -1, 'WITHSCORES'))
          break
        default:
          value = await conn.get(key) || ''
      }
      
      return { key, value: value || '', key_type: type }
    })
  },

  // 设置键值
  async set_key_value({ host, port, password, db, key, value, key_type }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      switch (key_type) {
        case 'string':
          await conn.set(key, value)
          break
        case 'hash': {
          const data = JSON.parse(value)
          if (Object.keys(data).length > 0) {
            await conn.hset(key, data)
          }
          break
        }
        case 'list': {
          const data = JSON.parse(value)
          if (data.length > 0) {
            await conn.del(key)
            await conn.rpush(key, ...data)
          }
          break
        }
        case 'set': {
          const data = JSON.parse(value)
          if (data.length > 0) {
            await conn.sadd(key, ...data)
          }
          break
        }
        default:
          await conn.set(key, value)
      }
      return true
    })
  },

  // 删除键
  async delete_key({ host, port, password, db, key }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      await conn.del(key)
      return true
    })
  },

  // 搜索键
  async search_keys({ host, port, password, db, pattern }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      return conn.keys(pattern)
    })
  },

  // 创建数据库（Redis 自动创建，只需 select）
  async create_database({ host, port, password, db }) {
    return executeRedisCommand(host, port, password, db, async () => {
      return true
    })
  },

  // 删除数据库（清空）
  async delete_database({ host, port, password, db }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      await conn.flushdb()
      return true
    })
  },

  // 清空数据库
  async flush_database({ host, port, password, db }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      await conn.flushdb()
      return true
    })
  },

  // 生成测试数据
  async generate_test_data({ host, port, password, db }, count) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      const keyTypes = ['string', 'hash', 'list', 'set']
      
      for (let i = 0; i < count; i++) {
        const keyType = keyTypes[Math.floor(Math.random() * keyTypes.length)]
        const keyName = `test_key_${i}_${Math.floor(Math.random() * 9000) + 1000}`
        
        switch (keyType) {
          case 'string':
            await conn.set(keyName, `value_${Math.floor(Math.random() * 10000)}`)
            break
          case 'hash':
            await conn.hset(keyName, {
              field1: `value${Math.floor(Math.random() * 100)}`,
              field2: `value${Math.floor(Math.random() * 100)}`
            })
            break
          case 'list':
            await conn.rpush(keyName,
              `item${Math.floor(Math.random() * 100)}`,
              `item${Math.floor(Math.random() * 100)}`,
              `item${Math.floor(Math.random() * 100)}`
            )
            break
          case 'set':
            await conn.sadd(keyName,
              `member${Math.floor(Math.random() * 100)}`,
              `member${Math.floor(Math.random() * 100)}`,
              `member${Math.floor(Math.random() * 100)}`
            )
            break
        }
      }
      return true
    })
  },

  // 测试连接（独立命令）
  async test_connection({ host, port, password }) {
    try {
      await executeRedisCommand(host, port, password, 0, async (conn) => {
        await conn.ping()
      })
      return { success: true, message: '连接成功' }
    } catch (e) {
      return { success: false, message: `连接失败: ${e.message}` }
    }
  },

  // 获取 SLOWLOG（历史命令记录）
  // 只读模式：不修改 Redis 配置，使用服务器原有阈值
  // SLOWLOG GET count 最多返回 slowlog-max-len 条（默认 128），超出部分 Redis 已滚动覆盖
  // 若需要更多历史，需在 redis.conf 中调大 slowlog-max-len 并重启 Redis
  async slowlog_get({ host, port, password }) {
    return executeRedisCommand(host, port, password, 0, async (conn) => {
      console.log(`[ws-proxy][slowlog] 开始获取 ${host}:${port} 的 SLOWLOG...`)

      // 读取慢日志（不限数量，取回 Redis 当前 slowlog-max-len 范围内的全部记录）
      const raw = await conn.slowlog('GET', 9999)
      console.log(`[ws-proxy][slowlog] SLOWLOG GET 返回: ${Array.isArray(raw) ? raw.length + ' 条' : JSON.stringify(raw)}`)

      if (!Array.isArray(raw)) return []

      // 服务端过滤：只保留用户数据操作指令，隐藏一切工具/驱动/诊断类指令
      // 白名单思路：凡是 GET/SET/DEL/EXPIRE/HGET/HSET/LPUSH/LRANGE/SMEMBERS/ZRANGE
      // 等数据操作才保留，其他全部过滤
      const filtered = raw.filter(entry => {
        const cmd = Array.isArray(entry[3]) ? (entry[3][0] || '').toUpperCase() : ''

        // 1. 心跳/连接握手类（ioredis 每次新连接都会产生）
        if (cmd === 'PING') return false
        if (cmd === 'CLIENT') return false
        if (cmd === 'AUTH') return false

        // 2. 配置/诊断类（非用户数据操作）
        if (cmd === 'CONFIG') return false
        if (cmd === 'INFO') return false
        if (cmd === 'COMMAND') return false

        // 3. SLOWLOG 自身指令
        if (cmd === 'SLOWLOG') return false

        // 4. 安全过滤：MONITOR（虽然已移除实时监控，但保留过滤）
        if (cmd === 'MONITOR') return false

        return true
      })

      if (filtered.length !== raw.length) {
        console.log(`[ws-proxy][slowlog] 过滤掉 ${raw.length - filtered.length} 条噪音，返回 ${filtered.length} 条`)
      }

      return filtered.map(entry => ({
        id: entry[0],
        time: entry[1],
        costMs: entry[2],
        cmd: Array.isArray(entry[3]) ? (entry[3][0] || '') : '',
        args: Array.isArray(entry[3]) ? entry[3].slice(1) : [],
        // Redis SLOWLOG 协议：entry[4]=client_addr, entry[5]=client_name（Redis 7+）
        // 任何 Redis 版本均不含 db 字段，db 字段不可用
        client: entry[4] || '',     // "127.0.0.1:52341" 格式
      }))
    })
  },

  // 获取内存分析信息
  // 支持 cursor 参数：首次请求不传 cursor（全量分析），后续翻页传 cursor 继续扫描
  async get_memory_info({ host, port, password, db, cursor }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      console.log(`[ws-proxy][memory] 开始分析 ${host}:${port} db=${db} 的内存...`)

      // 获取内存基本信息
      const info = await conn.info('memory')
      const lines = info.split('\n')
      const infoMap = {}
      for (const line of lines) {
        const idx = line.indexOf(':')
        if (idx > 0) {
          const key = line.substring(0, idx).trim()
          const value = line.substring(idx + 1).trim()
          infoMap[key] = value
        }
      }

      const usedMemory = parseInt(infoMap['used_memory']) || 0
      const usedMemoryPeak = parseInt(infoMap['used_memory_peak']) || 0
      const memFragmentationRatio = parseFloat(infoMap['mem_fragmentation_ratio']) || 0
      const maxmemory = parseInt(infoMap['maxmemory']) || 0

      // 格式化字节数
      function formatBytes(bytes) {
        const KB = 1024
        const MB = KB * 1024
        const GB = MB * 1024

        if (bytes >= GB) {
          return (bytes / GB).toFixed(2) + 'GB'
        } else if (bytes >= MB) {
          return (bytes / MB).toFixed(2) + 'MB'
        } else if (bytes >= KB) {
          return (bytes / KB).toFixed(2) + 'KB'
        } else {
          return bytes + 'B'
        }
      }

      // 使用 DBSIZE 获取 key 总数（O(1)，瞬间返回）
      const totalKeys = await conn.dbsize()
      console.log(`[ws-proxy][memory] DBSIZE 返回: ${totalKeys}`)

      // 分页扫描：支持 cursor 参数继续扫描下一批
      const SCAN_BATCH_SIZE = 100
      const keyMemoryList = []
      const typeStats = {}
      let nextCursor = '0'  // 返回给前端的下一页 cursor

      if (totalKeys > 0) {
        // cursor 参数：前端翻页时传入上次返回的 nextCursor 继续扫描
        const scanCursor = cursor || '0'
        const [newCursor, keys] = await conn.scan(scanCursor, 'COUNT', SCAN_BATCH_SIZE)
        nextCursor = newCursor
        console.log(`[ws-proxy][memory] SCAN cursor=${scanCursor} 返回 ${keys.length} 个 key，nextCursor=${nextCursor}`)

        if (keys.length > 0) {
          // 合并 Pipeline：TYPE 和 MEMORY USAGE 交错发送，1 次 RTT，无并发竞争
          // 格式：TYPE(k0), MEMORY(k0), TYPE(k1), MEMORY(k1), ...
          const pipe = conn.pipeline()
          for (const key of keys) {
            pipe.type(key)
            pipe.memory('USAGE', key)
          }
          const rawResults = await pipe.exec()

          // 解析交错结果：偶数索引 = [err, type], 奇数索引 = [err, size]
          for (let i = 0; i < keys.length; i++) {
            const key = keys[i]
            const keyType = rawResults[i * 2]?.[1] || 'none'
            const size = rawResults[i * 2 + 1]?.[1] || 0

            keyMemoryList.push({
              key,
              size,
              size_human: formatBytes(size),
              key_type: keyType
            })

            // 统计类型
            if (!typeStats[keyType]) {
              typeStats[keyType] = { count: 0, memory_bytes: 0 }
            }
            typeStats[keyType].count++
            typeStats[keyType].memory_bytes += size
          }
        }
      }

      // 按内存大小排序
      keyMemoryList.sort((a, b) => b.size - a.size)

      // 已扫描的 key 数量
      const scannedKeysCount = keyMemoryList.length

      // 计算类型统计（基于本次扫描的样本）
      const totalMemory = Object.values(typeStats).reduce((sum, stat) => sum + stat.memory_bytes, 0)
      const keyTypeStats = Object.entries(typeStats).map(([keyType, stat]) => ({
        key_type: keyType,
        count: stat.count,
        memory_bytes: stat.memory_bytes,
        memory_percent: totalMemory > 0 ? (stat.memory_bytes / totalMemory * 100) : 0
      }))

      // 计算过期键占比（简化计算）
      const expiredKeysRatio = 0

      const result = {
        used_memory: usedMemory,
        used_memory_human: formatBytes(usedMemory),
        used_memory_peak: usedMemoryPeak,
        used_memory_peak_human: formatBytes(usedMemoryPeak),
        mem_fragmentation_ratio: memFragmentationRatio,
        maxmemory: maxmemory,
        keys_count: totalKeys,
        expired_keys_ratio: expiredKeysRatio,
        large_keys_count: scannedKeysCount,
        key_memory_list: keyMemoryList,
        key_type_stats: keyTypeStats,
        next_cursor: nextCursor  // 返回给前端用于翻页
      }

      console.log(`[ws-proxy][memory] 内存分析完成: 总计 ${totalKeys} 个键，本次扫描 ${scannedKeysCount} 个，nextCursor=${nextCursor}`)
      return result
    })
  },

  // 全量扫描键类型分布（只查 TYPE 不查 MEMORY USAGE，速度快）
  // 前端异步调用，不阻塞主界面加载
  async get_type_distribution({ host, port, password, db }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      console.log(`[ws-proxy][type-dist] 开始全量 TYPE 扫描 ${host}:${port} db=${db}...`)
      const BATCH_SIZE = 200
      const typeCounts = {}
      let cursor = '0'

      do {
        const [newCursor, keys] = await conn.scan(cursor, 'COUNT', BATCH_SIZE)
        cursor = newCursor

        if (keys.length > 0) {
          // Pipeline 批量 TYPE，1 次 RTT 搞定一批
          const pipe = conn.pipeline()
          for (const key of keys) {
            pipe.type(key)
          }
          const results = await pipe.exec()

          for (const [err, keyType] of results) {
            if (!err && keyType && keyType !== 'none') {
              typeCounts[keyType] = (typeCounts[keyType] || 0) + 1
            }
          }
        }
      } while (cursor !== '0')

      const total = Object.values(typeCounts).reduce((s, c) => s + c, 0)
      const keyTypeStats = Object.entries(typeCounts).map(([keyType, count]) => ({
        key_type: keyType,
        count,
        memory_bytes: 0,  // 全量 TYPE 扫描不查内存
        memory_percent: total > 0 ? (count / total * 100) : 0
      }))

      console.log(`[ws-proxy][type-dist] 全量 TYPE 扫描完成: ${total} 个键，类型分布:`, typeCounts)
      return keyTypeStats
    })
  },
}

// WebSocket 服务器
const wss = new WebSocketServer({ 
  port: PORT,
  maxPayload: 100 * 1024 * 1024 // 100MB
})

wss.on('error', (err) => {
  console.error(`[ws-proxy] WebSocket 服务器错误:`, err.message)
  if (err.code === 'EADDRINUSE') {
    console.error(`[ws-proxy] 端口 ${PORT} 已被占用，请先停止占用该端口的进程`)
  }
})

wss.on('connection', (ws) => {
  console.log(`[ws-proxy] 客户端已连接 (当前连接数: ${wss.clients.size})`)
  
  ws.on('message', async (data) => {
    let msg
    try {
      msg = JSON.parse(data.toString())
      const { id, command, args } = msg
      
      console.log(`[ws-proxy] 收到命令:`, command, args)

      if (!handlers[command]) {
        ws.send(JSON.stringify({ id, error: `未知命令: ${command}` }))
        return
      }
      
      // 分离额外参数（如 count）
      const extraArgs = msg.extraArgs || {}
      const result = await handlers[command](args, ...Object.values(extraArgs))
      console.log(`[ws-proxy] 命令 ${command} 执行结果:`, result)
      ws.send(JSON.stringify({ id, result }))
    } catch (err) {
      console.error(`[ws-proxy] 命令执行错误:`, err.message, err.stack)
      ws.send(JSON.stringify({ id: msg?.id, error: err.message }))
    }
  })
  
  ws.on('close', () => {
    console.log(`[ws-proxy] 客户端已断开 (当前连接数: ${wss.clients.size})`)
  })
  
  ws.on('error', (err) => {
    console.error(`[ws-proxy] WebSocket 错误:`, err.message)
  })
})

console.log(`[ws-proxy] Redis WebSocket 代理已启动，端口: ${PORT}`)
