//! 系统路径保护白名单：防止误删系统关键目录

use std::path::{Component, Path};

const PROTECTED_PREFIXES: &[&[&str]] = &[
    &["Windows", "System32"],
    &["Windows", "SysWOW64"],
    &["Windows", "WinSxS"],
    &["Windows", "Boot"],
    &["Windows", "CSC"],
    &["Program Files"],
    &["Program Files (x86)"],
    &["ProgramData"],
    &["$Recycle.Bin"],
    &["System Volume Information"],
    &["Recovery"],
    &["Boot"],
    &["Recovery"],
];

const PROTECTED_FILES: &[&str] = &[
    "pagefile.sys",
    "hiberfil.sys",
    "swapfile.sys",
    "memory.dmp",
    "dumpstack.log",
    "dumpstack.log.tmp",
    "ntldr",
    "ntdetect.com",
    "bootmgr",
    "bootsect.bak",
];

const PROTECTED_EXTS: &[&str] = &[".sys", ".drv"];

pub fn is_protected_path(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        let name_lower = name.to_string_lossy().to_ascii_lowercase();
        if PROTECTED_FILES.iter().any(|&p| p == name_lower) {
            return true;
        }
    }

    if let Some(ext) = path.extension() {
        let ext_lower = format!(".{}", ext.to_string_lossy().to_ascii_lowercase());
        if PROTECTED_EXTS.iter().any(|&e| e == ext_lower) {
            return true;
        }
    }

    let components: Vec<String> = path
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().to_ascii_lowercase()),
            _ => None,
        })
        .collect();

    for prefix in PROTECTED_PREFIXES {
        if components.len() >= prefix.len() {
            let matches = prefix
                .iter()
                .zip(components.iter())
                .all(|(expected, actual)| expected.to_ascii_lowercase() == *actual);
            if matches {
                return true;
            }
        }
    }

    false
}

pub fn is_system_drive_root(path: &Path) -> bool {
    let s = path.to_string_lossy();
    let s = s.trim_end_matches('\\');
    s.len() == 2 && s.ends_with(':') && s.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false)
}
