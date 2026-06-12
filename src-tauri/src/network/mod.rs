//! 网络诊断模块：网卡信息、DNS、延迟、测速、公网 IP、后台监控

pub mod adapter;
pub mod dns;
pub mod latency;
pub mod public_ip;
pub mod speed;

pub use adapter::{get_adapter_info, AdapterDiagnostic};
pub use dns::{test_dns_resolution, DnsDiagnostic};
pub use latency::{test_latency, LatencyResult};
pub use public_ip::{get_public_ip, PublicIpInfo};
pub use speed::{test_speed, SpeedResult};

use serde::Serialize;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::{AppHandle, Emitter};

use crate::system::school;

const POLL_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Serialize)]
pub struct NetworkStatus {
    pub is_online: bool,
    pub server_reachable: bool,
    pub ip: String,
    pub public_internet: bool,
    pub campus_network: bool,
}

/// 上一次的网络状态（用于检测变化）
static PREV_ONLINE: AtomicBool = AtomicBool::new(false);

/// 获取本机 IP（UDP 探测，不产生实际流量）
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("223.5.5.5:53").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

/// 检查认证服务器是否可达（TCP 探测）
pub fn check_server_reachable() -> bool {
    let profile = school::load_school_profile();
    let addr = match format!("{}:80", profile.auth_server).parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok()
}

/// 检查公网是否可达（连接百度）
pub fn check_public_internet() -> bool {
    TcpStream::connect_timeout(
        &"110.242.68.66:80".parse().unwrap(),
        Duration::from_secs(3),
    )
    .is_ok()
}

/// 启动后台网络状态轮询（每 30 秒）
/// - emit `network-status`：持续状态更新（前端标题栏用）
/// - emit `network-lost`：断网瞬间（前端 Toast + 系统通知）
/// - emit `network-restored`：恢复瞬间（前端通知 + 系统通知）
/// - 更新系统托盘 tooltip 和菜单
pub fn start_network_monitor(app: AppHandle) {
    tracing::info!("启动网络监控线程");
    thread::spawn(move || {
        let initial = check_server_reachable();
        tracing::info!("网络监控初始状态: server_reachable={}", initial);
        PREV_ONLINE.store(initial, Ordering::Relaxed);
        update_tray(&app, initial);

        loop {
            let reachable = check_server_reachable();
            let prev = PREV_ONLINE.swap(reachable, Ordering::Relaxed);

            // 持续状态事件（前端标题栏用）
            let public = check_public_internet();
            let status = NetworkStatus {
                is_online: reachable,
                server_reachable: reachable,
                ip: get_local_ip().unwrap_or_default(),
                public_internet: public,
                campus_network: reachable, // 复用同一次探测结果，避免冗余 TCP 连接
            };
            let _ = app.emit("network-status", status);

            // 状态变化 → 更新托盘 + 发送系统通知
            if prev && !reachable {
                tracing::warn!("网络状态变化: 已连接 → 已断开");
                let _ = app.emit("network-lost", ());
                update_tray(&app, false);
                send_notification(&app, "网络已断开", "正在尝试自动重连...");
            } else if !prev && reachable {
                tracing::info!("网络状态变化: 已断开 → 已恢复");
                let _ = app.emit("network-restored", ());
                update_tray(&app, true);
                send_notification(&app, "网络已恢复", "网络连接已恢复正常");
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}

/// 更新系统托盘 tooltip 和菜单
fn update_tray(app: &AppHandle, is_online: bool) {
    let Some(tray) = app.tray_by_id("main") else {
        return;
    };

    let tooltip = if is_online {
        "绍理闪连 - 已连接"
    } else {
        "绍理闪连 - 未连接"
    };
    let _ = tray.set_tooltip(Some(tooltip));

    // 重建菜单（更新状态文字）
    let status_text = if is_online {
        "✓ 已连接"
    } else {
        "✗ 未连接"
    };
    let status_item = MenuItemBuilder::with_id("status", status_text)
        .enabled(false)
        .build(app);
    let show_item = MenuItemBuilder::with_id("show", "打开绍理闪连").build(app);
    let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app);

    if let (Ok(status), Ok(show), Ok(quit)) = (status_item, show_item, quit_item) {
        if let Ok(menu) = MenuBuilder::new(app)
            .item(&status)
            .separator()
            .item(&show)
            .item(&quit)
            .build()
        {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

/// 发送系统通知
fn send_notification(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}
