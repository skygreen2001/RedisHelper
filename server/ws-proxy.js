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

  // 获取所有键
  async get_keys({ host, port, password, db }) {
    return executeRedisCommand(host, port, password, db, async (conn) => {
      return conn.keys('*')
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
