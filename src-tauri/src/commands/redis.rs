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