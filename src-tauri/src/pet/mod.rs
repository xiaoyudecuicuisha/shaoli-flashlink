//! 桌面宠物模块：配置、窗口生命周期、文件 I/O、光标位置

pub mod commands;
pub mod storage;

use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

// ── 自定义宠物条目 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPet {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub file_type: String,
}

// ── 心情消息 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodMessages {
    #[serde(default = "default_happy_messages")]
    pub happy: Vec<String>,
    #[serde(default = "default_sad_messages")]
    pub sad: Vec<String>,
    #[serde(default = "default_busy_messages")]
    pub busy: Vec<String>,
    #[serde(default = "default_sleep_messages")]
    pub sleep: Vec<String>,
}

impl Default for MoodMessages {
    fn default() -> Self {
        Self {
            happy: default_happy_messages(),
            sad: default_sad_messages(),
            busy: default_busy_messages(),
            sleep: default_sleep_messages(),
        }
    }
}

fn default_happy_messages() -> Vec<String> {
    vec!["开心~".into(), "✨".into(), "❤️".into(), "嘿嘿".into()]
}
fn default_sad_messages() -> Vec<String> {
    vec!["呜呜...".into(), "(难过)".into(), "网络断开了...".into()]
}
fn default_busy_messages() -> Vec<String> {
    vec!["忙碌中...".into(), "稍等~".into()]
}
fn default_sleep_messages() -> Vec<String> {
    vec!["困了...".into(), "💤".into(), "zzZ".into()]
}

// ── 气泡样式 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleStyle {
    #[serde(default = "default_bubble_duration")]
    pub duration: u32,
    #[serde(default = "default_bubble_font_size")]
    pub font_size: u32,
    #[serde(default = "default_bubble_bg_color")]
    pub bg_color: String,
    #[serde(default = "default_bubble_text_color")]
    pub text_color: String,
}

impl Default for BubbleStyle {
    fn default() -> Self {
        Self {
            duration: default_bubble_duration(),
            font_size: default_bubble_font_size(),
            bg_color: default_bubble_bg_color(),
            text_color: default_bubble_text_color(),
        }
    }
}

fn default_bubble_duration() -> u32 {
    3000
}
fn default_bubble_font_size() -> u32 {
    12
}
fn default_bubble_bg_color() -> String {
    "#1e1e1e".into()
}
fn default_bubble_text_color() -> String {
    "#ffffff".into()
}

// ── 右键菜单项 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub action: String, // "interact" | "shake" | "settings" | "close"
    #[serde(default)]
    pub icon: String,
}

impl Default for MenuItem {
    fn default() -> Self {
        default_menu_items()
    }
}

fn default_menu_items() -> MenuItem {
    MenuItem {
        id: "interact".into(),
        label: "摸摸我".into(),
        action: "interact".into(),
        icon: String::new(),
    }
}

fn default_menu_items_vec() -> Vec<MenuItem> {
    vec![
        MenuItem { id: "interact".into(), label: "摸摸我".into(), action: "interact".into(), icon: String::new() },
        MenuItem { id: "shake".into(), label: "摇一摇".into(), action: "shake".into(), icon: String::new() },
        MenuItem { id: "settings".into(), label: "打开设置".into(), action: "settings".into(), icon: String::new() },
        MenuItem { id: "close".into(), label: "关闭宠物".into(), action: "close".into(), icon: String::new() },
    ]
}

fn default_click_animation() -> String {
    "random".into()
}

// ── 宠物配置（持久化） ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_pet")]
    pub current_pet: String,
    #[serde(default = "default_size")]
    pub size: u32,
    #[serde(default = "default_pet_name")]
    pub default_pet_name: String,
    #[serde(default)]
    pub custom_pets: Vec<CustomPet>,
    #[serde(default)]
    pub reaction_network: bool,
    #[serde(default = "default_true")]
    pub reaction_course: bool,
    #[serde(default = "default_true")]
    pub reaction_qzone: bool,
    #[serde(default = "default_true")]
    pub reaction_convert: bool,
    // ── 互动自定义 ──
    #[serde(default = "default_click_messages")]
    pub click_messages: Vec<String>,
    #[serde(default = "default_shake_message")]
    pub shake_message: String,
    #[serde(default)]
    pub mood_messages: MoodMessages,
    #[serde(default = "default_click_animation")]
    pub click_animation: String,
    #[serde(default)]
    pub bubble_style: BubbleStyle,
    #[serde(default = "default_menu_items_vec")]
    pub menu_items: Vec<MenuItem>,
}

fn default_pet() -> String {
    "default".to_string()
}
fn default_size() -> u32 {
    180
}
fn default_pet_name() -> String {
    "默认小伙伴".to_string()
}
fn default_true() -> bool {
    true
}
fn default_click_messages() -> Vec<String> {
    vec![
        "喵~".into(),
        "汪!".into(),
        "嘿嘿".into(),
        "(害羞)".into(),
        "❤️".into(),
        "✨".into(),
        "你好呀~".into(),
        "(开心)".into(),
    ]
}
fn default_shake_message() -> String {
    "别摇啦~".into()
}

impl Default for PetConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            current_pet: "default".to_string(),
            size: 180,
            default_pet_name: "默认小伙伴".to_string(),
            custom_pets: Vec::new(),
            reaction_network: true,
            reaction_course: true,
            reaction_qzone: true,
            reaction_convert: true,
            click_messages: default_click_messages(),
            shake_message: default_shake_message(),
            mood_messages: MoodMessages::default(),
            click_animation: default_click_animation(),
            bubble_style: BubbleStyle::default(),
            menu_items: default_menu_items_vec(),
        }
    }
}

// ── 宠物信息（列表用） ──

#[derive(Debug, Clone, Serialize)]
pub struct PetInfo {
    pub id: String,
    pub name: String,
    pub source: String, // "builtin" | "custom"
}

// ── Tauri Managed State ──

pub struct PetState {
    pub config: Arc<Mutex<PetConfig>>,
    pub window_open: Arc<AtomicBool>,
}

impl PetState {
    pub fn new() -> Self {
        let config = storage::load_config().unwrap_or_default();
        Self {
            config: Arc::new(Mutex::new(config)),
            window_open: Arc::new(AtomicBool::new(false)),
        }
    }
}
