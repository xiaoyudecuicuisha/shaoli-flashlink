//! 深澜认证流程：获取 Challenge → 加密密码 → 登录 → 验证在线状态

use super::base64::srun_base64_encode;
use super::xencode::xencode;
use crate::system::school;
use hmac::{Hmac, Mac};
use md5::Md5;
use sha1::{Digest, Sha1};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacMd5 = Hmac<Md5>;

/// 解析 JSONP 响应：jQuery123_456({"key":"value"})
fn parse_jsonp(text: &str) -> Option<serde_json::Value> {
    let start = text.find('(')?;
    let end = text.rfind(')')?;
    let json_str = &text[start + 1..end];
    match serde_json::from_str(json_str) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::error!("JSONP JSON 解析失败: {}, 原文: {}", e, &text[..text.len().min(200)]);
            None
        }
    }
}

/// 获取当前时间戳（毫秒）
fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// 获取 Challenge Token
async fn get_challenge(client: &reqwest::Client, username: &str, ip: &str) -> Result<String, String> {
    let profile = school::load_school_profile();
    let ts = timestamp_ms();
    let callback = format!("jQuery{}_{}", ts, ts);

    tracing::info!("get_challenge: username={}, ip={}", username, ip);

    let resp = client
        .get(format!("http://{}/cgi-bin/get_challenge", profile.auth_server))
        .query(&[
            ("callback", callback.as_str()),
            ("username", username),
            ("ip", ip),
            ("_", &ts.to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::error!("get_challenge 网络请求失败: {}", e);
            format!("网络请求失败: {}", e)
        })?;

    let text = resp
        .text()
        .await
        .map_err(|e| {
            tracing::error!("get_challenge 读取响应失败: {}", e);
            format!("读取响应失败: {}", e)
        })?;

    let data = parse_jsonp(&text).ok_or("Challenge 响应解析失败")?;

    if data["error"] == "ok" {
        let token = data["challenge"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or("Challenge 字段缺失".to_string())?;
        tracing::info!("get_challenge 成功, token 长度: {}", token.len());
        Ok(token)
    } else {
        let err_msg = data["error_msg"].as_str().unwrap_or("未知错误");
        tracing::error!("get_challenge 失败: {}", err_msg);
        Err(format!("获取 Challenge 失败: {}", err_msg))
    }
}

/// 执行登录
pub async fn do_login(
    username: &str,
    password: &str,
    operator: &str,
    ip: &str,
) -> Result<String, String> {
    let profile = school::load_school_profile();
    let full_username = format!("{}{}", username, operator);
    tracing::info!("do_login: full_username={}, ip={}, auth_server={}", full_username, ip, profile.auth_server);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 1. 获取 Challenge
    let token = get_challenge(&client, &full_username, ip).await?;

    // 2. 构造 Info JSON
    let info_json = serde_json::json!({
        "username": full_username,
        "password": password,
        "ip": ip,
        "acid": "1",
        "enc_ver": "srun_bx1"
    })
    .to_string();

    // 3. XEncode + 自定义 Base64
    let xencoded = xencode(info_json.as_bytes(), token.as_bytes());
    let info_b64 = srun_base64_encode(&xencoded);
    let info_param = format!("{{SRBX1}}{}", info_b64);

    // 4. HMAC-MD5
    let mut mac =
        HmacMd5::new_from_slice(token.as_bytes()).map_err(|e| format!("HMAC 初始化失败: {}", e))?;
    mac.update(password.as_bytes());
    let hmd5 = hex::encode(mac.finalize().into_bytes());
    let password_param = format!("{{MD5}}{}", hmd5);

    // 5. 构造 Chksum (SHA1)
    let chkstr = [
        token.as_str(),
        &full_username,
        token.as_str(),
        hmd5.as_str(),
        token.as_str(),
        "1",
        token.as_str(),
        ip,
        token.as_str(),
        "200",
        token.as_str(),
        "1",
        token.as_str(),
        info_param.as_str(),
    ]
    .concat();
    let mut hasher = Sha1::new();
    hasher.update(chkstr.as_bytes());
    let chksum = hex::encode(hasher.finalize());

    // 6. 发送登录请求
    let ts = timestamp_ms();
    let callback = format!("jQuery{}_{}", ts, ts);

    let resp = client
        .get(format!("http://{}/cgi-bin/srun_portal", profile.auth_server))
        .query(&[
            ("callback", callback.as_str()),
            ("action", "login"),
            ("username", full_username.as_str()),
            ("password", password_param.as_str()),
            ("ac_id", "1"),
            ("ip", ip),
            ("chksum", chksum.as_str()),
            ("info", info_param.as_str()),
            ("n", "200"),
            ("type", "1"),
            ("os", "windows+10"),
            ("name", "windows"),
            ("double_stack", "0"),
            ("_", &ts.to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::error!("srun_portal 登录请求失败: {}", e);
            format!("登录请求失败: {}", e)
        })?;

    let text = resp
        .text()
        .await
        .map_err(|e| {
            tracing::error!("srun_portal 读取登录响应失败: {}", e);
            format!("读取登录响应失败: {}", e)
        })?;

    let data = parse_jsonp(&text).ok_or("登录响应解析失败")?;

    if data["error"] == "ok" {
        tracing::info!("srun_portal 登录成功");
        Ok("登录成功".to_string())
    } else {
        let err_msg = data["error_msg"]
            .as_str()
            .unwrap_or("未知错误")
            .to_string();
        tracing::error!("srun_portal 登录失败: {}", err_msg);
        Err(err_msg)
    }
}

/// 检查是否已在线
pub async fn check_online() -> bool {
    let profile = school::load_school_profile();
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("check_online: 创建 HTTP 客户端失败: {}", e);
            return false;
        }
    };

    match client
        .get(format!("http://{}/cgi-bin/rad_user_info", profile.auth_server))
        .send()
        .await
    {
        Ok(resp) => {
            if let Ok(text) = resp.text().await {
                let text = text.trim();
                let online = !text.is_empty() && !text.starts_with("not_online");
                tracing::info!("check_online: 结果={}, 响应长度={}", online, text.len());
                online
            } else {
                tracing::error!("check_online: 读取响应失败");
                false
            }
        }
        Err(e) => {
            tracing::error!("check_online: 请求失败: {}", e);
            false
        }
    }
}

/// 获取本机 IP（通过 UDP 探测）
pub fn get_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("223.5.5.5:53").ok()?;
    let addr = socket.local_addr().ok()?;
    let ip = addr.ip().to_string();
    tracing::info!("get_ip: 获取本机 IP = {}", ip);
    Some(ip)
}

/// 检查认证服务器是否可达
#[allow(dead_code)]
pub async fn check_server_reachable() -> bool {
    let profile = school::load_school_profile();
    use std::net::TcpStream;
    use std::time::Duration;
    let addr = match format!("{}:80", profile.auth_server).parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok()
}
