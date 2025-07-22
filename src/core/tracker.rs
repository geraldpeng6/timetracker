use crate::core::enhanced_platform::get_best_monitor;
use crate::core::monitor::{EnhancedWindowInfo, EnhancedWindowMonitor, PermissionStatus};
use crate::core::platform::{get_active_window, WindowInfo};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct ActivityRecord {
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    #[serde(rename = "duration_seconds")]
    pub duration: u64, // 持续时间（秒）
    pub process_id: u32,
    // 新增字段，支持增强的窗口信息
    #[serde(default)]
    pub app_path: Option<String>,
    #[serde(default)]
    pub bundle_id: Option<String>,
    #[serde(default)]
    pub window_geometry: Option<WindowGeometry>,
    #[serde(default)]
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ActivityRecord {
    pub fn new(window_info: WindowInfo) -> Self {
        Self {
            app_name: window_info.app_name,
            window_title: window_info.window_title,
            start_time: Utc::now(),
            end_time: None,
            duration: 0,
            process_id: window_info.process_id,
            app_path: None,
            bundle_id: None,
            window_geometry: None,
            confidence: 0.5, // 旧系统的默认置信度
        }
    }

    pub fn new_enhanced(window_info: EnhancedWindowInfo) -> Self {
        Self {
            app_name: window_info.app_name,
            window_title: window_info.window_title,
            start_time: Utc::now(),
            end_time: None,
            duration: 0,
            process_id: window_info.process_id,
            app_path: window_info.app_path,
            bundle_id: window_info.bundle_id,
            window_geometry: window_info.geometry.map(|g| WindowGeometry {
                x: g.x,
                y: g.y,
                width: g.width,
                height: g.height,
            }),
            confidence: window_info.confidence as f32,
        }
    }

    pub fn finish(&mut self) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.duration = (now - self.start_time).num_seconds() as u64;
    }
}

// 新的数据文件格式，支持更好的数据管理
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeTrackerData {
    pub activities: Vec<ActivityRecord>,
    pub current_activity: Option<ActivityRecord>,
    pub last_updated: DateTime<Utc>,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl Default for TimeTrackerData {
    fn default() -> Self {
        Self {
            activities: Vec::new(),
            current_activity: None,
            last_updated: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

// 用于数据聚合的结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySession {
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_duration: u64,
    pub activity_count: usize,
}

pub struct TimeTracker {
    pub current_activity: Option<ActivityRecord>,
    pub data: TimeTrackerData,
    pub data_file: String,
    pub interval: Duration,
    pub enhanced_monitor: Option<Box<dyn EnhancedWindowMonitor + Send>>,
    pub use_enhanced_monitoring: bool,
}

impl TimeTracker {
    pub fn new(data_file: String, interval_seconds: u64) -> Self {
        let interval = Duration::from_secs(interval_seconds.max(1)); // 最小1秒

        // 延迟初始化增强监控器，避免在TUI启动时阻塞
        Self {
            current_activity: None,
            data: TimeTrackerData::default(),
            data_file,
            interval,
            enhanced_monitor: None,
            use_enhanced_monitoring: false,
        }
    }

    /// 初始化增强监控器（延迟初始化）
    pub fn initialize_monitor(&mut self) {
        if self.enhanced_monitor.is_none() {
            match std::panic::catch_unwind(get_best_monitor) {
                Ok(monitor) => {
                    log::info!("成功初始化增强监控器");
                    self.enhanced_monitor = Some(monitor);
                    self.use_enhanced_monitoring = true;
                }
                Err(_) => {
                    log::warn!("增强监控器初始化失败，将使用基础监控");
                    self.use_enhanced_monitoring = false;
                }
            }
        }
    }

    /// 检查并请求必要的权限
    pub async fn check_permissions(&self) -> Result<()> {
        log::info!("检查窗口监控权限...");

        if let Some(monitor) = &self.enhanced_monitor {
            let permissions = monitor.check_permissions();
            for (name, status) in permissions {
                match status {
                    PermissionStatus::Granted => {
                        log::info!("权限 {name} 已授予");
                    }
                    PermissionStatus::Denied => {
                        log::warn!("权限 {name} 被拒绝");
                        return Err(anyhow::anyhow!("权限被拒绝: {name}"));
                    }
                    PermissionStatus::NotRequired => {
                        log::info!("权限 {name} 不需要");
                    }
                    PermissionStatus::Unknown => {
                        log::warn!("权限 {name} 状态未知");
                    }
                }
            }
        } else {
            log::info!("未启用增强监控，跳过权限检查");
        }

        Ok(())
    }

    pub fn load_data(&mut self) -> Result<()> {
        // 使用超时机制避免长时间阻塞
        let path = Path::new(&self.data_file);

        if !path.exists() {
            // 文件不存在，使用默认数据
            self.data = TimeTrackerData::default();
            return Ok(());
        }

        // 尝试读取文件，如果失败则使用默认数据
        match fs::read_to_string(&self.data_file) {
            Ok(file_content) => {
                if file_content.trim().is_empty() {
                    self.data = TimeTrackerData::default();
                    return Ok(());
                }

                // 尝试解析数据，如果失败则使用默认数据
                match serde_json::from_str::<TimeTrackerData>(&file_content) {
                    Ok(data) => {
                        self.data = data;
                    }
                    Err(_) => {
                        // 尝试加载旧格式
                        match serde_json::from_str::<Vec<ActivityRecord>>(&file_content) {
                            Ok(activities) => {
                                self.data = TimeTrackerData {
                                    activities,
                                    current_activity: None,
                                    last_updated: Utc::now(),
                                    version: env!("CARGO_PKG_VERSION").to_string(),
                                };
                                // 异步保存，避免阻塞
                                let _ = self.save_data();
                            }
                            Err(_e) => {
                                // 解析失败，使用默认数据，不备份以避免阻塞
                                self.data = TimeTrackerData::default();
                            }
                        }
                    }
                }
            }
            Err(_e) => {
                // 读取失败，使用默认数据
                self.data = TimeTrackerData::default();
            }
        }

        Ok(())
    }

    pub fn save_data(&self) -> Result<()> {
        let mut data = self.data.clone();

        // 如果有当前活动，动态计算其持续时间
        if let Some(mut current) = self.current_activity.clone() {
            current.duration = (Utc::now() - current.start_time).num_seconds() as u64;
            data.current_activity = Some(current);
        } else {
            data.current_activity = None;
        }

        data.last_updated = Utc::now();
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(&self.data_file, json)?;
        Ok(())
    }

    pub async fn start_monitoring(&mut self) -> Result<()> {
        // 在开始监控时才初始化监控器
        self.initialize_monitor();

        log::info!("开始监控，间隔: {:?}", self.interval);
        log::info!("使用增强监控: {}", self.use_enhanced_monitoring);

        // 检查权限
        if let Err(e) = self.check_permissions().await {
            log::warn!("权限检查失败，将使用降级模式: {}", e);
            self.use_enhanced_monitoring = false;
        }

        let mut interval_timer = time::interval(self.interval);
        let mut error_count = 0;
        const MAX_ERRORS: u32 = 10;

        loop {
            interval_timer.tick().await;

            // 尝试使用增强监控系统
            let window_result = if self.use_enhanced_monitoring {
                if let Some(monitor) = &mut self.enhanced_monitor {
                    match monitor.get_active_window() {
                        Ok(Some(enhanced_info)) => {
                            error_count = 0; // 重置错误计数
                            match self.update_activity_enhanced(enhanced_info) {
                                Ok(_) => {
                                    log::debug!("增强活动更新成功");
                                    continue;
                                }
                                Err(e) => {
                                    log::error!("增强活动更新失败: {}", e);
                                    Err(e)
                                }
                            }
                        }
                        Ok(None) => {
                            log::debug!("增强监控未检测到活动窗口");
                            // 降级到基础监控
                            get_active_window()
                        }
                        Err(e) => {
                            log::warn!("增强监控失败，尝试降级到基础监控: {}", e);
                            // 降级到基础监控
                            get_active_window()
                        }
                    }
                } else {
                    log::debug!("增强监控未初始化，使用基础监控");
                    get_active_window()
                }
            } else {
                // 使用基础监控系统
                get_active_window()
            };

            // 处理监控结果
            match window_result {
                Ok(window_info) => {
                    error_count = 0; // 重置错误计数

                    match self.update_activity(window_info) {
                        Ok(_) => {
                            log::debug!("基础活动更新成功");
                        }
                        Err(e) => {
                            log::error!("基础活动更新失败: {}", e);
                            error_count += 1;
                            if error_count >= MAX_ERRORS {
                                log::error!("连续错误次数过多，退出监控");
                                return Err(e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error_count += 1;
                    log::warn!(
                        "获取活动窗口失败 (错误 {}/{}): {}",
                        error_count,
                        MAX_ERRORS,
                        e
                    );

                    if error_count >= MAX_ERRORS {
                        log::error!("连续获取活动窗口失败次数过多，退出监控");
                        return Err(anyhow::anyhow!("连续获取活动窗口失败: {}", e));
                    }

                    // 等待一段时间再重试
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    pub fn update_activity_enhanced(&mut self, window_info: EnhancedWindowInfo) -> Result<()> {
        let activity_key = format!("{} - {}", window_info.app_name, window_info.window_title);

        // 检查是否需要切换活动
        let should_switch = match &self.current_activity {
            None => true,
            Some(current) => {
                let current_key = format!("{} - {}", current.app_name, current.window_title);
                current_key != activity_key
            }
        };

        if should_switch {
            // 结束当前活动
            if let Some(mut current) = self.current_activity.take() {
                current.finish();
                log::info!(
                    "活动结束: {} - {} ({}秒, 置信度: {:.2})",
                    current.app_name,
                    current.window_title,
                    current.duration,
                    current.confidence
                );
                self.data.activities.push(current);
                self.save_data()?;
            }

            // 开始新活动
            let new_activity = ActivityRecord::new_enhanced(window_info.clone());
            log::info!(
                "新活动开始: {} - {} (置信度: {:.2})",
                new_activity.app_name,
                new_activity.window_title,
                new_activity.confidence
            );

            // 如果有应用路径或bundle ID，记录额外信息
            if let Some(ref path) = window_info.app_path {
                log::debug!("应用路径: {}", path);
            }
            if let Some(ref bundle_id) = window_info.bundle_id {
                log::debug!("Bundle ID: {}", bundle_id);
            }
            if let Some(ref geometry) = window_info.geometry {
                log::debug!(
                    "窗口位置: {}x{} at ({}, {})",
                    geometry.width,
                    geometry.height,
                    geometry.x,
                    geometry.y
                );
            }

            self.current_activity = Some(new_activity);

            // 立即保存数据，包含当前活动，以便TUI能实时看到
            if let Err(e) = self.save_data() {
                log::warn!("保存当前活动数据失败: {}", e);
            }
        } else {
            // 即使没有切换活动，也要定期保存当前活动的状态
            if let Err(e) = self.save_data() {
                log::warn!("保存当前活动状态失败: {}", e);
            }
        }

        Ok(())
    }

    pub fn update_activity(&mut self, window_info: WindowInfo) -> Result<()> {
        let activity_key = format!("{} - {}", window_info.app_name, window_info.window_title);

        // 检查是否需要切换活动
        let should_switch = match &self.current_activity {
            None => true,
            Some(current) => {
                let current_key = format!("{} - {}", current.app_name, current.window_title);
                current_key != activity_key
            }
        };

        if should_switch {
            // 结束当前活动
            if let Some(mut current) = self.current_activity.take() {
                current.finish();
                log::info!(
                    "活动结束: {} - {} ({}秒)",
                    current.app_name,
                    current.window_title,
                    current.duration
                );
                self.data.activities.push(current);
                self.save_data()?;
            }

            // 开始新活动
            let new_activity = ActivityRecord::new(window_info);
            log::info!(
                "新活动开始: {} - {}",
                new_activity.app_name,
                new_activity.window_title
            );
            self.current_activity = Some(new_activity);

            // 立即保存数据，包含当前活动，以便TUI能实时看到
            if let Err(e) = self.save_data() {
                log::warn!("保存当前活动数据失败: {}", e);
            }
        } else {
            // 即使没有切换活动，也要定期保存当前活动的状态
            if let Err(e) = self.save_data() {
                log::warn!("保存当前活动状态失败: {}", e);
            }
        }

        Ok(())
    }

    pub fn get_statistics(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();

        for activity in &self.data.activities {
            let key = format!("{} - {}", activity.app_name, activity.window_title);
            *stats.entry(key).or_insert(0) += activity.duration;
        }

        // 包含当前活动的时间
        if let Some(current) = &self.current_activity {
            let key = format!("{} - {}", current.app_name, current.window_title);
            let current_duration = (Utc::now() - current.start_time).num_seconds() as u64;
            *stats.entry(key).or_insert(0) += current_duration;
        }

        stats
    }

    pub fn get_recent_activities(&self, limit: usize) -> Vec<&ActivityRecord> {
        let mut recent: Vec<&ActivityRecord> = self.data.activities.iter().collect();
        recent.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        recent.into_iter().take(limit).collect()
    }

    pub fn get_activities_by_app(&self) -> HashMap<String, Vec<&ActivityRecord>> {
        let mut by_app = HashMap::new();

        for activity in &self.data.activities {
            by_app
                .entry(activity.app_name.clone())
                .or_insert_with(Vec::new)
                .push(activity);
        }

        by_app
    }

    pub fn get_total_time(&self) -> u64 {
        let mut total = self.data.activities.iter().map(|a| a.duration).sum::<u64>();

        // 加上当前活动的时间
        if let Some(current) = &self.current_activity {
            total += (Utc::now() - current.start_time).num_seconds() as u64;
        }

        total
    }

    // 新功能：获取聚合的活动会话（相同应用和窗口的连续活动合并）
    pub fn get_activity_sessions(&self) -> Vec<ActivitySession> {
        let mut sessions = Vec::new();
        let mut current_session: Option<ActivitySession> = None;

        // 按时间排序活动
        let mut sorted_activities = self.data.activities.clone();
        sorted_activities.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        for activity in sorted_activities {
            let activity_key = format!("{} - {}", activity.app_name, activity.window_title);

            match &mut current_session {
                None => {
                    // 开始新会话
                    current_session = Some(ActivitySession {
                        app_name: activity.app_name.clone(),
                        window_title: activity.window_title.clone(),
                        start_time: activity.start_time,
                        end_time: activity.end_time.unwrap_or(activity.start_time),
                        total_duration: activity.duration,
                        activity_count: 1,
                    });
                }
                Some(session) => {
                    let session_key = format!("{} - {}", session.app_name, session.window_title);

                    // 检查是否是同一个应用和窗口，且时间间隔不超过5分钟
                    let time_gap = (activity.start_time - session.end_time).num_seconds();

                    if session_key == activity_key && time_gap <= 300 {
                        // 5分钟内
                        // 扩展当前会话
                        session.end_time = activity.end_time.unwrap_or(activity.start_time);
                        session.total_duration += activity.duration;
                        session.activity_count += 1;
                    } else {
                        // 结束当前会话，开始新会话
                        sessions.push(session.clone());
                        current_session = Some(ActivitySession {
                            app_name: activity.app_name.clone(),
                            window_title: activity.window_title.clone(),
                            start_time: activity.start_time,
                            end_time: activity.end_time.unwrap_or(activity.start_time),
                            total_duration: activity.duration,
                            activity_count: 1,
                        });
                    }
                }
            }
        }

        // 添加最后一个会话
        if let Some(session) = current_session {
            sessions.push(session);
        }

        sessions
    }

    pub fn stop_monitoring(&mut self) -> Result<()> {
        if let Some(mut current) = self.current_activity.take() {
            current.finish();
            log::info!(
                "监控停止，最后活动: {} - {} ({}秒)",
                current.app_name,
                current.window_title,
                current.duration
            );
            self.data.activities.push(current);
            self.save_data()?;
        }
        Ok(())
    }

    // 获取活动数据的引用（用于导出等功能）
    pub fn get_activities(&self) -> &Vec<ActivityRecord> {
        &self.data.activities
    }

    /// 导出为 JSON 格式
    pub fn export_json(&self) -> Result<String> {
        let json = serde_json::to_string_pretty(&self.data)?;
        Ok(json)
    }

    /// 导出为 CSV 格式
    pub fn export_csv(&self) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("app_name,window_title,start_time,end_time,duration,process_id\n");

        for activity in &self.data.activities {
            let end_time = activity
                .end_time
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                activity.app_name,
                activity.window_title,
                activity.start_time.format("%Y-%m-%d %H:%M:%S"),
                end_time,
                activity.duration,
                activity.process_id
            ));
        }

        Ok(csv)
    }
}
