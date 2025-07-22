// 集成测试
// 测试各个模块的集成功能

use anyhow::Result;
use timetracker::{
    config::{app::AppConfig, manager::ConfigManager},
    core::{enhanced_platform::HybridWindowMonitor, monitor::EnhancedWindowMonitor},
    ui::themes::Theme,
    utils::permissions::PermissionManager,
};

#[tokio::test]
async fn test_config_system() -> Result<()> {
    // 测试配置系统
    println!("测试配置系统...");

    // 测试默认配置
    let default_config = AppConfig::default();
    assert_eq!(default_config.monitor_interval, 1);
    assert_eq!(default_config.daemon_mode, true);
    assert_eq!(default_config.ui.theme, "default");

    // 测试配置验证
    assert!(default_config.validate().is_ok());

    // 测试配置修复
    let mut broken_config = default_config.clone();
    broken_config.monitor_interval = 0; // 无效值
    broken_config.ui.theme = "invalid_theme".to_string(); // 无效主题

    let fixes = broken_config.fix();
    assert!(!fixes.is_empty());
    assert!(broken_config.validate().is_ok());

    println!("✓ 配置系统测试通过");
    Ok(())
}

#[tokio::test]
async fn test_monitoring_system() -> Result<()> {
    // 测试监控系统
    println!("测试监控系统...");

    // 创建混合监控器
    let mut monitor = HybridWindowMonitor::new();

    // 测试权限检查
    let permissions = monitor.check_permissions();
    assert!(!permissions.is_empty());

    // 测试监控器状态
    let status = monitor.get_status();
    assert!(matches!(
        status.monitor_type,
        timetracker::core::monitor::MonitorType::MacOS
    ));

    // 测试窗口获取（可能失败，但不应该崩溃）
    match monitor.get_active_window() {
        Ok(Some(window_info)) => {
            assert!(!window_info.app_name.is_empty());
            assert!(window_info.confidence >= 0.0 && window_info.confidence <= 1.0);
            println!("✓ 成功获取窗口信息: {}", window_info.app_name);
        }
        Ok(None) => {
            println!("✓ 未检测到活动窗口（正常）");
        }
        Err(e) => {
            println!("⚠ 窗口获取失败（可能是权限问题）: {}", e);
        }
    }

    println!("✓ 监控系统测试通过");
    Ok(())
}

#[test]
fn test_theme_system() {
    // 测试主题系统
    println!("测试主题系统...");

    // 测试所有主题
    let themes = Theme::all();
    assert!(!themes.is_empty());

    for theme in &themes {
        // 测试主题样式方法
        let _ = theme.title_style();
        let _ = theme.border_style();
        let _ = theme.selected_style();
        let _ = theme.text_style();
        let _ = theme.success_style();
        let _ = theme.warning_style();
        let _ = theme.error_style();
        let _ = theme.inactive_style();
    }

    // 测试主题名称获取
    let names = Theme::names();
    assert!(names.contains(&"Default".to_string()));
    assert!(names.contains(&"Dark".to_string()));
    assert!(names.contains(&"Light".to_string()));

    // 测试按名称获取主题
    let dark_theme = Theme::by_name("dark");
    assert_eq!(dark_theme.name, "Dark");

    let unknown_theme = Theme::by_name("unknown");
    assert_eq!(unknown_theme.name, "Default");

    println!("✓ 主题系统测试通过");
}

#[test]
fn test_permission_system() {
    // 测试权限系统
    println!("测试权限系统...");

    let manager = PermissionManager::new();

    // 测试权限检查
    match manager.check_all_permissions() {
        Ok(permissions) => {
            assert!(!permissions.is_empty());

            for (name, status) in &permissions {
                assert!(!name.is_empty());

                // 测试权限状态方法
                let _ = status.is_available();
                let _ = status.needs_user_action();
                let _ = status.description();
                let _ = status.icon();
            }

            println!("✓ 权限检查完成，发现 {} 个权限项", permissions.len());
        }
        Err(e) => {
            println!("⚠ 权限检查失败: {}", e);
        }
    }

    // 测试权限验证
    match manager.validate_permissions() {
        Ok(validation) => {
            println!("✓ 权限验证完成");
            println!("  - 可用权限: {}", validation.available_permissions.len());
            println!("  - 缺失权限: {}", validation.missing_permissions.len());
            println!("  - 警告项目: {}", validation.warnings.len());

            // 测试验证结果方法
            let _ = validation.all_available();
            let _ = validation.has_basic_permissions();
            let _ = validation.permissions_needing_action();
        }
        Err(e) => {
            println!("⚠ 权限验证失败: {}", e);
        }
    }

    // 测试权限报告生成
    match manager.generate_permission_report() {
        Ok(report) => {
            assert!(!report.is_empty());
            println!("✓ 权限报告生成成功");
        }
        Err(e) => {
            println!("⚠ 权限报告生成失败: {}", e);
        }
    }

    println!("✓ 权限系统测试通过");
}

#[tokio::test]
async fn test_config_manager() -> Result<()> {
    // 测试配置管理器
    println!("测试配置管理器...");

    // 创建配置管理器
    let manager = ConfigManager::new()?;

    // 测试配置验证
    match manager.validate() {
        Ok(_) => println!("✓ 配置验证通过"),
        Err(e) => println!("⚠ 配置验证失败: {}", e),
    }

    // 测试健康检查
    let health_issues = manager.health_check()?;
    if health_issues.is_empty() {
        println!("✓ 配置健康检查通过");
    } else {
        println!("⚠ 发现配置问题:");
        for issue in &health_issues {
            println!("  - {}", issue);
        }
    }

    // 测试配置摘要
    let summary = manager.summary();
    assert!(!summary.is_empty());
    println!("✓ 配置摘要生成成功");

    println!("✓ 配置管理器测试通过");
    Ok(())
}

#[test]
fn test_utility_functions() {
    // 测试工具函数
    println!("测试工具函数...");

    // 测试时间格式化
    use timetracker::utils::time::format_duration;

    assert_eq!(format_duration(0), "0s");
    assert_eq!(format_duration(59), "59s");
    assert_eq!(format_duration(60), "1m 0s");
    assert_eq!(format_duration(3661), "1h 1m 1s");

    // 测试间隔验证
    use timetracker::utils::validation::validate_interval;

    assert!(validate_interval(1).is_ok());
    assert!(validate_interval(3600).is_ok());
    assert!(validate_interval(0).is_err());

    println!("✓ 工具函数测试通过");
}

#[test]
fn test_error_handling() {
    // 测试错误处理
    println!("测试错误处理...");

    // 测试配置错误处理
    let mut config = AppConfig::default();
    config.monitor_interval = 0; // 无效值

    match config.validate() {
        Ok(_) => panic!("应该返回错误"),
        Err(e) => {
            assert!(e.to_string().contains("监控间隔"));
            println!("✓ 配置错误正确捕获: {}", e);
        }
    }

    println!("✓ 错误处理测试通过");
}

// 性能测试
#[tokio::test]
async fn test_performance() -> Result<()> {
    println!("测试性能...");

    use std::time::Instant;

    // 测试配置加载性能
    let start = Instant::now();
    let _config = AppConfig::default();
    let config_time = start.elapsed();
    assert!(
        config_time.as_millis() < 100,
        "配置创建耗时过长: {:?}",
        config_time
    );

    // 测试主题创建性能
    let start = Instant::now();
    let _themes = Theme::all();
    let theme_time = start.elapsed();
    assert!(
        theme_time.as_millis() < 50,
        "主题创建耗时过长: {:?}",
        theme_time
    );

    println!("✓ 性能测试通过");
    println!("  - 配置创建: {:?}", config_time);
    println!("  - 主题创建: {:?}", theme_time);

    Ok(())
}
