//! 网课助手模块：平台适配、题库管理、WebView 窗口

pub mod platform;
pub mod qbank;
pub mod window;

use std::sync::{Arc, Mutex};

use serde::Serialize;

use self::qbank::QBankEntry;

/// 网课助手托管状态
pub struct CourseState {
    #[allow(dead_code)]
    pub running: Arc<Mutex<bool>>,
    pub progress: Arc<Mutex<CourseProgress>>,
    pub qbank: Arc<Mutex<Vec<QBankEntry>>>,
}

impl Default for CourseState {
    fn default() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
            progress: Arc::new(Mutex::new(CourseProgress::default())),
            qbank: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// 刷课进度
#[derive(Default, Clone, Serialize)]
pub struct CourseProgress {
    pub videos_completed: u32,
    pub quizzes_answered: u32,
    pub quizzes_missed: u32,
    pub current_chapter: String,
    pub status: String,
}
