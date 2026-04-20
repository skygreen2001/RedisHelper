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
        
        // 尝试检查0-15号数据库
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
    
    pub fn search_keys(&mut self, pattern: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let keys: Vec<String> = self.conn.keys(pattern)?;
        Ok(keys)
    }
}
