//! 网课平台适配器：统一接口，支持多平台扩展

/// 平台适配器 trait
///
/// 每个网课平台实现此 trait，window.rs 通过 `&dyn PlatformAdapter` 操作，
/// 新增平台只需添加一个 struct + impl，不改核心代码。
pub trait PlatformAdapter: Send + Sync {
    /// 平台标识（用于窗口 label 前缀）
    fn name(&self) -> &str;

    /// 平台入口 URL
    fn entry_url(&self) -> &str;

    /// 注入脚本内容（include_str!）
    fn script(&self) -> &str;

    /// 窗口 label
    fn window_label(&self) -> String {
        format!("course-{}", self.name())
    }
}

/// 超星学习通平台
pub struct ChaoxingPlatform;

impl PlatformAdapter for ChaoxingPlatform {
    fn name(&self) -> &str {
        "chaoxing"
    }

    fn entry_url(&self) -> &str {
        "https://i.chaoxing.com"
    }

    fn script(&self) -> &str {
        include_str!("../../scripts/chaoxing.js")
    }
}
