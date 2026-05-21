use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Mutex;
use std::sync::OnceLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    #[serde(default)]
    pub readonly: bool,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
    #[serde(default = "default_debug_log_enabled")]
    pub debug_log_enabled: bool,
}

fn default_debug_log_enabled() -> bool {
    false
}

/// 全局配置管理器缓存
pub static GLOBAL_CONFIG_MANAGER: OnceLock<Mutex<ConfigManager>> = OnceLock::new();

/// 获取全局配置管理器
pub fn get_global_config_manager() -> &'static Mutex<ConfigManager> {
    GLOBAL_CONFIG_MANAGER.get_or_init(|| {
        let manager = ConfigManager::new().expect("Failed to initialize config manager");
        Mutex::new(manager)
    })
}

/// 检查 debug 日志是否启用
pub fn is_debug_log_enabled() -> bool {
    let manager = get_global_config_manager().lock().unwrap();
    manager.get_debug_log_enabled()
}

/// Debug 日志打印宏
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::storage::config::is_debug_log_enabled() {
            eprintln!($($arg)*);
        }
    };
}

pub use crate::debug_println;

pub struct ConfigManager {
    config: Config,
    config_path: String,
}

impl ConfigManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_config(&config_path)?;
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    fn get_config_path() -> Result<String, Box<dyn std::error::Error>> {
        let home_dir = std::env::var("HOME").map_err(|e| format!("Failed to get home directory: {}", e))?;
        let config_dir = Path::new(&home_dir).join(".redis-helper");
        
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        let config_path = config_dir.join("config.json");
        Ok(config_path.to_str().unwrap().to_string())
    }
    
    fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        if Path::new(path).exists() {
            let mut file = File::open(path).map_err(|e| format!("Failed to open config file: {}", e))?;
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| format!("Failed to read config file: {}", e))?;
            
            let config: Config = serde_json::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))?;
            Ok(config)
        } else {
            // 返回默认配置
            Ok(Config { 
                servers: Vec::new(),
                debug_log_enabled: false,
            })
        }
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config).map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.config_path)
            .map_err(|e| format!("Failed to open config file for writing: {}", e))?;
        
        file.write_all(content.as_bytes()).map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }
    
    pub fn add_server(&mut self, server: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 检查是否已存在相同ID的服务器
        if self.config.servers.iter().any(|s| s.id == server.id) {
            return Err(Box::from("Server with this ID already exists"));
        }
        
        self.config.servers.push(server);
        Ok(())
    }
    
    pub fn edit_server(&mut self, server: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(index) = self.config.servers.iter().position(|s| s.id == server.id) {
            // 保持原有创建时间
            let created = self.config.servers[index].created.clone();
            self.config.servers[index] = ServerConfig {
                created,
                ..server
            };
            Ok(())
        } else {
            Err(Box::from("Server not found"))
        }
    }
    
    pub fn delete_server(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let initial_len = self.config.servers.len();
        self.config.servers.retain(|s| s.id != id);
        
        if self.config.servers.len() == initial_len {
            return Err(Box::from("Server not found"));
        }
        
        Ok(())
    }
    
    pub fn get_servers(&self) -> Vec<ServerConfig> {
        self.config.servers.clone()
    }
    
    pub fn set_servers(&mut self, servers: Vec<ServerConfig>) {
        self.config.servers = servers;
    }
    
    pub fn get_debug_log_enabled(&self) -> bool {
        self.config.debug_log_enabled
    }
    
    pub fn set_debug_log_enabled(&mut self, enabled: bool) {
        self.config.debug_log_enabled = enabled;
    }
}