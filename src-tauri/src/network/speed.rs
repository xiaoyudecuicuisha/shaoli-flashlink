//! 网络速度测试：通过 CDN 下载计算 Mbps

use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct SpeedResult {
    pub download_speed_mbps: f64,
    pub download_size_bytes: u64,
    pub duration_ms: u64,
    pub success: bool,
}

/// HTTP 下载测速
/// 优先使用国内可达的 CDN 源，任一成功即返回
pub async fn test_speed() -> SpeedResult {
    // 按优先级排列：Cloudflare Anycast（全球可达）→ 腾讯 CDN（国内极快）
    let urls = vec![
        // Cloudflare 10MB — 全球有 PoPs，国内也可达
        "https://speed.cloudflare.com/__down?during=download&bytes=10485760",
        // 腾讯 CDN 微信安装包前 10MB — 国内 CDN 节点多
        "https://dldir1.qq.com/weixin/Windows/WeChatSetup.exe",
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    for url in &urls {
        let start = Instant::now();
        // 腾讯 CDN 需要 Range 头来只下载前 10MB
        let mut req = client.get(*url);
        if url.contains("qq.com") {
            req = req.header("Range", "bytes=0-10485759");
        }

        match req.send().await {
            Ok(resp) => {
                if let Ok(bytes) = resp.bytes().await {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let size = bytes.len() as u64;

                    if size > 0 && duration_ms > 0 {
                        // 计算 Mbps: bytes * 8 / (ms / 1000) / 1_000_000
                        let mbps = (size as f64 * 8.0) / (duration_ms as f64 / 1000.0)
                            / 1_000_000.0;

                        return SpeedResult {
                            download_speed_mbps: (mbps * 100.0).round() / 100.0,
                            download_size_bytes: size,
                            duration_ms,
                            success: true,
                        };
                    }
                }
            }
            Err(_) => continue,
        }
    }

    SpeedResult {
        download_speed_mbps: 0.0,
        download_size_bytes: 0,
        duration_ms: 0,
        success: false,
    }
}