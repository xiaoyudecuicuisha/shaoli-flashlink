//! 白板存储层：JSON 文件读写、目录管理、文件名清理

use std::fs;
use std::path::PathBuf;
use chrono::Local;

use super::{WhiteboardData, WhiteboardInfo};

/// 获取白板存储目录
/// 优先使用 custom_dir，否则默认 {desktop}/白板/
pub fn get_whiteboard_dir(custom_dir: Option<&str>) -> PathBuf {
    if let Some(dir) = custom_dir {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    let desktop = dirs::desktop_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
    desktop.join("白板")
}

/// 确保目录存在
fn ensure_dir(custom_dir: Option<&str>) -> Result<PathBuf, String> {
    let dir = get_whiteboard_dir(custom_dir);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    Ok(dir)
}

/// 获取白板文件路径
fn get_board_path(name: &str, custom_dir: Option<&str>) -> PathBuf {
    let safe_name = sanitize_filename(name);
    get_whiteboard_dir(custom_dir).join(format!("{}.json", safe_name))
}

/// 清理文件名中的非法字符
fn sanitize_filename(name: &str) -> String {
    let invalid = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
    name.chars()
        .map(|c| if invalid.contains(&c) { '_' } else { c })
        .collect()
}

/// 列出所有白板
pub fn list_whiteboards(custom_dir: Option<&str>) -> Result<Vec<WhiteboardInfo>, String> {
    let dir = ensure_dir(custom_dir)?;
    let mut boards = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<WhiteboardData>(&content) {
                        boards.push(WhiteboardInfo {
                            name: data.name,
                            created_at: data.created_at,
                            updated_at: data.updated_at,
                            board_type: detect_board_type(&data.data),
                        });
                    }
                }
            }
        }
    }

    boards.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(boards)
}

/// 检测白板类型
fn detect_board_type(data: &serde_json::Value) -> String {
    if let Some(children) = data.get("children").and_then(|v| v.as_array()) {
        for child in children {
            if let Some(type_str) = child.get("type").and_then(|v| v.as_str()) {
                if type_str.contains("mind") {
                    return "mindmap".to_string();
                }
                if type_str.contains("flow") {
                    return "flowchart".to_string();
                }
            }
        }
    }
    "freehand".to_string()
}

/// 加载白板数据
pub fn load_whiteboard(name: &str, custom_dir: Option<&str>) -> Result<WhiteboardData, String> {
    let path = get_board_path(name, custom_dir);
    if !path.exists() {
        return Err(format!("白板「{}」不存在", name));
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("读取白板失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析白板数据失败: {}", e))
}

/// 保存白板数据
pub fn save_whiteboard(name: &str, data: serde_json::Value, custom_dir: Option<&str>) -> Result<(), String> {
    ensure_dir(custom_dir)?;
    let path = get_board_path(name, custom_dir);
    let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let board_data = if path.exists() {
        let existing = load_whiteboard(name, custom_dir)?;
        WhiteboardData {
            name: name.to_string(),
            created_at: existing.created_at,
            updated_at: now,
            data,
        }
    } else {
        WhiteboardData {
            name: name.to_string(),
            created_at: now.clone(),
            updated_at: now,
            data,
        }
    };

    let json = serde_json::to_string_pretty(&board_data)
        .map_err(|e| format!("序列化白板数据失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("保存白板失败: {}", e))?;

    Ok(())
}

/// 删除白板
pub fn delete_whiteboard(name: &str, custom_dir: Option<&str>) -> Result<(), String> {
    let path = get_board_path(name, custom_dir);
    if !path.exists() {
        return Err(format!("白板「{}」不存在", name));
    }

    fs::remove_file(&path).map_err(|e| format!("删除白板失败: {}", e))?;
    Ok(())
}

/// 检查白板是否存在
pub fn whiteboard_exists(name: &str, custom_dir: Option<&str>) -> bool {
    get_board_path(name, custom_dir).exists()
}

/// 加载白板数据（返回原始 JSON）
pub fn load_whiteboard_data(name: &str, custom_dir: Option<&str>) -> Result<serde_json::Value, String> {
    let data = load_whiteboard(name, custom_dir)?;
    Ok(data.data)
}

/// 保存白板数据（从原始 JSON）
pub fn save_whiteboard_data(name: &str, data: serde_json::Value, custom_dir: Option<&str>) -> Result<(), String> {
    save_whiteboard(name, data, custom_dir)
}
