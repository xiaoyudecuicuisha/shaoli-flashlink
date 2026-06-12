//! 宠物配置存储：JSON 文件读写（~/.shaoli/pets/）

use super::PetConfig;
use std::fs;
use std::path::PathBuf;

fn pets_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".shaoli")
        .join("pets")
}

fn config_path() -> PathBuf {
    pets_dir().join("config.json")
}

fn custom_dir() -> PathBuf {
    pets_dir().join("custom")
}

/// 确保目录结构存在
pub fn ensure_dirs() {
    let _ = fs::create_dir_all(pets_dir());
    let _ = fs::create_dir_all(custom_dir());
}

/// 加载宠物配置
pub fn load_config() -> Result<PetConfig, String> {
    ensure_dirs();
    let path = config_path();
    if !path.exists() {
        return Ok(PetConfig::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("读取宠物配置失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析宠物配置失败: {}", e))
}

/// 保存宠物配置
pub fn save_config(config: &PetConfig) -> Result<(), String> {
    ensure_dirs();
    let path = config_path();
    let json = serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("写入宠物配置失败: {}", e))
}

/// 获取自定义宠物目录路径
pub fn get_custom_dir() -> PathBuf {
    ensure_dirs();
    custom_dir()
}

/// 删除自定义宠物文件
pub fn delete_custom_pet(file_name: &str) -> Result<(), String> {
    let path = custom_dir().join(file_name);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
    }
    Ok(())
}
