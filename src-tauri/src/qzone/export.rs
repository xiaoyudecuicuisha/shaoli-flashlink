//! QQ 空间数据导出：Excel、HTML、图片下载

use std::fs;
use std::path::Path;

use super::models::Moment;

/// 导出为 Excel 文件
/// 格式参考原 Python 项目：说说列表包含时间、内容、图片、评论四列
pub fn export_excel(moments: &[Moment], path: &Path) -> Result<(), String> {
    use rust_xlsxwriter::{Format, Workbook, Color};

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("说说列表").map_err(|e| e.to_string())?;

    // 表头格式：加粗、居中、浅蓝背景、边框
    let header_format = Format::new()
        .set_bold()
        .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter)
        .set_align(rust_xlsxwriter::FormatAlign::Center)
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White)
        .set_border(rust_xlsxwriter::FormatBorder::Thin)
        .set_text_wrap();

    // 内容单元格格式：文本换行、垂直居中、边框
    let wrap_format = Format::new()
        .set_align(rust_xlsxwriter::FormatAlign::Top)
        .set_text_wrap()
        .set_border(rust_xlsxwriter::FormatBorder::Thin);

    // 时间列格式：居中
    let time_format = Format::new()
        .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter)
        .set_align(rust_xlsxwriter::FormatAlign::Center)
        .set_text_wrap()
        .set_border(rust_xlsxwriter::FormatBorder::Thin);

    // 写入表头（带格式）
    worksheet.write_string_with_format(0, 0, "时间", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 1, "内容", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 2, "图片链接", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 3, "评论", &header_format).map_err(|e| e.to_string())?;

    // 设置列宽（按参考项目比例）
    worksheet.set_column_width(0, 22.0).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 65.0).map_err(|e| e.to_string())?;
    worksheet.set_column_width(2, 50.0).map_err(|e| e.to_string())?;
    worksheet.set_column_width(3, 55.0).map_err(|e| e.to_string())?;

    // 冻结首行（表头始终可见）
    worksheet.set_freeze_panes(1, 0).map_err(|e| e.to_string())?;

    // 写入数据
    for (i, moment) in moments.iter().enumerate() {
        let row = (i + 1) as u32;

        worksheet.write_string_with_format(row, 0, &moment.time, &time_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 1, &moment.content, &wrap_format).map_err(|e| e.to_string())?;

        // 图片链接：每个 URL 单独一行，便于阅读
        let images_text = if moment.images.is_empty() {
            String::new()
        } else {
            moment.images.join("\n")
        };
        worksheet.write_string_with_format(row, 2, &images_text, &wrap_format).map_err(|e| e.to_string())?;

        // 评论格式化：每条评论单独一行
        let comments_text: Vec<String> = moment.comments.iter().map(|c| {
            format!("[{}] {}: {}", c.time, c.nickname, c.content)
        }).collect();
        worksheet.write_string_with_format(row, 3, &comments_text.join("\n"), &wrap_format).map_err(|e| e.to_string())?;

        // 根据内容行数设置行高（每行约15像素，最少25像素）
        let content_lines = moment.content.chars().filter(|&c| c == '\n').count() + 1;
        let comment_lines = moment.comments.len().max(1);
        let image_lines = moment.images.len().max(1);
        let max_lines = content_lines.max(comment_lines).max(image_lines) as u32;
        let row_height = (max_lines * 15).max(25).min(400); // 限制最大行高
        worksheet.set_row_height(row, row_height as f64).map_err(|e| e.to_string())?;
    }

    // 保存文件
    workbook.save(path).map_err(|e| format!("保存 Excel 失败: {}", e))?;

    Ok(())
}

/// 导出为 HTML 文件
/// 对应 Python 的 render_html 函数
pub fn export_html(moments: &[Moment], path: &Path, uin: &str, nickname: &str) -> Result<(), String> {
    let avatar_url = format!("https://q.qlogo.cn/headimg_dl?dst_uin={}&spec=640&img_type=jpg", uin);
    
    let mut posts_html = String::new();
    
    for moment in moments {
        // 处理内容
        let content_parts: Vec<&str> = moment.content.splitn(2, '：').collect();
        let (display_nickname, message) = if content_parts.len() == 2 {
            (content_parts[0], content_parts[1])
        } else {
            (nickname, moment.content.as_str())
        };
        
        // HTML 转义 + 替换表情（先转义防 XSS，再替换 emoji 图片标签）
        let display_nickname = replace_em_to_img(&html_escape(display_nickname));
        let message = replace_em_to_img(&html_escape(message));
        
        // 生成图片 HTML
        let mut image_html = String::from(r#"<div class="image">"#);
        for img_url in &moment.images {
            if !img_url.is_empty() && img_url.starts_with("http") {
                // 将图片替换为高清图
                let high_res_url = img_url
                    .replace("/m&ek=1&kp=1", "/s&ek=1&kp=1")
                    .replace("!/m/", "!/s/");
                image_html.push_str(&format!(r#"<img src="{}" alt="图片">"#, html_escape(&high_res_url)));
            }
        }
        image_html.push_str("</div>");
        
        // 生成评论 HTML
        let mut comment_html = String::new();
        for comment in &moment.comments {
            let comment_nickname = replace_em_to_img(&html_escape(&comment.nickname));
            let comment_content = replace_em_to_img(&html_escape(&comment.content));
            let comment_avatar_url = format!(
                "https://q.qlogo.cn/headimg_dl?dst_uin={}&spec=640&img_type=jpg",
                comment.uin
            );
            
            comment_html.push_str(&format!(
                r#"
                <div class="comments">
                    <div class="comment">
                        <div class="avatar">
                            <img src="{}" alt="评论头像">
                        </div>
                        <div class="nickname">{}</div>
                        <div class="time">{}</div>
                        <div class="message">{}</div>
                    </div>
                </div>
                "#,
                comment_avatar_url, comment_nickname, html_escape(&comment.time), comment_content
            ));
        }
        
        // 生成单条说说 HTML
        posts_html.push_str(&format!(
            r#"
            <div class="post">
                <div class="avatar">
                    <img src="{}" alt="头像">
                </div>
                <div class="content">
                    <div class="nickname">{}</div>
                    <div class="time">{}</div>
                    <div class="message">{}</div>
                    {}
                </div>
                {}
            </div>
            "#,
            avatar_url, display_nickname, html_escape(&moment.time), message, image_html, comment_html
        ));
    }
    
    // 生成完整 HTML
    let html_template = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>QQ空间动态 - {}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            background-color: #f5f5f5;
            margin: 0;
            padding: 20px;
        }}
        .post {{
            background-color: #333;
            color: #fff;
            padding: 20px;
            margin: 20px auto;
            border-radius: 10px;
            max-width: 800px;
        }}
        .avatar {{
            float: left;
            margin-right: 20px;
        }}
        .avatar img {{
            width: 50px;
            height: 50px;
            border-radius: 50%;
        }}
        .content {{
            overflow: hidden;
        }}
        .nickname {{
            font-size: 1.2em;
            font-weight: bold;
        }}
        .time {{
            color: #999;
            font-size: 0.9em;
        }}
        .message {{
            margin-top: 10px;
            font-size: 1.1em;
        }}
        .image {{
            margin-top: 10px;
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            grid-gap: 10px;
            justify-items: center;
        }}
        .image img {{
            width: 100%;
            height: auto;
            object-fit: cover;
            max-width: 33vw;
            max-height: 33vh;
            border-radius: 10px;
            cursor: pointer;
        }}
        .comments {{
            margin-top: 5px;
            background-color: #444;
            padding: 2px 10px 10px 10px;
            border-radius: 10px;
            clear: both;
        }}
        .comment {{
            margin-top: 10px;
            padding: 10px;
            background-color: #555;
            border-radius: 10px;
            color: #fff;
        }}
        .comment .avatar img {{
            width: 30px;
            height: 30px;
        }}
        .comment .nickname {{
            font-size: 1em;
            font-weight: bold;
        }}
        .comment .time {{
            font-size: 0.8em;
            color: #aaa;
        }}
        h1 {{
            text-align: center;
            color: #333;
        }}
        .stats {{
            text-align: center;
            color: #666;
            margin-bottom: 20px;
        }}
    </style>
</head>
<body>
    <h1>QQ空间动态回忆</h1>
    <div class="stats">共 {} 条说说</div>
    {}
    <script>
        document.querySelectorAll(".image img").forEach(img => {{
            img.addEventListener("click", function() {{
                window.open(this.src, '_blank');
            }});
        }});
    </script>
</body>
</html>"#,
        html_escape(nickname), moments.len(), posts_html
    );
    
    // 写入文件
    fs::write(path, html_template)
        .map_err(|e| format!("保存 HTML 失败: {}", e))?;
    
    Ok(())
}

/// 下载图片
/// 图片命名格式参考原 Python 项目：{昵称}_{内容截断}.jpg
pub async fn download_images(
    moments: &[Moment],
    path: &Path,
    client: &reqwest::Client,
) -> Result<usize, String> {
    fs::create_dir_all(path).map_err(|e| format!("创建图片目录失败: {}", e))?;
    
    let mut downloaded = 0;
    
    for moment in moments {
        // 解析昵称和内容：格式通常为 "大树一动不动：内容部分"
        let (nickname, content_part) = match moment.content.find('：') {
            Some(pos) => {
                let nick = moment.content[..pos].trim().to_string();
                let content = moment.content[pos + '：'.len_utf8()..].trim().to_string();
                (nick, content)
            }
            None => {
                // 无分隔符时，取前8个字符作为昵称替代
                let nick = moment.content.chars().take(8).collect::<String>();
                (nick, moment.content.clone())
            }
        };
        
        let safe_nickname = sanitize_filename(&nickname);
        
        for (i, img_url) in moment.images.iter().enumerate() {
            if img_url.is_empty() || !img_url.starts_with("http") {
                continue;
            }
            
            // 生成文件名：{昵称}_{内容截断}.jpg，参考原项目格式
            let safe_content = sanitize_filename(&content_part);
            let content_abbr: String = if safe_content.len() > 40 {
                safe_content.chars().take(40).collect()
            } else {
                safe_content.clone()
            };
            
            let filename = if content_abbr.is_empty() {
                format!("{}_{}.jpg", safe_nickname, i)
            } else {
                format!("{}_{}.jpg", safe_nickname, content_abbr)
            };
            let filepath = path.join(&filename);
            
            // 如果文件已存在，添加时间戳（类似原项目 _1780752732 后缀）
            let filepath = if filepath.exists() {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if content_abbr.is_empty() {
                    path.join(format!("{}_{}_{}.jpg", safe_nickname, i, timestamp))
                } else {
                    path.join(format!("{}_{}_{}.jpg", safe_nickname, content_abbr, timestamp))
                }
            } else {
                filepath
            };
            
            // 下载图片
            match client.get(img_url).send().await {
                Ok(response) => {
                    if let Ok(bytes) = response.bytes().await {
                        if fs::write(&filepath, &bytes).is_ok() {
                            downloaded += 1;
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    }
    
    Ok(downloaded)
}

/// 清理文件名中的非法字符
fn sanitize_filename(name: &str) -> String {
    let re = regex::Regex::new(r#"[\\/:*?"<>|\r\n]+"#).unwrap();
    let name = re.replace_all(name, "_");
    name.trim().to_string()
}

/// HTML 实体编码，防止 XSS
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 替换表情为 HTML 图片标签
fn replace_em_to_img(content: &str) -> String {
    let re = regex::Regex::new(r"\[em\](.*?)\[/em\]").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let emoji_code = &caps[1];
        // 只允许安全字符（字母、数字、下划线），防止 XSS 注入
        if emoji_code.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            format!(
                r#"<img src="http://qzonestyle.gtimg.cn/qzone/em/{}.gif" alt="{}" style="display:inline;vertical-align:middle;">"#,
                emoji_code, emoji_code
            )
        } else {
            format!("[em]{}[/em]", html_escape(emoji_code))
        }
    })
    .to_string()
}
