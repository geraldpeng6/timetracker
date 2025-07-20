use crate::platform::{get_active_window, WindowInfo};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivityRecord {
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    #[serde(rename = "duration_seconds")]
    pub duration: u64, // 持续时间（秒）
    pub process_id: u32,
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

#[derive(Debug)]
pub struct TimeTracker {
    pub current_activity: Option<ActivityRecord>,
    pub data: TimeTrackerData,
    pub data_file: String,
    pub interval: Duration,
}

impl TimeTracker {
    pub fn new(data_file: String, interval_seconds: u64) -> Self {
        let interval = Duration::from_secs(interval_seconds.max(1)); // 最小1秒
        Self {
            current_activity: None,
            data: TimeTrackerData::default(),
            data_file,
            interval,
        }
    }

    pub fn load_data(&mut self) -> Result<()> {
        if Path::new(&self.data_file).exists() {
            let file_content = fs::read_to_string(&self.data_file)?;
            if !file_content.trim().is_empty() {
                // 尝试加载新格式
                match serde_json::from_str::<TimeTrackerData>(&file_content) {
                    Ok(data) => {
                        self.data = data;
                    }
                    Err(_) => {
                        // 尝试加载旧格式（直接的活动数组）
                        match serde_json::from_str::<Vec<ActivityRecord>>(&file_content) {
                            Ok(activities) => {
                                self.data = TimeTrackerData {
                                    activities,
                                    last_updated: Utc::now(),
                                    version: env!("CARGO_PKG_VERSION").to_string(),
                                };
                                // 立即保存为新格式
                                self.save_data()?;
                            }
                            Err(e) => {
                                log::warn!("无法解析数据文件，创建新文件: {}", e);
                                // 备份损坏的文件
                                let backup_file = format!("{}.backup", self.data_file);
                                fs::copy(&self.data_file, backup_file)?;
                                self.data = TimeTrackerData::default();
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_data(&self) -> Result<()> {
        let mut data = self.data.clone();
        data.last_updated = Utc::now();
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(&self.data_file, json)?;
        Ok(())
    }

    pub async fn start_monitoring(&mut self) -> Result<()> {
        log::info!("开始监控，间隔: {:?}", self.interval);

        let mut interval_timer = time::interval(self.interval);

        loop {
            interval_timer.tick().await;

            match get_active_window() {
                Ok(window_info) => {
                    self.update_activity(window_info)?;
                }
                Err(e) => {
                    log::warn!("获取活动窗口失败: {}", e);
                }
            }
        }
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
}
