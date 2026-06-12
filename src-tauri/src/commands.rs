//! Tauri 命令路由层
//!
//! 所有 #[tauri::command] 函数的集中注册点，按功能分区组织：
//! 认证 / Hosts / 自启 / 配置 / 网络 / 格式转换 / QQ空间 / 白板 / 清理 / 网课 / 宠物

use crate::network;
use crate::srun;
use crate::system::{autostart, config, elevate, hosts};
use crate::qzone;
use crate::whiteboard;
use crate::cleaner;
use crate::course;
use crate::utils::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

// ── 格式转换任务管理器 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvertStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertTask {
    pub id: String,
    pub input_path: String,
    pub output_path: String,
    pub target_format: String,
    pub status: ConvertStatus,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertTaskStatus {
    pub id: String,
    pub status: ConvertStatus,
    pub progress: f32,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConvertItem {
    pub input_path: String,
    pub target_format: String,
}

pub type ConvertTaskManager = Arc<Mutex<HashMap<String, ConvertTask>>>;

pub type ProcessHandleMap = Arc<Mutex<HashMap<String, tokio::process::Child>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertResult {
    pub task_id: String,
    pub output_path: String,
}

// ── 响应类型 ──

#[derive(Serialize)]
pub struct HostsStatusResponse {
    pub fixed_count: usize,
    pub total: usize,
    pub sites: Vec<SiteInfoResponse>,
}

#[derive(Serialize)]
pub struct SiteInfoResponse {
    pub domain: String,
    pub name: String,
    pub is_fixed: bool,
}

impl From<hosts::HostsStatus> for HostsStatusResponse {
    fn from(s: hosts::HostsStatus) -> Self {
        Self {
            fixed_count: s.fixed_count,
            total: s.total,
            sites: s
                .sites
                .into_iter()
                .map(|s| SiteInfoResponse {
                    domain: s.domain,
                    name: s.name,
                    is_fixed: s.is_fixed,
                })
                .collect(),
        }
    }
}

// ── 认证命令 ──

#[tauri::command]
pub async fn login(
    username: String,
    password: String,
    operator: String,
) -> Result<String, AppError> {
    tracing::info!("login 命令: username={}, operator={}", username, operator);
    let ip = srun::get_ip().ok_or("无法获取本机IP")?;
    let full_operator = match operator.as_str() {
        "telecom" | "电信" => "@telecom",
        "cmcc" | "移动" => "@cmcc",
        "unicom" | "联通" => "@unicom",
        other => {
            tracing::warn!("login: 未知运营商 '{}', 直接使用", other);
            other
        },
    };
    let result = srun::do_login(&username, &password, full_operator, &ip).await.map_err(AppError::from);
    match &result {
        Ok(msg) => tracing::info!("login 成功: {}", msg),
        Err(e) => tracing::error!("login 失败: {}", e),
    }
    result
}

#[tauri::command]
pub async fn check_online() -> bool {
    srun::check_online().await
}

#[tauri::command]
pub async fn get_ip() -> Option<String> {
    srun::get_ip()
}

#[tauri::command]
pub async fn check_server_reachable() -> bool {
    network::check_server_reachable()
}

// ── Hosts 命令 ──

#[tauri::command]
pub async fn get_hosts_status() -> HostsStatusResponse {
    hosts::check_hosts_status().into()
}

#[tauri::command]
pub async fn fix_hosts() -> Result<String, AppError> {
    hosts::fix_hosts().map_err(AppError::from)
}

#[tauri::command]
pub async fn restore_hosts() -> Result<String, AppError> {
    hosts::restore_hosts().map_err(AppError::from)
}

// ── 开机自启命令 ──

#[tauri::command]
pub async fn check_autostart() -> bool {
    autostart::check_autostart()
}

#[tauri::command]
pub async fn setup_autostart() -> Result<String, AppError> {
    autostart::setup_autostart().map_err(AppError::from)
}

#[tauri::command]
pub async fn remove_autostart() -> Result<String, AppError> {
    autostart::remove_autostart().map_err(AppError::from)
}

// ── 管理员提权命令 ──

#[tauri::command]
pub async fn is_admin() -> bool {
    elevate::is_admin()
}

#[tauri::command]
pub async fn elevate_and_restart(
    app: tauri::AppHandle,
    args: Vec<String>,
) -> Result<bool, AppError> {
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let elevated = elevate::elevate_and_run(&arg_refs).map_err(AppError::from)?;
    if elevated {
        app.exit(0);
    }
    Ok(elevated)
}

// ── 配置命令 ──

#[tauri::command]
pub async fn load_config() -> config::AppConfig {
    config::load_config()
}

/// 合并保存配置（只覆盖传入的字段）
#[tauri::command]
pub async fn save_config(patch: serde_json::Value) -> Result<(), AppError> {
    config::save_config_patch(&patch).map_err(AppError::from)
}

// ── 账号管理命令 ──

/// 获取所有已保存的账号列表
#[tauri::command]
pub async fn get_accounts() -> Vec<config::AccountEntry> {
    config::get_accounts()
}

/// 保存/更新账号并设为活跃
#[tauri::command]
pub async fn save_account(
    username: String,
    password: String,
    operator: String,
) -> Result<(), AppError> {
    config::save_account(&username, &password, &operator).map_err(AppError::from)
}

/// 删除指定账号
#[tauri::command]
pub async fn delete_account(username: String) -> Result<(), AppError> {
    config::delete_account(&username).map_err(AppError::from)
}

/// 切换活跃账号
#[tauri::command]
pub async fn set_active_account(username: String) -> Result<(), AppError> {
    config::set_active_account(&username).map_err(AppError::from)
}

// ── 网络状态 ──

#[tauri::command]
pub async fn get_network_status() -> network::NetworkStatus {
    tracing::info!("get_network_status: 开始检测网络状态");
    tokio::task::spawn_blocking(|| {
        let campus = network::check_server_reachable();
        let public = network::check_public_internet();
        let ip = srun::get_ip().unwrap_or_default();
        tracing::info!("get_network_status: campus={}, public={}, ip={}", campus, public, ip);
        network::NetworkStatus {
            is_online: campus || public,
            server_reachable: campus,
            ip,
            public_internet: public,
            campus_network: campus,
        }
    })
    .await
    .unwrap_or(network::NetworkStatus {
        is_online: false,
        server_reachable: false,
        ip: String::new(),
        public_internet: false,
        campus_network: false,
    })
}

// ── 网络诊断 ──

#[tauri::command]
pub async fn get_adapter_info() -> network::AdapterDiagnostic {
    tracing::info!("get_adapter_info: 开始获取网卡信息");
    tokio::task::spawn_blocking(network::get_adapter_info)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("get_adapter_info: 失败: {}", e);
            network::AdapterDiagnostic {
                adapters: vec![],
                active_adapter: String::new(),
                vpn_detected: false,
                proxy_enabled: false,
                proxy_address: String::new(),
            }
        })
}

#[tauri::command]
pub async fn get_dns_diagnostic() -> network::DnsDiagnostic {
    tracing::info!("get_dns_diagnostic: 开始 DNS 诊断");
    tokio::task::spawn_blocking(network::test_dns_resolution)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("get_dns_diagnostic: 失败: {}", e);
            network::DnsDiagnostic {
                dns_servers: vec![],
                resolution_tests: vec![],
            }
        })
}

#[tauri::command]
pub async fn get_latency() -> Vec<network::LatencyResult> {
    tracing::info!("get_latency: 开始延迟测试");
    tokio::task::spawn_blocking(network::test_latency)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn get_speed() -> network::SpeedResult {
    tracing::info!("get_speed: 开始速度测试");
    network::test_speed().await
}

#[tauri::command]
pub async fn get_public_ip() -> network::PublicIpInfo {
    tracing::info!("get_public_ip: 开始获取公网 IP");
    network::get_public_ip().await
}

// ── 打开链接 ──

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), AppError> {
    // 仅允许安全协议，防止打开 file:// 或 javascript: 等危险 URL
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:") {
        webbrowser::open(&url).map_err(|e| AppError::Other(e.to_string()))
    } else {
        Err(AppError::Other("不支持的 URL 协议".to_string()))
    }
}

// ── CLI 参数 ──

#[tauri::command]
pub async fn get_cli_args() -> String {
    let args: Vec<String> = std::env::args().collect();
    args.get(1).cloned().unwrap_or_default()
}

// ── 构建时间 ──

#[tauri::command]
pub fn get_build_time() -> String {
    env!("BUILD_TIME").to_string()
}

// ── 格式转换 ──

/// Word COM 格式代码
fn word_format_code(ext: &str) -> Option<i32> {
    match ext {
        "doc" => Some(0),   // wdFormatDocument
        "docx" => Some(12), // wdFormatXMLDocument
        "pdf" => Some(17),  // wdFormatPDF
        "rtf" => Some(6),   // wdFormatRTF
        "txt" => Some(2),   // wdFormatText (Unicode)
        _ => None,
    }
}

/// Excel COM 格式代码
fn excel_format_code(ext: &str) -> Option<i32> {
    match ext {
        "xls" => Some(56),  // xlExcel8
        "xlsx" => Some(51), // xlOpenXMLWorkbook
        "csv" => Some(6),   // xlCSV
        "pdf" => Some(0),   // 特殊处理：ExportAsFixedFormat
        _ => None,
    }
}

/// PowerPoint COM 格式代码
fn powerpoint_format_code(ext: &str) -> Option<i32> {
    match ext {
        "ppt" => Some(0),   // ppSaveAsPresentation (旧格式)
        "pptx" => Some(24), // ppSaveAsOpenXMLPresentation
        "pdf" => Some(32),  // ppSaveAsPDF
        _ => None,
    }
}

/// 获取目标格式的 COM SaveAs 格式代码
fn get_format_code(source_ext: &str, target_ext: &str) -> Option<i32> {
    match source_ext {
        "doc" | "docx" | "rtf" | "txt" => word_format_code(target_ext),
        "xls" | "xlsx" | "csv" => excel_format_code(target_ext),
        "ppt" | "pptx" => powerpoint_format_code(target_ext),
        _ => None,
    }
}

/// 获取源格式支持的目标格式列表
#[tauri::command]
pub async fn get_supported_formats(input_path: String) -> Vec<String> {
    let ext = Path::new(&input_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "doc" | "docx" | "rtf" | "txt" => {
            ["doc", "docx", "pdf", "rtf", "txt"]
                .iter().filter(|&&f| f != ext.as_str()).map(|s| s.to_string()).collect()
        }
        "xls" | "xlsx" | "csv" => {
            ["xls", "xlsx", "pdf", "csv"]
                .iter().filter(|&&f| f != ext.as_str()).map(|s| s.to_string()).collect()
        }
        "ppt" | "pptx" => {
            ["ppt", "pptx", "pdf"]
                .iter().filter(|&&f| f != ext.as_str()).map(|s| s.to_string()).collect()
        }
        _ => vec![],
    }
}

/// 生成任务 ID
fn generate_task_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("convert_{}_{}", timestamp, rand::random::<u32>())
}

/// Office 文档格式转换（调用 Word/WPS COM 接口）
/// 返回任务 ID
#[tauri::command]
pub async fn convert_document(
    input_path: String,
    target_format: String,
    task_manager: tauri::State<'_, ConvertTaskManager>,
    process_map: tauri::State<'_, ProcessHandleMap>,
) -> Result<ConvertResult, AppError> {
    let input = Path::new(&input_path);
    if !input.exists() {
        return Err(AppError::Other("文件不存在".to_string()));
    }

    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let supported = ["doc", "docx", "rtf", "txt", "xls", "xlsx", "csv", "ppt", "pptx"];
    if !supported.contains(&ext.as_str()) {
        return Err(AppError::Other(format!("不支持的源格式: .{}", ext)));
    }

    let target_ext = target_format.trim_start_matches('.').to_lowercase();
    let format_code = get_format_code(&ext, &target_ext)
        .ok_or(AppError::Other(format!("不支持的转换: .{} → .{}", ext, target_ext)))?;

    let output = input.with_extension(target_ext);
    let output_str = output.to_string_lossy().to_string();

    let task_id = generate_task_id();
    let task = ConvertTask {
        id: task_id.clone(),
        input_path: input_path.clone(),
        output_path: output_str.clone(),
        target_format: target_format.clone(),
        status: ConvertStatus::Running,
        progress: 0.0,
    };

    {
        let mut tasks = task_manager.lock().await;
        tasks.insert(task_id.clone(), task);
    }

    let script = include_str!("../scripts/convert_to_pdf.ps1");
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("shaoli_convert_{}.ps1", task_id));
    std::fs::write(&script_path, script).map_err(|e| AppError::Io(format!("创建转换脚本失败: {}", e)))?;

    let task_id_clone = task_id.clone();
    let task_manager_clone = task_manager.inner().clone();
    let process_map_clone = process_map.inner().clone();
    let input_path_clone = input_path.clone();
    let output_str_clone = output_str.clone();
    let script_path_clone = script_path.clone();

    tokio::spawn(async move {
        let child = tokio::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script_path_clone.to_string_lossy(),
                "-InputPath",
                &input_path_clone,
                "-OutputPath",
                &output_str_clone,
                "-Format",
                &format_code.to_string(),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match child {
            Ok(child) => {
                // 立即将进程句柄存入 HashMap，然后取出并释放锁
                // 避免在 wait 期间持有 process_map 锁导致 cancel 操作阻塞
                let child_opt = {
                    let mut processes = process_map_clone.lock().await;
                    processes.insert(task_id_clone.clone(), child);
                    processes.remove(&task_id_clone)
                };

                if let Some(child) = child_opt {
                    let result = child.wait_with_output().await;
                    let _ = std::fs::remove_file(&script_path_clone);

                    let mut tasks = task_manager_clone.lock().await;
                    if let Some(task) = tasks.get_mut(&task_id_clone) {
                        match result {
                            Ok(output) => {
                                if output.status.success() {
                                    if Path::new(&output_str_clone).exists() {
                                        task.status = ConvertStatus::Completed;
                                        task.progress = 100.0;
                                    } else {
                                        task.status = ConvertStatus::Failed("转换完成但未生成目标文件".to_string());
                                    }
                                } else {
                                    let mut msg = String::from_utf8_lossy(&output.stderr).to_string();
                                    if msg.is_empty() {
                                        msg = String::from_utf8_lossy(&output.stdout).to_string();
                                    }
                                    if msg.is_empty() {
                                        msg = "转换失败，请确认已安装 Microsoft Office 或 WPS".to_string();
                                    }
                                    task.status = ConvertStatus::Failed(msg);
                                }
                            }
                            Err(e) => {
                                task.status = ConvertStatus::Failed(format!("进程执行失败: {}", e));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let _ = std::fs::remove_file(&script_path_clone);
                let mut tasks = task_manager_clone.lock().await;
                if let Some(task) = tasks.get_mut(&task_id_clone) {
                    task.status = ConvertStatus::Failed(format!("启动转换失败: {}", e));
                }
            }
        }
    });

    Ok(ConvertResult {
        task_id: task_id.clone(),
        output_path: output_str,
    })
}

/// 查询转换任务状态
#[tauri::command]
pub async fn get_convert_status(
    task_id: String,
    task_manager: tauri::State<'_, ConvertTaskManager>,
) -> Result<ConvertTaskStatus, AppError> {
    let tasks = task_manager.lock().await;
    if let Some(task) = tasks.get(&task_id) {
        let (output_path, error) = match &task.status {
            ConvertStatus::Completed => (Some(task.output_path.clone()), None),
            ConvertStatus::Failed(e) => (None, Some(e.clone())),
            _ => (None, None),
        };
        
        Ok(ConvertTaskStatus {
            id: task.id.clone(),
            status: task.status.clone(),
            progress: task.progress,
            output_path,
            error,
        })
    } else {
        Err(AppError::Other("任务不存在".to_string()))
    }
}

/// 查询所有转换任务状态
#[tauri::command]
pub async fn get_all_convert_status(
    task_manager: tauri::State<'_, ConvertTaskManager>,
) -> Result<Vec<ConvertTaskStatus>, AppError> {
    let tasks = task_manager.lock().await;
    let result: Vec<ConvertTaskStatus> = tasks
        .values()
        .map(|task| {
            let (output_path, error) = match &task.status {
                ConvertStatus::Completed => (Some(task.output_path.clone()), None),
                ConvertStatus::Failed(e) => (None, Some(e.clone())),
                _ => (None, None),
            };
            ConvertTaskStatus {
                id: task.id.clone(),
                status: task.status.clone(),
                progress: task.progress,
                output_path,
                error,
            }
        })
        .collect();
    Ok(result)
}

/// 取消转换任务
#[tauri::command]
pub async fn cancel_convert(
    task_id: String,
    task_manager: tauri::State<'_, ConvertTaskManager>,
    process_map: tauri::State<'_, ProcessHandleMap>,
) -> Result<(), AppError> {
    {
        let mut processes = process_map.lock().await;
        if let Some(mut child) = processes.remove(&task_id) {
            child.kill().await.map_err(|e| AppError::Other(format!("终止进程失败: {}", e)))?;
        }
    }

    {
        let mut tasks = task_manager.lock().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.status = ConvertStatus::Cancelled;
        }
    }

    Ok(())
}

/// 批量转换
#[tauri::command]
pub async fn batch_convert(
    files: Vec<BatchConvertItem>,
    task_manager: tauri::State<'_, ConvertTaskManager>,
    process_map: tauri::State<'_, ProcessHandleMap>,
) -> Result<Vec<ConvertResult>, AppError> {
    let mut results = Vec::new();
    
    for file in files {
        let result = convert_document(
            file.input_path,
            file.target_format,
            task_manager.clone(),
            process_map.clone(),
        ).await?;
        results.push(result);
    }
    
    Ok(results)
}

/// 取消所有转换任务
#[tauri::command]
pub async fn cancel_all_convert(
    task_manager: tauri::State<'_, ConvertTaskManager>,
    process_map: tauri::State<'_, ProcessHandleMap>,
) -> Result<(), AppError> {
    {
        let mut processes = process_map.lock().await;
        for (_, mut child) in processes.drain() {
            let _ = child.kill().await;
            let _ = child.wait().await; // 必须 wait 以回收进程句柄，避免 zombie
        }
    }

    {
        let mut tasks = task_manager.lock().await;
        for (_, task) in tasks.iter_mut() {
            if matches!(task.status, ConvertStatus::Pending | ConvertStatus::Running) {
                task.status = ConvertStatus::Cancelled;
            }
        }
    }

    Ok(())
}

// ── QQ 空间历史命令 ──

#[tauri::command]
pub async fn qzone_get_qr_code(state: tauri::State<'_, qzone::QzoneState>) -> Result<String, AppError> {
    // 每次生成新二维码前清空累积的 cookies，避免上一次失败/过期的 cookies 污染本次登录
    {
        let mut map = state.cookie_map.lock().map_err(|e| AppError::Other(e.to_string()))?;
        map.clear();
    }

    // 先访问 xlogin 获取 pt_login_sig 等初始 cookie（QQ 扫码登录的强制前置步骤）
    qzone::auth::get_initial_cookies(&state.client, &state.cookie_map).await?;

    let (image_data, qrsig) =
        qzone::auth::get_qr_code(&state.client, &state.cookie_map).await?;

    let mut progress = state.progress.lock().map_err(|e| AppError::Other(e.to_string()))?;
    progress.status = format!("qrsig:{}", qrsig);

    use base64::Engine;
    let base64 = base64::engine::general_purpose::STANDARD.encode(&image_data);
    Ok(format!("data:image/png;base64,{}", base64))
}

#[tauri::command]
pub async fn qzone_poll_login(
    state: tauri::State<'_, qzone::QzoneState>,
    qrsig: String,
) -> Result<qzone::models::LoginStatus, AppError> {
    let status = qzone::auth::poll_login(
        &state.client,
        &state.no_redirect_client,
        &state.cookie_map,
        &qrsig,
    )
    .await;

    if let qzone::models::LoginStatus::Success { ref cookies } = status {
        let mut saved_cookies = state.cookies.lock().map_err(|e| AppError::Other(e.to_string()))?;
        *saved_cookies = Some(cookies.clone());
    }

    Ok(status)
}

#[tauri::command]
pub async fn qzone_get_user_info(state: tauri::State<'_, qzone::QzoneState>) -> Result<qzone::models::QzoneUserInfo, AppError> {
    let cookies = state.cookies.lock().map_err(|e| AppError::Other(e.to_string()))?
        .clone()
        .ok_or(AppError::Auth("未登录".to_string()))?;
    
    let user_info = qzone::api::get_user_info(&state.client, &cookies).await?;
    
    let mut saved_user_info = state.user_info.lock().map_err(|e| AppError::Other(e.to_string()))?;
    *saved_user_info = Some(user_info.clone());
    
    Ok(user_info)
}

#[tauri::command]
pub async fn qzone_start_fetch(
    state: tauri::State<'_, qzone::QzoneState>,
    options: qzone::models::FetchOptions,
) -> Result<(), AppError> {
    let cookies = state.cookies.lock().map_err(|e| AppError::Other(e.to_string()))?
        .clone()
        .ok_or(AppError::Auth("未登录".to_string()))?;
    
    state.cancel_token.store(false, std::sync::atomic::Ordering::SeqCst);

    {
        let mut progress = state.progress.lock().map_err(|e| AppError::Other(e.to_string()))?;
        progress.status = "正在获取数据...".to_string();
        progress.is_running = true;
    }
    
    let moments_state = state.moments.clone();
    let progress_state = state.progress.clone();
    let cancel_token = state.cancel_token.clone();
    let client = state.client.clone();
    
    tokio::spawn(async move {
        use std::sync::atomic::Ordering;

        let total = match qzone::api::get_message_count(&client, &cookies).await {
            Ok(t) => t,
            Err(e) => {
                let mut progress = progress_state.lock().unwrap_or_else(|e| e.into_inner());
                progress.status = format!("获取消息数量失败: {}", e);
                progress.is_running = false;
                return;
            }
        };
        
        let mut all_moments = Vec::new();
        let mut all_friends = Vec::new();
        
        let batch_size = 10;
        let total_batches = (total / batch_size) + 1;
        
        for i in 0..total_batches {
            if cancel_token.load(Ordering::SeqCst) {
                let mut progress = progress_state.lock().unwrap_or_else(|e| e.into_inner());
                progress.status = "已终止".to_string();
                progress.is_running = false;
                return;
            }
            
            let offset = i * batch_size;
            
            {
                let mut progress = progress_state.lock().unwrap_or_else(|e| e.into_inner());
                progress.total = total_batches as usize;
                progress.current = i as usize;
                progress.status = format!("正在获取第 {}/{} 批消息...", i + 1, total_batches);
            }
            
            match qzone::api::get_message(&client, &cookies, offset, batch_size).await {
                Ok(data) => {
                    let (batch_moments, batch_friends) = qzone::parser::parse_html_message(&data);
                    all_moments.extend(batch_moments);
                    all_friends.extend(batch_friends);
                }
                Err(e) => {
                    eprintln!("获取消息失败: {}", e);
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        
        if options.include_moments {
            let mut offset = 0;
            let page_size = 30;
            
            loop {
                if cancel_token.load(Ordering::SeqCst) {
                    let mut progress = progress_state.lock().unwrap_or_else(|e| e.into_inner());
                    progress.status = "已终止".to_string();
                    progress.is_running = false;
                    let partial_moments = qzone::parser::merge_moments(all_moments.clone(), Vec::new());
                    let mut moments = moments_state.lock().unwrap_or_else(|e| e.into_inner());
                    *moments = partial_moments;
                    return;
                }

                match qzone::api::get_visible_moments(&client, &cookies, page_size, offset).await {
                    Ok(json) => {
                        let moments = qzone::parser::parse_json_moments(&json);
                        if moments.is_empty() {
                            break;
                        }
                        all_moments.extend(moments);
                        offset += page_size;
                    }
                    Err(e) => {
                        eprintln!("获取未删除说说失败: {}", e);
                        break;
                    }
                }
                
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        
        let merged_moments = qzone::parser::merge_moments(all_moments, Vec::new());

        {
            let mut moments = moments_state.lock().unwrap_or_else(|e| e.into_inner());
            *moments = merged_moments.clone();
        }
        
        {
            let mut progress = progress_state.lock().unwrap_or_else(|e| e.into_inner());
            progress.moments_count = merged_moments.len();
            progress.friends_count = all_friends.len();
            progress.status = "获取完成".to_string();
            progress.is_running = false;
        }
    });
    
    Ok(())
}

#[tauri::command]
pub async fn qzone_stop_fetch(state: tauri::State<'_, qzone::QzoneState>) -> Result<(), AppError> {
    state.cancel_token.store(true, std::sync::atomic::Ordering::SeqCst);
    
    let mut progress = state.progress.lock().map_err(|e| AppError::Other(e.to_string()))?;
    progress.status = "已请求终止...".to_string();
    progress.is_running = false;
    
    Ok(())
}

#[tauri::command]
pub async fn qzone_get_default_output_dir() -> Result<String, AppError> {
    let desktop = dirs::desktop_dir()
        .ok_or_else(|| AppError::Other("无法获取桌面路径".to_string()))?;
    let qzone_dir = desktop.join("QQ空间");
    
    if !qzone_dir.exists() {
        std::fs::create_dir_all(&qzone_dir)
            .map_err(|e| AppError::Other(format!("创建 QQ空间 目录失败: {}", e)))?;
    }
    
    Ok(qzone_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn qzone_get_progress(state: tauri::State<'_, qzone::QzoneState>) -> Result<qzone::models::FetchProgress, AppError> {
    let progress = state.progress.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(progress.clone())
}

#[tauri::command]
pub async fn qzone_get_moments(
    state: tauri::State<'_, qzone::QzoneState>,
    page: usize,
    page_size: usize,
) -> Result<Vec<qzone::models::Moment>, AppError> {
    let moments = state.moments.lock().map_err(|e| AppError::Other(e.to_string()))?;
    let start = page * page_size;
    let end = std::cmp::min(start + page_size, moments.len());
    
    if start >= moments.len() {
        Ok(Vec::new())
    } else {
        Ok(moments[start..end].to_vec())
    }
}

#[tauri::command]
pub async fn qzone_export_excel(
    state: tauri::State<'_, qzone::QzoneState>,
    path: String,
) -> Result<String, AppError> {
    let moments = state.moments.lock().map_err(|e| AppError::Other(e.to_string()))?;
    let path = Path::new(&path);
    
    qzone::export::export_excel(&moments, path)?;
    
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn qzone_export_html(
    state: tauri::State<'_, qzone::QzoneState>,
    path: String,
) -> Result<String, AppError> {
    let moments = state.moments.lock().map_err(|e| AppError::Other(e.to_string()))?;
    let user_info = state.user_info.lock().map_err(|e| AppError::Other(e.to_string()))?;
    let path = Path::new(&path);
    
    let (uin, nickname) = match &*user_info {
        Some(info) => (info.uin.clone(), info.nickname.clone()),
        None => ("unknown".to_string(), "未知用户".to_string()),
    };
    
    qzone::export::export_html(&moments, path, &uin, &nickname)?;
    
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn qzone_download_images(
    state: tauri::State<'_, qzone::QzoneState>,
    path: String,
) -> Result<usize, AppError> {
    let moments = state.moments.lock().map_err(|e| AppError::Other(e.to_string()))?.clone();
    let path = Path::new(&path);
    let client = reqwest::Client::new();
    
    let count = qzone::export::download_images(&moments, path, &client).await?;
    
    Ok(count)
}

// ── 白板命令 ──

#[tauri::command]
pub async fn list_whiteboards() -> Result<Vec<whiteboard::WhiteboardInfo>, AppError> {
    whiteboard::commands::list_whiteboards().map_err(AppError::from)
}

#[tauri::command]
pub async fn load_whiteboard(name: String) -> Result<whiteboard::WhiteboardData, AppError> {
    whiteboard::commands::load_whiteboard(&name).map_err(AppError::from)
}

#[tauri::command]
pub async fn save_whiteboard(name: String, data: serde_json::Value) -> Result<(), AppError> {
    whiteboard::commands::save_whiteboard(&name, data).map_err(AppError::from)
}

#[tauri::command]
pub async fn delete_whiteboard(name: String) -> Result<(), AppError> {
    whiteboard::commands::delete_whiteboard(&name).map_err(AppError::from)
}

#[tauri::command]
pub async fn whiteboard_exists(name: String) -> Result<bool, AppError> {
    whiteboard::commands::whiteboard_exists(&name).map_err(AppError::from)
}

#[tauri::command]
pub async fn open_whiteboard_window(app: tauri::AppHandle, name: String) -> Result<(), AppError> {
    whiteboard::commands::open_whiteboard_window(&app, &name).map_err(AppError::from)
}

#[tauri::command]
pub async fn get_open_whiteboards(app: tauri::AppHandle) -> Vec<String> {
    whiteboard::commands::get_open_whiteboards(&app)
}

#[tauri::command]
pub async fn close_whiteboard_window(app: tauri::AppHandle, name: String) -> Result<(), AppError> {
    whiteboard::commands::close_whiteboard_window(&app, &name).map_err(AppError::from)
}

#[tauri::command]
pub async fn load_whiteboard_data(name: String) -> Result<serde_json::Value, AppError> {
    whiteboard::commands::load_whiteboard_data(&name).map_err(AppError::from)
}

#[tauri::command]
pub async fn save_whiteboard_data(name: String, data: serde_json::Value) -> Result<(), AppError> {
    whiteboard::commands::save_whiteboard_data(&name, data).map_err(AppError::from)
}

#[tauri::command]
pub async fn whiteboard_get_dir() -> Result<String, AppError> {
    whiteboard::commands::whiteboard_get_dir().map_err(AppError::from)
}

#[tauri::command]
pub async fn whiteboard_set_dir(dir: String) -> Result<(), AppError> {
    whiteboard::commands::whiteboard_set_dir(dir).map_err(AppError::from)
}

// ── 清理工具命令 ──

#[tauri::command]
pub async fn cleaner_get_rules() -> Vec<cleaner::rules::CleanRule> {
    cleaner::rules::load_all_rules()
}

#[tauri::command]
pub async fn cleaner_estimate(categories: Vec<String>) -> Vec<cleaner::rules::EstimateItem> {
    let all_rules = cleaner::rules::load_all_rules();
    let filtered: Vec<_> = all_rules
        .into_iter()
        .filter(|r| categories.contains(&r.category))
        .collect();
    cleaner::scan::estimate_rules(&filtered)
}

#[tauri::command]
pub async fn cleaner_scan(categories: Vec<String>) -> cleaner::scan::ScanResult {
    let all_rules = cleaner::rules::load_all_rules();
    let filtered: Vec<_> = all_rules
        .into_iter()
        .filter(|r| categories.contains(&r.category))
        .collect();
    cleaner::scan::scan_rules(&filtered)
}

#[tauri::command]
pub async fn cleaner_clean(
    paths: Vec<String>,
    mode: String,
) -> cleaner::clean::CleanResult {
    let clean_mode = match mode.as_str() {
        "permanent" => cleaner::clean::CleanMode::Permanent,
        _ => cleaner::clean::CleanMode::Recycle,
    };
    cleaner::clean::clean_items(&paths, &clean_mode)
}

#[tauri::command]
pub async fn cleaner_scan_empty_folders(root: String) -> Vec<cleaner::scan::ScanItem> {
    let path = std::path::Path::new(&root);
    cleaner::scan::scan_empty_folders(path)
}

#[tauri::command]
pub async fn cleaner_clean_empty_folders(paths: Vec<String>) -> cleaner::clean::CleanResult {
    cleaner::clean::clean_empty_folders(&paths)
}

#[tauri::command]
pub async fn large_file_scan(
    root: String,
    min_bytes: u64,
    limit: u32,
    skip_optional: bool,
    excludes: Vec<String>,
) -> Result<Vec<cleaner::mft::LargeFile>, AppError> {
    if elevate::is_admin() {
        // 管理员权限：使用 MFT 快速扫描
        cleaner::mft::scan_large_files(&root, min_bytes, limit, skip_optional, &excludes)
            .map_err(AppError::Other)
    } else {
        // 非管理员：降级为文件系统遍历扫描（无需管理员权限）
        cleaner::mft::scan_large_files_fallback(
            &root,
            min_bytes,
            limit as usize,
            skip_optional,
            &excludes,
        )
        .map_err(AppError::Other)
    }
}

#[tauri::command]
pub async fn large_file_delete(
    paths: Vec<String>,
    mode: String,
) -> cleaner::clean::CleanResult {
    let clean_mode = match mode.as_str() {
        "permanent" => cleaner::clean::CleanMode::Permanent,
        _ => cleaner::clean::CleanMode::Recycle,
    };
    cleaner::clean::clean_items(&paths, &clean_mode)
}

#[tauri::command]
pub async fn uninstall_scan() -> Vec<cleaner::uninstall::InstalledApp> {
    cleaner::uninstall::scan_installed_apps()
}

#[tauri::command]
pub async fn uninstall_standard(
    app_name: String,
    install_path: String,
    reg_key: String,
) -> cleaner::uninstall::UninstallResult {
    let app = cleaner::uninstall::InstalledApp {
        name: app_name,
        version: String::new(),
        publisher: String::new(),
        install_path,
        uninstall_cmd: String::new(),
        quiet_cmd: String::new(),
        icon_path: String::new(),
        reg_key,
        is_risky: false,
        risk_reason: String::new(),
        size_bytes: 0,
    };
    cleaner::uninstall::standard_uninstall(&app)
}

#[tauri::command]
pub async fn uninstall_force(
    app_name: String,
    install_path: String,
    reg_key: String,
) -> cleaner::uninstall::UninstallResult {
    let app = cleaner::uninstall::InstalledApp {
        name: app_name,
        version: String::new(),
        publisher: String::new(),
        install_path,
        uninstall_cmd: String::new(),
        quiet_cmd: String::new(),
        icon_path: String::new(),
        reg_key,
        is_risky: false,
        risk_reason: String::new(),
        size_bytes: 0,
    };
    cleaner::uninstall::force_uninstall(&app)
}

#[tauri::command]
pub async fn uninstall_scan_residue(
    app_name: String,
    install_path: String,
    reg_key: String,
) -> Vec<cleaner::uninstall::ResidueItem> {
    let app = cleaner::uninstall::InstalledApp {
        name: app_name,
        version: String::new(),
        publisher: String::new(),
        install_path,
        uninstall_cmd: String::new(),
        quiet_cmd: String::new(),
        icon_path: String::new(),
        reg_key,
        is_risky: false,
        risk_reason: String::new(),
        size_bytes: 0,
    };
    cleaner::uninstall::scan_residue(&app)
}

#[tauri::command]
pub async fn uninstall_clean_residue(
    items: Vec<cleaner::uninstall::ResidueItem>,
) -> cleaner::clean::CleanResult {
    cleaner::uninstall::clean_residue(&items)
}

#[tauri::command]
pub async fn export_logs(target_path: String) -> Result<(), AppError> {
    let log_dir = dirs::home_dir()
        .ok_or(AppError::Io("无法获取用户目录".to_string()))?
        .join(".shaoli")
        .join("logs");

    if !log_dir.exists() {
        return Err(AppError::Io("日志目录不存在".to_string()));
    }

    let zip_file = std::fs::File::create(&target_path)
        .map_err(|e| AppError::Io(format!("创建压缩文件失败: {}", e)))?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(&log_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let name = path.strip_prefix(&log_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            zip.start_file(name, options)
                .map_err(|e| AppError::Io(format!("添加文件到压缩包失败: {}", e)))?;
            let content = std::fs::read(path)
                .map_err(|e| AppError::Io(format!("读取日志文件失败: {}", e)))?;
            zip.write_all(&content)
                .map_err(|e| AppError::Io(format!("写入压缩包失败: {}", e)))?;
        }
    }

    zip.finish()
        .map_err(|e| AppError::Io(format!("完成压缩失败: {}", e)))?;

    tracing::info!("日志已导出到: {}", target_path);
    Ok(())
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<String, AppError> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater()
        .map_err(|e| AppError::Other(format!("获取更新器失败: {}", e)))?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.to_string();
            update.download_and_install(|_, _| {}, || {})
                .await
                .map_err(|e| AppError::Other(format!("下载或安装更新失败: {}", e)))?;
            Ok(format!("已更新到版本 {}，请重启应用生效", version))
        }
        Ok(None) => {
            Ok("当前已是最新版本".to_string())
        }
        Err(e) => Err(AppError::Other(format!("检查更新失败: {}", e))),
    }
}

// ── 网课助手命令 ──

#[tauri::command]
pub async fn course_open_window(app: tauri::AppHandle) -> Result<(), AppError> {
    let platform = course::platform::ChaoxingPlatform;
    course::window::open_window(&app, &platform).map_err(AppError::from)
}

#[tauri::command]
pub async fn course_close_window(app: tauri::AppHandle) -> Result<(), AppError> {
    let platform = course::platform::ChaoxingPlatform;
    course::window::close_window(&app, &platform).map_err(AppError::from)
}

#[tauri::command]
pub async fn course_import_qbank(
    state: tauri::State<'_, course::CourseState>,
    path: String,
) -> Result<usize, AppError> {
    let entries = course::qbank::import_file(&path).map_err(AppError::from)?;
    let total = course::qbank::save_bank(&entries).map_err(AppError::from)?;

    if let Ok(bank) = course::qbank::get_bank() {
        let mut qbank = state.qbank.lock().map_err(|e| AppError::Other(e.to_string()))?;
        *qbank = bank;
    }

    Ok(total)
}

#[tauri::command]
pub async fn course_get_qbank_info(
    state: tauri::State<'_, course::CourseState>,
) -> Result<serde_json::Value, AppError> {
    let info = course::qbank::get_bank_info().map_err(AppError::from)?;
    let bank_len = {
        let bank = state.qbank.lock().map_err(|e| AppError::Other(e.to_string()))?;
        bank.len()
    };
    Ok(serde_json::json!({
        "count": info.count,
        "source": info.source,
        "loaded": bank_len,
    }))
}

#[tauri::command]
pub async fn course_delete_qbank(
    state: tauri::State<'_, course::CourseState>,
) -> Result<(), AppError> {
    course::qbank::delete_bank().map_err(AppError::from)?;
    let mut qbank = state.qbank.lock().map_err(|e| AppError::Other(e.to_string()))?;
    qbank.clear();
    Ok(())
}

#[tauri::command]
pub async fn course_match_question(
    state: tauri::State<'_, course::CourseState>,
    question: String,
) -> Result<Option<String>, AppError> {
    let bank = state.qbank.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(course::qbank::match_question(&question, &bank).map(|(answer, _score)| answer))
}

#[tauri::command]
pub async fn course_report_progress(
    state: tauri::State<'_, course::CourseState>,
    app: tauri::AppHandle,
    videos: u32,
    quizzes: u32,
    chapter: String,
    status: String,
) -> Result<(), AppError> {
    {
        let mut progress = state.progress.lock().map_err(|e| AppError::Other(e.to_string()))?;
        progress.videos_completed = videos;
        progress.quizzes_answered = quizzes;
        progress.current_chapter = chapter.clone();
        progress.status = status.clone();
    }

    let _ = app.emit("course-progress", serde_json::json!({
        "videos_completed": videos,
        "quizzes_answered": quizzes,
        "current_chapter": chapter,
        "status": status,
    }));

    Ok(())
}

#[tauri::command]
pub async fn course_start(app: tauri::AppHandle) -> Result<(), AppError> {
    use tauri::Manager;
    let window = app
        .get_webview_window("course-chaoxing")
        .ok_or_else(|| AppError::Other("网课窗口未打开，请先点击“打开网课窗口”".to_string()))?;
    window
        .eval("if(window.__COURSE_HELPER__){window.__COURSE_HELPER__.start()}else{console.warn('CourseHelper not loaded')}")
        .map_err(|e| AppError::Other(format!("启动刷课失败: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn course_stop(app: tauri::AppHandle) -> Result<(), AppError> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("course-chaoxing") {
        let _ = window.eval("if(window.__COURSE_HELPER__){window.__COURSE_HELPER__.stop()}");
    }
    Ok(())
}

#[tauri::command]
pub async fn course_set_speed(app: tauri::AppHandle, speed: f64) -> Result<(), AppError> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("course-chaoxing") {
        let js = format!(
            "if(window.__COURSE_HELPER__){{window.__COURSE_HELPER__.setSpeed({})}}",
            speed
        );
        let _ = window.eval(js);
    }
    Ok(())
}
