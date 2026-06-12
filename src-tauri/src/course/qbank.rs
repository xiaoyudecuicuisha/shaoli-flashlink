//! 题库管理：多格式解析(TXT/JSON/XLSX/DOCX)、存储、模糊匹配

use std::fs;
use std::io::Read;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// 题库条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QBankEntry {
    pub question: String,
    pub answer: String,
}

/// 题库信息
#[derive(Debug, Clone, Serialize)]
pub struct QBankInfo {
    pub count: usize,
    pub source: String,
}

// ── 持久化路径 ──

fn bank_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".shaoli")
        .join("qbank.json")
}

/// 读取本地题库
pub fn get_bank() -> Result<Vec<QBankEntry>, String> {
    let path = bank_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("读取题库失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("题库文件解析失败: {}", e))
}

/// 保存题库（追加去重）
pub fn save_bank(new_entries: &[QBankEntry]) -> Result<usize, String> {
    let mut existing = get_bank().unwrap_or_default();

    let mut seen: std::collections::HashSet<String> = existing
        .iter()
        .map(|e| e.question.trim().to_lowercase())
        .collect();

    let mut _added = 0;
    for entry in new_entries {
        let key = entry.question.trim().to_lowercase();
        if !key.is_empty() && seen.insert(key) {
            existing.push(entry.clone());
            _added += 1;
        }
    }

    let path = bank_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let content =
        serde_json::to_string_pretty(&existing).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("写入题库失败: {}", e))?;

    Ok(existing.len())
}

/// 删除题库
pub fn delete_bank() -> Result<(), String> {
    let path = bank_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("删除题库失败: {}", e))?;
    }
    Ok(())
}

/// 获取题库信息
pub fn get_bank_info() -> Result<QBankInfo, String> {
    let bank = get_bank()?;
    let source = bank_path()
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "qbank.json".to_string());
    Ok(QBankInfo {
        count: bank.len(),
        source,
    })
}

// ── 文件导入 ──

/// 根据文件后缀自动选择解析器
pub fn import_file(path_str: &str) -> Result<Vec<QBankEntry>, String> {
    let path = Path::new(path_str);
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => match ext.to_lowercase().as_str() {
            "txt" => parse_txt(path),
            "json" => parse_json(path),
            "xlsx" | "xls" => parse_xlsx(path),
            "docx" => parse_docx(path),
            _ => Err(format!(
                "不支持的文件格式: .{}，支持: .txt .json .xlsx .docx",
                ext
            )),
        },
        None => Err("无法识别文件格式".into()),
    }
}

/// TXT 解析：支持 "题目#答案" 或 "题目\t答案" 格式
fn parse_txt(path: &Path) -> Result<Vec<QBankEntry>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let re_hash = Regex::new(r"^(.+?)#(.+)$").unwrap();
    let re_tab = Regex::new(r"^(.+?)\t(.+)$").unwrap();

    let mut entries = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(caps) = re_hash.captures(line) {
            entries.push(QBankEntry {
                question: caps[1].trim().to_string(),
                answer: caps[2].trim().to_string(),
            });
        } else if let Some(caps) = re_tab.captures(line) {
            entries.push(QBankEntry {
                question: caps[1].trim().to_string(),
                answer: caps[2].trim().to_string(),
            });
        }
    }

    Ok(entries)
}

/// JSON 解析：直接反序列化
fn parse_json(path: &Path) -> Result<Vec<QBankEntry>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("JSON 解析失败: {}", e))
}

/// XLSX/XLS 解析：A列题目，B列答案（第一行为表头时跳过）
fn parse_xlsx(path: &Path) -> Result<Vec<QBankEntry>, String> {
    let mut workbook =
        open_workbook_auto(path).map_err(|e| format!("打开 Excel 文件失败: {}", e))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "Excel 文件无工作表".to_string())?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("读取工作表失败: {}", e))?;

    let mut entries = Vec::new();
    let mut first_row = true;

    for row in range.rows() {
        if first_row {
            first_row = false;
            if let Some(cell) = row.first() {
                let text = match cell {
                    Data::String(s) => s.to_lowercase(),
                    _ => String::new(),
                };
                if text.contains("题") || text.contains("question") || text.contains("问题") {
                    continue; // 跳过表头
                }
            }
        }

        if row.len() < 2 {
            continue;
        }

        let question = match &row[0] {
            Data::String(s) => s.trim().to_string(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            _ => continue,
        };

        let answer = match &row[1] {
            Data::String(s) => s.trim().to_string(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            _ => continue,
        };

        if !question.is_empty() && !answer.is_empty() {
            entries.push(QBankEntry { question, answer });
        }
    }

    Ok(entries)
}

/// DOCX 解析：zip + quick-xml 提取段落文本
/// 假设题库格式为每两段一组：第一段题目，第二段答案
fn parse_docx(path: &Path) -> Result<Vec<QBankEntry>, String> {
    let file =
        fs::File::open(path).map_err(|e| format!("打开 DOCX 文件失败: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("DOCX 解压失败: {}", e))?;

    let mut doc_xml = String::new();
    if let Ok(mut entry) = archive.by_name("word/document.xml") {
        entry
            .read_to_string(&mut doc_xml)
            .map_err(|e| format!("读取 document.xml 失败: {}", e))?;
    } else {
        return Err("DOCX 文件中未找到 word/document.xml".into());
    }

    let paragraphs = extract_paragraphs(&doc_xml)?;

    let mut entries = Vec::new();
    let mut i = 0;
    while i + 1 < paragraphs.len() {
        let question = paragraphs[i].trim().to_string();
        let answer = paragraphs[i + 1].trim().to_string();
        if !question.is_empty() && !answer.is_empty() {
            entries.push(QBankEntry { question, answer });
        }
        i += 2;
    }

    Ok(entries)
}

/// 从 DOCX XML 中提取所有段落文本
fn extract_paragraphs(xml: &str) -> Result<Vec<String>, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    let mut paragraphs = Vec::new();
    let mut current_text = String::new();
    let mut in_paragraph = false;
    let mut in_run = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"w:p" {
                    in_paragraph = true;
                    current_text.clear();
                } else if name.as_ref() == b"w:r" {
                    in_run = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_paragraph && in_run {
                    if let Ok(text) = e.unescape() {
                        current_text.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"w:p" {
                    if in_paragraph {
                        paragraphs.push(current_text.clone());
                    }
                    in_paragraph = false;
                } else if name.as_ref() == b"w:r" {
                    in_run = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML 解析错误: {}", e)),
            _ => {}
        }
    }

    Ok(paragraphs)
}

// ── 模糊匹配 ──

const SIMILARITY_THRESHOLD: f64 = 0.6;

/// 预处理字符串：去除非 CJK/字母/数字字符，转小写
fn clean_string(s: &str) -> String {
    s.chars()
        .filter(|c| {
            ('\u{2E80}'..='\u{9FFF}').contains(c) // CJK
                || c.is_ascii_alphanumeric()
        })
        .collect::<String>()
        .to_lowercase()
}

/// 在题库中模糊匹配题目，返回最佳答案和相似度分数
pub fn match_question(question: &str, bank: &[QBankEntry]) -> Option<(String, f64)> {
    let cleaned_q = clean_string(question);
    if cleaned_q.is_empty() {
        return None;
    }

    bank.iter()
        .filter_map(|entry| {
            let cleaned_entry = clean_string(&entry.question);
            if cleaned_entry.is_empty() {
                return None;
            }
            let score = strsim::sorensen_dice(&cleaned_q, &cleaned_entry);
            if score > SIMILARITY_THRESHOLD {
                Some((entry.answer.clone(), score))
            } else {
                None
            }
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}
