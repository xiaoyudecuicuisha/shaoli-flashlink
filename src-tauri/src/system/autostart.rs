//! 开机自启管理：HKCU 注册表（主） + schtasks 任务计划（备）

use std::process::Command;

const REG_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "SXLGnet-autologin";

/// 检查是否已设置开机自启
pub fn check_autostart() -> bool {
    // 优先检查注册表方式
    if check_registry_autostart() {
        return true;
    }
    // 回退检查计划任务方式
    Command::new("schtasks")
        .args(["/query", "/tn", APP_NAME])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_registry_autostart() -> bool {
    let output = Command::new("reg")
        .args([
            "query",
            &format!(r"HKCU\{}", REG_KEY),
            "/v",
            APP_NAME,
        ])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// 设置开机自启（优先使用 HKCU 注册表，无需管理员权限）
pub fn setup_autostart() -> Result<String, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取程序路径失败: {}", e))?
        .to_string_lossy()
        .to_string();

    // 方案1：HKCU 注册表（无需管理员权限，最可靠）
    let reg_result = Command::new("reg")
        .args([
            "add",
            &format!(r"HKCU\{}", REG_KEY),
            "/v",
            APP_NAME,
            "/t",
            "REG_SZ",
            "/d",
            &format!("\"{}\" --silent", exe_path),
            "/f",
        ])
        .output();

    match reg_result {
        Ok(o) if o.status.success() => {
            return Ok("开机自启设置成功".to_string());
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            // 注册表方式失败，尝试计划任务
            let _ = err; // 记录但不返回
        }
        Err(_) => {}
    }

    // 方案2：计划任务（先尝试最高权限）
    let output = Command::new("schtasks")
        .args([
            "/create",
            "/tn",
            APP_NAME,
            "/tr",
            &format!("\"{}\" --silent", exe_path),
            "/sc",
            "onlogon",
            "/rl",
            "highest",
            "/f",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let _ = set_task_settings();
            return Ok("开机自启设置成功".to_string());
        }
        Ok(_) => {
            // 管理员权限创建失败，尝试以普通用户权限创建
            let output2 = Command::new("schtasks")
                .args([
                    "/create",
                    "/tn",
                    APP_NAME,
                    "/tr",
                    &format!("\"{}\" --silent", exe_path),
                    "/sc",
                    "onlogon",
                    "/f",
                ])
                .output()
                .map_err(|e| format!("执行 schtasks 失败: {}", e))?;

            if output2.status.success() {
                let _ = set_task_settings();
                return Ok("开机自启设置成功（普通权限模式）".to_string());
            }

            let err = String::from_utf8_lossy(&output2.stderr);
            Err(format!("创建计划任务失败: {}", err))
        }
        Err(e) => Err(format!("执行 schtasks 失败: {}", e)),
    }
}

fn set_task_settings() -> Result<(), String> {
    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "$t = Get-ScheduledTask -TaskName '{}'; $s = $t.Settings; \
                 $s.Hidden = $true; $s.DisallowStartIfOnBatteries = $false; \
                 $s.StopIfGoingOnBatteries = $false; Set-ScheduledTask -TaskName '{}' -Settings $s",
                APP_NAME, APP_NAME
            ),
        ])
        .output();
    Ok(())
}

/// 取消开机自启
pub fn remove_autostart() -> Result<String, String> {
    let mut removed = false;

    let reg_output = Command::new("reg")
        .args([
            "delete",
            &format!(r"HKCU\{}", REG_KEY),
            "/v",
            APP_NAME,
            "/f",
        ])
        .output();

    if let Ok(o) = reg_output {
        if o.status.success() {
            removed = true;
        }
    }

    let task_output = Command::new("schtasks")
        .args(["/delete", "/tn", APP_NAME, "/f"])
        .output();

    match task_output {
        Ok(o) if o.status.success() => {
            removed = true;
        }
        _ => {}
    }

    if removed {
        Ok("已取消开机自启".to_string())
    } else {
        Err("未找到开机自启项".to_string())
    }
}
