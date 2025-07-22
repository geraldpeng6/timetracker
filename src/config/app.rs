// 应用程序配置
// 统一管理所有配置项

use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use toml;

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 配置版本
    #[serde(default = "default_config_version")]
    pub version: String,
    /// 数据文件路径
    pub data_file: String,
    /// 监控间隔（秒）
    pub monitor_interval: u64,
    /// 是否启用守护进程模式
    pub daemon_mode: bool,
    /// UI 配置
    pub ui: UiConfig,
    /// 导出配置
    pub export: ExportConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 活跃度检测配置
    #[serde(default)]
    pub activity: ActivityDetectionConfig,
}

/// 默认配置版本
fn default_config_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// 默认主题
    pub theme: String,
    /// 是否启用鼠标支持
    pub mouse_enabled: bool,
    /// 默认排序方式
    pub default_sort_by: String,
    /// 默认排序顺序
    pub default_sort_order: String,
    /// 默认视图模式
    pub default_view_mode: String,
    /// 刷新间隔（毫秒）
    pub refresh_interval: u64,
    /// 日分割点（小时，0-23）
    #[serde(default = "default_day_split_hour")]
    pub day_split_hour: u8,
}

/// 默认日分割点（0点）
fn default_day_split_hour() -> u8 {
    0
}

/// 导出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// 默认导出格式
    pub default_format: String,
    /// 默认导出路径
    pub default_path: String,
    /// 是否包含详细信息
    pub include_details: bool,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志文件路径
    pub file_path: Option<String>,
    /// 是否启用控制台输出
    pub console_enabled: bool,
}

/// 活跃度检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetectionConfig {
    /// 是否启用活跃度检测
    pub enabled: bool,
    /// 闲置超时时间（秒）
    pub idle_timeout: u64,
    /// 检测间隔（毫秒）
    pub check_interval: u64,
    /// 视频应用列表（这些应用即使闲置也记录）
    pub video_apps: Vec<String>,
    /// 视频网站列表
    pub video_sites: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: default_config_version(),
            data_file: "timetracker_data.json".to_string(),
            monitor_interval: 1,
            daemon_mode: true,
            ui: UiConfig::default(),
            export: ExportConfig::default(),
            logging: LoggingConfig::default(),
            activity: ActivityDetectionConfig::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            mouse_enabled: true,
            default_sort_by: "duration".to_string(),
            default_sort_order: "descending".to_string(),
            default_view_mode: "unified".to_string(),
            refresh_interval: 1000,
            day_split_hour: default_day_split_hour(),
        }
    }
}

impl UiConfig {
    /// 验证UI配置
    pub fn validate(&self) -> Result<()> {
        // 验证主题
        let valid_themes = [
            "default",
            "dark",
            "light",
            "high_contrast",
            "eye_care",
            "blue",
        ];
        if !valid_themes.contains(&self.theme.as_str()) {
            return Err(anyhow::anyhow!(
                "不支持的主题: {}，支持的主题: {:?}",
                self.theme,
                valid_themes
            ));
        }

        // 验证排序方式
        let valid_sort_by = ["duration", "name", "last_used", "count"];
        if !valid_sort_by.contains(&self.default_sort_by.as_str()) {
            return Err(anyhow::anyhow!(
                "不支持的排序方式: {}，支持的方式: {:?}",
                self.default_sort_by,
                valid_sort_by
            ));
        }

        // 验证排序顺序
        let valid_sort_order = ["ascending", "descending"];
        if !valid_sort_order.contains(&self.default_sort_order.as_str()) {
            return Err(anyhow::anyhow!(
                "不支持的排序顺序: {}，支持的顺序: {:?}",
                self.default_sort_order,
                valid_sort_order
            ));
        }

        // 验证视图模式
        let valid_view_mode = ["unified", "separate", "compact"];
        if !valid_view_mode.contains(&self.default_view_mode.as_str()) {
            return Err(anyhow::anyhow!(
                "不支持的视图模式: {}，支持的模式: {:?}",
                self.default_view_mode,
                valid_view_mode
            ));
        }

        // 验证刷新间隔
        if self.refresh_interval < 100 || self.refresh_interval > 10000 {
            return Err(anyhow::anyhow!(
                "刷新间隔必须在100-10000毫秒之间，当前值: {}",
                self.refresh_interval
            ));
        }

        Ok(())
    }

    /// 修复UI配置
    pub fn fix(&mut self) -> Vec<String> {
        let mut fixes = Vec::new();

        // 修复主题
        let valid_themes = [
            "default",
            "dark",
            "light",
            "high_contrast",
            "eye_care",
            "blue",
        ];
        if !valid_themes.contains(&self.theme.as_str()) {
            self.theme = "default".to_string();
            fixes.push("主题已重置为默认主题".to_string());
        }

        // 修复排序方式
        let valid_sort_by = ["duration", "name", "last_used", "count"];
        if !valid_sort_by.contains(&self.default_sort_by.as_str()) {
            self.default_sort_by = "duration".to_string();
            fixes.push("排序方式已重置为按时长排序".to_string());
        }

        // 修复排序顺序
        let valid_sort_order = ["ascending", "descending"];
        if !valid_sort_order.contains(&self.default_sort_order.as_str()) {
            self.default_sort_order = "descending".to_string();
            fixes.push("排序顺序已重置为降序".to_string());
        }

        // 修复视图模式
        let valid_view_mode = ["unified", "separate", "compact"];
        if !valid_view_mode.contains(&self.default_view_mode.as_str()) {
            self.default_view_mode = "unified".to_string();
            fixes.push("视图模式已重置为统一视图".to_string());
        }

        // 修复刷新间隔
        if self.refresh_interval < 100 {
            self.refresh_interval = 100;
            fixes.push("刷新间隔已修正为100毫秒".to_string());
        } else if self.refresh_interval > 10000 {
            self.refresh_interval = 10000;
            fixes.push("刷新间隔已修正为10000毫秒".to_string());
        }

        fixes
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            default_format: "json".to_string(),
            default_path: "./exports".to_string(),
            include_details: true,
        }
    }
}

impl ExportConfig {
    /// 验证导出配置
    pub fn validate(&self) -> Result<()> {
        // 验证导出格式
        let valid_formats = ["json", "csv", "xlsx", "toml"];
        if !valid_formats.contains(&self.default_format.as_str()) {
            return Err(anyhow::anyhow!(
                "不支持的导出格式: {}，支持的格式: {:?}",
                self.default_format,
                valid_formats
            ));
        }

        // 验证导出路径（检查是否为有效路径）
        if self.default_path.is_empty() {
            return Err(anyhow::anyhow!("导出路径不能为空"));
        }

        // 检查路径是否包含无效字符
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        if self
            .default_path
            .chars()
            .any(|c| invalid_chars.contains(&c))
        {
            return Err(anyhow::anyhow!(
                "导出路径包含无效字符: {}",
                self.default_path
            ));
        }

        Ok(())
    }

    /// 修复导出配置
    pub fn fix(&mut self) -> Vec<String> {
        let mut fixes = Vec::new();

        // 修复导出格式
        let valid_formats = ["json", "csv", "xlsx", "toml"];
        if !valid_formats.contains(&self.default_format.as_str()) {
            self.default_format = "json".to_string();
            fixes.push("导出格式已重置为JSON".to_string());
        }

        // 修复导出路径
        if self.default_path.is_empty() {
            self.default_path = "./exports".to_string();
            fixes.push("导出路径已重置为./exports".to_string());
        }

        // 清理路径中的无效字符
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        let original_path = self.default_path.clone();
        self.default_path = self
            .default_path
            .chars()
            .filter(|c| !invalid_chars.contains(c))
            .collect();

        if self.default_path != original_path {
            fixes.push(format!(
                "导出路径已清理无效字符: {} -> {}",
                original_path, self.default_path
            ));
        }

        fixes
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: None,
            console_enabled: true,
        }
    }
}

impl Default for ActivityDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_timeout: 300,    // 5分钟
            check_interval: 1000, // 1秒
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
        }
    }
}

impl ActivityDetectionConfig {
    /// 转换为ActivityConfig
    pub fn to_activity_config(&self) -> crate::core::activity_detector::ActivityConfig {
        crate::core::activity_detector::ActivityConfig {
            enabled: self.enabled,
            idle_timeout: self.idle_timeout,
            check_interval: self.check_interval,
            video_apps: self.video_apps.clone(),
            video_sites: self.video_sites.clone(),
        }
    }

    /// 验证活跃度检测配置
    pub fn validate(&self) -> Result<()> {
        if self.idle_timeout == 0 {
            return Err(anyhow::anyhow!("闲置超时时间不能为0"));
        }

        if self.idle_timeout > 86400 {
            return Err(anyhow::anyhow!("闲置超时时间不能超过24小时"));
        }

        if self.check_interval < 100 {
            return Err(anyhow::anyhow!("检测间隔不能小于100毫秒"));
        }

        if self.check_interval > 60000 {
            return Err(anyhow::anyhow!("检测间隔不能超过60秒"));
        }

        Ok(())
    }

    /// 修复活跃度检测配置
    pub fn fix(&mut self) -> Vec<String> {
        let mut fixes = Vec::new();

        if self.idle_timeout == 0 {
            self.idle_timeout = 300;
            fixes.push("闲置超时时间已修正为300秒".to_string());
        } else if self.idle_timeout > 86400 {
            self.idle_timeout = 86400;
            fixes.push("闲置超时时间已修正为86400秒（24小时）".to_string());
        }

        if self.check_interval < 100 {
            self.check_interval = 1000;
            fixes.push("检测间隔已修正为1000毫秒".to_string());
        } else if self.check_interval > 60000 {
            self.check_interval = 60000;
            fixes.push("检测间隔已修正为60000毫秒".to_string());
        }

        fixes
    }
}

impl LoggingConfig {
    /// 验证日志配置
    pub fn validate(&self) -> Result<()> {
        // 验证日志级别
        let valid_levels = ["trace", "debug", "info", "warn", "error", "off"];
        if !valid_levels.contains(&self.level.as_str()) {
            return Err(anyhow::anyhow!(
                "不支持的日志级别: {}，支持的级别: {:?}",
                self.level,
                valid_levels
            ));
        }

        // 验证日志文件路径（如果设置了）
        if let Some(ref path) = self.file_path {
            if path.is_empty() {
                return Err(anyhow::anyhow!("日志文件路径不能为空字符串"));
            }

            // 检查路径是否包含无效字符
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            if path.chars().any(|c| invalid_chars.contains(&c)) {
                return Err(anyhow::anyhow!("日志文件路径包含无效字符: {}", path));
            }
        }

        Ok(())
    }

    /// 修复日志配置
    pub fn fix(&mut self) -> Vec<String> {
        let mut fixes = Vec::new();

        // 修复日志级别
        let valid_levels = ["trace", "debug", "info", "warn", "error", "off"];
        if !valid_levels.contains(&self.level.as_str()) {
            self.level = "info".to_string();
            fixes.push("日志级别已重置为info".to_string());
        }

        // 修复日志文件路径
        if let Some(ref mut path) = self.file_path {
            if path.is_empty() {
                self.file_path = None;
                fixes.push("空的日志文件路径已清除".to_string());
            } else {
                // 清理路径中的无效字符
                let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
                let original_path = path.clone();
                *path = path
                    .chars()
                    .filter(|c| !invalid_chars.contains(c))
                    .collect();

                if *path != original_path {
                    fixes.push(format!(
                        "日志文件路径已清理无效字符: {} -> {}",
                        original_path, path
                    ));
                }

                // 如果清理后路径为空，则清除
                if path.is_empty() {
                    self.file_path = None;
                    fixes.push("清理后的日志文件路径为空，已清除".to_string());
                }
            }
        }

        fixes
    }
}

impl AppConfig {
    /// 获取配置文件路径
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = config_dir().ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?;

        let app_config_dir = config_dir.join("timetracker");
        std::fs::create_dir_all(&app_config_dir)?;

        Ok(app_config_dir.join("config.toml"))
    }

    /// 加载配置
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;

            // 尝试解析配置
            match toml::from_str::<AppConfig>(&content) {
                Ok(mut config) => {
                    // 验证配置
                    if let Err(e) = config.validate() {
                        log::warn!("配置验证失败: {}，尝试自动修复", e);

                        // 尝试自动修复
                        let fixes = config.fix();
                        if !fixes.is_empty() {
                            log::info!("配置已自动修复:");
                            for fix in &fixes {
                                log::info!("  - {}", fix);
                            }

                            // 保存修复后的配置
                            config.save()?;
                        }
                    }

                    Ok(config)
                }
                Err(e) => {
                    log::error!("配置文件解析失败: {}，使用默认配置", e);

                    // 备份损坏的配置文件
                    let backup_path = config_path.with_extension("toml.backup");
                    if let Err(backup_err) = std::fs::copy(&config_path, &backup_path) {
                        log::warn!("无法备份损坏的配置文件: {}", backup_err);
                    } else {
                        log::info!("损坏的配置文件已备份到: {:?}", backup_path);
                    }

                    // 使用默认配置并保存
                    let config = Self::default();
                    config.save()?;
                    Ok(config)
                }
            }
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        use crate::utils::validation::*;

        // 验证监控间隔
        validate_interval(self.monitor_interval)?;

        // 验证数据文件路径
        validate_file_path(&self.data_file)?;

        // 验证 UI 配置
        self.ui.validate()?;

        // 验证导出配置
        self.export.validate()?;

        // 验证日志配置
        self.logging.validate()?;

        // 验证活跃度检测配置
        self.activity.validate()?;

        Ok(())
    }

    /// 修复配置（自动修正无效值）
    pub fn fix(&mut self) -> Vec<String> {
        let mut fixes = Vec::new();

        // 修复监控间隔
        if self.monitor_interval == 0 {
            self.monitor_interval = 1;
            fixes.push("监控间隔已修正为1秒".to_string());
        } else if self.monitor_interval > 3600 {
            self.monitor_interval = 3600;
            fixes.push("监控间隔已修正为3600秒（1小时）".to_string());
        }

        // 修复UI配置
        fixes.extend(self.ui.fix());

        // 修复导出配置
        fixes.extend(self.export.fix());

        // 修复日志配置
        fixes.extend(self.logging.fix());

        // 修复活跃度检测配置
        fixes.extend(self.activity.fix());

        fixes
    }

    /// 合并配置（用于命令行参数覆盖）
    pub fn merge_with_args(
        mut self,
        data_file: Option<String>,
        interval: Option<u64>,
        daemon: Option<bool>,
    ) -> Self {
        if let Some(file) = data_file {
            self.data_file = file;
        }

        if let Some(interval) = interval {
            self.monitor_interval = interval;
        }

        if let Some(daemon) = daemon {
            self.daemon_mode = daemon;
        }

        self
    }

    /// 检查是否需要迁移配置
    pub fn needs_migration(&self) -> bool {
        let current_version = env!("CARGO_PKG_VERSION");
        self.version != current_version
    }

    /// 迁移配置到当前版本
    pub fn migrate(&mut self) -> Result<Vec<String>> {
        let mut migration_log = Vec::new();
        let current_version = env!("CARGO_PKG_VERSION");

        if self.version == current_version {
            return Ok(migration_log);
        }

        migration_log.push(format!(
            "开始配置迁移: {} -> {}",
            self.version, current_version
        ));

        // 根据版本进行迁移
        match self.version.as_str() {
            "0.1.0" | "0.1.1" | "0.1.2" => {
                migration_log.extend(self.migrate_from_v0_1_x()?);
            }
            "0.2.0" | "0.2.1" => {
                migration_log.extend(self.migrate_from_v0_2_x()?);
            }
            _ => {
                // 对于未知版本，进行通用迁移
                migration_log.extend(self.migrate_generic()?);
            }
        }

        // 更新版本号
        self.version = current_version.to_string();
        migration_log.push(format!("配置版本已更新到: {}", current_version));

        // 验证迁移后的配置
        if let Err(e) = self.validate() {
            migration_log.push(format!("迁移后配置验证失败: {}，尝试自动修复", e));
            let fixes = self.fix();
            migration_log.extend(fixes);
        }

        Ok(migration_log)
    }

    /// 从 v0.1.x 迁移
    fn migrate_from_v0_1_x(&mut self) -> Result<Vec<String>> {
        let mut log = Vec::new();

        // v0.1.x 可能没有某些新字段，使用默认值
        if self.ui.theme == "default" {
            // 检查是否需要更新主题名称
            log.push("主题配置已更新".to_string());
        }

        // 添加新的配置项
        if self.ui.refresh_interval < 100 {
            self.ui.refresh_interval = 1000;
            log.push("刷新间隔已更新为1000ms".to_string());
        }

        log.push("从 v0.1.x 迁移完成".to_string());
        Ok(log)
    }

    /// 从 v0.2.x 迁移
    fn migrate_from_v0_2_x(&mut self) -> Result<Vec<String>> {
        let log = vec!["从 v0.2.x 迁移完成".to_string()];
        Ok(log)
    }

    /// 通用迁移
    fn migrate_generic(&mut self) -> Result<Vec<String>> {
        let mut log = Vec::new();

        // 执行通用的配置修复
        let fixes = self.fix();
        log.extend(fixes);

        log.push("通用配置迁移完成".to_string());
        Ok(log)
    }

    /// 获取配置摘要
    pub fn summary(&self) -> String {
        format!(
            "TimeTracker 配置 v{}\n\
            - 数据文件: {}\n\
            - 监控间隔: {}秒\n\
            - 守护进程: {}\n\
            - 主题: {}\n\
            - 鼠标支持: {}\n\
            - 导出格式: {}\n\
            - 日志级别: {}",
            self.version,
            self.data_file,
            self.monitor_interval,
            if self.daemon_mode { "启用" } else { "禁用" },
            self.ui.theme,
            if self.ui.mouse_enabled {
                "启用"
            } else {
                "禁用"
            },
            self.export.default_format,
            self.logging.level
        )
    }
}
