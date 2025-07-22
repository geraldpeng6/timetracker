// 用户活跃度检测模块
// 检测用户是否处于活跃状态，闲置时不记录窗口活动

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{GetLastInputInfo, LASTINPUTINFO};

#[cfg(target_os = "linux")]
use std::process::Command;

/// 用户活跃状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActivityStatus {
    /// 用户活跃
    Active,
    /// 用户闲置
    Idle,
    /// 正在观看视频（即使闲置也记录）
    WatchingVideo,
    /// 未知状态
    Unknown,
}

impl ActivityStatus {
    /// 是否应该记录窗口活动
    pub fn should_record(&self) -> bool {
        match self {
            Self::Active | Self::WatchingVideo => true,
            Self::Idle | Self::Unknown => false,
        }
    }

    /// 获取状态描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Active => "活跃",
            Self::Idle => "闲置",
            Self::WatchingVideo => "观看视频",
            Self::Unknown => "未知",
        }
    }

    /// 获取状态图标
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Active => "🟢",
            Self::Idle => "🟡",
            Self::WatchingVideo => "📺",
            Self::Unknown => "❓",
        }
    }
}

/// 活跃度检测配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityConfig {
    /// 闲置超时时间（秒）
    pub idle_timeout: u64,
    /// 是否启用活跃度检测
    pub enabled: bool,
    /// 视频应用列表（这些应用即使闲置也记录）
    pub video_apps: Vec<String>,
    /// 视频网站列表
    pub video_sites: Vec<String>,
    /// 检测间隔（毫秒）
    pub check_interval: u64,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            idle_timeout: 300, // 5分钟
            enabled: true,
            video_apps: vec![
                "VLC".to_string(),
                "QuickTime Player".to_string(),
                "IINA".to_string(),
                "PotPlayer".to_string(),
                "MPC-HC".to_string(),
                "Windows Media Player".to_string(),
                "Netflix".to_string(),
                "YouTube".to_string(),
                "Bilibili".to_string(),
                "爱奇艺".to_string(),
                "腾讯视频".to_string(),
                "优酷".to_string(),
            ],
            video_sites: vec![
                "youtube.com".to_string(),
                "bilibili.com".to_string(),
                "netflix.com".to_string(),
                "iqiyi.com".to_string(),
                "v.qq.com".to_string(),
                "youku.com".to_string(),
                "twitch.tv".to_string(),
                "vimeo.com".to_string(),
            ],
            check_interval: 1000, // 1秒
        }
    }
}

/// 用户活跃度检测器
pub struct ActivityDetector {
    config: ActivityConfig,
    last_activity_time: SystemTime,
    last_check_time: SystemTime,
    current_status: ActivityStatus,
}

impl ActivityDetector {
    /// 创建新的活跃度检测器
    pub fn new(config: ActivityConfig) -> Self {
        let now = SystemTime::now();
        Self {
            config,
            last_activity_time: now,
            last_check_time: now,
            current_status: ActivityStatus::Unknown,
        }
    }

    /// 使用默认配置创建检测器
    pub fn with_default_config() -> Self {
        Self::new(ActivityConfig::default())
    }

    /// 检测当前用户活跃状态
    pub fn detect_activity(
        &mut self,
        current_app: Option<&str>,
        current_window: Option<&str>,
    ) -> Result<ActivityStatus> {
        if !self.config.enabled {
            return Ok(ActivityStatus::Active);
        }

        let now = SystemTime::now();

        // 检查是否需要更新活跃度状态
        if now
            .duration_since(self.last_check_time)
            .unwrap_or(Duration::ZERO)
            .as_millis()
            < self.config.check_interval as u128
        {
            return Ok(self.current_status.clone());
        }

        self.last_check_time = now;

        // 获取系统闲置时间
        let idle_time = self.get_system_idle_time()?;

        // 检查是否正在观看视频
        let is_watching_video = self.is_watching_video(current_app, current_window);

        // 确定活跃状态
        let status = if is_watching_video {
            ActivityStatus::WatchingVideo
        } else if idle_time.as_secs() > self.config.idle_timeout {
            ActivityStatus::Idle
        } else {
            ActivityStatus::Active
        };

        // 更新状态
        if status != ActivityStatus::Idle {
            self.last_activity_time = now;
        }

        self.current_status = status.clone();
        Ok(status)
    }

    /// 获取当前活跃状态
    pub fn current_status(&self) -> &ActivityStatus {
        &self.current_status
    }

    /// 获取上次活跃时间
    pub fn last_activity_time(&self) -> SystemTime {
        self.last_activity_time
    }

    /// 获取闲置时长
    pub fn idle_duration(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.last_activity_time)
            .unwrap_or(Duration::ZERO)
    }

    /// 检查是否正在观看视频
    fn is_watching_video(&self, current_app: Option<&str>, current_window: Option<&str>) -> bool {
        // 检查应用名称
        if let Some(app_name) = current_app {
            for video_app in &self.config.video_apps {
                if app_name.to_lowercase().contains(&video_app.to_lowercase()) {
                    return true;
                }
            }
        }

        // 检查窗口标题中的视频网站
        if let Some(window_title) = current_window {
            for video_site in &self.config.video_sites {
                if window_title
                    .to_lowercase()
                    .contains(&video_site.to_lowercase())
                {
                    return true;
                }
            }

            // 检查常见的视频相关关键词
            let video_keywords = [
                "播放",
                "视频",
                "电影",
                "电视剧",
                "直播",
                "play",
                "video",
                "movie",
                "stream",
            ];
            for keyword in &video_keywords {
                if window_title.to_lowercase().contains(keyword) {
                    return true;
                }
            }
        }

        false
    }

    /// 获取系统闲置时间
    #[cfg(target_os = "macos")]
    fn get_system_idle_time(&self) -> Result<Duration> {
        let output = Command::new("ioreg")
            .args(&["-c", "IOHIDSystem"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // 查找 HIDIdleTime
        for line in output_str.lines() {
            if line.contains("HIDIdleTime") {
                if let Some(time_str) = line.split('=').nth(1) {
                    if let Ok(nanoseconds) = time_str.trim().parse::<u64>() {
                        return Ok(Duration::from_nanos(nanoseconds));
                    }
                }
            }
        }

        // 如果无法获取，返回0
        Ok(Duration::ZERO)
    }

    /// 获取系统闲置时间 (Windows)
    #[cfg(target_os = "windows")]
    fn get_system_idle_time(&self) -> Result<Duration> {
        unsafe {
            let mut last_input_info = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };

            if GetLastInputInfo(&mut last_input_info) != 0 {
                let current_time = winapi::um::sysinfoapi::GetTickCount();
                let idle_time = current_time - last_input_info.dwTime;
                Ok(Duration::from_millis(idle_time as u64))
            } else {
                Ok(Duration::ZERO)
            }
        }
    }

    /// 获取系统闲置时间 (Linux)
    #[cfg(target_os = "linux")]
    fn get_system_idle_time(&self) -> Result<Duration> {
        // 尝试使用 xprintidle
        if let Ok(output) = Command::new("xprintidle").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(milliseconds) = output_str.trim().parse::<u64>() {
                    return Ok(Duration::from_millis(milliseconds));
                }
            }
        }

        // 备用方法：使用 xssstate
        if let Ok(output) = Command::new("xssstate").args(&["-i"]).output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(seconds) = output_str.trim().parse::<u64>() {
                    return Ok(Duration::from_secs(seconds));
                }
            }
        }

        // 如果都失败了，返回0
        Ok(Duration::ZERO)
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ActivityConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &ActivityConfig {
        &self.config
    }

    /// 强制设置为活跃状态
    pub fn force_active(&mut self) {
        self.last_activity_time = SystemTime::now();
        self.current_status = ActivityStatus::Active;
    }

    /// 获取活跃度统计信息
    pub fn get_stats(&self) -> ActivityStats {
        let total_idle_time = if self.current_status == ActivityStatus::Idle {
            self.idle_duration()
        } else {
            Duration::ZERO
        };

        ActivityStats {
            current_status: self.current_status.clone(),
            last_activity_time: self.last_activity_time,
            idle_duration: total_idle_time,
            idle_timeout: Duration::from_secs(self.config.idle_timeout),
            detection_enabled: self.config.enabled,
        }
    }
}

/// 活跃度统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStats {
    pub current_status: ActivityStatus,
    pub last_activity_time: SystemTime,
    pub idle_duration: Duration,
    pub idle_timeout: Duration,
    pub detection_enabled: bool,
}

impl ActivityStats {
    /// 格式化闲置时长
    pub fn format_idle_duration(&self) -> String {
        let secs = self.idle_duration.as_secs();
        if secs < 60 {
            format!("{}秒", secs)
        } else if secs < 3600 {
            format!("{}分{}秒", secs / 60, secs % 60)
        } else {
            format!("{}小时{}分", secs / 3600, (secs % 3600) / 60)
        }
    }

    /// 获取状态描述
    pub fn status_description(&self) -> String {
        format!(
            "{} {}",
            self.current_status.icon(),
            self.current_status.description()
        )
    }
}
