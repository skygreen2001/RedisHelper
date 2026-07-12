/**
 * Redis 审计模块
 * 使用 Redis List 存储审计日志（兼容 Redis 4.x）
 */
use crate::storage::config::debug_println;
use redis::{Client};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 审计条目结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: i64,
    pub server_id: String,
    pub server_name: String,
    pub db: u8,
    pub client_ip: String,
    pub command: String,
    pub args: Vec<String>,
    pub cost_ms: i64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// 命令统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandStats {
    pub command: String,
    pub count: i64,
    pub total_cost_ms: i64,
    pub avg_cost_ms: f64,
    pub success_count: i64,
    pub error_count: i64,
    pub success_rate: f64,
}

/// 审计配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub list_key: String,
    pub max_len: usize,
    pub enabled: bool,
    pub filter_commands: Vec<String>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            list_key: "redis:audit:logs".to_string(),
            max_len: 1_000_000,
            enabled: true,
            filter_commands: vec![
                "PING".to_string(),
                "INFO".to_string(),
                "CLIENT".to_string(),
                "SLOWLOG".to_string(),
                "DEBUG".to_string(),
                "CONFIG".to_string(),
                "DBSIZE".to_string(),
                "SELECT".to_string(),
                "ECHO".to_string(),
            ],
        }
    }
}

/// 审计上下文信息
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuditContext {
    pub server_id: String,
    pub server_name: String,
    pub db: u8,
    pub client_ip: String,
}

/// 简单的审计器（简化版）
#[allow(dead_code)]
pub struct Auditor {
    config: AuditConfig,
}

#[allow(dead_code)]
impl Auditor {
    pub fn new(config: AuditConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &AuditConfig {
        &self.config
    }

    /// 检查命令是否应该被过滤
    pub fn should_filter(&self, command: &str) -> bool {
        self.config.filter_commands.iter()
            .any(|cmd| cmd.eq_ignore_ascii_case(command))
    }

    /// 拦截命令并记录（同步版本）
    pub fn intercept<F, T>(
        &self,
        redis_url: &str,
        context: &AuditContext,
        command: &str,
        args: Vec<String>,
        execute: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Result<T, Box<dyn std::error::Error + Send + Sync>>,
    {
        // 检查是否启用审计
        if !self.config.enabled || self.should_filter(command) {
            return execute();
        }

        let start = Instant::now();
        let result = execute();
        let cost_ms = start.elapsed().as_millis() as i64;

        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            server_id: context.server_id.clone(),
            server_name: context.server_name.clone(),
            db: context.db,
            client_ip: context.client_ip.clone(),
            command: command.to_string(),
            args: args.clone(),
            cost_ms,
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
        };

        // 同步写入（简单的实现）
        if let Err(e) = write_entry_to_list_sync(redis_url, &entry, self.config.max_len) {
            eprintln!("[AUDIT] Failed to write entry: {}", e);
        }

        result
    }

    /// 更新配置
    pub fn update_config(&mut self, enabled: bool, filter_commands: Vec<String>) {
        self.config.enabled = enabled;
        self.config.filter_commands = filter_commands;
    }
}

/// 同步写入单条记录到 Redis List（使用 LPUSH + LTRIM）
pub fn write_entry_to_list_sync(
    redis_url: &str,
    entry: &AuditEntry,
    max_len: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug_println!("[AUDIT WRITE] Attempting to write audit entry to Redis: {}", redis_url);
    
    let client = Client::open(redis_url)?;
    let mut conn = client.get_connection()?;
    
    // 序列化为 JSON
    let json = serde_json::to_string(entry)?;
    debug_println!("[AUDIT WRITE] Serialized entry: {} bytes", json.len());
    
    // LPUSH 新记录
    let _: () = redis::cmd("LPUSH")
        .arg("redis:audit:logs")
        .arg(&json)
        .query(&mut conn)?;
    debug_println!("[AUDIT WRITE] LPUSH successful");
    
    // LTRIM 保持列表长度
    let _: () = redis::cmd("LTRIM")
        .arg("redis:audit:logs")
        .arg(0)
        .arg(max_len as isize - 1)
        .query(&mut conn)?;
    debug_println!("[AUDIT WRITE] LTRIM successful, max_len: {}", max_len);
    
    Ok(())
}

/// 查询审计日志（使用 LRANGE）
#[allow(clippy::too_many_arguments)]
pub async fn get_audit_logs(
    redis_url: &str,
    list_key: &str,
    server_id: Option<&str>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    command: Option<&str>,
    limit: usize,
    offset: usize,
) -> Result<Vec<AuditEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    // 获取列表长度
    let list_len: usize = redis::cmd("LLEN")
        .arg(list_key)
        .query_async(&mut conn)
        .await?;

    if list_len == 0 {
        return Ok(Vec::new());
    }

    // 计算查询范围
    let start_idx = offset;
    let end_idx = (offset + limit).min(list_len) - 1;

    // 使用 LRANGE 获取记录（从新到旧）
    let raw_entries: Vec<String> = redis::cmd("LRANGE")
        .arg(list_key)
        .arg(start_idx as isize)
        .arg(end_idx as isize)
        .query_async(&mut conn)
        .await?;

    let mut entries: Vec<AuditEntry> = Vec::new();

    for json in raw_entries {
        if let Ok(entry) = serde_json::from_str::<AuditEntry>(&json) {
            // 应用过滤条件
            if let Some(sid) = server_id {
                if entry.server_id != sid {
                    continue;
                }
            }
            
            if let Some(st) = start_time {
                if entry.timestamp < st {
                    continue;
                }
            }
            
            if let Some(et) = end_time {
                if entry.timestamp > et {
                    continue;
                }
            }
            
            if let Some(cmd) = command {
                if entry.command != cmd {
                    continue;
                }
            }

            entries.push(entry);
        }
    }

    // 排序：按时间倒序（LRANGE 已经是按时间倒序的）
    
    Ok(entries)
}

/// 获取命令统计
pub async fn get_command_stats(
    redis_url: &str,
    list_key: &str,
    server_id: Option<&str>,
) -> Result<Vec<CommandStats>, Box<dyn std::error::Error + Send + Sync>> {
    // 获取最近 10000 条记录进行统计
    let logs = get_audit_logs(
        redis_url,
        list_key,
        server_id,
        None,
        None,
        None,
        10000,
        0,
    )
    .await?;

    let mut stats_map: std::collections::HashMap<String, CommandStats> = 
        std::collections::HashMap::new();

    for log in logs {
        let entry = stats_map.entry(log.command.clone()).or_insert(CommandStats {
            command: log.command,
            count: 0,
            total_cost_ms: 0,
            avg_cost_ms: 0.0,
            success_count: 0,
            error_count: 0,
            success_rate: 0.0,
        });

        entry.count += 1;
        entry.total_cost_ms += log.cost_ms;
        if log.success {
            entry.success_count += 1;
        } else {
            entry.error_count += 1;
        }
    }

    // 计算平均值
    let stats: Vec<CommandStats> = stats_map
        .into_values()
        .map(|mut s| {
            s.avg_cost_ms = if s.count > 0 {
                s.total_cost_ms as f64 / s.count as f64
            } else {
                0.0
            };
            s.success_rate = if s.count > 0 {
                s.success_count as f64 / s.count as f64 * 100.0
            } else {
                0.0
            };
            s
        })
        .collect();

    Ok(stats)
}

/// 清空审计日志
pub async fn clear_audit_logs(
    redis_url: &str,
    list_key: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    // 删除 List
    let deleted: usize = redis::cmd("DEL")
        .arg(list_key)
        .query_async(&mut conn)
        .await?;
    
    println!("[AUDIT CLEAR] Deleted {} keys for list: {}", deleted, list_key);
    
    Ok(())
}
