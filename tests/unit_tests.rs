// 单元测试
// 测试各个模块的单独功能

use timetracker::{
    config::app::{AppConfig, ExportConfig, LoggingConfig, UiConfig},
    ui::themes::Theme,
    utils::permissions::PermissionStatus,
};

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();

        assert_eq!(config.monitor_interval, 1);
        assert_eq!(config.daemon_mode, true);
        assert_eq!(config.data_file, "timetracker_data.json");
        assert!(!config.version.is_empty());
    }

    #[test]
    fn test_app_config_validation() {
        let mut config = AppConfig::default();

        // 测试有效配置
        assert!(config.validate().is_ok());

        // 测试无效监控间隔
        config.monitor_interval = 0;
        assert!(config.validate().is_err());

        // 重置并测试其他无效值
        config = AppConfig::default();
        config.ui.theme = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_app_config_fix() {
        let mut config = AppConfig::default();

        // 设置无效值
        config.monitor_interval = 0;
        config.ui.theme = "invalid".to_string();
        config.export.default_format = "invalid".to_string();
        config.logging.level = "invalid".to_string();

        // 执行修复
        let fixes = config.fix();

        // 验证修复结果
        assert!(!fixes.is_empty());
        assert!(config.validate().is_ok());
        assert_eq!(config.monitor_interval, 1);
        assert_eq!(config.ui.theme, "default");
        assert_eq!(config.export.default_format, "json");
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_ui_config_validation() {
        let mut config = UiConfig::default();

        // 测试有效配置
        assert!(config.validate().is_ok());

        // 测试无效主题
        config.theme = "invalid".to_string();
        assert!(config.validate().is_err());

        // 测试无效刷新间隔
        config = UiConfig::default();
        config.refresh_interval = 50; // 太小
        assert!(config.validate().is_err());

        config.refresh_interval = 20000; // 太大
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_export_config_validation() {
        let mut config = ExportConfig::default();

        // 测试有效配置
        assert!(config.validate().is_ok());

        // 测试无效格式
        config.default_format = "invalid".to_string();
        assert!(config.validate().is_err());

        // 测试空路径
        config = ExportConfig::default();
        config.default_path = "".to_string();
        assert!(config.validate().is_err());

        // 测试包含无效字符的路径
        config.default_path = "path<with>invalid:chars".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_logging_config_validation() {
        let mut config = LoggingConfig::default();

        // 测试有效配置
        assert!(config.validate().is_ok());

        // 测试无效日志级别
        config.level = "invalid".to_string();
        assert!(config.validate().is_err());

        // 测试空文件路径
        config = LoggingConfig::default();
        config.file_path = Some("".to_string());
        assert!(config.validate().is_err());

        // 测试包含无效字符的文件路径
        config.file_path = Some("path<with>invalid:chars.log".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_migration() {
        let mut config = AppConfig::default();
        config.version = "0.1.0".to_string();

        // 测试是否需要迁移
        assert!(config.needs_migration());

        // 执行迁移
        let migration_log = config.migrate().unwrap();
        assert!(!migration_log.is_empty());

        // 验证迁移后版本
        assert_eq!(config.version, env!("CARGO_PKG_VERSION"));
        assert!(!config.needs_migration());
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let default_theme = Theme::default();
        assert_eq!(default_theme.name, "Default");

        let dark_theme = Theme::dark();
        assert_eq!(dark_theme.name, "Dark");

        let light_theme = Theme::light();
        assert_eq!(light_theme.name, "Light");

        let high_contrast_theme = Theme::high_contrast();
        assert_eq!(high_contrast_theme.name, "High Contrast");

        let eye_care_theme = Theme::eye_care();
        assert_eq!(eye_care_theme.name, "Eye Care");

        let blue_theme = Theme::blue();
        assert_eq!(blue_theme.name, "Blue");
    }

    #[test]
    fn test_theme_by_name() {
        assert_eq!(Theme::by_name("dark").name, "Dark");
        assert_eq!(Theme::by_name("light").name, "Light");
        assert_eq!(Theme::by_name("high_contrast").name, "High Contrast");
        assert_eq!(Theme::by_name("eye_care").name, "Eye Care");
        assert_eq!(Theme::by_name("blue").name, "Blue");
        assert_eq!(Theme::by_name("unknown").name, "Default");
    }

    #[test]
    fn test_theme_names() {
        let names = Theme::names();
        assert!(names.contains(&"Default".to_string()));
        assert!(names.contains(&"Dark".to_string()));
        assert!(names.contains(&"Light".to_string()));
        assert!(names.contains(&"High Contrast".to_string()));
        assert!(names.contains(&"Eye Care".to_string()));
        assert!(names.contains(&"Blue".to_string()));
    }

    #[test]
    fn test_theme_styles() {
        let theme = Theme::default();

        // 测试所有样式方法都能正常调用
        let _ = theme.title_style();
        let _ = theme.border_style();
        let _ = theme.selected_style();
        let _ = theme.text_style();
        let _ = theme.success_style();
        let _ = theme.warning_style();
        let _ = theme.error_style();
        let _ = theme.inactive_style();
        let _ = theme.highlight_style();
        let _ = theme.table_header_style();
        let _ = theme.table_row_style();
        let _ = theme.chart_style();
        let _ = theme.input_style();
        let _ = theme.input_focus_style();
    }
}

#[cfg(test)]
mod permission_tests {
    use super::*;

    #[test]
    fn test_permission_status_methods() {
        // 测试 Granted 状态
        let granted = PermissionStatus::Granted;
        assert!(granted.is_available());
        assert!(!granted.needs_user_action());
        assert_eq!(granted.description(), "已授予");
        assert_eq!(granted.icon(), "✅");

        // 测试 Denied 状态
        let denied = PermissionStatus::Denied;
        assert!(!denied.is_available());
        assert!(denied.needs_user_action());
        assert_eq!(denied.description(), "被拒绝");
        assert_eq!(denied.icon(), "❌");

        // 测试 NotDetermined 状态
        let not_determined = PermissionStatus::NotDetermined;
        assert!(!not_determined.is_available());
        assert!(not_determined.needs_user_action());
        assert_eq!(not_determined.description(), "未确定");
        assert_eq!(not_determined.icon(), "⚠️");

        // 测试 Restricted 状态
        let restricted = PermissionStatus::Restricted;
        assert!(!restricted.is_available());
        assert!(restricted.needs_user_action());
        assert_eq!(restricted.description(), "受限制");
        assert_eq!(restricted.icon(), "🚫");

        // 测试 NotRequired 状态
        let not_required = PermissionStatus::NotRequired;
        assert!(not_required.is_available());
        assert!(!not_required.needs_user_action());
        assert_eq!(not_required.description(), "不需要");
        assert_eq!(not_required.icon(), "➖");

        // 测试 Unknown 状态
        let unknown = PermissionStatus::Unknown;
        assert!(!unknown.is_available());
        assert!(!unknown.needs_user_action());
        assert_eq!(unknown.description(), "未知");
        assert_eq!(unknown.icon(), "❓");
    }
}

#[cfg(test)]
mod utility_tests {
    use timetracker::utils::{time::format_duration, validation::*};

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(1), "1s");
        assert_eq!(format_duration(59), "59s");
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(61), "1m 1s");
        assert_eq!(format_duration(3600), "1h 0m 0s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(86400), "24h 0m 0s");
    }

    #[test]
    fn test_validate_interval() {
        assert!(validate_interval(1).is_ok());
        assert!(validate_interval(60).is_ok());
        assert!(validate_interval(3600).is_ok());

        assert!(validate_interval(0).is_err());
        assert!(validate_interval(u64::MAX).is_err());
    }

    #[test]
    fn test_validate_file_path() {
        assert!(validate_file_path("valid_file.txt").is_ok());
        assert!(validate_file_path("./data/file.json").is_ok());
        assert!(validate_file_path("/absolute/path/file.log").is_ok());

        assert!(validate_file_path("").is_err());
        assert!(validate_file_path("file<with>invalid:chars").is_err());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_config_error_messages() {
        let mut config = AppConfig::default();

        // 测试监控间隔错误
        config.monitor_interval = 0;
        match config.validate() {
            Err(e) => assert!(e.to_string().contains("监控间隔")),
            Ok(_) => panic!("应该返回错误"),
        }

        // 测试主题错误
        config = AppConfig::default();
        config.ui.theme = "invalid".to_string();
        match config.validate() {
            Err(e) => assert!(e.to_string().contains("主题")),
            Ok(_) => panic!("应该返回错误"),
        }
    }

    #[test]
    fn test_graceful_degradation() {
        // 测试配置修复的优雅降级
        let mut config = AppConfig::default();

        // 设置多个无效值
        config.monitor_interval = 0;
        config.ui.theme = "invalid".to_string();
        config.ui.refresh_interval = 50;
        config.export.default_format = "invalid".to_string();
        config.logging.level = "invalid".to_string();

        // 执行修复
        let fixes = config.fix();

        // 验证所有问题都被修复
        assert!(!fixes.is_empty());
        assert!(config.validate().is_ok());

        // 验证修复后的值都是合理的默认值
        assert!(config.monitor_interval > 0);
        assert!([
            "default",
            "dark",
            "light",
            "high_contrast",
            "eye_care",
            "blue"
        ]
        .contains(&config.ui.theme.as_str()));
        assert!(config.ui.refresh_interval >= 100);
        assert!(["json", "csv", "xlsx", "toml"].contains(&config.export.default_format.as_str()));
        assert!(["trace", "debug", "info", "warn", "error", "off"]
            .contains(&config.logging.level.as_str()));
    }
}
