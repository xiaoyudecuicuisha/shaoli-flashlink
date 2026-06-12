//! 清理执行：回收站删除 / 永久删除

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::protect::is_protected_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanMode {
    Recycle,
    Permanent,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanResult {
    pub cleaned_count: u64,
    pub freed_bytes: u64,
    pub failed_count: u64,
    pub errors: Vec<String>,
}

/// Execute cleanup on a list of file paths
pub fn clean_items(paths: &[String], mode: &CleanMode) -> CleanResult {
    let mut result = CleanResult {
        cleaned_count: 0,
        freed_bytes: 0,
        failed_count: 0,
        errors: Vec::new(),
    };

    for path_str in paths {
        let path = Path::new(path_str);

        if !path.exists() {
            continue;
        }

        if is_protected_path(path) {
            result.errors.push(format!("跳过受保护路径: {}", path_str));
            result.failed_count += 1;
            continue;
        }

        let size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);

        let success = match mode {
            CleanMode::Recycle => move_to_recycle(path),
            CleanMode::Permanent => permanent_delete(path),
        };

        if success {
            result.cleaned_count += 1;
            result.freed_bytes += size;
        } else {
            result.failed_count += 1;
            result.errors.push(format!("删除失败: {}", path_str));
        }
    }

    result
}

fn move_to_recycle(path: &Path) -> bool {
    trash::delete(path).is_ok()
}

fn permanent_delete(path: &Path) -> bool {
    if path.is_dir() {
        std::fs::remove_dir_all(path).is_ok()
    } else {
        std::fs::remove_file(path).is_ok()
    }
}

/// Clean empty folders
pub fn clean_empty_folders(paths: &[String]) -> CleanResult {
    let mut result = CleanResult {
        cleaned_count: 0,
        freed_bytes: 0,
        failed_count: 0,
        errors: Vec::new(),
    };

    for path_str in paths {
        let path = Path::new(path_str);

        if !path.exists() || !path.is_dir() {
            continue;
        }

        if is_protected_path(path) {
            result.errors.push(format!("跳过受保护路径: {}", path_str));
            result.failed_count += 1;
            continue;
        }

        if std::fs::remove_dir(path).is_ok() {
            result.cleaned_count += 1;
        } else {
            result.failed_count += 1;
        }
    }

    result
}
