use serde::{Deserialize, Serialize};

use crate::redis::connection::{RedisConnection, SlowlogRaw, KeyMemoryItem, KeyTypeStat, MemoryInfo, KeyStatItem, KeysResponse};
use crate::storage::config::debug_println;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    #[serde(default)]
    pub limit: Option<usize>, // 可选的键数量限制，用于分页加载
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
    let t0 = std::time::Instant::now();
    debug_println!("[DEBUG] connect: 建立连接 {}:{}", req.host, req.port);
    // 使用连接池复用连接
    match RedisConnection::from_cache(&req.host, req.port, req.password.as_deref(), req.db) {
        Ok(mut conn) => {
            let t1 = std::time::Instant::now();
            debug_println!("[DEBUG] connect: 获取/创建连接成功，耗时 {:?}", t1 - t0);
            conn.ping().map_err(|e| e.to_string())?;
            let t2 = std::time::Instant::now();
            debug_println!("[DEBUG] connect: PING 成功，耗时 {:?}", t2 - t1);
            debug_println!("[DEBUG] connect: 总耗时 {:?}", t2 - t0);
            Ok(true)
        }
        Err(e) => {
            let t_err = std::time::Instant::now();
            debug_println!("[DEBUG] connect: 连接失败，耗时 {:?}, error: {}", t_err - t0, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn get_databases(req: ConnectRequest) -> Result<Vec<(u8, usize)>, String> {
    let t0 = std::time::Instant::now();
    debug_println!("[DEBUG] get_databases: 开始获取数据库列表 {}:{}", req.host, req.port);
    // 使用连接池复用连接
    match RedisConnection::from_cache(&req.host, req.port, req.password.as_deref(), 0) {
        Ok(mut conn) => {
            let t1 = std::time::Instant::now();
            debug_println!("[DEBUG] get_databases: 获取/创建连接成功，耗时 {:?}", t1 - t0);
            let result = conn.get_databases();
            let t2 = std::time::Instant::now();
            debug_println!("[DEBUG] get_databases: INFO keyspace 成功，耗时 {:?}", t2 - t1);
            debug_println!("[DEBUG] get_databases: 总耗时 {:?}", t2 - t0);
            result.map_err(|e| e.to_string())
        }
        Err(e) => {
            let t_err = std::time::Instant::now();
            debug_println!("[DEBUG] get_databases: 连接失败，耗时 {:?}, error: {}", t_err - t0, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn get_keys(req: ConnectRequest) -> Result<KeysResponse, String> {
    let t0 = std::time::Instant::now();
    let limit_str = req.limit.map(|l| l.to_string()).unwrap_or_else(|| "无限制".to_string());
    debug_println!("[DEBUG] get_keys: 开始获取键列表 {}:{}/{} (限制: {})", req.host, req.port, req.db, limit_str);
    // 使用连接池复用连接
    match RedisConnection::from_cache(&req.host, req.port, req.password.as_deref(), req.db) {
        Ok(mut conn) => {
            let t1 = std::time::Instant::now();
            debug_println!("[DEBUG] get_keys: 获取/创建连接成功，耗时 {:?}", t1 - t0);
            let result = conn.get_keys(req.limit);
            let t2 = std::time::Instant::now();
            match &result {
                Ok(resp) => {
                    debug_println!("[DEBUG] get_keys: SCAN 成功，获取 {} 个键（总计 {} 个），耗时 {:?}", 
                        resp.keys.len(), resp.total, t2 - t1);
                }
                Err(e) => {
                    debug_println!("[DEBUG] get_keys: SCAN 失败，耗时 {:?}, error: {}", t2 - t1, e);
                }
            }
            debug_println!("[DEBUG] get_keys: 总耗时 {:?}", t2 - t0);
            result.map_err(|e| e.to_string())
        }
        Err(e) => {
            let t_err = std::time::Instant::now();
            debug_println!("[DEBUG] get_keys: 连接失败，耗时 {:?}, error: {}", t_err - t0, e);
            Err(e.to_string())
        }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SlowlogRequest {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

/// 可序列化的 SLOWLOG 条目（Tauri IPC 传输用）
#[derive(Debug, Serialize, Deserialize)]
pub struct SlowlogEntry {
    pub id: u64,
    pub time: u64,
    pub cost_ms: u64,
    pub cmd: String,
    pub args: Vec<String>,
    pub client: String,
}

impl From<SlowlogRaw> for SlowlogEntry {
    fn from(raw: SlowlogRaw) -> Self {
        Self {
            id: raw.id,
            time: raw.time,
            cost_ms: raw.cost_ms,
            cmd: raw.cmd,
            args: raw.args,
            client: raw.client,
        }
    }
}

/// 噪音命令黑名单（与 ws-proxy 保持一致）
const SLOWLOG_NOISE_CMDS: &[&str] = &[
    "PING", "CLIENT", "AUTH", "CONFIG", "INFO",
    "COMMAND", "SLOWLOG", "MONITOR",
];

#[tauri::command]
pub fn slowlog_get(req: SlowlogRequest) -> Result<Vec<SlowlogEntry>, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            let raw_entries = conn.slowlog_get().map_err(|e| e.to_string())?;
            let filtered: Vec<SlowlogEntry> = raw_entries
                .into_iter()
                .filter(|e| !SLOWLOG_NOISE_CMDS.contains(&e.cmd.to_uppercase().as_str()))
                .map(SlowlogEntry::from)
                .collect();
            Ok(filtered)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 内存分析响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfoResponse {
    pub used_memory: u64,
    pub used_memory_human: String,
    pub used_memory_peak: u64,
    pub used_memory_peak_human: String,
    pub mem_fragmentation_ratio: f64,
    pub maxmemory: u64,
    pub keys_count: usize,
    pub expired_keys_ratio: f64,
    pub large_keys_count: usize,
    pub key_memory_list: Vec<KeyMemoryItem>,
    pub key_type_stats: Vec<KeyTypeStat>,
}

impl From<MemoryInfo> for MemoryInfoResponse {
    fn from(info: MemoryInfo) -> Self {
        Self {
            used_memory: info.used_memory,
            used_memory_human: info.used_memory_human,
            used_memory_peak: info.used_memory_peak,
            used_memory_peak_human: info.used_memory_peak_human,
            mem_fragmentation_ratio: info.mem_fragmentation_ratio,
            maxmemory: info.maxmemory,
            keys_count: 0,
            expired_keys_ratio: 0.0,
            large_keys_count: 0,
            key_memory_list: vec![],
            key_type_stats: vec![],
        }
    }
}

#[tauri::command]
pub fn get_memory_info(req: ConnectRequest) -> Result<MemoryInfoResponse, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            
            let memory_info = conn.get_memory_info().map_err(|e| e.to_string())?;
            
            let (key_memory_list, key_type_stats, keys_count) = 
                conn.scan_keys_memory().map_err(|e| e.to_string())?;
            
            let expired_keys_ratio = if keys_count > 0 {
                (key_memory_list.iter().filter(|item| item.size > 0).count() as f64 / keys_count as f64) * 100.0
            } else {
                0.0
            };
            
            let mut response: MemoryInfoResponse = MemoryInfoResponse::from(memory_info);
            response.keys_count = keys_count;
            response.expired_keys_ratio = expired_keys_ratio;
            response.large_keys_count = key_memory_list.len();
            response.key_memory_list = key_memory_list;
            response.key_type_stats = key_type_stats;
            
            Ok(response)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 获取完整的服务器信息
#[tauri::command]
pub fn get_server_info(req: ConnectRequest) -> Result<std::collections::HashMap<String, String>, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.select(req.db).map_err(|e| e.to_string())?;
            conn.get_server_info().map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 获取键值统计
#[tauri::command]
pub fn get_key_stats(req: ConnectRequest) -> Result<Vec<KeyStatItem>, String> {
    match RedisConnection::new(&req.host, req.port, req.password) {
        Ok(mut conn) => {
            conn.get_key_stats().map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}