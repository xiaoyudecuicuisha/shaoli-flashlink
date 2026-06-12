//! 绍理闪连 V3.0 入口点
//!
//! 支持双模式运行：
//! - GUI 模式（无参数）：启动 Tauri 应用窗口
//! - CLI 模式（有参数）：执行命令行任务（静默登录、自启设置等）

// 发布模式禁止控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;

fn main() {
    // ── 设置崩溃报告 ──
    setup_crash_handler();

    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 {
        // CLI 模式
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(shaoli_flashlink_lib::handle_cli_args());
    } else {
        // GUI 模式
        shaoli_flashlink_lib::run();
    }
}

fn setup_crash_handler() {
    std::panic::set_hook(Box::new(|info| {
        let crash_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".shaoli")
            .join("crash");

        if let Err(e) = std::fs::create_dir_all(&crash_dir) {
            eprintln!("创建崩溃目录失败: {}", e);
            return;
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let crash_file = crash_dir.join(format!("crash-{}.txt", timestamp));

        let mut content = String::new();
        content.push_str("绍理闪连 V3.0 崩溃报告\n");
        content.push_str("========================\n\n");
        content.push_str(&format!("时间: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

        // Panic 信息
        content.push_str("Panic 信息:\n");
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            content.push_str(&format!("  {}\n", s));
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            content.push_str(&format!("  {}\n", s));
        } else {
            content.push_str("  (未知错误)\n");
        }

        // 位置信息
        if let Some(location) = info.location() {
            content.push_str(&format!("位置: {}:{}:{}\n",
                location.file(),
                location.line(),
                location.column()
            ));
        }

        // Backtrace
        content.push_str("\nBacktrace:\n");
        let backtrace = std::backtrace::Backtrace::force_capture();
        content.push_str(&format!("{:?}\n", backtrace));

        // 写入文件
        match std::fs::File::create(&crash_file) {
            Ok(mut file) => {
                let _ = file.write_all(content.as_bytes());
                eprintln!("崩溃报告已保存到: {}", crash_file.display());
            }
            Err(e) => {
                eprintln!("保存崩溃报告失败: {}", e);
            }
        }
    }));
}
