//! QQ 空间历史动态获取：QR 登录、API 调用、解析、导出

pub mod auth;
pub mod api;
pub mod parser;
pub mod models;
pub mod export;

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use self::models::{Cookies, FetchProgress, Moment, QzoneUserInfo};

/// 全局 Qzone 状态
pub struct QzoneState {
    pub client: reqwest::Client,
    pub no_redirect_client: reqwest::Client,
    pub cookies: Arc<Mutex<Option<Cookies>>>,
    pub cookie_map: Arc<Mutex<HashMap<String, String>>>,
    pub user_info: Arc<Mutex<Option<QzoneUserInfo>>>,
    pub moments: Arc<Mutex<Vec<Moment>>>,
    pub progress: Arc<Mutex<FetchProgress>>,
    pub cancel_token: Arc<AtomicBool>,
}

impl QzoneState {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .build()
            .unwrap_or_default();

        let no_redirect_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_default();

        Self {
            client,
            no_redirect_client,
            cookies: Arc::new(Mutex::new(None)),
            cookie_map: Arc::new(Mutex::new(HashMap::new())),
            user_info: Arc::new(Mutex::new(None)),
            moments: Arc::new(Mutex::new(Vec::new())),
            progress: Arc::new(Mutex::new(FetchProgress {
                total: 0,
                current: 0,
                moments_count: 0,
                friends_count: 0,
                status: "idle".to_string(),
                is_running: false,
            })),
            cancel_token: Arc::new(AtomicBool::new(false)),
        }
    }
}
