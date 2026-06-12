//! 网课 WebView 窗口管理：创建/关闭窗口、注入脚本

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use super::platform::PlatformAdapter;

/// 打开网课平台窗口
///
/// 参照 `whiteboard/commands.rs` 的 `open_whiteboard_window()` 模式，
/// 使用 `WebviewUrl::External` 加载外部网课页面并注入自动化脚本。
pub fn open_window(app: &AppHandle, platform: &dyn PlatformAdapter) -> Result<(), String> {
    let label = platform.window_label();

    if let Some(window) = app.get_webview_window(&label) {
        return window.set_focus().map_err(|e| e.to_string());
    }

    let webview_url = WebviewUrl::External(
        tauri::Url::parse(platform.entry_url())
            .map_err(|e| format!("URL 格式错误: {}", e))?,
    );

    WebviewWindowBuilder::new(app, &label, webview_url)
        .title("网课助手")
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .center()
        .resizable(true)
        .initialization_script(platform.script())
        // 允许 window.open() 请求（超星课程页面大量使用新窗口导航）
        .on_new_window(|_url, _features| {
            tauri::webview::NewWindowResponse::Allow
        })
        .build()
        .map_err(|e| format!("创建窗口失败: {}", e))?;

    Ok(())
}

/// 关闭网课平台窗口
pub fn close_window(app: &AppHandle, platform: &dyn PlatformAdapter) -> Result<(), String> {
    let label = platform.window_label();
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
