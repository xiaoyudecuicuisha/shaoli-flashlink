//! 统一错误类型定义，所有 Tauri 命令返回 Result<T, AppError>

use serde::Serialize;
use std::fmt;

/// 应用统一错误类型
#[derive(Debug)]
pub enum AppError {
    /// 网络错误
    Network(String),
    /// IO 错误
    Io(String),
    /// 配置错误
    Config(String),
    /// 认证错误
    Auth(String),
    /// 其他错误
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Network(msg) => write!(f, "网络错误: {}", msg),
            AppError::Io(msg) => write!(f, "IO 错误: {}", msg),
            AppError::Config(msg) => write!(f, "配置错误: {}", msg),
            AppError::Auth(msg) => write!(f, "认证错误: {}", msg),
            AppError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// 从 reqwest::Error 转换
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

/// 从 std::io::Error 转换
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

/// 从 serde_json::Error 转换
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Config(err.to_string())
    }
}

/// 从 String 转换
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Other(err)
    }
}

/// 从 &str 转换
impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Other(err.to_string())
    }
}
