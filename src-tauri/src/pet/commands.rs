//! 宠物 Tauri 命令：配置读写、窗口管理、上传/删除/改名、光标位置

use super::{storage, CustomPet, PetConfig, PetInfo, PetState};
use std::fs;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

// ── 获取全局鼠标位置（用于点击穿透检测） ──

#[tauri::command]
pub fn pet_get_cursor_pos() -> Result<(i32, i32), String> {
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    use windows::Win32::Foundation::POINT;

    let mut point = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut point).map_err(|e| format!("获取鼠标位置失败: {}", e))?;
    }
    Ok((point.x, point.y))
}

// ── 配置读写 ──

#[tauri::command]
pub fn pet_get_config(state: State<'_, PetState>) -> Result<PetConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn pet_save_config(state: State<'_, PetState>, patch: PetConfig) -> Result<(), String> {
    storage::save_config(&patch)?;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = patch;
    Ok(())
}

// ── 窗口管理 ──

#[tauri::command]
pub async fn pet_open_window(app: AppHandle, state: State<'_, PetState>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("pet-window") {
        state
            .window_open
            .store(true, std::sync::atomic::Ordering::SeqCst);
        return w.set_focus().map_err(|e| e.to_string());
    }

    let config = state.config.lock().map_err(|e| e.to_string())?;
    let size = config.size as f64;
    // 窗口尺寸需要比宠物图片大，以容纳右键菜单和气泡
    let win_size = size + 120.0;
    drop(config); // 释放锁

    // 使用 run_on_main_thread 在主线程创建窗口
    // 关键：command 必须先返回，让主线程空闲后再执行窗口创建
    // 否则 build() 会死锁（主线程等 IPC 返回，IPC 等 build 完成，build 等主线程）
    let app_clone = app.clone();
    let state_open = state.window_open.clone();

    app.run_on_main_thread(move || {
        let url = WebviewUrl::App("pet/pet.html".into());
        let result = WebviewWindowBuilder::new(&app_clone, "pet-window", url)
            .title("桌面宠物")
            .inner_size(win_size, win_size)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .build();

        match result {
            Ok(_window) => {
                state_open.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            Err(e) => {
                tracing::error!("Pet window creation failed: {}", e);
            }
        }
    }).map_err(|e| format!("调度主线程失败: {}", e))?;

    // 立即返回，窗口创建在主线程空闲后异步完成
    Ok(())
}

#[tauri::command]
pub fn pet_close_window(app: AppHandle, state: State<'_, PetState>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("pet-window") {
        w.close().map_err(|e| e.to_string())?;
    }
    state
        .window_open
        .store(false, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

// ── 宠物列表 ──

#[tauri::command]
pub fn pet_list_pets(state: State<'_, PetState>) -> Result<Vec<PetInfo>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let mut pets = vec![PetInfo {
        id: "default".to_string(),
        name: "默认小伙伴".to_string(),
        source: "builtin".to_string(),
    }];

    for cp in &config.custom_pets {
        pets.push(PetInfo {
            id: cp.id.clone(),
            name: cp.name.clone(),
            source: "custom".to_string(),
        });
    }

    Ok(pets)
}

// ── 上传自定义宠物 ──

#[tauri::command]
pub fn pet_upload(
    state: State<'_, PetState>,
    source_path: String,
    name: String,
) -> Result<CustomPet, String> {
    let src = std::path::Path::new(&source_path);
    if !src.exists() {
        return Err("源文件不存在".to_string());
    }

    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !["gif", "apng", "png"].contains(&ext.as_str()) {
        return Err("不支持的文件格式，请上传 GIF / APNG / PNG".to_string());
    }

    let metadata = fs::metadata(src).map_err(|e| e.to_string())?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err("文件过大，请上传 10MB 以内的图片".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let file_name = format!("{}.{}", id, ext);
    let dest = storage::get_custom_dir().join(&file_name);

    fs::copy(src, &dest).map_err(|e| format!("复制文件失败: {}", e))?;

    let pet = CustomPet {
        id: id.clone(),
        name,
        file_name,
        file_type: ext,
    };

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.custom_pets.push(pet.clone());
    storage::save_config(&config)?;

    Ok(pet)
}

// ── 重命名宠物 ──

#[tauri::command]
pub fn pet_rename(state: State<'_, PetState>, pet_id: String, new_name: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    let pet = config
        .custom_pets
        .iter_mut()
        .find(|p| p.id == pet_id)
        .ok_or("宠物不存在")?;

    pet.name = new_name;
    storage::save_config(&config)?;

    Ok(())
}

// ── 删除自定义宠物 ──

#[tauri::command]
pub fn pet_delete_custom(state: State<'_, PetState>, pet_id: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    if let Some(pos) = config.custom_pets.iter().position(|p| p.id == pet_id) {
        let pet = config.custom_pets.remove(pos);
        storage::delete_custom_pet(&pet.file_name)?;

        if config.current_pet == pet_id {
            config.current_pet = "default".to_string();
        }
        storage::save_config(&config)?;
    }

    Ok(())
}

// ── 读取宠物文件为 base64 ──

#[tauri::command]
pub fn pet_read_file(state: State<'_, PetState>, pet_id: String) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;

    if pet_id == "default" {
        return Err("默认宠物无需 base64 加载".to_string());
    }

    let pet = config
        .custom_pets
        .iter()
        .find(|p| p.id == pet_id)
        .ok_or("宠物不存在")?;

    let path = storage::get_custom_dir().join(&pet.file_name);
    let data = fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let mime = match pet.file_type.as_str() {
        "gif" => "image/gif",
        "apng" => "image/apng",
        "png" => "image/png",
        _ => "image/gif",
    };

    Ok(format!("data:{};base64,{}", mime, b64))
}

// ── 统一设置宠物名称（支持默认 + 自定义） ──

#[tauri::command]
pub fn pet_set_name(state: State<'_, PetState>, pet_id: String, new_name: String) -> Result<(), String> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err("名称不能为空".to_string());
    }
    if trimmed.chars().count() > 20 {
        return Err("名称过长（最多 20 个字符）".to_string());
    }
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    if pet_id == "default" {
        config.default_pet_name = trimmed.to_string();
    } else {
        let pet = config
            .custom_pets
            .iter_mut()
            .find(|p| p.id == pet_id)
            .ok_or("宠物不存在")?;
        pet.name = trimmed.to_string();
    }
    storage::save_config(&config)?;
    Ok(())
}

// ── 打开宠物设置（由悬浮窗右键触发：拉起主窗口并跳到 /pet 页面） ──

#[tauri::command]
pub fn pet_open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.show();
        let _ = main_win.unminimize();
        let _ = main_win.set_focus();
    }
    let _ = app.emit("navigate-to-pet", ());
    Ok(())
}
