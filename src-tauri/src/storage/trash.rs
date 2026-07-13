use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use chrono::{DateTime, Utc};

/// 废键项保存的连接信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrashConnection {
    pub host: String,
    pub port: u16,
    /// ACL 用户名（Redis >= 6.0），可选
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub password: Option<String>,
    pub db: u8,
}

/// 单个废键项
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrashItem {
    pub id: String,
    pub server_id: String,
    pub key: String,
    pub value: String,
    pub key_type: String,
    pub connection: TrashConnection,
    pub deleted_at: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrashData {
    pub items: Vec<TrashItem>,
}

pub struct TrashManager {
    data: TrashData,
    trash_path: String,
}

impl TrashManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let trash_path = Self::get_trash_path()?;
        let data = Self::load_trash(&trash_path)?;
        Ok(Self { data, trash_path })
    }

    fn get_trash_path() -> Result<String, Box<dyn std::error::Error>> {
        let config_dir = crate::storage::get_app_config_dir()?;
        let trash_path = config_dir.join("trash.json");
        Ok(trash_path.to_str().unwrap().to_string())
    }

    fn load_trash(path: &str) -> Result<TrashData, Box<dyn std::error::Error>> {
        if Path::new(path).exists() {
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let data: TrashData = serde_json::from_str(&content)?;
            Ok(data)
        } else {
            Ok(TrashData { items: Vec::new() })
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.data)?;
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&self.trash_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn add_item(&mut self, item: TrashItem) -> Result<(), Box<dyn std::error::Error>> {
        self.data.items.push(item);
        Ok(())
    }

    pub fn remove_item(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let initial_len = self.data.items.len();
        self.data.items.retain(|item| item.id != id);
        if self.data.items.len() == initial_len {
            return Err(Box::from("Trash item not found"));
        }
        Ok(())
    }

    pub fn remove_items(&mut self, ids: &[String]) -> Result<usize, Box<dyn std::error::Error>> {
        let id_set: std::collections::HashSet<String> = ids.iter().cloned().collect();
        let initial_len = self.data.items.len();
        self.data.items.retain(|item| !id_set.contains(&item.id));
        Ok(initial_len - self.data.items.len())
    }

    pub fn get_items_by_server(&self, server_id: &str) -> Vec<TrashItem> {
        self.data.items.iter().filter(|item| item.server_id == server_id).cloned().collect()
    }

    pub fn get_item(&self, id: &str) -> Option<TrashItem> {
        self.data.items.iter().find(|item| item.id == id).cloned()
    }

    pub fn cleanup_expired(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let initial_len = self.data.items.len();
        self.data.items.retain(|item| {
            if let Ok(expires_at) = DateTime::parse_from_rfc3339(&item.expires_at) {
                expires_at > now
            } else {
                true
            }
        });
        let removed = initial_len - self.data.items.len();
        if removed > 0 {
            self.save()?;
        }
        Ok(removed)
    }
}
