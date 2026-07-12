/**
 * 审计相关的 Tauri 命令
 * 使用 Redis List 存储审计日志
 */
use serde::Deserialize;
use crate::redis::audit::{
    AuditEntry, AuditConfig, CommandStats,
    get_audit_logs, get_command_stats, clear_audit_logs,
};
use crate::storage::config::debug_println;

/// 审计查询请求
#[derive(Debug, Deserialize)]
pub struct AuditLogRequest {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: Option<String>,
    pub password: Option<String>,
    pub server_id: Option<String>,
    #[allow(dead_code)]
    pub server_name: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub command: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 审计统计请求
#[derive(Debug, Deserialize)]
pub struct AuditStatsRequest {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: Option<String>,
    pub password: Option<String>,
    pub server_id: Option<String>,
}

fn build_redis_url(host: &str, port: u16, username: Option<&str>, password: Option<&str>) -> String {
    match (username, password) {
        (Some(user), Some(pass)) if !user.is_empty() && !pass.is_empty() => {
            format!("redis://{}:{}@{}:{}", user, pass, host, port)
        }
        (Some(user), _) if !user.is_empty() => {
            format!("redis://{}@{}:{}", user, host, port)
        }
        (_, Some(pass)) if !pass.is_empty() => {
            format!("redis://:{}@{}:{}", pass, host, port)
        }
        _ => {
            format!("redis://{}:{}", host, port)
        }
    }
}

/// 获取审计日志
#[tauri::command]
pub async fn audit_get_logs(req: AuditLogRequest) -> Result<Vec<AuditEntry>, String> {
    let redis_url = build_redis_url(&req.host, req.port, req.username.as_deref(), req.password.as_deref());
    let list_key = "redis:audit:logs";
    
    debug_println!("[AUDIT] Getting logs from {}:{}", req.host, req.port);
    
    let logs = get_audit_logs(
        &redis_url,
        list_key,
        req.server_id.as_deref(),
        req.start_time,
        req.end_time,
        req.command.as_deref(),
        req.limit.unwrap_or(50),
        req.offset.unwrap_or(0),
    )
    .await
    .map_err(|e| e.to_string())?;
    
    debug_println!("[AUDIT] Found {} log entries", logs.len());
    
    Ok(logs)
}

/// 获取审计统计
#[tauri::command]
pub async fn audit_get_stats(req: AuditStatsRequest) -> Result<Vec<CommandStats>, String> {
    let redis_url = build_redis_url(&req.host, req.port, req.username.as_deref(), req.password.as_deref());
    let list_key = "redis:audit:logs";
    
    debug_println!("[AUDIT] Getting stats from {}:{}", req.host, req.port);
    
    let stats = get_command_stats(
        &redis_url,
        list_key,
        req.server_id.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;
    
    debug_println!("[AUDIT] Found {} command types", stats.len());
    
    Ok(stats)
}

/// 清空审计日志
#[tauri::command]
pub async fn audit_clear(
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
) -> Result<bool, String> {
    let redis_url = build_redis_url(&host, port, username.as_deref(), password.as_deref());
    let list_key = "redis:audit:logs";
    
    debug_println!("[AUDIT] Clearing logs from {}:{}", host, port);
    
    clear_audit_logs(&redis_url, list_key)
        .await
        .map_err(|e| e.to_string())?;
    
    debug_println!("[AUDIT] Logs cleared successfully");
    
    Ok(true)
}

/// 获取审计配置
#[tauri::command]
pub fn audit_get_config() -> Result<AuditConfig, String> {
    // 返回默认配置
    Ok(AuditConfig::default())
}

/// 更新审计配置（仅内存中保存，实际需要持久化）
#[tauri::command]
pub fn audit_update_config(enabled: bool, filter_commands: Vec<String>) -> Result<bool, String> {
    debug_println!("[AUDIT] Config updated: enabled={}, filter_count={}", enabled, filter_commands.len());
    Ok(true)
}

/// 测试审计功能（生成测试数据）
#[tauri::command]
pub async fn audit_generate_test_data(
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    count: Option<usize>,
) -> Result<usize, String> {
    let redis_url = build_redis_url(&host, port, username.as_deref(), password.as_deref());
    let list_key = "redis:audit:logs";
    
    let count = count.unwrap_or(100);
    
    debug_println!("[AUDIT] Generating {} test entries", count);
    
    // 创建测试数据
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|e| e.to_string())?;
    let mut conn = client.get_multiplexed_async_connection().await
        .map_err(|e| e.to_string())?;
    
    let commands = ["GET", "SET", "DEL", "HSET", "HGET", "LPUSH", "LRANGE"];
    let keys = ["user:1", "session:abc", "cache:data", "queue:jobs", "config:app"];
    
    for i in 0..count {
        let cmd = commands[i % commands.len()];
        let key = keys[i % keys.len()];
        let timestamp = chrono::Utc::now().timestamp_millis() - (i as i64 * 1000);
        
        let args_json = if cmd == "SET" || cmd == "HSET" {
            let key_str: &str = key;
            let value = format!("value_{}", i);
            serde_json::to_string(&[key_str, value.as_str()]).unwrap()
        } else {
            let key_str: &str = key;
            serde_json::to_string(&[key_str]).unwrap()
        };
        
        let id = uuid::Uuid::new_v4().to_string();
        
        // 构建审计条目（使用结构体序列化，确保字段名正确）
        let entry = AuditEntry {
            id,
            timestamp,
            server_id: "test-server".to_string(),
            server_name: "测试服务器".to_string(),
            db: 0,
            client_ip: "127.0.0.1".to_string(),
            command: cmd.to_string(),
            args: serde_json::from_str::<Vec<String>>(&args_json).unwrap_or_default(),
            cost_ms: (i % 50) as i64,
            success: true,
            error_message: None,
        };
        
        // 序列化为 JSON
        let entry_json = serde_json::to_string(&entry).unwrap();
        
        // LPUSH 到列表
        let _: () = redis::cmd("LPUSH")
            .arg(list_key)
            .arg(entry_json)
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    // 修剪列表保持长度
    let _: () = redis::cmd("LTRIM")
        .arg(list_key)
        .arg(0)
        .arg(1_000_000isize - 1)
        .query_async(&mut conn)
        .await
        .map_err(|e| e.to_string())?;
    
    debug_println!("[AUDIT] Generated {} test entries successfully", count);
    
    Ok(count)
}
