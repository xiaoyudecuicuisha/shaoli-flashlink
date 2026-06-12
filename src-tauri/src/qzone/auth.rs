//! QQ 空间 QR 登录流程：xlogin → ptqrshow → ptqrlogin → check_sig

use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::models::{Cookies, LoginStatus};

const XLOGIN_URL: &str = "https://xui.ptlogin2.qq.com/cgi-bin/xlogin?\
proxy_url=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv6%2Fportal%2Fproxy.html&\
daid=5&hide_title_bar=1&low_login=0&qlogin_auto_login=1&no_verifyimg=1&\
link_target=blank&appid=549000912&style=22&target=self&\
s_url=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv5%2Floginsucc.html%3Fpara%3Dizone&\
pt_qr_app=%E6%89%8B%E6%9C%BAQQ%E7%A9%BA%E9%97%B4&\
pt_qr_link=https%3A%2F%2Fz.qzone.com%2Fdownload.html&\
self_regurl=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv6%2Freg%2Findex.html&\
pt_qr_help_link=https%3A%2F%2Fz.qzone.com%2Fdownload.html&pt_no_auth=0";

const LOGIN_SUCC_REFERER: &str =
    "https://qzs.qq.com/qzone/v5/loginsucc.html?para=izone";

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// 计算 bkn 鉴权参数（对应 Python 的 bkn 函数）
pub fn bkn(p_skey: &str) -> i64 {
    let mut t: i64 = 5381;
    for c in p_skey.chars() {
        t = t.wrapping_add(t.wrapping_shl(5)).wrapping_add(c as i64);
    }
    t & 2147483647
}

/// 计算 ptqrtoken（对应 Python 的 ptqrToken 函数）
pub fn ptqr_token(qrsig: &str) -> i64 {
    let mut e: i64 = 0;
    for c in qrsig.chars() {
        e = e.wrapping_add(e.wrapping_shl(5)).wrapping_add(c as i64);
    }
    2147483647 & e
}

/// 把 Set-Cookie 头合并进 cookie_map
/// 自动跳过"删除型" Set-Cookie（value 为空、或带 Max-Age=0），防止后写覆盖前写真实值。
fn merge_set_cookies(
    resp: &reqwest::Response,
    cookie_map: &Arc<Mutex<HashMap<String, String>>>,
) {
    let mut map = cookie_map.lock().unwrap_or_else(|e| e.into_inner());
    for v in resp.headers().get_all("set-cookie").iter() {
        if let Ok(s) = v.to_str() {
            let mut parts = s.split(';');
            let first = parts.next().unwrap_or("").trim();
            if let Some((name, val)) = first.split_once('=') {
                let name = name.trim();
                let val = val.trim();
                let is_delete = val.is_empty()
                    || parts.any(|attr| {
                        let attr = attr.trim();
                        if attr.eq_ignore_ascii_case("Max-Age=0") {
                            return true;
                        }
                        attr.split_once('=')
                            .map(|(k, v)| {
                                k.trim().eq_ignore_ascii_case("max-age") && v.trim() == "0"
                            })
                            .unwrap_or(false)
                    });
                if !is_delete {
                    map.insert(name.to_string(), val.to_string());
                }
            }
        }
    }
}

/// 把 cookie_map 序列化成 Cookie 请求头
fn cookie_header(cookie_map: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let map = cookie_map.lock().unwrap_or_else(|e| e.into_inner());
    map.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Step 1：访问 xlogin 拿 `pt_login_sig` 等初始 cookies。
/// 这是 QQ 扫码登录的强制前置步骤，否则后续 check_sig 无法拿到 p_skey。
pub async fn get_initial_cookies(
    client: &Client,
    cookie_map: &Arc<Mutex<HashMap<String, String>>>,
) -> Result<(), String> {
    let resp = client
        .get(XLOGIN_URL)
        .header("User-Agent", USER_AGENT)
        .header("Referer", "https://qzs.qq.com/")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("xlogin 请求失败: {}", e))?;

    merge_set_cookies(&resp, cookie_map);

    let map = cookie_map.lock().map_err(|e| e.to_string())?;
    if !map.contains_key("pt_login_sig") {
        return Err("xlogin 阶段未返回 pt_login_sig".to_string());
    }
    Ok(())
}

/// Step 2：获取登录二维码，同时把 ptqrshow 阶段返回的 cookies 并入 cookie_map。
/// 返回 (二维码图片数据, qrsig)。
pub async fn get_qr_code(
    client: &Client,
    cookie_map: &Arc<Mutex<HashMap<String, String>>>,
) -> Result<(Vec<u8>, String), String> {
    let url = "https://ssl.ptlogin2.qq.com/ptqrshow?appid=549000912&e=2&l=M&s=3&d=72&v=4&t=0.8692955245720428&daid=5&pt_3rd_aid=0";

    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Referer", XLOGIN_URL)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("获取二维码失败: {}", e))?;

    merge_set_cookies(&response, cookie_map);

    let qrsig = cookie_map
        .lock()
        .map_err(|e| e.to_string())?
        .get("qrsig")
        .cloned()
        .ok_or("获取 qrsig 失败")?;

    let body = response
        .bytes()
        .await
        .map_err(|e| format!("读取二维码数据失败: {}", e))?;

    Ok((body.to_vec(), qrsig))
}

/// Step 3 + 4：轮询登录状态；扫码成功后请求 check_sig 拿最终 cookies。
pub async fn poll_login(
    client: &Client,
    no_redirect_client: &Client,
    cookie_map: &Arc<Mutex<HashMap<String, String>>>,
    qrsig: &str,
) -> LoginStatus {
    let ptqrtoken = ptqr_token(qrsig);

    let login_sig = cookie_map
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get("pt_login_sig")
        .cloned()
        .unwrap_or_default();

    let url = format!(
        "https://ssl.ptlogin2.qq.com/ptqrlogin?\
u1=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv5%2Floginsucc.html%3Fpara%3Dizone&\
ptqrtoken={}&ptredirect=0&h=1&t=1&g=1&from_ui=1&ptlang=2052&\
action=0-0-{}&js_ver=20032614&js_type=1&\
login_sig={}&pt_uistyle=40&aid=549000912&daid=5&",
        ptqrtoken,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        login_sig,
    );

    let response = match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Referer", XLOGIN_URL)
        .header("Cookie", cookie_header(cookie_map))
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return LoginStatus::Failed { message: format!("请求失败: {}", e) },
    };

    merge_set_cookies(&response, cookie_map);

    let text = match response.text().await {
        Ok(t) => t,
        Err(e) => return LoginStatus::Failed { message: format!("读取响应失败: {}", e) },
    };

    if text.contains("二维码未失效") {
        LoginStatus::WaitingScan
    } else if text.contains("二维码认证中") {
        LoginStatus::Scanned
    } else if text.contains("二维码已失效") {
        LoginStatus::Failed {
            message: "二维码已失效，请点击刷新二维码".to_string(),
        }
    } else if text.contains("ptuiCB('0','0'") || text.contains("登录成功") {
        let ptsigx = match extract_ptsigx(&text) {
            Some(s) => s,
            None => return LoginStatus::Failed { message: "解析 ptsigx 失败".to_string() },
        };

        let uin = cookie_map
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get("uin")
            .cloned()
            .unwrap_or_default();

        let check_url = format!(
            "https://ptlogin2.qzone.qq.com/check_sig?pttype=1&uin={}&\
service=ptqrlogin&nodirect=0&ptsigx={}&\
s_url=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv5%2Floginsucc.html%3Fpara%3Dizone&\
f_url=&ptlang=2052&ptredirect=100&aid=549000912&daid=5&\
j_later=0&low_login_hour=0&regmaster=0&pt_login_type=3&\
pt_aid=0&pt_aaid=16&pt_light=0&pt_3rd_aid=0",
            uin, ptsigx
        );

        match no_redirect_client
            .get(&check_url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", LOGIN_SUCC_REFERER)
            .header("Cookie", cookie_header(cookie_map))
            .send()
            .await
        {
            Ok(r) => {
                merge_set_cookies(&r, cookie_map);
                let map = cookie_map.lock().unwrap_or_else(|e| e.into_inner());
                let p_skey = map.get("p_skey").cloned().unwrap_or_default();
                let uin = map.get("uin").cloned().unwrap_or_default();
                let skey = map.get("skey").cloned().unwrap_or_default();
                let p_uin = map.get("p_uin").cloned().unwrap_or_default();
                let pt4_token = map.get("pt4_token").cloned().unwrap_or_default();
                drop(map);

                if p_skey.is_empty() {
                    LoginStatus::Failed {
                        message: "获取 cookie 失败：check_sig 响应未包含 p_skey".to_string(),
                    }
                } else {
                    LoginStatus::Success {
                        cookies: Cookies {
                            p_skey,
                            uin,
                            skey,
                            p_uin,
                            pt4_token,
                        },
                    }
                }
            }
            Err(e) => LoginStatus::Failed { message: format!("获取 cookie 失败: {}", e) },
        }
    } else {
        LoginStatus::Failed {
            message: format!(
                "用户取消登录或未知响应: {}",
                {
                    let end = text
                        .char_indices()
                        .take_while(|&(i, _)| i < 80)
                        .last()
                        .map(|(i, c)| i + c.len_utf8())
                        .unwrap_or(0);
                    &text[..end]
                }
            ),
        }
    }
}

/// 从响应文本中提取 ptsigx
fn extract_ptsigx(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"ptsigx=(.*?)&").ok()?;
    re.captures(text)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}
