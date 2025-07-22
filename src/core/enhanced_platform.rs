// 增强的窗口监控平台
// 提供跨平台的高性能窗口监控功能

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// 重新导出监控器模块
pub use crate::core::monitor::{
    EnhancedWindowInfo, EnhancedWindowMonitor, MonitorType, PermissionStatus, WindowGeometry,
};

use crate::core::activity_detector::{ActivityConfig, ActivityDetector};
use crate::core::monitor::{fallback::FallbackMonitor, macos::MacOSMonitor};

#[cfg(target_os = "windows")]
use crate::core::monitor::windows::WindowsMonitor;

#[cfg(target_os = "linux")]
use crate::core::monitor::linux::LinuxMonitor;

/// 监控器状态信息
#[derive(Debug, Clone)]
pub struct MonitorStatus {
    pub monitor_type: MonitorType,
    pub is_using_fallback: bool,
    pub error_count: u32,
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
    pub cache_valid: bool,
}

/// 混合窗口监控器
/// 结合平台特定监控器和回退方案，提供最佳的窗口监控体验
pub struct HybridWindowMonitor {
    primary_monitor: Box<dyn EnhancedWindowMonitor + Send>,
    fallback_monitor: FallbackMonitor,
    monitor_type: MonitorType,
    cache: Arc<Mutex<Option<EnhancedWindowInfo>>>,
    cache_timestamp: Arc<Mutex<SystemTime>>,
    cache_duration: Duration,
    error_count: u32,
    max_errors: u32,
    use_fallback: bool,
    consecutive_failures: u32,
    last_error: Option<String>,
    activity_detector: ActivityDetector,
}

impl Default for HybridWindowMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridWindowMonitor {
    /// 创建新的混合监控器实例
    pub fn new() -> Self {
        let (primary_monitor, monitor_type) = Self::create_platform_monitor();

        Self {
            primary_monitor,
            fallback_monitor: FallbackMonitor::new(),
            monitor_type,
            cache: Arc::new(Mutex::new(None)),
            cache_timestamp: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
            cache_duration: Duration::from_millis(100),
            error_count: 0,
            max_errors: 3,
            use_fallback: false,
            consecutive_failures: 0,
            last_error: None,
            activity_detector: ActivityDetector::with_default_config(),
        }
    }

    /// 使用自定义活跃度配置创建监控器
    pub fn with_activity_config(activity_config: ActivityConfig) -> Self {
        let (primary_monitor, monitor_type) = Self::create_platform_monitor();

        Self {
            primary_monitor,
            fallback_monitor: FallbackMonitor::new(),
            monitor_type,
            cache: Arc::new(Mutex::new(None)),
            cache_timestamp: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
            cache_duration: Duration::from_millis(100),
            error_count: 0,
            max_errors: 3,
            use_fallback: false,
            consecutive_failures: 0,
            last_error: None,
            activity_detector: ActivityDetector::new(activity_config),
        }
    }

    /// 创建平台特定的监控器
    fn create_platform_monitor() -> (Box<dyn EnhancedWindowMonitor + Send>, MonitorType) {
        #[cfg(target_os = "windows")]
        {
            (Box::new(WindowsMonitor::new()), MonitorType::Windows)
        }
        #[cfg(target_os = "macos")]
        {
            (Box::new(MacOSMonitor::new()), MonitorType::MacOS)
        }
        #[cfg(target_os = "linux")]
        {
            (Box::new(LinuxMonitor::new()), MonitorType::Linux)
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            (Box::new(FallbackMonitor::new()), MonitorType::Fallback)
        }
    }

    /// 检查缓存是否有效
    fn is_cache_valid(&self) -> bool {
        if let (Ok(cache), Ok(timestamp)) = (self.cache.lock(), self.cache_timestamp.lock()) {
            cache.is_some() && timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration
        } else {
            false
        }
    }

    /// 更新缓存
    fn update_cache(&self, window_info: Option<EnhancedWindowInfo>) {
        if let (Ok(mut cache), Ok(mut timestamp)) = (self.cache.lock(), self.cache_timestamp.lock())
        {
            *cache = window_info;
            *timestamp = SystemTime::now();
        }
    }

    /// 获取监控器状态信息
    pub fn get_status(&self) -> MonitorStatus {
        MonitorStatus {
            monitor_type: self.monitor_type.clone(),
            is_using_fallback: self.use_fallback,
            error_count: self.error_count,
            consecutive_failures: self.consecutive_failures,
            last_error: self.last_error.clone(),
            cache_valid: self.is_cache_valid(),
        }
    }

    /// 重置监控器状态
    pub fn reset(&mut self) {
        self.error_count = 0;
        self.consecutive_failures = 0;
        self.last_error = None;
        self.use_fallback = false;

        // 清除缓存
        if let (Ok(mut cache), Ok(mut timestamp)) = (self.cache.lock(), self.cache_timestamp.lock())
        {
            *cache = None;
            *timestamp = SystemTime::UNIX_EPOCH;
        }
    }

    /// 强制使用回退监控器
    pub fn force_fallback(&mut self) {
        self.use_fallback = true;
        log::info!("Forced to use fallback monitor");
    }

    /// 获取活跃度检测器的引用
    pub fn activity_detector(&self) -> &ActivityDetector {
        &self.activity_detector
    }

    /// 获取活跃度检测器的可变引用
    pub fn activity_detector_mut(&mut self) -> &mut ActivityDetector {
        &mut self.activity_detector
    }

    /// 更新活跃度检测配置
    pub fn update_activity_config(&mut self, config: ActivityConfig) {
        self.activity_detector.update_config(config);
    }

    /// 强制设置为活跃状态
    pub fn force_active(&mut self) {
        self.activity_detector.force_active();
    }

    /// 获取窗口信息（不考虑活跃度检测）
    fn get_window_info_without_activity_check(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        if self.use_fallback {
            // 使用回退监控器
            match self.fallback_monitor.get_active_window() {
                Ok(window_info) => {
                    // 回退监控器成功，重置错误计数
                    self.consecutive_failures = 0;
                    self.last_error = None;
                    Ok(window_info)
                }
                Err(e) => {
                    self.consecutive_failures += 1;
                    let error_msg = format!("Fallback monitor failed: {}", e);
                    self.last_error = Some(error_msg.clone());
                    log::error!(
                        "{} (consecutive failures: {})",
                        error_msg,
                        self.consecutive_failures
                    );
                    Err(anyhow::anyhow!(error_msg))
                }
            }
        } else {
            // 尝试使用主监控器
            match self.primary_monitor.get_active_window() {
                Ok(window_info) => {
                    // 成功，重置错误计数
                    self.error_count = 0;
                    self.consecutive_failures = 0;
                    self.last_error = None;
                    Ok(window_info)
                }
                Err(e) => {
                    // 失败，增加错误计数
                    self.error_count += 1;
                    self.consecutive_failures += 1;
                    let error_msg =
                        format!("Primary monitor ({}) failed: {}", self.monitor_type, e);
                    self.last_error = Some(error_msg.clone());

                    log::warn!(
                        "{} (attempt {}/{})",
                        error_msg,
                        self.error_count,
                        self.max_errors
                    );

                    if self.error_count >= self.max_errors {
                        log::warn!(
                            "Switching to fallback monitor after {} failures",
                            self.max_errors
                        );
                        self.use_fallback = true;
                    }

                    // 尝试回退监控器
                    match self.fallback_monitor.get_active_window() {
                        Ok(window_info) => {
                            log::info!("Fallback monitor succeeded after primary failure");
                            Ok(window_info)
                        }
                        Err(fallback_err) => {
                            let combined_error = format!("Both primary and fallback monitors failed. Primary: {}, Fallback: {}", e, fallback_err);
                            self.last_error = Some(combined_error.clone());
                            log::error!("{}", combined_error);
                            Err(anyhow::anyhow!(combined_error))
                        }
                    }
                }
            }
        }
    }

    /// 获取缓存的窗口信息
    fn get_cached_window_info(&self) -> Option<EnhancedWindowInfo> {
        self.cache.lock().ok()?.clone()
    }

    /// 重置错误计数
    pub fn reset_error_count(&mut self) {
        self.error_count = 0;
        self.use_fallback = false;
    }

    /// 获取当前使用的监控器类型
    pub fn get_current_monitor_type(&self) -> MonitorType {
        if self.use_fallback {
            MonitorType::Fallback
        } else {
            self.monitor_type.clone()
        }
    }

    /// 获取监控器统计信息
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("monitor_type".to_string(), self.monitor_type.to_string());
        stats.insert("using_fallback".to_string(), self.use_fallback.to_string());
        stats.insert("error_count".to_string(), self.error_count.to_string());
        stats.insert(
            "cache_duration_ms".to_string(),
            self.cache_duration.as_millis().to_string(),
        );
        stats
    }
}

impl EnhancedWindowMonitor for HybridWindowMonitor {
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        // 检查缓存
        if self.is_cache_valid() {
            return Ok(self.get_cached_window_info());
        }

        // 首先获取窗口信息（不考虑活跃度）
        let window_info_result = self.get_window_info_without_activity_check();

        // 如果成功获取到窗口信息，检查用户活跃度
        match &window_info_result {
            Ok(Some(window_info)) => {
                // 检测用户活跃状态
                let activity_status = self.activity_detector.detect_activity(
                    Some(&window_info.app_name),
                    Some(&window_info.window_title),
                )?;

                // 如果用户不活跃且不在观看视频，返回None（不记录）
                if !activity_status.should_record() {
                    log::debug!(
                        "用户处于闲置状态 ({}), 跳过记录",
                        activity_status.description()
                    );
                    return Ok(None);
                }

                log::debug!(
                    "用户活跃状态: {}, 记录窗口活动",
                    activity_status.description()
                );
            }
            _ => {
                // 如果获取窗口信息失败，仍然更新活跃度检测器
                let _ = self.activity_detector.detect_activity(None, None);
            }
        }

        let result = window_info_result;

        // 更新缓存
        if let Ok(ref window_info) = result {
            self.update_cache(window_info.clone());
        }

        result
    }

    fn check_permissions(&self) -> Vec<(String, PermissionStatus)> {
        let mut all_permissions = vec![];

        // 获取主监控器权限
        let primary_permissions = self.primary_monitor.check_permissions();
        for (name, status) in primary_permissions {
            all_permissions.push((format!("{} ({})", name, self.monitor_type), status));
        }

        // 获取回退监控器权限
        let fallback_permissions = self.fallback_monitor.check_permissions();
        for (name, status) in fallback_permissions {
            all_permissions.push((format!("{} (Fallback)", name), status));
        }

        all_permissions
    }

    fn request_permissions(&self) -> Result<()> {
        // 请求主监控器权限
        if let Err(e) = self.primary_monitor.request_permissions() {
            log::warn!("Failed to request primary monitor permissions: {}", e);
        }

        // 请求回退监控器权限
        self.fallback_monitor.request_permissions()?;

        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        let mut capabilities = vec![];

        // 添加主监控器能力
        let primary_capabilities = self.primary_monitor.get_capabilities();
        for capability in primary_capabilities {
            capabilities.push(format!("{} ({})", capability, self.monitor_type));
        }

        // 添加回退监控器能力
        let fallback_capabilities = self.fallback_monitor.get_capabilities();
        for capability in fallback_capabilities {
            capabilities.push(format!("{} (Fallback)", capability));
        }

        // 添加混合监控器特有能力
        capabilities.push("Automatic fallback".to_string());
        capabilities.push("Error recovery".to_string());
        capabilities.push("Performance caching".to_string());

        capabilities
    }

    fn supports_real_time(&self) -> bool {
        self.primary_monitor.supports_real_time() || self.fallback_monitor.supports_real_time()
    }

    fn supports_geometry(&self) -> bool {
        if self.use_fallback {
            self.fallback_monitor.supports_geometry()
        } else {
            self.primary_monitor.supports_geometry()
        }
    }
}

/// 检查所有监控器的权限状态
pub fn check_all_permissions() -> HashMap<String, PermissionStatus> {
    let mut permissions = HashMap::new();

    // 创建各平台监控器并检查权限
    #[cfg(target_os = "windows")]
    {
        let monitor = WindowsMonitor::new();
        for (name, status) in monitor.check_permissions() {
            permissions.insert(format!("Windows.{}", name), status);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let monitor = MacOSMonitor::new();
        for (name, status) in monitor.check_permissions() {
            permissions.insert(format!("macOS.{}", name), status);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let monitor = LinuxMonitor::new();
        for (name, status) in monitor.check_permissions() {
            permissions.insert(format!("Linux.{}", name), status);
        }
    }

    // 检查回退监控器权限
    let fallback_monitor = FallbackMonitor::new();
    for (name, status) in fallback_monitor.check_permissions() {
        permissions.insert(format!("Fallback.{}", name), status);
    }

    permissions
}

/// 请求所有必要的权限
pub fn request_all_permissions() -> Result<()> {
    println!("正在检查和请求窗口监控权限...\n");

    // 使用混合监控器来请求权限
    let monitor = HybridWindowMonitor::new();

    println!("检查权限状态:");
    let permissions = monitor.check_permissions();
    for (name, status) in permissions {
        match status {
            PermissionStatus::Granted => println!("  ✓ {} - 已授予", name),
            PermissionStatus::Denied => println!("  ✗ {} - 被拒绝", name),
            PermissionStatus::NotRequired => println!("  - {} - 不需要", name),
            PermissionStatus::Unknown => println!("  ? {} - 未知状态", name),
        }
    }

    println!("\n请求权限:");
    monitor.request_permissions()?;

    Ok(())
}

/// 获取最佳可用的监控器
pub fn get_best_monitor() -> Box<dyn EnhancedWindowMonitor + Send> {
    Box::new(HybridWindowMonitor::new())
}

/// 测试所有监控器的功能
pub fn test_all_monitors() -> Result<()> {
    println!("测试所有窗口监控器...\n");

    let mut hybrid_monitor = HybridWindowMonitor::new();

    // 测试权限
    println!("权限检查:");
    let permissions = hybrid_monitor.check_permissions();
    for (name, status) in permissions {
        println!("  {}: {:?}", name, status);
    }
    println!();

    // 测试能力
    println!("监控器能力:");
    let capabilities = hybrid_monitor.get_capabilities();
    for capability in capabilities {
        println!("  - {}", capability);
    }
    println!();

    // 测试窗口检测
    println!("测试窗口检测:");
    match hybrid_monitor.get_active_window() {
        Ok(Some(window_info)) => {
            println!("  ✓ 成功检测到活动窗口:");
            println!("    应用: {}", window_info.app_name);
            println!("    窗口: {}", window_info.window_title);
            println!("    进程ID: {}", window_info.process_id);
            println!("    置信度: {:.2}", window_info.confidence);
            if let Some(geometry) = window_info.geometry {
                println!(
                    "    几何: {}x{} at ({}, {})",
                    geometry.width, geometry.height, geometry.x, geometry.y
                );
            }
            if let Some(path) = window_info.app_path {
                println!("    路径: {}", path);
            }
        }
        Ok(None) => {
            println!("  ⚠ 未检测到活动窗口");
        }
        Err(e) => {
            println!("  ✗ 窗口检测失败: {}", e);
        }
    }

    // 显示统计信息
    println!("\n监控器统计:");
    let stats = hybrid_monitor.get_stats();
    for (key, value) in stats {
        println!("  {}: {}", key, value);
    }

    Ok(())
}
