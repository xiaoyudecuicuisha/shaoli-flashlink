//! 网络延迟测试：TCP 连接到多个目标并计时

use serde::Serialize;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize)]
pub struct LatencyResult {
    pub host: String,
    pub host_desc: String,
    pub latency_ms: Option<u64>,
    pub reachable: bool,
}

/// TCP Connect 延迟测试到多个目标
/// 每目标 3 秒超时
pub fn test_latency() -> Vec<LatencyResult> {
    let targets = vec![
        ("110.242.68.66:80", "百度"),
        ("223.5.5.5:53", "阿里 DNS"),
        ("119.29.29.29:53", "腾讯 DNS"),
        ("114.114.114.114:53", "114 DNS"),
    ];

    let mut results = Vec::new();

    for (addr, desc) in &targets {
        let start = Instant::now();
        let reachable = match addr.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(sock_addr) = addrs.next() {
                    TcpStream::connect_timeout(&sock_addr, Duration::from_secs(3)).is_ok()
                } else {
                    false
                }
            }
            Err(_) => false,
        };
        let elapsed = start.elapsed().as_millis() as u64;

        results.push(LatencyResult {
            host: addr.to_string(),
            host_desc: desc.to_string(),
            latency_ms: if reachable { Some(elapsed) } else { None },
            reachable,
        });
    }

    results
}