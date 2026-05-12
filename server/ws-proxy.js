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

// 更简单的 Redis 连接管理，让 ioredis 自己处理连接
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
}

// WebSocket 服务器
const wss = new WebSocketServer({ port: PORT })

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
