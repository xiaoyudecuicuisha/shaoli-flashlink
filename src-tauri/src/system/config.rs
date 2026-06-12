//! 应用配置管理：账号存储、AES-256-GCM 密码加密、配置加载/保存/迁移

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use sha2::{Sha256, Digest};
use base64::Engine;

// ── 账号条目 ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountEntry {
    pub username: String,
    pub password: String,
    pub operator: String, // "cmcc" | "unicom" | "telecom"
}

// ── 应用配置 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub accounts: Vec<AccountEntry>,
    #[serde(default)]
    pub active_account: String, // 当前活跃账号的 username
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub pet_enabled: bool,
    #[serde(default)]
    pub whiteboard_dir: String, // 白板存储目录，空字符串表示使用默认路径
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            active_account: String::new(),
            autostart: false,
            theme: "dark".to_string(),
            pet_enabled: false,
            whiteboard_dir: String::new(),
        }
    }
}

/// 旧版配置格式（用于迁移）
#[derive(Debug, Clone, Deserialize)]
pub struct LegacyConfig {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub operator: String,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

// ── 路径 ──

/// 获取配置文件路径（与 exe 同目录）
fn config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config.json")
}

// ── 密码加密 ──

/// 获取机器 ID（Windows MachineGuid）
fn get_machine_id() -> String {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\Cryptography", KEY_READ) {
        if let Ok(guid) = key.get_value::<String, _>("MachineGuid") {
            return guid;
        }
    }
    // Fallback: 生成随机 ID 并持久化，避免所有用户共享同一密钥
    let fallback_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".shaoli");
    let fallback_path = fallback_dir.join("machine-id");
    if let Ok(id) = fs::read_to_string(&fallback_path) {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return id;
        }
    }
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("failed to generate random machine ID");
    let id = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let _ = fs::create_dir_all(&fallback_dir);
    let _ = fs::write(&fallback_path, &id);
    id
}

/// 从机器 ID + 盐值派生 256 位密钥 (v1)
fn derive_key() -> [u8; 32] {
    let machine_id = get_machine_id();
    let mut hasher = Sha256::new();
    hasher.update(b"sxl-flashlink-salt-");
    hasher.update(machine_id.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// 从机器 ID + 新盐值派生 256 位密钥 (v2)
fn derive_key_v2() -> [u8; 32] {
    let machine_id = get_machine_id();
    let mut hasher = Sha256::new();
    // 可通过编译时环境变量 SALT_V2 覆盖（CI 构建时注入随机值）
    let salt = option_env!("SALT_V2").unwrap_or("sxl-flashlink-v2-default");
    hasher.update(salt.as_bytes());
    hasher.update(machine_id.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// 加密密码（AES-256-GCM）
fn encrypt_password(plaintext: &str) -> String {
    let key = derive_key_v2();
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();

    // 生成随机 12 字节 nonce（无 fallback，失败即报错）
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)
        .expect("CRITICAL: failed to generate random nonce — cannot encrypt securely");
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap_or_default();

    // 格式: nonce_base64:ciphertext_base64
    let nonce_b64 = base64::engine::general_purpose::STANDARD.encode(&nonce_bytes);
    let ct_b64 = base64::engine::general_purpose::STANDARD.encode(&ciphertext);
    format!("enc:v2:{}:{}", nonce_b64, ct_b64)
}

/// 解密密码（AES-256-GCM）—— 兼容 v1 和 v2 格式
fn decrypt_password(encrypted: &str) -> String {
    // 识别版本并选择对应密钥
    let (version, key) = if encrypted.starts_with("enc:v2:") {
        ("enc:v2:", derive_key_v2())
    } else if encrypted.starts_with("enc:v1:") {
        ("enc:v1:", derive_key())
    } else {
        return encrypted.to_string(); // 明文，直接返回
    };

    let parts: Vec<&str> = encrypted[version.len()..].splitn(2, ':').collect();
    if parts.len() != 2 {
        return encrypted.to_string(); // 格式错误，返回原值
    }

    let nonce_bytes = match base64::engine::general_purpose::STANDARD.decode(parts[0]) {
        Ok(b) => b,
        Err(_) => return encrypted.to_string(),
    };
    let ciphertext = match base64::engine::general_purpose::STANDARD.decode(parts[1]) {
        Ok(b) => b,
        Err(_) => return encrypted.to_string(),
    };

    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);

    match cipher.decrypt(nonce, ciphertext.as_ref()) {
        Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_default(),
        Err(_) => encrypted.to_string(), // 解密失败，返回原值
    }
}

// ── 加载 ──

/// 加载配置（自动迁移旧版格式，自动解密密码）
pub fn load_config() -> AppConfig {
    let path = config_path();
    tracing::info!("load_config: 配置路径 = {:?}", path);

    if !path.exists() {
        tracing::info!("load_config: 配置文件不存在，尝试从旧版迁移");
        // 尝试从旧版 config.ini 迁移
        if let Some(config) = migrate_from_ini(&path) {
            tracing::info!("load_config: 从旧版 config.ini 迁移成功");
            return config;
        }
        tracing::info!("load_config: 使用默认配置");
        return AppConfig::default();
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("load_config: 读取配置文件失败: {}", e);
            return AppConfig::default();
        }
    };

    // 先尝试新版格式
    if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
        tracing::info!("load_config: 新版格式解析成功, 账号数={}", config.accounts.len());
        // 解密密码
        for account in &mut config.accounts {
            account.password = decrypt_password(&account.password);
        }
        return config;
    }

    // 回退：尝试旧版格式（有顶层 username/password/operator）
    if let Ok(legacy) = serde_json::from_str::<LegacyConfig>(&content) {
        tracing::info!("load_config: 旧版格式解析成功, 迁移到新格式");
        let mut config = AppConfig {
            autostart: legacy.autostart,
            theme: legacy.theme,
            ..Default::default()
        };
        // 如果旧版有账号数据，迁移到 accounts 数组
        if !legacy.username.is_empty() {
            let operator = normalize_operator(&legacy.operator);
            config.accounts.push(AccountEntry {
                username: legacy.username,
                password: legacy.password,
                operator: operator.clone(),
            });
            config.active_account = config.accounts[0].username.clone();
        }
        // 静默保存新版格式（加密）
        let _ = save_config(&config);
        return config;
    }

    tracing::warn!("load_config: 无法解析配置文件，使用默认配置");
    AppConfig::default()
}

// ── 保存 ──

/// 保存完整配置（加密密码）
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    tracing::info!("save_config: 保存配置到 {:?}", path);

    // 创建加密版本的配置
    let mut encrypted_config = config.clone();
    for account in &mut encrypted_config.accounts {
        account.password = encrypt_password(&account.password);
    }

    let json = serde_json::to_string_pretty(&encrypted_config).map_err(|e| {
        tracing::error!("save_config: JSON 序列化失败: {}", e);
        e.to_string()
    })?;
    fs::write(&path, json).map_err(|e| {
        tracing::error!("save_config: 写入文件失败: {}", e);
        format!("保存配置失败: {}", e)
    })?;

    tracing::info!("save_config: 保存成功");
    Ok(())
}

/// Merge 保存：只覆盖 patch 中的非空/非默认字段
pub fn save_config_patch(patch: &serde_json::Value) -> Result<(), String> {
    let mut config = load_config();

    if let Some(accounts) = patch.get("accounts") {
        if let Ok(accs) = serde_json::from_value::<Vec<AccountEntry>>(accounts.clone()) {
            config.accounts = accs;
        }
    }
    if let Some(active) = patch.get("active_account").and_then(|v| v.as_str()) {
        config.active_account = active.to_string();
    }
    if let Some(autostart) = patch.get("autostart").and_then(|v| v.as_bool()) {
        config.autostart = autostart;
    }
    if let Some(theme) = patch.get("theme").and_then(|v| v.as_str()) {
        config.theme = theme.to_string();
    }
    if let Some(dir) = patch.get("whiteboard_dir").and_then(|v| v.as_str()) {
        config.whiteboard_dir = dir.to_string();
    }
    if let Some(enabled) = patch.get("pet_enabled").and_then(|v| v.as_bool()) {
        config.pet_enabled = enabled;
    }

    save_config(&config)
}

// ── 账号管理 ──

/// 保存/更新账号并设为活跃
pub fn save_account(username: &str, password: &str, operator: &str) -> Result<(), String> {
    tracing::info!("save_account: username={}, operator={}", username, operator);
    let mut config = load_config();
    let op = normalize_operator(operator);

    // 查找已有账号并更新
    if let Some(entry) = config.accounts.iter_mut().find(|a| a.username == username) {
        tracing::info!("save_account: 更新已有账号");
        entry.password = password.to_string();
        entry.operator = op;
    } else {
        tracing::info!("save_account: 新增账号");
        config.accounts.push(AccountEntry {
            username: username.to_string(),
            password: password.to_string(),
            operator: op,
        });
    }

    config.active_account = username.to_string();
    save_config(&config)
}

/// 删除指定账号
pub fn delete_account(username: &str) -> Result<(), String> {
    let mut config = load_config();
    config.accounts.retain(|a| a.username != username);
    if config.active_account == username {
        config.active_account = config
            .accounts
            .first()
            .map(|a| a.username.clone())
            .unwrap_or_default();
    }
    save_config(&config)
}

/// 切换活跃账号
pub fn set_active_account(username: &str) -> Result<(), String> {
    let mut config = load_config();
    if config.accounts.iter().any(|a| a.username == username) {
        config.active_account = username.to_string();
        save_config(&config)
    } else {
        Err("账号不存在".to_string())
    }
}

/// 获取所有账号
pub fn get_accounts() -> Vec<AccountEntry> {
    load_config().accounts
}

// ── 工具函数 ──

/// 归一化运营商名称
fn normalize_operator(op: &str) -> String {
    match op {
        "cmcc" | "移动" | "中国移动" => "cmcc".to_string(),
        "unicom" | "联通" | "中国联通" => "unicom".to_string(),
        "telecom" | "电信" | "中国电信" => "telecom".to_string(),
        _ => "cmcc".to_string(),
    }
}

/// 从旧版 Python config.ini 迁移
fn migrate_from_ini(_new_path: &PathBuf) -> Option<AppConfig> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))?;
    let ini_path = exe_dir.join("config.ini");
    if !ini_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&ini_path).ok()?;
    let mut config = AppConfig::default();
    let mut username = String::new();
    let mut password = String::new();
    let mut operator = String::new();

    for line in content.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("username = ") {
            username = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("password = ") {
            password = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("operator = ") {
            operator = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("autostart = ") {
            config.autostart = val.trim() == "True";
        } else if let Some(val) = line.strip_prefix("theme = ") {
            config.theme = val.trim().to_string();
        }
    }

    // 迁移账号到 accounts 数组
    if !username.is_empty() {
        let op = normalize_operator(&operator);
        config.accounts.push(AccountEntry {
            username: username.clone(),
            password,
            operator: op,
        });
        config.active_account = username;
    }

    // 保存为新格式
    let _ = save_config(&config);
    Some(config)
}
