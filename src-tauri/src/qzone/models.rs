//! QQ 空间数据模型：说说、评论、Cookie、登录状态、进度

use serde::{Deserialize, Serialize};

/// 说说结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Moment {
    pub time: String,
    pub content: String,
    pub images: Vec<String>,
    pub comments: Vec<Comment>,
}

/// 评论结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub time: String,
    pub content: String,
    pub nickname: String,
    pub uin: String,
}

/// Cookie 存储
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookies {
    pub p_skey: String,
    pub uin: String,
    pub skey: String,
    pub p_uin: String,
    pub pt4_token: String,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QzoneUserInfo {
    pub uin: String,
    pub nickname: String,
    pub avatar_url: String,
}

/// 登录状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LoginStatus {
    #[serde(rename = "waiting")]
    WaitingScan,
    #[serde(rename = "scanned")]
    Scanned,
    #[serde(rename = "success")]
    Success { cookies: Cookies },
    #[serde(rename = "failed")]
    Failed { message: String },
}

/// 获取进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchProgress {
    pub total: usize,
    pub current: usize,
    pub moments_count: usize,
    pub friends_count: usize,
    pub status: String,
    pub is_running: bool,
}

/// 获取选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOptions {
    pub fetch_all: bool,
    pub start_year: Option<i32>,
    pub start_month: Option<u32>,
    pub end_year: Option<i32>,
    pub end_month: Option<u32>,
    pub include_moments: bool,
    pub include_comments: bool,
    pub include_forwards: bool,
}
