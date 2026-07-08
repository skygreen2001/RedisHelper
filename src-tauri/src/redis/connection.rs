use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::storage::config::debug_println;

/// 获取键列表的响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct KeysResponse {
    pub keys: Vec<String>,
    pub total: usize,
}

/// 连接缓存 - 按 host:port:password:db 缓存连接，使用 Mutex 保护
struct ConnectionCache {
    connections: HashMap<String, Connection>,
}

impl ConnectionCache {
    fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }
}

// 全局连接缓存
static CONNECTION_CACHE: std::sync::OnceLock<Mutex<ConnectionCache>> = 
    std::sync::OnceLock::new();

fn get_cache() -> &'static Mutex<ConnectionCache> {
    CONNECTION_CACHE.get_or_init(|| Mutex::new(ConnectionCache::new()))
}

pub struct RedisConnection {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    db: u8,
}

/// 构造 redis:// URL，支持 ACL 用户名（Redis >= 6.0）
/// 仅当密码非空时才附加 :password 部分；用户名支持则总是附加（如果有）
fn build_redis_url(host: &str, port: u16, username: Option<&str>, password: Option<&str>) -> String {
    let user = username.map(|s| s.trim()).filter(|s| !s.is_empty());
    let pass = password.map(|s| s.to_string()).filter(|s| !s.is_empty());

    match (user, pass) {
        (Some(u), Some(p)) => format!("redis://{}:{}@{}:{}", u, p, host, port),
        (Some(u), None) => format!("redis://{}@{}:{}", u, host, port),
        (None, Some(p)) => format!("redis://:{}@{}:{}", p, host, port),
        (None, None) => format!("redis://{}:{}", host, port),
    }
}

impl RedisConnection {
    /// 创建连接并支持 ACL 用户名（Redis >= 6.0）
    pub fn new_with_auth(
        host: &str,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Self, Box<dyn Error>> {
        let url = build_redis_url(host, port, username.as_deref(), password.as_deref());
        debug_println!("[DEBUG] RedisConnection::new: {} (user={:?})", url, username);

        let client = Client::open(url)?;
        let mut conn = client.get_connection()?;

        // ACL（Redis >= 6.0）：先发送 AUTH <username> <password>，回退到 AUTH <password>
        match (username.as_deref(), password.as_deref()) {
            (Some(u), Some(p)) if !u.is_empty() => {
                let _: () = redis::cmd("AUTH").arg(u).arg(p).query(&mut conn)?;
            }
            (Some(u), None) if !u.is_empty() => {
                let _: () = redis::cmd("AUTH").arg(u).arg("").query(&mut conn)?;
            }
            (None, Some(p)) if !p.is_empty() => {
                let _: () = redis::cmd("AUTH").arg(p).query(&mut conn)?;
            }
            _ => {}
        }

        // 缓存连接
        let cache_key = format!(
            "{}:{}:{}:{}:{}",
            host,
            port,
            username.as_deref().unwrap_or(""),
            password.as_deref().unwrap_or(""),
            0
        );
        let mut cache = get_cache().lock().unwrap();
        cache.connections.insert(cache_key, conn);

        Ok(Self {
            host: host.to_string(),
            port,
            username,
            password,
            db: 0,
        })
    }

    /// 从缓存获取或创建连接，支持 ACL 用户名
    pub fn from_cache(
        host: &str,
        port: u16,
        username: Option<&str>,
        password: Option<&str>,
        db: u8,
    ) -> Result<Self, Box<dyn Error>> {
        let user_trim = username.map(|s| s.trim()).filter(|s| !s.is_empty());
        let cache_key = format!(
            "{}:{}:{}:{}:{}",
            host,
            port,
            user_trim.unwrap_or(""),
            password.unwrap_or(""),
            db
        );

        {
            let mut cache = get_cache().lock().unwrap();

            // 检查缓存
            if let Some(_conn) = cache.connections.get_mut(&cache_key) {
                // 验证连接是否存活
                let result: Result<String, _> = redis::cmd("PING").query(_conn);
                match result {
                    Ok(_) => {
                        debug_println!("[DEBUG] 连接池: 复用现有连接 {}", cache_key);
                        return Ok(Self {
                            host: host.to_string(),
                            port,
                            username: user_trim.map(|s| s.to_string()),
                            password: password.map(|s| s.to_string()),
                            db,
                        });
                    }
                    Err(_) => {
                        debug_println!("[DEBUG] 连接池: 连接已失效，移除 {}", cache_key);
                        cache.connections.remove(&cache_key);
                    }
                }
            }
        }

        // 创建新连接
        debug_println!("[DEBUG] 连接池: 创建新连接 {}", cache_key);
        let url = build_redis_url(host, port, user_trim, password);

        let client = Client::open(url)?;
        let mut conn = client.get_connection()?;

        // ACL（Redis >= 6.0）：根据是否提供用户名/密码分别处理
        match (user_trim, password) {
            (Some(u), Some(p)) if !p.is_empty() => {
                let _: () = redis::cmd("AUTH").arg(u).arg(p).query(&mut conn)?;
            }
            (Some(u), None) => {
                let _: () = redis::cmd("AUTH").arg(u).arg("").query(&mut conn)?;
            }
            (None, Some(p)) if !p.is_empty() => {
                let _: () = redis::cmd("AUTH").arg(p).query(&mut conn)?;
            }
            _ => {}
        }

        // 选择数据库
        let _: () = redis::cmd("SELECT").arg(db).query(&mut conn)?;

        let mut cache = get_cache().lock().unwrap();
        cache.connections.insert(cache_key, conn);

        Ok(Self {
            host: host.to_string(),
            port,
            username: user_trim.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
            db,
        })
    }
    
    /// 获取连接缓存的 key
    fn cache_key(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.host,
            self.port,
            self.username.as_deref().unwrap_or(""),
            self.password.as_deref().unwrap_or(""),
            self.db
        )
    }
    
    pub fn select(&mut self, db: u8) -> Result<(), Box<dyn Error>> {
        self.db = db;
        let key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        if let Some(conn) = cache.connections.get_mut(&key) {
            let _: () = redis::cmd("SELECT").arg(db).query(conn)?;
        }
        Ok(())
    }
    
    pub fn ping(&mut self) -> Result<(), Box<dyn Error>> {
        let key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        if let Some(conn) = cache.connections.get_mut(&key) {
            let _: String = redis::cmd("PING").query(conn)?;
        }
        Ok(())
    }
    
    pub fn get_databases(&mut self) -> Result<Vec<(u8, usize)>, Box<dyn Error>> {
        let key = format!(
            "{}:{}:{}:{}:{}",
            self.host,
            self.port,
            self.username.as_deref().unwrap_or(""),
            self.password.as_deref().unwrap_or(""),
            0
        );
        
        let mut databases = Vec::new();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        // 使用 INFO keyspace 获取数据库统计，这比逐个遍历数据库执行 KEYS * 高效得多
        let info_str: String = redis::cmd("INFO").arg("keyspace").query(conn)?;
        
        drop(cache); // 释放锁
        
        for line in info_str.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(idx) = line.find(':') {
                let db_key = line[..idx].trim().to_string();
                let value = line[idx + 1..].trim().to_string();
                
                // 解析格式: keys=10,expires=2,avg_ttl=3600
                let mut keys = 0;
                for part in value.split(',') {
                    let kv: Vec<_> = part.split('=').collect();
                    if kv.len() == 2 && kv[0] == "keys" {
                        keys = kv[1].parse().unwrap_or(0);
                        break;
                    }
                }
                
                // 只添加有键的数据库
                if keys > 0 {
                    // 从 "db0" 提取数据库编号
                    if let Some(db_num) = db_key.strip_prefix("db") {
                        if let Ok(db) = db_num.parse::<u8>() {
                            databases.push((db, keys));
                        }
                    }
                }
            }
        }
        
        // 按数据库编号排序
        databases.sort_by_key(|a| a.0);
        
        Ok(databases)
    }
    
    pub fn get_keys(&mut self, limit: Option<usize>) -> Result<KeysResponse, Box<dyn Error>> {
        let key = self.cache_key();
        let mut keys = Vec::new();
        let limit = limit.unwrap_or(0); // 0 表示不限制
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        // 获取数据库总键数（用于显示）
        let total: usize = redis::cmd("DBSIZE").query(conn)?;
        
        // 使用 SCAN 命令替代 KEYS 命令，避免阻塞 Redis 服务器
        let mut cursor = 0;
        
        loop {
            let (new_cursor, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("COUNT")
                .arg(1000)
                .query(conn)?;
            
            cursor = new_cursor;
            keys.extend(batch);
            
            // 如果达到限制，停止扫描
            if limit > 0 && keys.len() >= limit {
                keys.truncate(limit);
                break;
            }
            
            if cursor == 0 {
                break;
            }
        }
        
        Ok(KeysResponse { keys, total })
    }
    
    pub fn get_key_value(&mut self, key: &str) -> Result<(String, String), Box<dyn Error>> {
        let cache_key = self.cache_key();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        let key_type: String = redis::cmd("TYPE").arg(key).query(conn)?;
        
        let value = match key_type.as_str() {
            "string" => {
                let val: Option<String> = conn.get(key)?;
                val.unwrap_or_default()
            }
            "list" => {
                let val: Vec<String> = conn.lrange(key, 0, -1)?;
                serde_json::to_string(&val)?
            }
            "set" => {
                let val: Vec<String> = conn.smembers(key)?;
                serde_json::to_string(&val)?
            }
            "zset" => {
                let val: Vec<(String, f64)> = conn.zrange_withscores(key, 0, -1)?;
                serde_json::to_string(&val)?
            }
            "hash" => {
                let val: std::collections::HashMap<String, String> = conn.hgetall(key)?;
                serde_json::to_string(&val)?
            }
            _ => "".to_string(),
        };
        
        Ok((value, key_type))
    }
    
    pub fn set_key_value(&mut self, key: &str, value: &str, key_type: &str) -> Result<(), Box<dyn Error>> {
        let cache_key = self.cache_key();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        match key_type {
            "string" => {
                let _: () = conn.set(key, value)?;
            }
            "list" => {
                // 先删除旧值
                let _: () = conn.del::<_, ()>(key)?;
                // 解析JSON数组
                let values: Vec<String> = serde_json::from_str(value)?;
                for val in values {
                    let _: () = conn.lpush::<_, _, ()>(key, val)?;
                }
            }
            "set" => {
                // 先删除旧值
                let _: () = conn.del::<_, ()>(key)?;
                // 解析JSON数组
                let values: Vec<String> = serde_json::from_str(value)?;
                for val in values {
                    let _: () = conn.sadd::<_, _, ()>(key, val)?;
                }
            }
            "zset" => {
                // 先删除旧值
                let _: () = conn.del::<_, ()>(key)?;
                // 解析JSON数组
                let values: Vec<(String, f64)> = serde_json::from_str(value)?;
                for (val, score) in values {
                    let _: () = conn.zadd::<_, _, _, ()>(key, val, score)?;
                }
            }
            "hash" => {
                // 先删除旧值
                let _: () = conn.del::<_, ()>(key)?;
                // 解析JSON对象
                let values: std::collections::HashMap<String, String> = serde_json::from_str(value)?;
                for (field, val) in values {
                    let _: () = conn.hset::<_, _, _, ()>(key, field, val)?;
                }
            }
            _ => {
                return Err(Box::from("Unsupported key type"));
            }
        }
        
        Ok(())
    }
    
    pub fn delete_key(&mut self, key: &str) -> Result<(), Box<dyn Error>> {
        let cache_key = self.cache_key();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        let _: () = conn.del::<_, ()>(key)?;
        Ok(())
    }

    pub fn delete_keys(&mut self, keys: &[&str]) -> Result<(), Box<dyn Error>> {
        if keys.is_empty() {
            return Ok(());
        }
        
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        let _: () = redis::cmd("DEL").arg(keys).query(conn)?;
        Ok(())
    }
    
    pub fn search_keys(&mut self, pattern: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let cache_key = self.cache_key();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        let keys: Vec<String> = conn.keys(pattern)?;
        Ok(keys)
    }
    
    pub fn flushdb(&mut self) -> Result<(), Box<dyn Error>> {
        let cache_key = self.cache_key();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        let _: () = redis::cmd("FLUSHDB").query(conn)?;
        Ok(())
    }

    /// 获取 SLOWLOG 历史记录（只读，不修改 Redis 配置）
    /// Redis 7+ 返回格式: [id, timestamp_us, duration_us, [cmd, args...], client_addr, client_name]
    pub fn slowlog_get(&mut self) -> Result<Vec<SlowlogRaw>, Box<dyn Error>> {
        let cache_key = self.cache_key();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        let raw: Vec<redis::Value> = redis::cmd("SLOWLOG").arg("GET").arg(9999).query(conn)?;
        let entries: Vec<SlowlogRaw> = raw
            .into_iter()
            .filter_map(|v| {
                if let redis::Value::Array(items) = v {
                    Some(parse_slowlog_entry(items))
                } else {
                    None
                }
            })
            .collect();
        Ok(entries)
    }

    /// 获取当前 db 的键总数（O(1)）
    pub fn dbsize(&mut self) -> Result<usize, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        let count: usize = redis::cmd("DBSIZE").query(conn)?;
        Ok(count)
    }

    /// 获取内存基本信息
    pub fn get_memory_info(&mut self) -> Result<MemoryInfo, Box<dyn Error>> {
        let cache_key = self.cache_key();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        let info_str: String = redis::cmd("INFO")
            .arg("memory")
            .query(conn)?;
        
        drop(cache); // 释放锁
        
        let mut info_map = std::collections::HashMap::new();
        for line in info_str.lines() {
            if let Some(idx) = line.find(':') {
                let key = line[..idx].trim().to_string();
                let value = line[idx + 1..].trim().to_string();
                info_map.insert(key, value);
            }
        }
        
        let used_memory: u64 = info_map.get("used_memory")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let used_memory_peak: u64 = info_map.get("used_memory_peak")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let mem_fragmentation_ratio: f64 = info_map.get("mem_fragmentation_ratio")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let maxmemory: u64 = info_map.get("maxmemory")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        Ok(MemoryInfo {
            used_memory,
            used_memory_human: Self::format_bytes(used_memory),
            used_memory_peak,
            used_memory_peak_human: Self::format_bytes(used_memory_peak),
            mem_fragmentation_ratio,
            maxmemory,
        })
    }

    /// 获取完整的服务器信息
    pub fn get_server_info(&mut self) -> Result<std::collections::HashMap<String, String>, Box<dyn Error>> {
        let cache_key = self.cache_key();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        let info_str: String = redis::cmd("INFO").query(conn)?;
        
        drop(cache); // 释放锁
        
        let mut info_map = std::collections::HashMap::new();
        for line in info_str.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(idx) = line.find(':') {
                let key = line[..idx].trim().to_string();
                let value = line[idx + 1..].trim().to_string();
                info_map.insert(key, value);
            }
        }
        
        Ok(info_map)
    }
    
    /// 获取有数据的数据库的键统计（使用 INFO keyspace 命令）
    pub fn get_key_stats(&mut self) -> Result<Vec<KeyStatItem>, Box<dyn Error>> {
        let key = format!(
            "{}:{}:{}:{}:{}",
            self.host,
            self.port,
            self.username.as_deref().unwrap_or(""),
            self.password.as_deref().unwrap_or(""),
            0
        );
        
        let mut stats = Vec::new();
        
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&key)
            .ok_or_else(|| Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected, "连接不存在")))?;
        
        // 首先获取 INFO keyspace，它已经包含了所有数据库的键信息
        let info_str: String = redis::cmd("INFO").arg("keyspace").query(conn)?;
        
        drop(cache); // 释放锁
        
        // 解析 keyspace 信息
        for line in info_str.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(idx) = line.find(':') {
                let db_key = line[..idx].trim().to_string();
                let value = line[idx + 1..].trim().to_string();
                
                // 解析格式: keys=10,expires=2,avg_ttl=3600
                let mut keys = 0;
                let mut expires = 0;
                let mut avg_ttl = 0;
                
                for part in value.split(',') {
                    let kv: Vec<_> = part.split('=').collect();
                    if kv.len() == 2 {
                        match kv[0] {
                            "keys" => keys = kv[1].parse().unwrap_or(0),
                            "expires" => expires = kv[1].parse().unwrap_or(0),
                            "avg_ttl" => avg_ttl = kv[1].parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                }
                
                // 只添加有键的数据库
                if keys > 0 {
                    stats.push(KeyStatItem {
                        db: db_key,
                        keys,
                        expires,
                        avg_ttl: avg_ttl as u64,
                    });
                }
            }
        }
        
        Ok(stats)
    }

    /// 辅助函数：格式化字节数为人类可读格式
    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        
        if bytes >= GB {
            format!("{:.2}GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2}MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2}KB", bytes as f64 / KB as f64)
        } else {
            format!("{}B", bytes)
        }
    }

    /// 扫描并获取键的内存信息（分页扫描，使用 DBSIZE 获取总数）
    /// cursor: 0 = 首次扫描，其他值 = 继续上一次扫描
    /// 返回: (key_memory_list, key_type_stats, total_keys, next_cursor)
    pub fn scan_keys_memory(&mut self, cursor: u64) -> ScanKeysMemoryResult {
        let cache_key = self.cache_key();
        const SCAN_BATCH_SIZE: u64 = 100;
        
        // 使用 DBSIZE 获取总 key 数（O(1)，瞬间返回）
        let total_keys: usize = {
            let mut cache = get_cache().lock().unwrap();
            let conn = cache.connections.get_mut(&cache_key)
                .ok_or_else(|| Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotConnected, "连接不存在")))?;
            redis::cmd("DBSIZE").query(conn)?
        };
        
        let mut key_memory_list = Vec::new();
        let mut type_stats: std::collections::HashMap<String, (usize, u64)> = std::collections::HashMap::new();
        let mut next_cursor: u64 = 0;
        
        if total_keys == 0 {
            return Ok((key_memory_list, Vec::new(), 0, next_cursor));
        }
        
        // SCAN 一批 key（最多 SCAN_BATCH_SIZE 个）
        let batch_keys: Vec<String> = {
            let mut cache = get_cache().lock().unwrap();
            let conn = cache.connections.get_mut(&cache_key)
                .ok_or_else(|| Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotConnected, "连接不存在")))?;
            
            let result: (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("COUNT")
                .arg(SCAN_BATCH_SIZE)
                .query(conn)?;
            next_cursor = result.0;
            result.1
        };
        
        if batch_keys.is_empty() {
            return Ok((key_memory_list, Vec::new(), total_keys, next_cursor));
        }
        
        // 合并 Pipeline：TYPE 和 MEMORY USAGE 交错发送，只需 1 次 RTT
        // 格式：TYPE(k0), MEMORY USAGE(k0), TYPE(k1), MEMORY USAGE(k1), ...
        let raw_results: Vec<redis::Value> = {
            let mut cache = get_cache().lock().unwrap();
            let conn = cache.connections.get_mut(&cache_key)
                .ok_or_else(|| Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotConnected, "连接不存在")))?;
            
            let mut pipe = redis::pipe();
            for key in &batch_keys {
                pipe.cmd("TYPE").arg(key.as_str());
                pipe.cmd("MEMORY").arg("USAGE").arg(key.as_str());
            }
            pipe.query(conn)?
        };
        
        // 解析交错结果：偶数索引 = TYPE (SimpleString), 奇数索引 = MEMORY USAGE (Int/Nil)
        for (i, key) in batch_keys.iter().enumerate() {
            let key_type = match raw_results.get(i * 2) {
                Some(redis::Value::SimpleString(s)) => s.clone(),
                Some(redis::Value::BulkString(s)) => String::from_utf8_lossy(s).to_string(),
                _ => String::from("none"),
            };
            let size = match raw_results.get(i * 2 + 1) {
                Some(redis::Value::Int(n)) => *n as u64,
                _ => 0,
            };
            
            key_memory_list.push(KeyMemoryItem {
                key: key.clone(),
                size,
                size_human: Self::format_bytes(size),
                key_type: key_type.clone(),
            });
            
            let entry = type_stats.entry(key_type.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += size;
        }
        
        key_memory_list.sort_by_key(|a| std::cmp::Reverse(a.size));
        
        let total_memory: u64 = type_stats.values().map(|(_, m)| m).sum();
        let key_type_stats: Vec<KeyTypeStat> = type_stats
            .into_iter()
            .map(|(key_type, (count, memory_bytes))| {
                let memory_percent = if total_memory > 0 {
                    memory_bytes as f64 / total_memory as f64 * 100.0
                } else {
                    0.0
                };
                KeyTypeStat {
                    key_type,
                    count,
                    memory_bytes,
                    memory_percent,
                }
            })
            .collect();
        
        Ok((key_memory_list, key_type_stats, total_keys, next_cursor))
    }

    /// 全量扫描所有键的 TYPE，用于准确的键类型分布统计
    /// 只查 TYPE 不查 MEMORY USAGE，TYPE 是 O(1) 且 Pipeline 极快
    /// 返回: HashMap<type_name, count>
    pub fn scan_all_types(&mut self) -> Result<std::collections::HashMap<String, usize>, Box<dyn Error>> {
        let cache_key = self.cache_key();
        const BATCH_SIZE: u64 = 200; // TYPE 比 MEMORY USAGE 快，可以用更大批次
        let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut cursor: u64 = 0;

        loop {
            // SCAN 一批 key
            let batch_keys: Vec<String> = {
                let mut cache = get_cache().lock().unwrap();
                let conn = cache.connections.get_mut(&cache_key)
                    .ok_or_else(|| Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotConnected, "连接不存在")))?;

                let result: (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("COUNT")
                    .arg(BATCH_SIZE)
                    .query(conn)?;
                cursor = result.0;
                result.1
            };

            if !batch_keys.is_empty() {
                // Pipeline 批量 TYPE，1 次 RTT 搞定一批
                let type_raw: Vec<redis::Value> = {
                    let mut cache = get_cache().lock().unwrap();
                    let conn = cache.connections.get_mut(&cache_key)
                        .ok_or_else(|| Box::new(std::io::Error::new(
                            std::io::ErrorKind::NotConnected, "连接不存在")))?;

                    let mut pipe = redis::pipe();
                    for key in &batch_keys {
                        pipe.cmd("TYPE").arg(key.as_str());
                    }
                    pipe.query(conn)?
                };

                for v in &type_raw {
                    let key_type = match v {
                        redis::Value::SimpleString(s) => s.clone(),
                        redis::Value::BulkString(s) => String::from_utf8_lossy(s).to_string(),
                        _ => String::from("none"),
                    };
                    if key_type != "none" {
                        *type_counts.entry(key_type).or_insert(0) += 1;
                    }
                }
            }

            if cursor == 0 {
                break;
            }
        }

        Ok(type_counts)
    }

    /// List: RPUSH - 在列表尾部添加一个或多个值，返回操作后列表的长度
    pub fn rpush(&mut self, key: &str, value: &str) -> Result<i64, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        Ok(conn.rpush(key, value)?)
    }

    /// List: LSET - 设置列表指定索引位置的值
    pub fn lset(&mut self, key: &str, index: i64, value: &str) -> Result<(), Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        let _: () = redis::cmd("LSET").arg(key).arg(index).arg(value).query(conn)?;
        Ok(())
    }

    /// List: LREM - 从列表中删除等于指定值的元素，count > 0 从头部删
    pub fn lrem(&mut self, key: &str, count: i64, value: &str) -> Result<i64, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        let result: i64 = redis::cmd("LREM").arg(key).arg(count).arg(value).query(conn)?;
        Ok(result)
    }

    /// Set: SADD - 向集合添加成员，返回新增成员数
    pub fn sadd(&mut self, key: &str, members: &[String]) -> Result<i64, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        Ok(conn.sadd::<_, _, i64>(key, members)?)
    }

    /// Set: SREM - 从集合移除成员，返回移除数量
    pub fn srem(&mut self, key: &str, members: &[String]) -> Result<i64, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        Ok(conn.srem::<_, _, i64>(key, members)?)
    }

    /// ZSet: ZADD - 向有序集合添加成员，返回新增成员数
    pub fn zadd(&mut self, key: &str, members: &[(String, f64)]) -> Result<i64, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        let mut cmd = redis::cmd("ZADD");
        let mut cmd = cmd.arg(key);
        for (m, s) in members {
            cmd = cmd.arg(*s).arg(m.as_str());
        }
        let result: i64 = cmd.query(conn)?;
        Ok(result)
    }

    /// ZSet: ZREM - 从有序集合移除成员，返回移除数量
    pub fn zrem(&mut self, key: &str, members: &[String]) -> Result<i64, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        let mut cmd = redis::cmd("ZREM");
        let mut cmd = cmd.arg(key);
        for m in members {
            cmd = cmd.arg(m.as_str());
        }
        let result: i64 = cmd.query(conn)?;
        Ok(result)
    }

    /// Hash: HSET - 设置哈希表字段值
    pub fn hset_single(&mut self, key: &str, field: &str, value: &str) -> Result<bool, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        let result: i64 = conn.hset::<_, _, _, i64>(key, field, value)?;
        Ok(result > 0)
    }

    /// Hash: HDEL - 删除哈希表字段
    pub fn hdel_fields(&mut self, key: &str, fields: &[String]) -> Result<i64, Box<dyn Error>> {
        let cache_key = self.cache_key();
        let mut cache = get_cache().lock().unwrap();
        let conn = cache.connections.get_mut(&cache_key)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "连接不存在")))?;
        let mut cmd = redis::cmd("HDEL");
        let mut cmd = cmd.arg(key);
        for f in fields {
            cmd = cmd.arg(f.as_str());
        }
        let result: i64 = cmd.query(conn)?;
        Ok(result)
    }
}

/// SLOWLOG 原始条目（内部解析用，不含 Serialize）
pub struct SlowlogRaw {
    pub id: u64,
    pub time: u64,
    pub cost_ms: u64,
    pub cmd: String,
    pub args: Vec<String>,
    pub client: String,
}

/// 内存基本信息
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub used_memory: u64,
    pub used_memory_human: String,
    pub used_memory_peak: u64,
    pub used_memory_peak_human: String,
    pub mem_fragmentation_ratio: f64,
    pub maxmemory: u64,
}

/// 单个键的内存信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyMemoryItem {
    pub key: String,
    pub size: u64,
    pub size_human: String,
    pub key_type: String,
}

/// 键类型统计
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyTypeStat {
    pub key_type: String,
    pub count: usize,
    pub memory_bytes: u64,
    pub memory_percent: f64,
}

/// 单个数据库的键统计
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyStatItem {
    pub db: String,
    pub keys: usize,
    pub expires: usize,
    pub avg_ttl: u64,
}

/// 键内存扫描结果类型别名
pub type ScanKeysMemoryResult = Result<(Vec<KeyMemoryItem>, Vec<KeyTypeStat>, usize, u64), Box<dyn Error>>;

fn parse_slowlog_entry(items: Vec<redis::Value>) -> SlowlogRaw {
    let get_u64 = |idx: usize| -> u64 {
        if idx < items.len() {
            match &items[idx] {
                redis::Value::Int(n) => *n as u64,
                _ => 0,
            }
        } else { 0 }
    };
    let get_str = |idx: usize| -> String {
        if idx < items.len() {
            match &items[idx] {
                redis::Value::BulkString(s) => String::from_utf8_lossy(s).to_string(),
                redis::Value::Int(i) => i.to_string(),
                _ => String::new(),
            }
        } else { String::new() }
    };
    let get_args = |idx: usize| -> (String, Vec<String>) {
        if idx < items.len() {
            if let redis::Value::Array(argv) = &items[idx] {
                let mut argv_strs: Vec<String> = argv.iter().map(|a| match a {
                    redis::Value::BulkString(s) => String::from_utf8_lossy(s).to_string(),
                    redis::Value::Int(i) => i.to_string(),
                    _ => String::new(),
                }).collect();
                let cmd = if !argv_strs.is_empty() { argv_strs.remove(0) } else { String::new() };
                (cmd, argv_strs)
            } else { (String::new(), Vec::new()) }
        } else { (String::new(), Vec::new()) }
    };

    let id = get_u64(0);
    let time = get_u64(1);
    let cost_us = get_u64(2);
    let (cmd, args) = get_args(3);
    let client = get_str(4);

    SlowlogRaw {
        id, time,
        cost_ms: cost_us, // 直接返回微秒，与 Web 端保持一致
        cmd, args, client,
    }
}
