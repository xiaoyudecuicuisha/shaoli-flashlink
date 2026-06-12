//! 白板模块：Drawnix 集成、窗口管理、数据读写

pub mod commands;
pub mod storage;

use serde::{Deserialize, Serialize};

/// 白板信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardInfo {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub board_type: String,
}

/// 白板数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardData {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub data: serde_json::Value,
}
