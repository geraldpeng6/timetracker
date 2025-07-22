// TimeTracker 库的主入口
// 提供模块化的公共接口

pub mod ai;
pub mod config;
pub mod core;
pub mod team; // v0.3.0 新增团队功能
pub mod ui;
pub mod utils;

// 重新导出核心类型
pub use core::{
    platform::{get_active_window, WindowInfo},
    tracker::{ActivityRecord, TimeTracker, TimeTrackerData},
};

// 重新导出 AI 相关类型
pub use ai::{
    analyzer::AIAnalyzer,
    client::{AIMessage, AIRequest, AIResponse, UnifiedAIClient},
    config::{AIConfig, AIModelConfig, AIProvider},
};

// 重新导出 UI 相关类型
pub use ui::{
    components::{SortBy, SortOrder, TabIndex, ViewMode},
    tui::TuiApp,
};

// 重新导出配置相关类型
pub use config::{app::AppConfig, manager::ConfigManager};

// 重新导出工具函数
pub use utils::{
    permissions::auto_request_permissions, time::format_duration, validation::validate_interval,
};
