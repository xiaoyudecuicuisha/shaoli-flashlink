//! 公网 IP + ISP + 地理信息查询（ipify/ip-api 主 + ip.sb/ipinfo.io 备）

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PublicIpInfo {
    pub public_ip: String,
    pub isp: String,
    pub country: String,
    pub city: String,
    pub success: bool,
}

/// 通过公共 API 获取公网 IP 信息
/// 优先使用国际源，失败时回退到国内友好源
pub async fn get_public_ip() -> PublicIpInfo {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .connect_timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            return PublicIpInfo {
                public_ip: "获取失败".to_string(),
                isp: "-".to_string(),
                country: "-".to_string(),
                city: "-".to_string(),
                success: false,
            }
        }
    };

    // 获取公网 IP：先 ipify，失败回退 ip.sb
    let ip = match client.get("https://api.ipify.org").send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) if !text.is_empty() => text,
            _ => get_ip_from_fallback(&client).await,
        },
        Err(_) => get_ip_from_fallback(&client).await,
    };

    if ip == "获取失败" {
        return PublicIpInfo {
            public_ip: ip,
            isp: "-".to_string(),
            country: "-".to_string(),
            city: "-".to_string(),
            success: false,
        };
    }

    // 获取地理位置：先 ip-api，失败回退 ipinfo.io
    if let Some(info) = get_geo_from_ipapi(&client, &ip).await {
        return info;
    }
    if let Some(info) = get_geo_from_ipinfo(&client, &ip).await {
        return info;
    }

    PublicIpInfo {
        public_ip: ip,
        isp: "-".to_string(),
        country: "-".to_string(),
        city: "-".to_string(),
        success: true,
    }
}

/// 备用 IP 获取（国内友好）
async fn get_ip_from_fallback(client: &reqwest::Client) -> String {
    if let Ok(resp) = client.get("https://api.ip.sb/ip").send().await {
        if let Ok(text) = resp.text().await {
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    "获取失败".to_string()
}

/// ip-api.com 地理信息
async fn get_geo_from_ipapi(client: &reqwest::Client, ip: &str) -> Option<PublicIpInfo> {
    let url = format!("https://ip-api.com/json/{}", ip);
    let resp = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let isp = json["isp"].as_str().unwrap_or("-").to_string();
    let country = json["country"].as_str().unwrap_or("-").to_string();
    let city = json["city"].as_str().unwrap_or("-").to_string();

    Some(PublicIpInfo {
        public_ip: ip.to_string(),
        isp,
        country,
        city,
        success: true,
    })
}

/// ipinfo.io 地理信息（备用）
async fn get_geo_from_ipinfo(client: &reqwest::Client, ip: &str) -> Option<PublicIpInfo> {
    let url = format!("https://ipinfo.io/{}/json", ip);
    let resp = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let org = json["org"].as_str().unwrap_or("-").to_string();
    let country = json["country"].as_str().unwrap_or("-").to_string();
    let city = json["city"].as_str().unwrap_or("-").to_string();

    Some(PublicIpInfo {
        public_ip: ip.to_string(),
        isp: org,
        country,
        city,
        success: true,
    })
}