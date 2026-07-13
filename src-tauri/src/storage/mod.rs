pub mod config;
pub mod trash;

use std::path::PathBuf;

/// 跨平台获取用户主目录。
///
/// Windows 上不存在 `HOME` 环境变量，需要使用 `USERPROFILE`；
/// Linux/macOS 上则使用 `HOME`。这里按平台优先级依次尝试。
pub fn get_home_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Windows 上 USERPROFILE 是完整的主目录路径
    #[cfg(target_os = "windows")]
    {
        if let Ok(dir) = std::env::var("USERPROFILE") {
            if !dir.is_empty() {
                return Ok(PathBuf::from(dir));
            }
        }
    }

    // 通用回退：HOME（Linux/macOS，部分 Windows 环境也会设置）
    if let Ok(dir) = std::env::var("HOME") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    Err("Failed to get home directory: environment variable not found".into())
}

/// 获取应用配置目录 `~/.redis-helper`，不存在则创建。
pub fn get_app_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = get_home_dir()?.join(".redis-helper");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    Ok(config_dir)
}
