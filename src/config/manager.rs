// 配置管理器
// 统一管理应用程序和 AI 配置

use crate::ai::config::AIConfig;
use crate::config::app::AppConfig;
use anyhow::Result;

/// 统一配置管理器
pub struct ConfigManager {
    pub app_config: AppConfig,
    pub ai_config: AIConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self> {
        let mut app_config = AppConfig::load()?;
        let ai_config = AIConfig::load()?;

        // 检查是否需要迁移配置
        if app_config.needs_migration() {
            log::info!("检测到配置需要迁移");
            match app_config.migrate() {
                Ok(migration_log) => {
                    for log_entry in migration_log {
                        log::info!("配置迁移: {}", log_entry);
                    }
                    // 保存迁移后的配置
                    app_config.save()?;
                    log::info!("配置迁移完成并已保存");
                }
                Err(e) => {
                    log::error!("配置迁移失败: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(Self {
            app_config,
            ai_config,
        })
    }

    /// 验证所有配置
    pub fn validate(&self) -> Result<()> {
        self.app_config.validate()?;
        self.ai_config.validate()?;
        Ok(())
    }

    /// 保存所有配置
    pub fn save(&self) -> Result<()> {
        self.app_config.save()?;
        self.ai_config.save()?;
        Ok(())
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> Result<()> {
        self.app_config = AppConfig::load()?;
        self.ai_config = AIConfig::load()?;
        Ok(())
    }

    /// 重置为默认配置
    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.app_config = AppConfig::default();
        self.ai_config = AIConfig::default();
        self.save()?;
        Ok(())
    }

    /// 导出配置
    pub fn export_config(&self, path: &str) -> Result<()> {
        let config_data = serde_json::json!({
            "app": self.app_config,
            "ai": self.ai_config,
            "exported_at": chrono::Utc::now(),
            "version": env!("CARGO_PKG_VERSION")
        });

        let content = serde_json::to_string_pretty(&config_data)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 导入配置
    pub fn import_config(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let config_data: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(app_config) = config_data.get("app") {
            self.app_config = serde_json::from_value(app_config.clone())?;
        }

        if let Some(ai_config) = config_data.get("ai") {
            self.ai_config = serde_json::from_value(ai_config.clone())?;
        }

        self.validate()?;
        self.save()?;
        Ok(())
    }

    /// 获取配置摘要
    pub fn summary(&self) -> String {
        format!("{}\n\n{}", self.app_config.summary(), "AI配置已加载")
    }

    /// 检查配置健康状态
    pub fn health_check(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // 检查应用配置
        if let Err(e) = self.app_config.validate() {
            issues.push(format!("应用配置问题: {}", e));
        }

        // 检查AI配置
        if let Err(e) = self.ai_config.validate() {
            issues.push(format!("AI配置问题: {}", e));
        }

        // 检查数据文件是否可访问
        let data_file = &self.app_config.data_file;
        if !data_file.is_empty() {
            if let Some(parent) = std::path::Path::new(data_file).parent() {
                if !parent.exists() {
                    issues.push(format!("数据文件目录不存在: {:?}", parent));
                }
            }
        }

        // 检查导出目录是否可访问
        let export_path = &self.app_config.export.default_path;
        if !export_path.is_empty() {
            let path = std::path::Path::new(export_path);
            if !path.exists() {
                if let Err(e) = std::fs::create_dir_all(path) {
                    issues.push(format!("无法创建导出目录 {}: {}", export_path, e));
                }
            }
        }

        Ok(issues)
    }

    /// 自动修复配置问题
    pub fn auto_fix(&mut self) -> Result<Vec<String>> {
        let mut fixes = Vec::new();

        // 修复应用配置
        fixes.extend(self.app_config.fix());

        // 保存修复后的配置
        if !fixes.is_empty() {
            self.save()?;
            fixes.push("配置已自动修复并保存".to_string());
        }

        Ok(fixes)
    }

    /// 备份当前配置
    pub fn backup(&self, backup_path: Option<&str>) -> Result<String> {
        let backup_file = if let Some(path) = backup_path {
            path.to_string()
        } else {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            format!("timetracker_config_backup_{}.json", timestamp)
        };

        self.export_config(&backup_file)?;
        Ok(backup_file)
    }
}
