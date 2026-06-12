//! QQ 空间数据解析：HTML (scraper) + JSON 解析、合并去重

use chrono::{NaiveDateTime, TimeZone, Utc};
use scraper::{Html, Selector};
use regex::Regex;

use super::models::{Comment, Moment};

/// 解析历史消息 HTML
/// 对应 Python 的 process_batch_messages 函数
pub fn parse_html_message(html_bytes: &[u8]) -> (Vec<Moment>, Vec<(String, String, String)>) {
    let mut moments = Vec::new();
    let mut friends = Vec::new();
    
    let message = try_decode(html_bytes);

    let html = process_old_html(&message);
    if !html.contains("li") {
        return (moments, friends);
    }
    
    let document = Html::parse_document(&html);
    
    let single_selector = Selector::parse("li.f-single.f-s-s").unwrap();
    let name_selector = Selector::parse("a.f-name.q_namecard").unwrap();
    let time_selector = Selector::parse("div.info-detail").unwrap();
    let text_selector = Selector::parse("p.txt-box-title.ellipsis-one").unwrap();
    let img_selector = Selector::parse("a.img-item img").unwrap();
    
    for element in document.select(&single_selector) {
        if let Some(friend_element) = element.select(&name_selector).next() {
            let friend_name = friend_element.text().collect::<String>();
            let friend_qq = friend_element
                .value()
                .attr("link")
                .map(|s| s[9..].to_string())
                .unwrap_or_default();
            let friend_link = friend_element
                .value()
                .attr("href")
                .map(|s| s.to_string())
                .unwrap_or_default();
            friends.push((friend_name, friend_qq, friend_link));
        }
        
        let time_element = element.select(&time_selector).next();
        let text_element = element.select(&text_selector).next();
        let img_element = element.select(&img_selector).next();
        
        if let (Some(time_el), Some(text_el)) = (time_element, text_element) {
            let put_time = time_el
                .text()
                .collect::<String>()
                .replace('\u{a0}', " ");
            let text = text_el
                .text()
                .collect::<String>()
                .replace('\u{a0}', " ");
            let img = img_element
                .and_then(|el| el.value().attr("src"))
                .map(|s| s.to_string());
            
            let images = img.into_iter().collect();
            
            moments.push(Moment {
                time: put_time,
                content: text,
                images,
                comments: Vec::new(),
            });
        }
    }
    
    (moments, friends)
}

/// 解析未删除说说 JSON
/// 对应 Python 的 get_visible_moments_list 中的解析逻辑
pub fn parse_json_moments(json: &serde_json::Value) -> Vec<Moment> {
    let mut moments = Vec::new();
    
    let msglist = match json.get("msglist").and_then(|v| v.as_array()) {
        Some(list) => list,
        None => return moments,
    };
    
    for item in msglist {
        let content = item
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let nickname = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("未知用户");
        let created_time = item
            .get("created_time")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        
        let time = format_timestamp(created_time);
        let content_with_nickname = format!("{} ：{}", nickname, content);
        
        let mut images = Vec::new();
        if let Some(pic_array) = item.get("pic").and_then(|v| v.as_array()) {
            for pic in pic_array {
                if let Some(url) = pic.get("url1").and_then(|v| v.as_str()) {
                    images.push(url.to_string());
                }
            }
        }
        if let Some(video_array) = item.get("video").and_then(|v| v.as_array()) {
            for video in video_array {
                if let Some(url) = video.get("url1").and_then(|v| v.as_str()) {
                    images.push(url.to_string());
                }
            }
        }
        
        let mut comments = Vec::new();
        if let Some(comment_array) = item.get("commentlist").and_then(|v| v.as_array()) {
            for comment in comment_array {
                let comment_content = comment
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let comment_time = comment
                    .get("createTime2")
                    .or_else(|| comment.get("create_time2"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let comment_nickname = comment
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let comment_uin = comment
                    .get("uin")
                    .and_then(|v| v.as_i64())
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                
                comments.push(Comment {
                    time: comment_time,
                    content: comment_content,
                    nickname: comment_nickname,
                    uin: comment_uin,
                });
            }
        }
        
        moments.push(Moment {
            time,
            content: content_with_nickname,
            images,
            comments,
        });
    }
    
    moments
}

/// 处理旧版 HTML 格式
/// 对应 Python 的 ToolsUtil.process_old_html
fn process_old_html(message: &str) -> String {
    let re = Regex::new(r"\\x[0-9a-fA-F]{2}").unwrap();
    let new_text = re.replace_all(message, |caps: &regex::Captures| {
        let hex = &caps[0];
        let byte = u8::from_str_radix(&hex[2..], 16).unwrap_or(b'?');
        (byte as char).to_string()
    });
    
    let start_string = "html:'";
    let end_string = "',opuin";

    if let Some(start_idx) = new_text.find(start_string) {
        let content_start = start_idx + start_string.len();
        if let Some(end_idx) = new_text.find(end_string) {
            let html = &new_text[content_start..end_idx];
            let re = Regex::new(r"\s+").unwrap();
            let html = re.replace_all(html, " ");
            return html.replace('\\', "");
        }
    }
    
    new_text.to_string()
}

/// 格式化时间戳
/// 对应 Python 的 ToolsUtil.format_timestamp
fn format_timestamp(timestamp: i64) -> String {
    if let Some(dt) = Utc.timestamp_opt(timestamp, 0).single() {
        dt.format("%Y年%m月%d日 %H:%M:%S").to_string()
    } else {
        "未知时间".to_string()
    }
}

/// 合并去重两个说说列表
pub fn merge_moments(list1: Vec<Moment>, list2: Vec<Moment>) -> Vec<Moment> {
    let mut merged = list1;
    
    for moment in list2 {
        let content = extract_content(&moment.content);
        let is_duplicate = merged.iter().any(|m| {
            let existing_content = extract_content(&m.content);
            existing_content == content
        });
        
        if !is_duplicate {
            merged.push(moment);
        }
    }
    
    merged.sort_by(|a, b| {
        let time_a = parse_time(&a.time);
        let time_b = parse_time(&b.time);
        time_b.cmp(&time_a)
    });
    
    merged
}

/// 提取说说内容（去掉昵称前缀）
fn extract_content(content: &str) -> &str {
    content.split_once('：').map(|x| x.1)
        .unwrap_or(content)
        .trim()
}

/// 解析时间字符串为可比较的值
fn parse_time(time_str: &str) -> NaiveDateTime {
    let formats = [
        "%Y年%m月%d日 %H:%M:%S",
        "%Y年%m月%d日 %H:%M",
    ];
    
    for format in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, format) {
            return dt;
        }
    }
    
    Utc.timestamp_opt(0, 0).single().unwrap().naive_utc()
}

/// 尝试多种编码解码
fn try_decode(data: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(data) {
        return text.to_string();
    }

    String::from_utf8_lossy(data).to_string()
}
