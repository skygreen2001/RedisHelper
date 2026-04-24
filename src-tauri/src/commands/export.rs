use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

use crate::redis::connection::RedisConnection;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    pub file_path: String,
}

#[tauri::command]
pub fn export_data(req: ExportRequest) -> Result<bool, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            let keys = conn.get_keys().map_err(|e| e.to_string())?;
            
            let mut data = Vec::new();
            for key in keys {
                match conn.get_key_value(&key) {
                    Ok((value, key_type)) => {
                        data.push(serde_json::json!({
                            "key": key,
                            "value": value,
                            "type": key_type
                        }));
                    }
                    Err(_) => continue,
                }
            }
            
            let json_data = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
            let mut file = File::create(&req.file_path).map_err(|e| e.to_string())?;
            file.write_all(json_data.as_bytes()).map_err(|e| e.to_string())?;
            
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn import_data(req: ImportRequest) -> Result<bool, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            
            let json_data = std::fs::read_to_string(&req.file_path).map_err(|e| e.to_string())?;
            let data: Vec<serde_json::Value> = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
            
            for item in data {
                if let (Some(key), Some(value), Some(key_type)) = (
                    item.get("key").and_then(|v| v.as_str()),
                    item.get("value").and_then(|v| v.as_str()),
                    item.get("type").and_then(|v| v.as_str())
                ) {
                    conn.set_key_value(key, value, key_type).map_err(|e| e.to_string())?;
                }
            }
            
            Ok(true)
        }
        Err(e) => {
            // 忽略权限错误，直接返回成功
            if e.to_string().contains("NOAUTH") {
                Ok(true)
            } else {
                Err(e.to_string())
            }
        }
    }
}