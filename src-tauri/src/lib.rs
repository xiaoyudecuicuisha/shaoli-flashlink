//! 绍理闪连 V3.0 核心库
//!
//! 包含 Tauri 应用构建、命令注册、系统托盘、网络监控、日志初始化等核心功能

mod commands;
mod network;
mod srun;
mod system;
mod qzone;
mod whiteboard;
mod cleaner;
mod course;
mod pet;
mod utils;

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tracing_subscriber::EnvFilter;
use tracing_appender::rolling;

/// 运行 Tauri 应用（GUI 模式）
pub fn run() {
    // ── 初始化日志系统 ──
    let log_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".shaoli")
        .join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = rolling::daily(&log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("绍理闪连 V3.0 启动");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(qzone::QzoneState::new())
        .manage(commands::ConvertTaskManager::default())
        .manage(commands::ProcessHandleMap::default())
        .manage(course::CourseState::default())
        .manage(pet::PetState::new())
        .setup(|app| {
            // ── 创建系统托盘 ──
            let status_item = MenuItemBuilder::with_id("status", "状态：检测中...")
                .enabled(false)
                .build(app)?;
            let show_item =
                MenuItemBuilder::with_id("show", "打开绍理闪连").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&status_item)
                .separator()
                .item(&show_item)
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("绍理闪连 - 检测中...")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // ── 根据显示器分辨率动态调整窗口大小 ──
            // 设计基准：1366×850 逻辑像素（保证所有 UI 完整显示）
            // 通过 scale_factor 做 DPI 归一化，避免高 DPI 显示器下窗口物理尺寸过大
            // 同时不超过屏幕逻辑尺寸的 90%，避免撑满
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.current_monitor() {
                    let physical = monitor.size();
                    let scale = monitor.scale_factor();
                    // 物理像素 → 逻辑像素（DPI 归一化）
                    let logical_w = physical.width as f64 / scale;
                    let logical_h = physical.height as f64 / scale;

                    // 设计基准（CSS 逻辑像素）
                    const BASE_W: f64 = 1366.0;
                    const BASE_H: f64 = 850.0;
                    // 屏幕的 90%（留边距）
                    let cap_w = logical_w * 0.9;
                    let cap_h = logical_h * 0.9;

                    // 取基准与屏幕上限的较小值
                    let target_logical_w = BASE_W.min(cap_w);
                    let target_logical_h = BASE_H.min(cap_h);

                    // 逻辑像素 → 物理像素（乘回 scale_factor）
                    // 最低 900×680 物理像素兜底
                    let w = ((target_logical_w * scale) as u32).max(900);
                    let h = ((target_logical_h * scale) as u32).max(680);

                    let _ = window.set_size(tauri::PhysicalSize::new(w, h));
                    let _ = window.center();
                }
            }

            // ── 拦截窗口关闭 → 最小化到托盘 ──
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            // ── 启动后台网络状态监控 ──
            let handle = app.handle().clone();
            network::start_network_monitor(handle);

            // ── 检测崩溃报告 ──
            let app_handle = app.handle().clone();
            check_crash_reports(&app_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::check_online,
            commands::get_ip,
            commands::check_server_reachable,
            commands::get_hosts_status,
            commands::fix_hosts,
            commands::restore_hosts,
            commands::check_autostart,
            commands::setup_autostart,
            commands::remove_autostart,
            commands::is_admin,
            commands::elevate_and_restart,
            commands::load_config,
            commands::save_config,
            commands::get_accounts,
            commands::save_account,
            commands::delete_account,
            commands::set_active_account,
            commands::get_network_status,
            commands::get_adapter_info,
            commands::get_dns_diagnostic,
            commands::get_latency,
            commands::get_speed,
            commands::get_public_ip,
            commands::open_url,
            commands::get_cli_args,
            commands::get_build_time,
            commands::convert_document,
            commands::get_convert_status,
            commands::get_all_convert_status,
            commands::cancel_convert,
            commands::batch_convert,
            commands::cancel_all_convert,
            commands::get_supported_formats,
            commands::qzone_get_qr_code,
            commands::qzone_poll_login,
            commands::qzone_get_user_info,
            commands::qzone_start_fetch,
            commands::qzone_stop_fetch,
            commands::qzone_get_progress,
            commands::qzone_get_moments,
            commands::qzone_export_excel,
            commands::qzone_export_html,
            commands::qzone_download_images,
            commands::qzone_get_default_output_dir,
            commands::list_whiteboards,
            commands::load_whiteboard,
            commands::save_whiteboard,
            commands::delete_whiteboard,
            commands::whiteboard_exists,
            commands::open_whiteboard_window,
            commands::get_open_whiteboards,
            commands::close_whiteboard_window,
            commands::load_whiteboard_data,
            commands::save_whiteboard_data,
            commands::whiteboard_get_dir,
            commands::whiteboard_set_dir,
            commands::cleaner_get_rules,
            commands::cleaner_estimate,
            commands::cleaner_scan,
            commands::cleaner_clean,
            commands::cleaner_scan_empty_folders,
            commands::cleaner_clean_empty_folders,
            commands::large_file_scan,
            commands::large_file_delete,
            commands::uninstall_scan,
            commands::uninstall_standard,
            commands::uninstall_force,
            commands::uninstall_scan_residue,
            commands::uninstall_clean_residue,
            commands::export_logs,
            commands::check_for_updates,
            // ── 网课助手命令 ──
            commands::course_open_window,
            commands::course_close_window,
            commands::course_import_qbank,
            commands::course_get_qbank_info,
            commands::course_delete_qbank,
            commands::course_match_question,
            commands::course_report_progress,
            commands::course_start,
            commands::course_stop,
            commands::course_set_speed,
            // ── 桌面宠物命令 ──
            pet::commands::pet_get_config,
            pet::commands::pet_save_config,
            pet::commands::pet_open_window,
            pet::commands::pet_close_window,
            pet::commands::pet_list_pets,
            pet::commands::pet_upload,
            pet::commands::pet_delete_custom,
            pet::commands::pet_rename,
            pet::commands::pet_set_name,
            pet::commands::pet_open_settings,
            pet::commands::pet_read_file,
            pet::commands::pet_get_cursor_pos,
        ])
        .run(tauri::generate_context!())
        .expect("绍理闪连启动失败");
}

/// 显示主窗口并置顶
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 检测崩溃报告并提示用户
fn check_crash_reports(app: &tauri::AppHandle) {
    let crash_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".shaoli")
        .join("crash");

    if !crash_dir.exists() {
        return;
    }

    // 查找最新的崩溃报告
    let mut reports: Vec<_> = std::fs::read_dir(&crash_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "txt")
                .unwrap_or(false)
        })
        .collect();

    if reports.is_empty() {
        return;
    }

    // 按修改时间排序，获取最新的
    reports.sort_by(|a, b| {
        b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .cmp(&a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH))
    });

    let latest_report = &reports[0];
    let report_path = latest_report.path();

    // 读取报告内容
    let content = std::fs::read_to_string(&report_path)
        .unwrap_or_else(|_| "无法读取崩溃报告".to_string());

    // 截取前 500 字节（安全处理 UTF-8 多字节字符边界）
    let summary = if content.len() > 500 {
        let end = content
            .char_indices()
            .take_while(|&(i, _)| i < 500)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}...", &content[..end])
    } else {
        content.clone()
    };

    // 使用 dialog 显示崩溃报告
    use tauri_plugin_dialog::DialogExt;
    let dialog = app.dialog();
    let message = format!(
        "检测到上次运行时发生了崩溃。\n\n崩溃报告已保存到:\n{}\n\n报告摘要:\n{}\n\n是否要删除此报告？",
        report_path.display(),
        summary
    );

    let report_path_clone = report_path.clone();
    dialog
        .message(message)
        .title("崩溃报告")
        .kind(tauri_plugin_dialog::MessageDialogKind::Warning)
        .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancelCustom(
            "删除报告".to_string(),
            "保留".to_string(),
        ))
        .show(move |result| {
            if result {
                // 用户选择删除
                let _ = std::fs::remove_file(&report_path_clone);
            }
        });
}

/// 静默登录（CLI --silent 模式）
pub async fn silent_login() {
    tracing::info!("silent_login: 开始静默登录");
    let config = system::config::load_config();
    // 从 accounts 获取活跃账号
    let active = config
        .accounts
        .iter()
        .find(|a| a.username == config.active_account)
        .or_else(|| config.accounts.first());

    let (username, password, operator_raw) = match active {
        Some(acc) => {
            tracing::info!("silent_login: 找到活跃账号, username={}", acc.username);
            (acc.username.clone(), acc.password.clone(), acc.operator.clone())
        },
        None => {
            tracing::error!("silent_login: 未配置账号密码");
            eprintln!("未配置账号密码");
            return;
        }
    };

    let operator = match operator_raw.as_str() {
        "telecom" | "电信" => "@telecom",
        "cmcc" | "移动" => "@cmcc",
        "unicom" | "联通" => "@unicom",
        _ => {
            tracing::warn!("silent_login: 未知运营商 '{}', 使用默认 @telecom", operator_raw);
            "@telecom"
        },
    };

    // 检查是否已在线
    if srun::check_online().await {
        tracing::info!("silent_login: 已在线，无需登录");
        println!("已在线，无需登录");
        return;
    }

    let ip = match srun::get_ip() {
        Some(ip) => {
            tracing::info!("silent_login: 获取本机 IP = {}", ip);
            ip
        },
        None => {
            tracing::error!("silent_login: 无法获取本机IP");
            eprintln!("无法获取本机IP");
            return;
        }
    };

    // 最多重试 3 次
    for attempt in 1..=3 {
        if attempt > 1 {
            tracing::info!("silent_login: 第{}次重试, 等待3秒", attempt);
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }

        tracing::info!("silent_login: 第{}次尝试登录", attempt);
        match srun::do_login(&username, &password, operator, &ip).await {
            Ok(msg) => {
                tracing::info!("silent_login: 登录成功 (第{}次): {}", attempt, msg);
                println!("静默登录成功 (第{}次): {}", attempt, msg);
                return;
            }
            Err(e) => {
                tracing::error!("silent_login: 登录失败 (第{}次): {}", attempt, e);
                eprintln!("静默登录失败 (第{}次): {}", attempt, e);
            }
        }
    }
    tracing::error!("silent_login: 全部重试均失败");
    eprintln!("静默登录全部重试均失败");
}

/// CLI 命令处理（由 main.rs 在 CLI 模式下调用）
pub async fn handle_cli_args() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return;
    }

    match args[1].as_str() {
        "--silent" => {
            silent_login().await;
        }
        "--setup-autostart" => match system::autostart::setup_autostart() {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("{}", e),
        },
        "--remove-autostart" => match system::autostart::remove_autostart() {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("{}", e),
        },
        "--fix-hosts" => match system::hosts::fix_hosts() {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("{}", e),
        },
        "--restore-hosts" => match system::hosts::restore_hosts() {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("{}", e),
        },
        _ => {}
    }
}
