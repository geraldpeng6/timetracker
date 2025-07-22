// 窗口监控器模块
// 提供跨平台的窗口监控功能

pub mod fallback;
pub mod linux;
pub mod macos;
pub mod windows;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// 窗口几何信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// 增强的窗口信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedWindowInfo {
    /// 应用程序名称
    pub app_name: String,
    /// 窗口标题
    pub window_title: String,
    /// 进程ID
    pub process_id: u32,
    /// 应用程序路径
    pub app_path: Option<String>,
    /// Bundle ID (macOS专用)
    pub bundle_id: Option<String>,
    /// 窗口几何信息
    pub geometry: Option<WindowGeometry>,
    /// 时间戳
    pub timestamp: SystemTime,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
}

/// 权限状态
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotRequired,
    Unknown,
}

/// 增强的窗口监控器trait
pub trait EnhancedWindowMonitor {
    /// 获取当前活动窗口信息
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>>;

    /// 检查所需权限
    fn check_permissions(&self) -> Vec<(String, PermissionStatus)>;

    /// 请求必要权限
    fn request_permissions(&self) -> Result<()>;

    /// 获取监控器能力
    fn get_capabilities(&self) -> Vec<String>;

    /// 检查是否支持实时监控
    fn supports_real_time(&self) -> bool {
        true
    }

    /// 检查是否支持窗口几何信息
    fn supports_geometry(&self) -> bool {
        false
    }
}

/// 监控器类型
#[derive(Debug, Clone)]
pub enum MonitorType {
    Windows,
    MacOS,
    Linux,
    Fallback,
}

impl std::fmt::Display for MonitorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorType::Windows => write!(f, "Windows"),
            MonitorType::MacOS => write!(f, "macOS"),
            MonitorType::Linux => write!(f, "Linux"),
            MonitorType::Fallback => write!(f, "Fallback"),
        }
    }
}
