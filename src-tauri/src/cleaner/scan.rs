//! 文件扫描：规则匹配、大小估算、空文件夹检测

use serde::Serialize;
use std::path::Path;
use walkdir::WalkDir;

use super::protect::is_protected_path;
use super::rules::CleanRule;

#[derive(Debug, Clone, Serialize)]
pub struct ScanItem {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub category: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub items: Vec<ScanItem>,
    pub total_size: u64,
    pub total_count: u64,
}

/// Scan files matching a rule
pub fn scan_rule(rule: &CleanRule) -> Vec<ScanItem> {
    let path = Path::new(&rule.path);

    if !path.exists() {
        return Vec::new();
    }

    if is_protected_path(path) {
        return Vec::new();
    }

    match rule.rule_type.as_str() {
        "file" => scan_single_file(path, &rule.category),
        "dir" => scan_directory(path, &rule.category, false),
        _ => scan_directory(path, &rule.category, true),
    }
}

fn scan_single_file(path: &Path, category: &str) -> Vec<ScanItem> {
    if !path.exists() || is_protected_path(path) {
        return Vec::new();
    }

    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    vec![ScanItem {
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        path: path.to_string_lossy().to_string(),
        size,
        category: category.to_string(),
        is_dir: false,
    }]
}

fn scan_directory(path: &Path, category: &str, recursive: bool) -> Vec<ScanItem> {
    let mut items = Vec::new();

    if recursive {
        for entry in WalkDir::new(path)
            .max_depth(100)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if is_protected_path(entry_path) {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.is_file() {
                items.push(ScanItem {
                    name: entry_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path: entry_path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    category: category.to_string(),
                    is_dir: false,
                });
            }
        }
    } else {
        // Non-recursive: only immediate children
        let entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return items,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if is_protected_path(&entry_path) {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.is_file() {
                items.push(ScanItem {
                    name: entry_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path: entry_path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    category: category.to_string(),
                    is_dir: false,
                });
            }
        }
    }

    items
}

/// Scan multiple rules and return combined result
pub fn scan_rules(rules: &[CleanRule]) -> ScanResult {
    let mut all_items = Vec::new();

    for rule in rules {
        let items = scan_rule(rule);
        all_items.extend(items);
    }

    let total_size = all_items.iter().map(|i| i.size).sum();
    let total_count = all_items.len() as u64;

    ScanResult {
        items: all_items,
        total_size,
        total_count,
    }
}

/// Estimate sizes for rule categories (lightweight scan, only sums sizes)
pub fn estimate_rules(rules: &[CleanRule]) -> Vec<super::rules::EstimateItem> {
    let mut estimates = Vec::new();

    for rule in rules {
        let path = Path::new(&rule.path);
        if !path.exists() || is_protected_path(path) {
            continue;
        }

        let (size, count) = match rule.rule_type.as_str() {
            "file" => {
                let s = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                (s, if s > 0 { 1 } else { 0 })
            }
            "dir" | _ => estimate_directory(path),
        };

        if size > 0 {
            estimates.push(super::rules::EstimateItem {
                name: rule.name.clone(),
                category: rule.category.clone(),
                estimated_bytes: size,
                file_count: count,
            });
        }
    }

    estimates
}

fn estimate_directory(path: &Path) -> (u64, u64) {
    let mut total_size = 0u64;
    let mut total_count = 0u64;

    for entry in WalkDir::new(path)
        .max_depth(100)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        if is_protected_path(entry_path) {
            continue;
        }

        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                total_size += metadata.len();
                total_count += 1;
            }
        }
    }

    (total_size, total_count)
}

/// Scan for empty folders
pub fn scan_empty_folders(root: &Path) -> Vec<ScanItem> {
    let mut items = Vec::new();

    if !root.exists() || !root.is_dir() {
        return items;
    }

    for entry in WalkDir::new(root)
        .max_depth(50)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if is_protected_path(path) {
            continue;
        }

        if path.is_dir() {
            if let Ok(mut entries) = std::fs::read_dir(path) {
                if entries.next().is_none() {
                    items.push(ScanItem {
                        name: path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        path: path.to_string_lossy().to_string(),
                        size: 0,
                        category: "空文件夹".to_string(),
                        is_dir: true,
                    });
                }
            }
        }
    }

    items
}
