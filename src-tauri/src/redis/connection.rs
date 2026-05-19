use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct RedisConnection {
    conn: Connection,
}

impl RedisConnection {
    pub fn new(host: &str, port: u16, password: Option<String>) -> Result<Self, Box<dyn Error>> {
        let url = match &password {
            Some(pass) => format!("redis://:{}@{}:{}", pass, host, port),
            None => format!("redis://{}:{}", host, port),
        };
        
        let client = Client::open(url)?;
        let mut conn = client.get_connection()?;
        
        // 发送 AUTH 命令如果有密码
        if let Some(pass) = password {
            let _: () = redis::cmd("AUTH").arg(pass).query(&mut conn)?;
        }
        
        Ok(Self { conn })
    }
    
    pub fn select(&mut self, db: u8) -> Result<(), Box<dyn Error>> {
        let _: () = redis::cmd("SELECT").arg(db).query(&mut self.conn)?;
        Ok(())
    }
    
    pub fn ping(&mut self) -> Result<(), Box<dyn Error>> {
        let _: String = redis::cmd("PING").query(&mut self.conn)?;
        Ok(())
    }
    
    pub fn get_databases(&mut self) -> Result<Vec<(u8, usize)>, Box<dyn Error>> {
        let mut databases = Vec::new();
        
        // 尝试检查0-15号数据库，只返回有key的数据库
        for db in 0..16 {
            self.select(db)?;
            let keys: Vec<String> = self.conn.keys("*")?;
            if !keys.is_empty() {
                databases.push((db, keys.len()));
            }
        }
        
        Ok(databases)
    }
    
    pub fn get_keys(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let keys: Vec<String> = self.conn.keys("*")?;
        Ok(keys)
    }
    
    pub fn get_key_value(&mut self, key: &str) -> Result<(String, String), Box<dyn Error>> {
        let key_type: String = redis::cmd("TYPE").arg(key).query(&mut self.conn)?;
        
        let value = match key_type.as_str() {
            "string" => {
                let val: Option<String> = self.conn.get(key)?;
                val.unwrap_or_else(|| "".to_string())
            }
            "list" => {
                let val: Vec<String> = self.conn.lrange(key, 0, -1)?;
                serde_json::to_string(&val)?
            }
            "set" => {
                let val: Vec<String> = self.conn.smembers(key)?;
                serde_json::to_string(&val)?
            }
            "zset" => {
                let val: Vec<(String, f64)> = self.conn.zrange_withscores(key, 0, -1)?;
                serde_json::to_string(&val)?
            }
            "hash" => {
                let val: std::collections::HashMap<String, String> = self.conn.hgetall(key)?;
                serde_json::to_string(&val)?
            }
            _ => "".to_string(),
        };
        
        Ok((value, key_type))
    }
    
    pub fn set_key_value(&mut self, key: &str, value: &str, key_type: &str) -> Result<(), Box<dyn Error>> {
        match key_type {
            "string" => {
                let _: () = self.conn.set(key, value)?;
            }
            "list" => {
                // 先删除旧值
                let _: () = self.conn.del::<_, ()>(key)?;
                // 解析JSON数组
                let values: Vec<String> = serde_json::from_str(value)?;
                for val in values {
                    let _: () = self.conn.lpush::<_, _, ()>(key, val)?;
                }
            }
            "set" => {
                // 先删除旧值
                let _: () = self.conn.del::<_, ()>(key)?;
                // 解析JSON数组
                let values: Vec<String> = serde_json::from_str(value)?;
                for val in values {
                    let _: () = self.conn.sadd::<_, _, ()>(key, val)?;
                }
            }
            "zset" => {
                // 先删除旧值
                let _: () = self.conn.del::<_, ()>(key)?;
                // 解析JSON数组
                let values: Vec<(String, f64)> = serde_json::from_str(value)?;
                for (val, score) in values {
                    let _: () = self.conn.zadd::<_, _, _, ()>(key, val, score)?;
                }
            }
            "hash" => {
                // 先删除旧值
                let _: () = self.conn.del::<_, ()>(key)?;
                // 解析JSON对象
                let values: std::collections::HashMap<String, String> = serde_json::from_str(value)?;
                for (field, val) in values {
                    let _: () = self.conn.hset::<_, _, _, ()>(key, field, val)?;
                }
            }
            _ => {
                return Err(Box::from("Unsupported key type"));
            }
        }
        
        Ok(())
    }
    
    pub fn delete_key(&mut self, key: &str) -> Result<(), Box<dyn Error>> {
        let _: () = self.conn.del::<_, ()>(key)?;
        Ok(())
    }

    pub fn delete_keys(&mut self, keys: &[&str]) -> Result<(), Box<dyn Error>> {
        if keys.is_empty() {
            return Ok(());
        }
        let _: () = redis::cmd("DEL").arg(keys).query(&mut self.conn)?;
        Ok(())
    }
    
    pub fn search_keys(&mut self, pattern: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let keys: Vec<String> = self.conn.keys(pattern)?;
        Ok(keys)
    }
    
    pub fn flushdb(&mut self) -> Result<(), Box<dyn Error>> {
        let _: () = redis::cmd("FLUSHDB").query(&mut self.conn)?;
        Ok(())
    }

    /// 获取 SLOWLOG 历史记录（只读，不修改 Redis 配置）
    /// Redis 7+ 返回格式: [id, timestamp_us, duration_us, [cmd, args...], client_addr, client_name]
    pub fn slowlog_get(&mut self) -> Result<Vec<SlowlogRaw>, Box<dyn Error>> {
        let raw: Vec<redis::Value> = redis::cmd("SLOWLOG").arg("GET").arg(9999).query(&mut self.conn)?;
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

    /// 获取内存基本信息
    pub fn get_memory_info(&mut self) -> Result<MemoryInfo, Box<dyn Error>> {
        let info_str: String = redis::cmd("INFO")
            .arg("memory")
            .query(&mut self.conn)?;
        
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

    /// 扫描并获取所有键的内存信息
    pub fn scan_keys_memory(&mut self) -> Result<(Vec<KeyMemoryItem>, Vec<KeyTypeStat>, usize), Box<dyn Error>> {
        let mut key_memory_list = Vec::new();
        let mut type_stats: std::collections::HashMap<String, (usize, u64)> = std::collections::HashMap::new();
        let mut cursor = 0;
        let mut total_keys = 0;
        
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("COUNT")
                .arg(100)
                .query(&mut self.conn)?;
            
            cursor = new_cursor;
            total_keys += keys.len();
            
            for key in keys {
                let key_type: String = redis::cmd("TYPE")
                    .arg(&key)
                    .query(&mut self.conn)?;
                
                let size: Option<u64> = redis::cmd("MEMORY")
                    .arg("USAGE")
                    .arg(&key)
                    .query(&mut self.conn)
                    .ok();
                
                let size = size.unwrap_or(0);
                
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
            
            if cursor == 0 {
                break;
            }
        }
        
        key_memory_list.sort_by(|a, b| b.size.cmp(&a.size));
        
        let large_keys_count = key_memory_list.len();
        // 不截断列表，让前端分页处理
        // key_memory_list.truncate(100);
        
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
        
        Ok((key_memory_list, key_type_stats, total_keys))
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
        cost_ms: cost_us / 1000,
        cmd, args, client,
    }
}
