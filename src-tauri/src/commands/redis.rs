use serde::{Deserialize, Serialize};

use crate::redis::connection::RedisConnection;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValueRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    pub key: String,
    pub value: String,
    pub key_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    pub pattern: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValueResponse {
    pub key: String,
    pub value: String,
    pub key_type: String,
}

#[tauri::command]
pub fn connect(req: ConnectRequest) -> Result<bool, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            conn.ping().map_err(|e| e.to_string())?;
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_databases(req: ConnectRequest) -> Result<Vec<(u8, usize)>, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.get_databases().map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_keys(req: ConnectRequest) -> Result<Vec<String>, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            conn.get_keys().map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_key_value(req: KeyRequest) -> Result<KeyValueResponse, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            let (value, key_type) = conn.get_key_value(&req.key).map_err(|e| e.to_string())?;
            Ok(KeyValueResponse {
                key: req.key,
                value,
                key_type,
            })
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_key_value(req: KeyValueRequest) -> Result<bool, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            conn.set_key_value(&req.key, &req.value, &req.key_type).map_err(|e| e.to_string())?;
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn delete_key(req: KeyRequest) -> Result<bool, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            conn.delete_key(&req.key).map_err(|e| e.to_string())?;
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn search_keys(req: SearchRequest) -> Result<Vec<String>, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            conn.search_keys(&req.pattern).map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn create_database(req: ConnectRequest) -> Result<bool, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            // Redis会自动创建不存在的数据库，只需要切换到该数据库即可
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn delete_database(req: ConnectRequest) -> Result<bool, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            // 清空数据库
            conn.flushdb().map_err(|e| e.to_string())?;
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn flush_database(req: ConnectRequest) -> Result<bool, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            // 清空数据库
            conn.flushdb().map_err(|e| e.to_string())?;
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn generate_test_data(req: ConnectRequest, count: u32) -> Result<bool, String> {
    use rand::Rng;
    
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            
            let mut rng = rand::thread_rng();
            let key_types = vec!["string", "hash", "list", "set"];
            
            for i in 0..count {
                let key_type = key_types[rng.gen_range(0..key_types.len())];
                let key_name = format!("test_key_{}_{}", i, rng.gen_range(1000..9999));
                
                match key_type {
                    "string" => {
                        let value = format!("value_{}", rng.gen_range(1..10000));
                        conn.set_key_value(&key_name, &value, "string").map_err(|e| e.to_string())?;
                    }
                    "hash" => {
                        let json_value = format!(
                            "{{\"field1\":\"value{}\",\"field2\":\"value{}\"}}",
                            rng.gen_range(1..100),
                            rng.gen_range(1..100)
                        );
                        conn.set_key_value(&key_name, &json_value, "hash").map_err(|e| e.to_string())?;
                    }
                    "list" => {
                        let json_value = format!(
                            "[\"item{}\",\"item{}\",\"item{}\"]",
                            rng.gen_range(1..100),
                            rng.gen_range(1..100),
                            rng.gen_range(1..100)
                        );
                        conn.set_key_value(&key_name, &json_value, "list").map_err(|e| e.to_string())?;
                    }
                    "set" => {
                        let json_value = format!(
                            "[\"member{}\",\"member{}\",\"member{}\"]",
                            rng.gen_range(1..100),
                            rng.gen_range(1..100),
                            rng.gen_range(1..100)
                        );
                        conn.set_key_value(&key_name, &json_value, "set").map_err(|e| e.to_string())?;
                    }
                    _ => {}
                }
            }
            
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}