//! 学校配置模块：认证服务器、校园站点列表、域名等（HMAC 完整性校验）

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

type HmacSha256 = Hmac<Sha256>;

/// HMAC 密钥，用于保护学校配置文件的完整性
const PROFILE_HMAC_KEY: &[u8] = b"sxl-profile-integrity-key-v1";

/// 学校配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolProfile {
    /// 学校名称
    pub name: String,
    /// 学校代码
    pub code: String,
    /// 认证服务器地址
    pub auth_server: String,
    /// 内网 IP
    pub internal_ip: String,
    /// 学校域名
    pub domain: String,
    /// 站点列表
    pub sites: Vec<SiteConfig>,
}

/// 站点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// 子域名
    pub subdomain: String,
    /// 站点名称
    pub name: String,
}

impl Default for SchoolProfile {
    fn default() -> Self {
        Self {
            name: "绍兴理工学院".to_string(),
            code: "sxlg".to_string(),
            auth_server: "10.210.0.2".to_string(),
            internal_ip: "10.1.0.209".to_string(),
            domain: "sxlg.edu.cn".to_string(),
            sites: vec![
                SiteConfig { subdomain: "jwxt".to_string(), name: "教务系统".to_string() },
                SiteConfig { subdomain: "jwc".to_string(), name: "教务处".to_string() },
                SiteConfig { subdomain: "jw".to_string(), name: "教务".to_string() },
                SiteConfig { subdomain: "www".to_string(), name: "学校官网".to_string() },
                SiteConfig { subdomain: "cas".to_string(), name: "统一认证(SSO)".to_string() },
                SiteConfig { subdomain: "oa".to_string(), name: "办公自动化".to_string() },
                SiteConfig { subdomain: "office".to_string(), name: "办公系统".to_string() },
                SiteConfig { subdomain: "lib".to_string(), name: "图书馆".to_string() },
                SiteConfig { subdomain: "dns".to_string(), name: "DNS管理".to_string() },
                SiteConfig { subdomain: "jcc".to_string(), name: "计财处".to_string() },
                SiteConfig { subdomain: "pay".to_string(), name: "缴费系统".to_string() },
                SiteConfig { subdomain: "kyc".to_string(), name: "科研处".to_string() },
                SiteConfig { subdomain: "app".to_string(), name: "应用平台".to_string() },
                SiteConfig { subdomain: "old".to_string(), name: "旧版网站".to_string() },
                SiteConfig { subdomain: "apply".to_string(), name: "融合门户".to_string() },
                SiteConfig { subdomain: "zsxx".to_string(), name: "招生信息网".to_string() },
                SiteConfig { subdomain: "cjyw".to_string(), name: "就业信息网".to_string() },
                SiteConfig { subdomain: "xgzx".to_string(), name: "学生工作部".to_string() },
                SiteConfig { subdomain: "ztb".to_string(), name: "招投标与采购中心".to_string() },
                SiteConfig { subdomain: "rsw".to_string(), name: "人事处".to_string() },
                SiteConfig { subdomain: "bwc".to_string(), name: "安全保卫部".to_string() },
                SiteConfig { subdomain: "cwb".to_string(), name: "计划财务处".to_string() },
                SiteConfig { subdomain: "zcgl".to_string(), name: "资产与实验室管理处".to_string() },
                SiteConfig { subdomain: "lm".to_string(), name: "后勤管理处".to_string() },
                SiteConfig { subdomain: "txy".to_string(), name: "腾讯云互联网学院".to_string() },
                SiteConfig { subdomain: "xdgc".to_string(), name: "人工智能学院".to_string() },
                SiteConfig { subdomain: "ffy".to_string(), name: "纺织服装学院".to_string() },
                SiteConfig { subdomain: "jgfy".to_string(), name: "建筑环境学院".to_string() },
                SiteConfig { subdomain: "yyjk".to_string(), name: "医药健康学院".to_string() },
                SiteConfig { subdomain: "yywx".to_string(), name: "语言文化学院".to_string() },
                SiteConfig { subdomain: "jjgl".to_string(), name: "经管学院".to_string() },
                SiteConfig { subdomain: "mks".to_string(), name: "马克思主义学院".to_string() },
                SiteConfig { subdomain: "ggjc".to_string(), name: "公共教育学院".to_string() },
                SiteConfig { subdomain: "cypxy".to_string(), name: "蔡元培学院".to_string() },
            ],
        }
    }
}

/// 获取学校配置文件路径
fn get_profile_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("shaoli");
    path.push("school.json");
    path
}

/// 加载学校配置（带 HMAC 完整性校验）
pub fn load_school_profile() -> SchoolProfile {
    let path = get_profile_path();
    
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match verify_and_parse_profile(&content, &path) {
                    Ok(profile) => return profile,
                    Err(e) => {
                        eprintln!("学校配置校验失败: {}, 使用默认配置", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("读取学校配置失败: {}, 使用默认配置", e);
            }
        }
    }
    
    let profile = SchoolProfile::default();

    save_signed_profile(&profile, &path);
    
    profile
}

/// 验证 HMAC 并解析配置；如果文件无 HMAC 则自动签名保存
fn verify_and_parse_profile(content: &str, path: &std::path::Path) -> Result<SchoolProfile, String> {
    let mut json_value: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    let stored_hmac = json_value.as_object_mut()
        .and_then(|obj| obj.remove("_hmac"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let canonical = serde_json::to_string(&json_value)
        .map_err(|e| format!("序列化失败: {}", e))?;

    let mut mac = HmacSha256::new_from_slice(PROFILE_HMAC_KEY)
        .map_err(|_| "HMAC 初始化失败".to_string())?;
    mac.update(canonical.as_bytes());
    let computed_hmac = mac.finalize().into_bytes();
    let computed_b64 = base64::engine::general_purpose::STANDARD.encode(&computed_hmac);

    match stored_hmac {
        Some(stored) if stored == computed_b64 => {
            let profile: SchoolProfile = serde_json::from_value(json_value)
                .map_err(|e| format!("解析配置失败: {}", e))?;
            Ok(profile)
        }
        Some(_) => {
            Err("学校配置文件 HMAC 校验失败（可能被篡改）".to_string())
        }
        None => {
            let profile: SchoolProfile = serde_json::from_value(json_value.clone())
                .map_err(|e| format!("解析配置失败: {}", e))?;
            
            if let Some(obj) = json_value.as_object_mut() {
                obj.insert("_hmac".to_string(), serde_json::Value::String(computed_b64));
                let signed = serde_json::to_string_pretty(&json_value)
                    .map_err(|e| format!("序列化失败: {}", e))?;
                let _ = fs::write(path, signed);
            }
            
            Ok(profile)
        }
    }
}

/// 保存签名后配置（含 HMAC）
fn save_signed_profile(profile: &SchoolProfile, path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut json_value) = serde_json::to_value(profile) {
        if let Ok(canonical) = serde_json::to_string(&json_value) {
            if let Ok(mut mac) = HmacSha256::new_from_slice(PROFILE_HMAC_KEY) {
                mac.update(canonical.as_bytes());
                let hmac_b64 = base64::engine::general_purpose::STANDARD.encode(&mac.finalize().into_bytes());
                if let Some(obj) = json_value.as_object_mut() {
                    obj.insert("_hmac".to_string(), serde_json::Value::String(hmac_b64));
                }
            }
        }
        if let Ok(content) = serde_json::to_string_pretty(&json_value) {
            let _ = fs::write(path, content);
        }
    }
}

/// 保存学校配置（含 HMAC 签名）
pub fn save_school_profile(profile: &SchoolProfile) -> Result<(), String> {
    let path = get_profile_path();
    save_signed_profile(profile, &path);
    Ok(())
}
