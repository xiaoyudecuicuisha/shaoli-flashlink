//! DNS 诊断：获取系统 DNS 服务器列表 + 域名解析测试

use serde::Serialize;
use std::net::ToSocketAddrs;

#[derive(Debug, Clone, Serialize)]
pub struct DnsDiagnostic {
    pub dns_servers: Vec<String>,
    pub resolution_tests: Vec<DnsResolutionResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsResolutionResult {
    pub domain: String,
    pub resolved_ips: Vec<String>,
    pub success: bool,
    pub duration_ms: u64,
}

/// 获取 DNS 服务器列表（通过 Windows 注册表）
pub fn get_dns_servers() -> Vec<String> {
    let mut servers = Vec::new();

    let hklm = match winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(
            r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
            winreg::enums::KEY_READ,
        ) {
        Ok(k) => k,
        Err(_) => return servers,
    };

    // 全局 DNS 服务器
    if let Ok(s) = hklm.get_value::<String, _>("DhcpNameServer") {
        for ip in s.split(&[',', ' '][..]) {
            let ip = ip.trim();
            if !ip.is_empty() {
                servers.push(ip.to_string());
            }
        }
    }
    if let Ok(s) = hklm.get_value::<String, _>("NameServer") {
        for ip in s.split(&[',', ' '][..]) {
            let ip = ip.trim();
            if !ip.is_empty() && !servers.contains(&ip.to_string()) {
                servers.push(ip.to_string());
            }
        }
    }

    // 从接口子键读取 DNS
    if let Ok(interfaces) =
        hklm.open_subkey_with_flags(r"Interfaces", winreg::enums::KEY_READ)
    {
        for name in interfaces.enum_keys().filter_map(|k| k.ok()) {
            if let Ok(iface) = interfaces.open_subkey_with_flags(&name, winreg::enums::KEY_READ) {
                if let Ok(s) = iface.get_value::<String, _>("DhcpNameServer") {
                    for ip in s.split(&[',', ' '][..]) {
                        let ip = ip.trim();
                        if !ip.is_empty() && !servers.contains(&ip.to_string()) {
                            servers.push(ip.to_string());
                        }
                    }
                }
                if let Ok(s) = iface.get_value::<String, _>("NameServer") {
                    for ip in s.split(&[',', ' '][..]) {
                        let ip = ip.trim();
                        if !ip.is_empty() && !servers.contains(&ip.to_string()) {
                            servers.push(ip.to_string());
                        }
                    }
                }
            }
        }
    }

    if servers.is_empty() {
        servers.push("未获取到 DNS 服务器".to_string());
    }

    servers.sort();
    servers.dedup();
    servers
}

/// 测试关键域名解析
pub fn test_dns_resolution() -> DnsDiagnostic {
    let servers = get_dns_servers();

    let domains = vec![
        "www.baidu.com",
        "www.zsit.edu.cn",
        "dns.qq.com",
    ];

    let mut results = Vec::new();
    for domain in &domains {
        let start = std::time::Instant::now();
        let resolved = format!("{}:80", domain).to_socket_addrs();
        let duration_ms = start.elapsed().as_millis() as u64;

        match resolved {
            Ok(addrs) => {
                let ips: Vec<String> = addrs
                    .take(4)
                    .filter(|a| a.is_ipv4())
                    .map(|a| a.ip().to_string())
                    .collect();
                results.push(DnsResolutionResult {
                    domain: domain.to_string(),
                    resolved_ips: if ips.is_empty() {
                        vec!["无法解析".to_string()]
                    } else {
                        ips
                    },
                    success: true,
                    duration_ms,
                });
            }
            Err(_) => {
                results.push(DnsResolutionResult {
                    domain: domain.to_string(),
                    resolved_ips: vec!["解析失败".to_string()],
                    success: false,
                    duration_ms,
                });
            }
        }
    }

    DnsDiagnostic {
        dns_servers: servers,
        resolution_tests: results,
    }
}