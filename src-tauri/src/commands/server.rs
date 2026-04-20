use serde::{Deserialize, Serialize};

use crate::storage::config::{ConfigManager, ServerConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerRequest {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConnectionRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub fn add_server(config: ServerRequest) -> Result<Vec<ServerConfig>, String> {
    let mut manager = ConfigManager::new().map_err(|e| e.to_string())?;
    manager.add_server(ServerConfig {
        id: config.id,
        name: config.name,
        host: config.host,
        port: config.port,
        password: config.password,
        db: config.db.unwrap_or(0),
        created: chrono::Local::now().to_string(),
        updated: chrono::Local::now().to_string(),
    }).map_err(|e| e.to_string())?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(manager.get_servers())
}

#[tauri::command]
pub fn edit_server(config: ServerRequest) -> Result<Vec<ServerConfig>, String> {
    let mut manager = ConfigManager::new().map_err(|e| e.to_string())?;
    manager.edit_server(ServerConfig {
        id: config.id,
        name: config.name,
        host: config.host,
        port: config.port,
        password: config.password,
        db: config.db.unwrap_or(0),
        created: "".to_string(), // 保持原有创建时间
        updated: chrono::Local::now().to_string(),
    }).map_err(|e| e.to_string())?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(manager.get_servers())
}

#[tauri::command]
pub fn delete_server(id: String) -> Result<Vec<ServerConfig>, String> {
    let mut manager = ConfigManager::new().map_err(|e| e.to_string())?;
    manager.delete_server(&id).map_err(|e| e.to_string())?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(manager.get_servers())
}

#[tauri::command]
pub fn get_servers() -> Result<Vec<ServerConfig>, String> {
    let manager = ConfigManager::new().map_err(|e| e.to_string())?;
    Ok(manager.get_servers())
}

#[tauri::command]
pub fn test_connection(req: TestConnectionRequest) -> Result<TestConnectionResponse, String> {
    use crate::redis::connection::RedisConnection;
    
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            match conn.ping() {
                Ok(_) => Ok(TestConnectionResponse {
                    success: true,
                    message: "连接成功".to_string(),
                }),
                Err(e) => Ok(TestConnectionResponse {
                    success: false,
                    message: format!("连接失败: {}", e),
                }),
            }
        }
        Err(e) => Ok(TestConnectionResponse {
            success: false,
            message: format!("连接失败: {}", e),
        }),
    }
}