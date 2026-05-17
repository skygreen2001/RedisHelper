use redis::{Client, Commands, Connection};
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
