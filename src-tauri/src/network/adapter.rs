//! 网络适配器信息 + VPN/代理检测

use serde::Serialize;
use sysinfo::Networks;

#[derive(Debug, Clone, Serialize)]
pub struct AdapterInfo {
    pub name: String,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,
    pub is_up: bool,
    pub is_vpn: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdapterDiagnostic {
    pub adapters: Vec<AdapterInfo>,
    pub active_adapter: String,
    pub vpn_detected: bool,
    pub proxy_enabled: bool,
    pub proxy_address: String,
}

/// VPN 关键字检测列表
const VPN_KEYWORDS: &[&str] = &[
    "vpn", "openvpn", "wireguard", "tunnel", "tap-", "tun-",
    "wintun", "nord", "express", "surfshark", "proton",
    "mullvad", "private", "zerotier", "tailscale",
    "cloudflare warp", "clash", "v2ray", "shadowsocks",
];

/// 获取网络适配器信息
pub fn get_adapter_info() -> AdapterDiagnostic {
    let mut adapters = Vec::new();
    let mut vpn_detected = false;
    let mut active_adapter = String::new();

    let networks = Networks::new_with_refreshed_list();

    for (name, network) in networks.iter() {
        let name_lower = name.to_lowercase();
        let is_vpn = VPN_KEYWORDS.iter().any(|kw| name_lower.contains(kw));

        if is_vpn {
            vpn_detected = true;
        }

        let ip_addresses: Vec<String> = network
            .ip_networks()
            .iter()
            .map(|ip| ip.addr.to_string())
            .collect();

        let is_up = !ip_addresses.is_empty();

        if is_up && active_adapter.is_empty() {
            active_adapter = name.to_string();
        }

        adapters.push(AdapterInfo {
            name: name.to_string(),
            mac_address: network.mac_address().to_string(),
            ip_addresses,
            is_up,
            is_vpn,
        });
    }

    // 注册表检测 VPN 连接（比网卡名更可靠）
    // 检查 Windows 网络配置文件中的 VPN 类型连接
    if !vpn_detected {
        vpn_detected = check_registry_vpn();
    }

    // 检测系统代理
    let (proxy_enabled, proxy_address) = check_system_proxy();

    AdapterDiagnostic {
        adapters,
        active_adapter,
        vpn_detected,
        proxy_enabled,
        proxy_address,
    }
}

/// 通过注册表检测 VPN 连接
/// 检查 HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles
/// 中是否有 VPN 类型的网络配置文件（Type == 2 表示 VPN）
fn check_registry_vpn() -> bool {
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let profiles = match hklm.open_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles",
        winreg::enums::KEY_READ,
    ) {
        Ok(k) => k,
        Err(_) => return false,
    };

    for name in profiles.enum_keys().filter_map(|k| k.ok()) {
        if let Ok(profile) = profiles.open_subkey_with_flags(&name, winreg::enums::KEY_READ) {
            // ProfileType: 1 = 有线/WiFi, 2 = VPN, 6 = WWAN
            let profile_type: u32 = profile.get_value("ProfileType").unwrap_or(0);
            if profile_type == 2 {
                return true;
            }
        }
    }

    false
}

/// 检测 Windows 系统代理设置
fn check_system_proxy() -> (bool, String) {
    let hkcu = match winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            winreg::enums::KEY_READ,
        ) {
        Ok(k) => k,
        Err(_) => return (false, String::new()),
    };

    let proxy_enable: u32 = hkcu.get_value("ProxyEnable").unwrap_or(0);
    if proxy_enable == 0 {
        return (false, String::new());
    }

    let proxy_server: String = hkcu
        .get_value("ProxyServer")
        .unwrap_or_default();

    (true, proxy_server)
}