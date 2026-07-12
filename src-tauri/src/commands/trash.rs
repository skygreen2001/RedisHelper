use serde::{Deserialize, Serialize};
use crate::storage::trash::{TrashItem, TrashManager, TrashConnection};
use crate::redis::connection::RedisConnection;

fn get_server_id(host: &str, port: u16) -> String {
    format!("{}:{}", host, port)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveToTrashRequest {
    pub host: String,
    pub port: u16,
    /// ACL 用户名（Redis >= 6.0），可选
    #[serde(default)]
    pub username: Option<String>,
    pub password: Option<String>,
    pub db: u8,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchMoveToTrashRequest {
    pub host: String,
    pub port: u16,
    /// ACL 用户名（Redis >= 6.0），可选
    #[serde(default)]
    pub username: Option<String>,
    pub password: Option<String>,
    pub db: u8,
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreFromTrashRequest {
    pub trash_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchRestoreFromTrashRequest {
    pub trash_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermanentDeleteRequest {
    pub trash_ids: Vec<String>,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct TrashItemResponse {
    pub id: String,
    pub key: String,
    pub value: String,
    pub key_type: String,
    pub host: String,
    pub port: u16,
    pub db: u8,
    pub deleted_at: String,
    pub expires_at: String,
    pub is_expired: bool,
}

fn to_response(item: &TrashItem) -> TrashItemResponse {
    let is_expired = match chrono::DateTime::parse_from_rfc3339(&item.expires_at) {
        Ok(expires_at) => expires_at <= chrono::Utc::now(),
        Err(_) => false,
    };
    TrashItemResponse {
        id: item.id.clone(),
        key: item.key.clone(),
        value: item.value.clone(),
        key_type: item.key_type.clone(),
        host: item.connection.host.clone(),
        port: item.connection.port,
        db: item.connection.db,
        deleted_at: item.deleted_at.clone(),
        expires_at: item.expires_at.clone(),
        is_expired,
    }
}

#[tauri::command]
pub fn move_to_trash(req: MoveToTrashRequest) -> Result<bool, String> {
    let password_clone = req.password.clone();
    let username_clone = req.username.clone();
    let server_id = get_server_id(&req.host, req.port);

    let (value, key_type) = match RedisConnection::new_with_auth(&req.host, req.port, req.username, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            conn.get_key_value(&req.key).map_err(|e| e.to_string())?
        }
        Err(e) => return Err(e.to_string()),
    };

    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::days(7);
    let item = TrashItem {
        id: uuid::Uuid::new_v4().to_string(),
        server_id,
        key: req.key.clone(),
        value,
        key_type,
        connection: TrashConnection {
            host: req.host.clone(),
            port: req.port,
            username: username_clone.clone(),
            password: password_clone.clone(),
            db: req.db,
        },
        deleted_at: now.to_rfc3339(),
        expires_at: expires.to_rfc3339(),
    };

    match RedisConnection::new_with_auth(&req.host, req.port, username_clone, password_clone) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            conn.delete_key(&req.key).map_err(|e| e.to_string())?;
        }
        Err(e) => return Err(e.to_string()),
    }

    let mut manager = TrashManager::new().map_err(|e| e.to_string())?;
    manager.add_item(item).map_err(|e| e.to_string())?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn batch_move_to_trash(req: BatchMoveToTrashRequest) -> Result<u32, String> {
    let password_clone = req.password.clone();
    let username_clone = req.username.clone();
    let server_id = get_server_id(&req.host, req.port);

    let mut conn = match RedisConnection::new_with_auth(&req.host, req.port, req.username, req.password) {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };
    conn.select(req.db).map_err(|e| e.to_string())?;

    let mut manager = TrashManager::new().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::days(7);
    let mut success_count: u32 = 0;
    let mut saved_keys: Vec<String> = Vec::new();

    for key in &req.keys {
        let (value, key_type) = match conn.get_key_value(key) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let item = TrashItem {
            id: uuid::Uuid::new_v4().to_string(),
            server_id: server_id.clone(),
            key: key.clone(),
            value,
            key_type,
            connection: TrashConnection {
                host: req.host.clone(),
                port: req.port,
                username: username_clone.clone(),
                password: password_clone.clone(),
                db: req.db,
            },
            deleted_at: now.to_rfc3339(),
            expires_at: expires.to_rfc3339(),
        };
        if manager.add_item(item).is_ok() {
            saved_keys.push(key.clone());
            success_count += 1;
        }
    }

    if !saved_keys.is_empty() {
        conn.delete_keys(&saved_keys.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .map_err(|e| e.to_string())?;
    }

    manager.save().map_err(|e| e.to_string())?;
    Ok(success_count)
}

#[tauri::command]
pub fn get_trash_items(host: String, port: u16) -> Result<Vec<TrashItemResponse>, String> {
    let mut manager = TrashManager::new().map_err(|e| e.to_string())?;
    let server_id = get_server_id(&host, port);
    let _ = manager.cleanup_expired();
    let items = manager.get_items_by_server(&server_id);
    Ok(items.iter().map(to_response).collect())
}

#[tauri::command]
pub fn restore_from_trash(req: RestoreFromTrashRequest) -> Result<bool, String> {
    let mut manager = TrashManager::new().map_err(|e| e.to_string())?;
    let item = match manager.get_item(&req.trash_id) {
        Some(item) => item,
        None => return Err("Trash item not found".to_string()),
    };
    let mut conn = match RedisConnection::new_with_auth(&item.connection.host, item.connection.port, item.connection.username.clone(), item.connection.password.clone()) {
        Ok(c) => c,
        Err(e) => return Err(format!("无法连接到原 Redis 服务器: {}", e)),
    };
    conn.select(item.connection.db).map_err(|e| e.to_string())?;
    conn.set_key_value(&item.key, &item.value, &item.key_type)
        .map_err(|e| format!("恢复键值失败: {}", e))?;
    manager.remove_item(&req.trash_id).map_err(|e| e.to_string())?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn batch_restore_from_trash(req: BatchRestoreFromTrashRequest) -> Result<u32, String> {
    let mut manager = TrashManager::new().map_err(|e| e.to_string())?;
    let mut success_count: u32 = 0;
    for trash_id in &req.trash_ids {
        let item = match manager.get_item(trash_id) {
            Some(item) => item,
            None => continue,
        };
        let mut conn = match RedisConnection::new_with_auth(&item.connection.host, item.connection.port, item.connection.username.clone(), item.connection.password.clone()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if conn.select(item.connection.db).is_err() { continue; }
        if conn.set_key_value(&item.key, &item.value, &item.key_type).is_ok() {
            let _ = manager.remove_item(trash_id);
            success_count += 1;
        }
    }
    manager.save().map_err(|e| e.to_string())?;
    Ok(success_count)
}

#[tauri::command]
pub fn permanent_delete_trash(req: PermanentDeleteRequest) -> Result<u32, String> {
    let mut manager = TrashManager::new().map_err(|e| e.to_string())?;
    let count = manager.remove_items(&req.trash_ids).map_err(|e| e.to_string())?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(count as u32)
}

#[tauri::command]
pub fn clear_expired_trash() -> Result<u32, String> {
    let mut manager = TrashManager::new().map_err(|e| e.to_string())?;
    let count = manager.cleanup_expired().map_err(|e| e.to_string())?;
    Ok(count as u32)
}
