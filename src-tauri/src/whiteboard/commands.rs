//! 白板 Tauri 命令：窗口管理、数据加载/保存/删除、目录配置

use tauri::Manager;
use tauri::WebviewUrl;
use tauri::WebviewWindowBuilder;

use super::storage;
use super::{WhiteboardData, WhiteboardInfo};
use crate::system::config;

/// 获取当前白板目录（从配置读取，空则用默认）
fn resolve_whiteboard_dir() -> Option<String> {
    let cfg = config::load_config();
    if cfg.whiteboard_dir.is_empty() {
        None
    } else {
        Some(cfg.whiteboard_dir)
    }
}

/// 获取白板存储目录（前端调用）
pub fn whiteboard_get_dir() -> Result<String, String> {
    let dir = storage::get_whiteboard_dir(resolve_whiteboard_dir().as_deref());
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建白板目录失败: {}", e))?;
    }
    Ok(dir.to_string_lossy().to_string())
}

/// 设置白板存储目录（前端调用）
pub fn whiteboard_set_dir(dir: String) -> Result<(), String> {
    if dir.trim().is_empty() {
        return Err("目录路径不能为空".to_string());
    }
    let path = std::path::PathBuf::from(&dir);
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let mut cfg = config::load_config();
    cfg.whiteboard_dir = dir;
    config::save_config(&cfg).map_err(|e| format!("保存配置失败: {}", e))?;
    Ok(())
}

/// 列出所有白板
pub fn list_whiteboards() -> Result<Vec<WhiteboardInfo>, String> {
    storage::list_whiteboards(resolve_whiteboard_dir().as_deref())
}

/// 加载白板数据
pub fn load_whiteboard(name: &str) -> Result<WhiteboardData, String> {
    storage::load_whiteboard(name, resolve_whiteboard_dir().as_deref())
}

/// 保存白板数据
pub fn save_whiteboard(name: &str, data: serde_json::Value) -> Result<(), String> {
    storage::save_whiteboard(name, data, resolve_whiteboard_dir().as_deref())
}

/// 删除白板
pub fn delete_whiteboard(name: &str) -> Result<(), String> {
    storage::delete_whiteboard(name, resolve_whiteboard_dir().as_deref())
}

/// 检查白板是否存在
pub fn whiteboard_exists(name: &str) -> Result<bool, String> {
    Ok(storage::whiteboard_exists(name, resolve_whiteboard_dir().as_deref()))
}

/// 清理文件名中的非法字符，用于窗口标识
fn sanitize_window_label(name: &str) -> String {
    let invalid = ['\\', '/', ':', '*', '?', '"', '<', '>', '|', ' ', '.'];
    name.chars()
        .map(|c| if invalid.contains(&c) { '-' } else { c })
        .collect()
}

/// 打开白板窗口
pub fn open_whiteboard_window(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    let window_label = format!("whiteboard-{}", sanitize_window_label(name));

    if let Some(window) = app.get_webview_window(&window_label) {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let url = WebviewUrl::App(
        format!("whiteboard/whiteboard.html?name={}", urlencoding::encode(name)).into(),
    );

    WebviewWindowBuilder::new(app, &window_label, url)
        .title(format!("白板 - {}", name))
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .center()
        .resizable(true)
        .build()
        .map_err(|e| format!("创建白板窗口失败: {}", e))?;

    Ok(())
}

/// 获取所有打开的白板窗口（过滤掉文件已被外部删除的条目）
pub fn get_open_whiteboards(app: &tauri::AppHandle) -> Vec<String> {
    app.webview_windows()
        .keys()
        .filter(|k| k.starts_with("whiteboard-"))
        .filter_map(|k| {
            let name = k.strip_prefix("whiteboard-")
                .unwrap_or(k)
                .replace('-', " ");
            if storage::whiteboard_exists(&name, resolve_whiteboard_dir().as_deref()) {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}

/// 关闭白板窗口
pub fn close_whiteboard_window(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    let window_label = format!("whiteboard-{}", sanitize_window_label(name));
    if let Some(window) = app.get_webview_window(&window_label) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 加载白板数据（返回原始 JSON）
pub fn load_whiteboard_data(name: &str) -> Result<serde_json::Value, String> {
    storage::load_whiteboard_data(name, resolve_whiteboard_dir().as_deref())
}

/// 保存白板数据（从原始 JSON）
pub fn save_whiteboard_data(name: &str, data: serde_json::Value) -> Result<(), String> {
    storage::save_whiteboard_data(name, data, resolve_whiteboard_dir().as_deref())
}
