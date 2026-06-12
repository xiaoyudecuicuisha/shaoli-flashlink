//! Hosts 文件修复：校园网站域名解析、备份/恢复

use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::school;

const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";
const HOSTS_BACKUP: &str = r"C:\Windows\System32\drivers\etc\hosts.backup-sxlgl";

#[derive(Serialize)]
pub struct HostsStatus {
    pub fixed_count: usize,
    pub total: usize,
    pub sites: Vec<SiteInfo>,
}

#[derive(Serialize)]
pub struct SiteInfo {
    pub domain: String,
    pub name: String,
    pub is_fixed: bool,
}

/// 获取完整域名
fn full_domain(sub: &str, domain: &str) -> String {
    if sub.contains('.') {
        sub.to_string()
    } else {
        format!("{}.{}", sub, domain)
    }
}

/// 读取 hosts 文件中已有的条目
fn get_existing_hosts_entries(domain: &str) -> HashSet<String> {
    let mut entries = HashSet::new();
    if let Ok(content) = fs::read_to_string(HOSTS_PATH) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1].contains(domain) {
                entries.insert(parts[1].to_string());
            }
        }
    }
    entries
}

/// 检查 hosts 修复状态
pub fn check_hosts_status() -> HostsStatus {
    let profile = school::load_school_profile();
    let existing = get_existing_hosts_entries(&profile.domain);
    let mut sites = Vec::new();
    let mut fixed_count = 0;

    for site in &profile.sites {
        let domain = full_domain(&site.subdomain, &profile.domain);
        let is_fixed = existing.contains(&domain);
        if is_fixed {
            fixed_count += 1;
        }
        sites.push(SiteInfo {
            domain,
            name: site.name.clone(),
            is_fixed,
        });
    }

    HostsStatus {
        fixed_count,
        total: profile.sites.len(),
        sites,
    }
}

/// 一键修复 hosts
pub fn fix_hosts() -> Result<String, String> {
    let profile = school::load_school_profile();
    
    if !Path::new(HOSTS_BACKUP).exists() {
        fs::copy(HOSTS_PATH, HOSTS_BACKUP)
            .map_err(|e| format!("备份 hosts 失败: {}", e))?;
    }

    let existing = get_existing_hosts_entries(&profile.domain);
    let mut new_lines = vec![format!("\r\n# {} internal network (managed by 绍理闪连)", profile.code.to_uppercase())];
    let mut added = 0;

    for site in &profile.sites {
        let domain = full_domain(&site.subdomain, &profile.domain);
        if !existing.contains(&domain) {
            new_lines.push(format!("{} {}", profile.internal_ip, domain));
            added += 1;
        }
    }

    if added == 0 {
        return Ok("所有站点已配置".to_string());
    }

    let mut content = fs::read_to_string(HOSTS_PATH)
        .map_err(|e| format!("读取 hosts 失败: {}", e))?;
    content.push_str(&new_lines.join("\r\n"));
    content.push_str("\r\n");

    fs::write(HOSTS_PATH, &content).map_err(|e| format!("写入 hosts 失败: {}", e))?;

    let _ = Command::new("ipconfig")
        .args(["/flushdns"])
        .output();

    Ok(format!("修复成功，新增 {} 个站点", added))
}

/// 恢复 hosts 默认
pub fn restore_hosts() -> Result<String, String> {
    let profile = school::load_school_profile();
    
    if Path::new(HOSTS_BACKUP).exists() {
        fs::copy(HOSTS_BACKUP, HOSTS_PATH)
            .map_err(|e| format!("恢复 hosts 失败: {}", e))?;
    } else {
        let content =
            fs::read_to_string(HOSTS_PATH).map_err(|e| format!("读取 hosts 失败: {}", e))?;
        let new_content: String = content
            .lines()
            .filter(|line| !line.contains(&profile.domain) && !line.contains(&profile.code.to_uppercase()))
            .collect::<Vec<_>>()
            .join("\r\n");
        fs::write(HOSTS_PATH, new_content).map_err(|e| format!("写入 hosts 失败: {}", e))?;
    }

    let _ = Command::new("ipconfig")
        .args(["/flushdns"])
        .output();

    Ok("已恢复默认".to_string())
}
