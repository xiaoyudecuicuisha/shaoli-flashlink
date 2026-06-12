//! UAC 管理员提权：检测权限、提权重启

use std::process::Command;

/// 检查当前是否以管理员权限运行
pub fn is_admin() -> bool {
    // 使用 PowerShell 检查管理员权限
    Command::new("powershell")
        .args([
            "-Command",
            "([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)",
        ])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.trim().eq_ignore_ascii_case("True")
        })
        .unwrap_or(false)
}

/// 以管理员权限重新启动自身，传入指定参数
/// 返回 Ok(true) 表示提权成功并已启动新进程（当前进程应退出）
/// 返回 Ok(false) 表示已是管理员
/// 返回 Err 表示提权失败
pub fn elevate_and_run(args: &[&str]) -> Result<bool, String> {
    if is_admin() {
        return Ok(false);
    }

    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取程序路径失败: {}", e))?
        .to_string_lossy()
        .to_string();

    // 转义单引号防止 PowerShell 命令注入
    let args_str = args
        .iter()
        .map(|a| a.replace("'", "''"))
        .collect::<Vec<_>>()
        .join(" ");

    // 对程序路径进行 PowerShell 单引号转义
    let exe_path_escaped = exe_path.replace("'", "''");

    // 使用 PowerShell 的 Start-Process 以管理员权限运行
    // -LiteralPath 避免路径中的特殊字符被 PowerShell 解析器误解
    let status = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Start-Process -LiteralPath '{}' -ArgumentList '{}' -Verb RunAs -WindowStyle Hidden",
                exe_path_escaped, args_str
            ),
        ])
        .status()
        .map_err(|e| format!("提权失败: {}", e))?;

    if status.success() {
        Ok(true)
    } else {
        Err("用户拒绝了 UAC 提权请求".to_string())
    }
}
