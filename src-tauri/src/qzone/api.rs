//! QQ 空间 API 调用：历史消息、可见动态、用户信息

use reqwest::Client;
use std::time::Duration;

use super::auth::bkn;
use super::models::{Cookies, QzoneUserInfo};

/// 获取历史消息列表
/// 对应 Python 的 RequestUtil.get_message
pub async fn get_message(
    client: &Client,
    cookies: &Cookies,
    offset: i32,
    count: i32,
) -> Result<Vec<u8>, String> {
    let g_tk = bkn(&cookies.p_skey);
    let uin = cookies.uin.trim_start_matches('o');
    
    let url = "https://user.qzone.qq.com/proxy/domain/ic2.qzone.qq.com/cgi-bin/feeds/feeds2_html_pav_all";
    
    let params = [
        ("uin", uin.to_string()),
        ("begin_time", "0".to_string()),
        ("end_time", "0".to_string()),
        ("getappnotification", "1".to_string()),
        ("getnotifi", "1".to_string()),
        ("has_get_key", "0".to_string()),
        ("offset", offset.to_string()),
        ("set", "0".to_string()),
        ("count", count.to_string()),
        ("useutf8", "1".to_string()),
        ("outputhtmlfeed", "1".to_string()),
        ("scope", "1".to_string()),
        ("format", "jsonp".to_string()),
        ("g_tk", g_tk.to_string()),
    ];
    
    let cookie_str = format!(
        "uin={};skey={};p_uin={};pt4_token={};p_skey={}",
        cookies.uin, cookies.skey, cookies.p_uin, cookies.pt4_token, cookies.p_skey
    );
    
    let response = client
        .get(url)
        .header("authority", "user.qzone.qq.com")
        .header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("accept-language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("cache-control", "no-cache")
        .header("cookie", cookie_str)
        .header("referer", format!("https://user.qzone.qq.com/{}/main", uin))
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .query(&params)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("请求历史消息失败: {}", e))?;
    
    let body = response
        .bytes()
        .await
        .map_err(|e| format!("读取历史消息失败: {}", e))?;
    
    Ok(body.to_vec())
}

/// 获取消息总数
/// 对应 Python 的 RequestUtil.get_message_count
pub async fn get_message_count(client: &Client, cookies: &Cookies) -> Result<i32, String> {
    let mut lower_bound: i32 = 0;
    let mut upper_bound: i32 = 10000000;
    let mut total = upper_bound / 2;
    
    while lower_bound <= upper_bound {
        match get_message(client, cookies, total, 100).await {
            Ok(data) => {
                let text = String::from_utf8_lossy(&data);
                if text.contains("li") {
                    lower_bound = total + 1;
                } else {
                    upper_bound = total - 1;
                }
            }
            Err(e) => {
                return Err(format!("获取消息数量失败: {}", e));
            }
        }
        total = (lower_bound + upper_bound) / 2;
    }
    
    Ok(total)
}

/// 获取用户信息
/// 对应 Python 的 RequestUtil.get_login_user_info
pub async fn get_user_info(client: &Client, cookies: &Cookies) -> Result<QzoneUserInfo, String> {
    let g_tk = bkn(&cookies.p_skey);
    let uin = cookies.uin.trim_start_matches('o');
    
    let url = format!(
        "https://r.qzone.qq.com/fcg-bin/cgi_get_portrait.fcg?g_tk={}&uins={}",
        g_tk, uin
    );
    
    let cookie_str = format!(
        "uin={};skey={};p_uin={};pt4_token={};p_skey={}",
        cookies.uin, cookies.skey, cookies.p_uin, cookies.pt4_token, cookies.p_skey
    );
    
    let response = client
        .get(&url)
        .header("cookie", cookie_str)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("获取用户信息失败: {}", e))?;
    
    let body = response
        .bytes()
        .await
        .map_err(|e| format!("读取用户信息失败: {}", e))?;
    
    let text = try_decode(&body);

    let text = text
        .trim()
        .strip_prefix("portraitCallBack(")
        .and_then(|s| s.strip_suffix(");"))
        .unwrap_or(&text);
    
    let json: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| format!("解析用户信息失败: {}", e))?;
    
    let info = json.get(uin).ok_or("未找到用户信息")?;
    
    let nickname = info
        .get(6)
        .and_then(|v| v.as_str())
        .unwrap_or("未知用户")
        .to_string();
    
    Ok(QzoneUserInfo {
        uin: uin.to_string(),
        nickname,
        avatar_url: format!("https://q.qlogo.cn/headimg_dl?dst_uin={}&spec=640&img_type=jpg", uin),
    })
}

/// 获取未删除说说列表
/// 对应 Python 的 GetAllMomentsUtil.get_visible_moments_list
pub async fn get_visible_moments(
    client: &Client,
    cookies: &Cookies,
    page_size: i32,
    offset: i32,
) -> Result<serde_json::Value, String> {
    let g_tk = bkn(&cookies.p_skey);
    let uin = cookies.uin.trim_start_matches('o');
    
    let url = "https://user.qzone.qq.com/proxy/domain/taotao.qq.com/cgi-bin/emotion_cgi_msglist_v6";
    
    let cookie_str = format!(
        "uin={};skey={};p_uin={};pt4_token={};p_skey={}",
        cookies.uin, cookies.skey, cookies.p_uin, cookies.pt4_token, cookies.p_skey
    );
    
    let params = [
        ("uin", uin.to_string()),
        ("ftype", "0".to_string()),
        ("sort", "0".to_string()),
        ("pos", offset.to_string()),
        ("num", page_size.to_string()),
        ("replynum", "100".to_string()),
        ("g_tk", g_tk.to_string()),
        ("callback", "_preloadCallback".to_string()),
        ("code_version", "1".to_string()),
        ("format", "jsonp".to_string()),
        ("need_private_comment", "1".to_string()),
    ];
    
    let response = client
        .get(url)
        .header("accept", "*/*")
        .header("cookie", cookie_str)
        .header("referer", format!("https://user.qzone.qq.com/{}/main", uin))
        .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
        .query(&params)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("获取说说列表失败: {}", e))?;
    
    let body = response
        .text()
        .await
        .map_err(|e| format!("读取说说列表失败: {}", e))?;
    
    let json_str = body
        .trim()
        .strip_prefix("_preloadCallback(")
        .and_then(|s| s.strip_suffix(");"))
        .unwrap_or(&body);
    
    serde_json::from_str(json_str)
        .map_err(|e| format!("解析说说列表失败: {}", e))
}

/// 尝试多种编码解码
fn try_decode(data: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(data) {
        return text.to_string();
    }

    String::from_utf8_lossy(data).to_string()
}
