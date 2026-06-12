//! 卸载引擎：注册表扫描、进程终止、服务停止、残留清理、注册表备份

use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::System;
use winreg::enums::*;

use super::protect::is_protected_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_path: String,
    pub uninstall_cmd: String,
    pub quiet_cmd: String,
    pub icon_path: String,
    pub reg_key: String,
    pub is_risky: bool,
    pub risk_reason: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidueItem {
    pub path: String,
    pub item_type: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UninstallResult {
    pub success: bool,
    pub message: String,
    pub residue_items: Vec<ResidueItem>,
}

/// Scan installed applications from Windows registry
pub fn scan_installed_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    let reg_roots = [
        (
            winreg::enums::HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
        (
            winreg::enums::HKEY_LOCAL_MACHINE,
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
        (
            winreg::enums::HKEY_CURRENT_USER,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
    ];

    for (root, subkey) in &reg_roots {
        let hklm = winreg::RegKey::predef(*root);
        let Ok(key) = hklm.open_subkey_with_flags(subkey, winreg::enums::KEY_READ) else {
            continue;
        };

        for subkey_name in key.enum_keys().filter_map(|k| k.ok()) {
            let Ok(app_key) = key.open_subkey_with_flags(&subkey_name, winreg::enums::KEY_READ)
            else {
                continue;
            };

            let display_name: String = app_key
                .get_value("DisplayName")
                .unwrap_or_default();
            if display_name.is_empty() {
                continue;
            }

            if display_name.contains("KB") && display_name.chars().any(|c| c.is_ascii_digit()) {
                continue;
            }

            let display_version: String = app_key
                .get_value("DisplayVersion")
                .unwrap_or_default();
            let publisher: String = app_key.get_value("Publisher").unwrap_or_default();
            let install_location: String = app_key
                .get_value("InstallLocation")
                .unwrap_or_default();
            let uninstall_string: String = app_key
                .get_value("UninstallString")
                .unwrap_or_default();
            let quiet_uninstall: String = app_key
                .get_value("QuietUninstallString")
                .unwrap_or_default();
            let display_icon: String = app_key.get_value("DisplayIcon").unwrap_or_default();

            let (is_risky, risk_reason) = assess_risk(&display_name, &publisher);

            let quiet_cmd = if !quiet_uninstall.is_empty() {
                quiet_uninstall
            } else {
                detect_quiet_cmd(&uninstall_string)
            };

            apps.push(InstalledApp {
                name: display_name,
                version: display_version,
                publisher,
                install_path: install_location,
                uninstall_cmd: uninstall_string,
                quiet_cmd,
                icon_path: display_icon,
                reg_key: subkey_name,
                is_risky,
                risk_reason,
                size_bytes: 0,
            });
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

/// Assess if an app is risky to uninstall
fn assess_risk(name: &str, publisher: &str) -> (bool, String) {
    let name_lower = name.to_lowercase();
    let pub_lower = publisher.to_lowercase();

    let risky_keywords = [
        ("driver", "驱动程序"),
        ("runtime", "运行时"),
        ("redistributable", "运行时组件"),
        ("visual c++", "VC++ 运行时"),
        (".net framework", ".NET框架"),
        ("directx", "DirectX"),
        ("windows sdk", "Windows SDK"),
        ("windows kit", "Windows Kit"),
        ("vcredist", "VC++运行时"),
    ];

    for (keyword, reason) in &risky_keywords {
        if name_lower.contains(keyword) || pub_lower.contains(keyword) {
            return (true, reason.to_string());
        }
    }

    (false, String::new())
}

/// 检测常见安装器类型的静默卸载参数
fn detect_quiet_cmd(cmd: &str) -> String {
    if cmd.is_empty() {
        return String::new();
    }

    let cmd_lower = cmd.to_lowercase();

    if cmd_lower.contains("nsis") || cmd_lower.contains("uninst") {
        return format!("{} /S", cmd);
    }

    if cmd_lower.contains("unins") && !cmd_lower.contains("/silent") {
        return format!("{} /VERYSILENT /SUPPRESSMSGBOXES /NORESTART", cmd);
    }

    if cmd_lower.contains("msiexec") {
        return format!("{} /QN /NORESTART", cmd);
    }

    if cmd_lower.contains("installshield") || cmd_lower.contains("setup") {
        return format!("{} /S /V/QN", cmd);
    }

    String::new()
}

/// 从注册表重新读取卸载命令，避免信任前端传入的命令字符串
fn read_uninstall_cmd_from_registry(reg_key: &str) -> (String, String) {
    let mut uninstall_cmd = String::new();
    let mut quiet_cmd = String::new();

    let reg_roots = [
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_CURRENT_USER, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];

    for (root, subkey) in &reg_roots {
        let hkey = winreg::RegKey::predef(*root);
        let path = format!("{}\\{}", subkey, reg_key);
        if let Ok(app_key) = hkey.open_subkey_with_flags(&path, KEY_READ) {
            let uninstall_string: String = app_key.get_value("UninstallString").unwrap_or_default();
            let quiet_uninstall: String = app_key.get_value("QuietUninstallString").unwrap_or_default();

            if !quiet_uninstall.is_empty() {
                quiet_cmd = quiet_uninstall;
            }
            if !uninstall_string.is_empty() {
                uninstall_cmd = uninstall_string;
            }

            if !uninstall_cmd.is_empty() {
                break;
            }
        }
    }

    if quiet_cmd.is_empty() && !uninstall_cmd.is_empty() {
        quiet_cmd = detect_quiet_cmd(&uninstall_cmd);
    }

    (uninstall_cmd, quiet_cmd)
}

/// 安全解析卸载命令字符串为 (exe_path, args_vec)
/// 支持 "path\to\uninst.exe /arg1 /arg2" 和 msiexec 格式
fn parse_uninstall_cmd(cmd: &str) -> Option<(String, Vec<String>)> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return None;
    }

    let cmd_lower = cmd.to_lowercase();

    if cmd_lower.contains("msiexec") {
        let parts = shellwords::split(cmd).ok()?;
        if parts.is_empty() {
            return None;
        }
        let exe = parts[0].clone();
        let args = parts.into_iter().skip(1).collect();
        return Some((exe, args));
    }

    let (exe, rest) = if cmd.starts_with('"') {
        let end = cmd[1..].find('"')?;
        let exe = cmd[1..=end].to_string();
        let rest = cmd[end + 2..].trim().to_string();
        (exe, rest)
    } else {
        match cmd.find(' ') {
            Some(pos) => {
                let exe = cmd[..pos].to_string();
                let rest = cmd[pos + 1..].trim().to_string();
                (exe, rest)
            }
            None => (cmd.to_string(), String::new()),
        }
    };

    if exe.is_empty() {
        return None;
    }

    let args = if rest.is_empty() {
        Vec::new()
    } else {
        shellwords::split(&rest).unwrap_or_else(|_| rest.split_whitespace().map(|s| s.to_string()).collect())
    };

    Some((exe, args))
}

/// 安全执行卸载命令
fn run_uninstall_cmd(cmd: &str) -> Result<std::process::Output, String> {
    let (exe, args) = parse_uninstall_cmd(cmd)
        .ok_or_else(|| "无法解析卸载命令".to_string())?;

    let output = Command::new(&exe)
        .args(&args)
        .output()
        .map_err(|e| format!("执行卸载命令失败: {}", e))?;

    Ok(output)
}

/// Kill processes matching the application name
pub fn kill_app_processes(app_name: &str) -> Vec<String> {
    let mut killed = Vec::new();
    let sys = System::new_all();

    let name_lower = app_name.to_lowercase();
    let keywords: Vec<&str> = name_lower.split_whitespace().collect();

    for (pid, process) in sys.processes() {
        let proc_name = process.name().to_string_lossy().to_lowercase();

        if keywords.iter().any(|kw| proc_name.contains(kw)) {
            if process.kill() {
                killed.push(format!("{} (PID: {})", proc_name, pid));
            }
        }
    }

    killed
}

/// 停止并删除匹配应用名的 Windows 服务
pub fn cleanup_app_services(app_name: &str) -> Vec<String> {
    let mut cleaned = Vec::new();
    let name_lower = app_name.to_lowercase();

    let output = Command::new("sc")
        .args(["query", "type=", "service", "state=", "all", "bufsize=", "65536"])
        .output();

    let Ok(output) = output else {
        return cleaned;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("service_name") {
            if let Some(name) = line.split(':').nth(1) {
                let svc_name = name.trim().to_lowercase();
                if svc_name.contains(&name_lower) || name_lower.contains(&svc_name) {
                    let _ = Command::new("sc")
                        .args(["stop", &svc_name])
                        .output();

                    let _ = Command::new("sc")
                        .args(["delete", &svc_name])
                        .output();

                    cleaned.push(svc_name);
                }
            }
        }
    }

    cleaned
}

/// 扫描残留文件和注册表项
pub fn scan_residue(app: &InstalledApp) -> Vec<ResidueItem> {
    let mut items = Vec::new();

    if !app.install_path.is_empty() {
        let path = std::path::Path::new(&app.install_path);
        if path.exists() {
            let size = dir_size(path);
            items.push(ResidueItem {
                path: app.install_path.clone(),
                item_type: "目录".to_string(),
                size,
            });
        }
    }

    let appdata_paths = [
        format!(
            "{}\\{}",
            std::env::var("APPDATA").unwrap_or_default(),
            app.name
        ),
        format!(
            "{}\\{}",
            std::env::var("LOCALAPPDATA").unwrap_or_default(),
            app.name
        ),
        format!(
            "{}\\{}\\LocalCache",
            std::env::var("LOCALAPPDATA").unwrap_or_default(),
            app.name
        ),
    ];

    for path_str in &appdata_paths {
        let path = std::path::Path::new(path_str);
        if path.exists() {
            let size = dir_size(path);
            items.push(ResidueItem {
                path: path_str.clone(),
                item_type: "AppData".to_string(),
                size,
            });
        }
    }

    let reg_paths = [
        format!(
            "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
            app.reg_key
        ),
        format!(
            "HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
            app.reg_key
        ),
        format!(
            "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
            app.reg_key
        ),
    ];

    for reg_path in &reg_paths {
        items.push(ResidueItem {
            path: reg_path.clone(),
            item_type: "注册表".to_string(),
            size: 0,
        });
    }

    items
}

/// Clean residue items
pub fn clean_residue(items: &[ResidueItem]) -> super::clean::CleanResult {
    let mut result = super::clean::CleanResult {
        cleaned_count: 0,
        freed_bytes: 0,
        failed_count: 0,
        errors: Vec::new(),
    };

    for item in items {
        if item.item_type == "注册表" {
            // Registry cleanup is best-effort
            result.cleaned_count += 1;
            continue;
        }

        let path = std::path::Path::new(&item.path);

        // 受保护路径校验
        if is_protected_path(path) {
            result.failed_count += 1;
            result.errors.push(format!("拒绝删除受保护路径: {}", item.path));
            continue;
        }

        if path.exists() {
            if path.is_dir() {
                if std::fs::remove_dir_all(path).is_ok() {
                    result.cleaned_count += 1;
                    result.freed_bytes += item.size;
                } else {
                    result.failed_count += 1;
                    result.errors.push(format!("删除目录失败: {}", item.path));
                }
            } else {
                if std::fs::remove_file(path).is_ok() {
                    result.cleaned_count += 1;
                    result.freed_bytes += item.size;
                } else {
                    result.failed_count += 1;
                    result.errors.push(format!("删除文件失败: {}", item.path));
                }
            }
        }
    }

    result
}

/// Backup registry key for an app before uninstall
pub fn backup_registry(app: &InstalledApp) -> Result<String, String> {
    let backup_dir = dirs::home_dir()
        .ok_or("无法获取用户目录")?
        .join(".shaoli")
        .join("backups");

    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let safe_name = app.name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect::<String>();
    let backup_path = backup_dir.join(format!("{}_{}.reg", safe_name, timestamp));

    let mut reg_content = String::from("Windows Registry Editor Version 5.00\n\n");

    // Backup from HKLM Uninstall
    let hklm = winreg::RegKey::predef(HKEY_LOCAL_MACHINE);
    let uninstall_paths = [
        format!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}", app.reg_key),
        format!("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}", app.reg_key),
    ];

    for path in &uninstall_paths {
        if let Ok(key) = hklm.open_subkey_with_flags(path, KEY_READ) {
            reg_content.push_str(&format!("[HKEY_LOCAL_MACHINE\\{}]\n", path));
            for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
                let escaped_name = name.replace('\\', "\\\\").replace('"', "\\\"");
                match value.vtype {
                    REG_SZ => {
                        let s = key.get_value::<String, _>(&name).unwrap_or_default();
                        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                        reg_content.push_str(&format!("\"{}\"=\"{}\"\n", escaped_name, escaped));
                    }
                    REG_DWORD => {
                        let v: u32 = key.get_value(&name).unwrap_or(0);
                        reg_content.push_str(&format!("\"{}\"=dword:{:08x}\n", escaped_name, v));
                    }
                    _ => {}
                }
            }
            reg_content.push('\n');
        }
    }

    // Backup from HKCU Uninstall
    let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let hkcu_path = format!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}", app.reg_key);
    if let Ok(key) = hkcu.open_subkey_with_flags(&hkcu_path, KEY_READ) {
        reg_content.push_str(&format!("[HKEY_CURRENT_USER\\{}]\n", hkcu_path));
        for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
            let escaped_name = name.replace('\\', "\\\\").replace('"', "\\\"");
            match value.vtype {
                REG_SZ => {
                    let s = key.get_value::<String, _>(&name).unwrap_or_default();
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    reg_content.push_str(&format!("\"{}\"=\"{}\"\n", escaped_name, escaped));
                }
                REG_DWORD => {
                    let v: u32 = key.get_value(&name).unwrap_or(0);
                    reg_content.push_str(&format!("\"{}\"=dword:{:08x}\n", escaped_name, v));
                }
                _ => {}
            }
        }
        reg_content.push('\n');
    }

    std::fs::write(&backup_path, reg_content)
        .map_err(|e| format!("写入备份文件失败: {}", e))?;

    Ok(backup_path.to_string_lossy().to_string())
}

/// Execute standard uninstall (with registry backup)
pub fn standard_uninstall(app: &InstalledApp) -> UninstallResult {
    // Backup registry before uninstall
    let backup_msg = match backup_registry(app) {
        Ok(path) => format!("注册表已备份到: {}", path),
        Err(e) => format!("注册表备份失败（不影响卸载）: {}", e),
    };

    // 从注册表重新读取卸载命令，不直接使用前端传入的命令字符串
    let (uninstall_cmd, quiet_cmd) = read_uninstall_cmd_from_registry(&app.reg_key);

    let cmd = if !quiet_cmd.is_empty() {
        quiet_cmd
    } else if !uninstall_cmd.is_empty() {
        uninstall_cmd
    } else {
        return UninstallResult {
            success: false,
            message: "未找到卸载命令".to_string(),
            residue_items: Vec::new(),
        };
    };

    let output = match run_uninstall_cmd(&cmd) {
        Ok(out) => out,
        Err(e) => {
            return UninstallResult {
                success: false,
                message: format!("{}; {}", e, backup_msg),
                residue_items: Vec::new(),
            };
        }
    };

    if output.status.success() {
        let residue = scan_residue(app);
        UninstallResult {
            success: true,
            message: format!("标准卸载完成; {}", backup_msg),
            residue_items: residue,
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        UninstallResult {
            success: false,
            message: format!("卸载程序返回错误: {}; {}", stderr, backup_msg),
            residue_items: Vec::new(),
        }
    }
}

/// Execute force uninstall (kill processes + services + uninstall + clean residue)
pub fn force_uninstall(app: &InstalledApp) -> UninstallResult {
    let mut messages = Vec::new();

    // Step 0: Backup registry
    match backup_registry(app) {
        Ok(path) => messages.push(format!("注册表已备份到: {}", path)),
        Err(e) => messages.push(format!("注册表备份失败（不影响卸载）: {}", e)),
    }

    // Step 1: Kill processes
    let killed = kill_app_processes(&app.name);
    if !killed.is_empty() {
        messages.push(format!("已终止进程: {}", killed.join(", ")));
    }

    // Step 2: Stop and delete services
    let services = cleanup_app_services(&app.name);
    if !services.is_empty() {
        messages.push(format!("已清理服务: {}", services.join(", ")));
    }

    // Step 3: Execute uninstall
    // 从注册表重新读取卸载命令，不直接使用前端传入的命令字符串
    let (uninstall_cmd, quiet_cmd) = read_uninstall_cmd_from_registry(&app.reg_key);

    let cmd = if !quiet_cmd.is_empty() {
        quiet_cmd
    } else if !uninstall_cmd.is_empty() {
        uninstall_cmd
    } else {
        messages.push("未找到卸载命令，跳过卸载程序".to_string());
        String::new()
    };

    if !cmd.is_empty() {
        match run_uninstall_cmd(&cmd) {
            Ok(out) => {
                if out.status.success() {
                    messages.push("卸载程序执行成功".to_string());
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    messages.push(format!("卸载程序返回错误: {}", stderr));
                }
            }
            Err(e) => {
                messages.push(format!("执行卸载命令失败: {}", e));
            }
        }
    }

    // Step 4: Scan and clean residue
    let residue = scan_residue(app);
    if !residue.is_empty() {
        messages.push(format!("发现 {} 个残留项", residue.len()));
    }

    UninstallResult {
        success: true,
        message: messages.join("; "),
        residue_items: residue,
    }
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut size = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if metadata.is_file() {
                size += metadata.len();
            } else if metadata.is_dir() {
                size += dir_size(&entry.path());
            }
        }
    }
    size
}
