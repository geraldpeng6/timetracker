// 活跃度检测功能集成测试

use anyhow::Result;
use std::time::Duration;
use timetracker::{
    config::{
        app::{ActivityDetectionConfig, AppConfig},
        manager::ConfigManager,
    },
    core::{
        activity_detector::{ActivityConfig, ActivityDetector, ActivityStatus},
        enhanced_platform::HybridWindowMonitor,
        monitor::EnhancedWindowMonitor,
    },
};

#[tokio::test]
async fn test_full_activity_detection_integration() -> Result<()> {
    println!("🧪 测试完整的活跃度检测集成");

    // 1. 测试配置系统集成
    println!("  ✓ 测试配置系统集成");
    let config_manager = ConfigManager::new()?;
    let app_config = &config_manager.app_config;

    // 验证活跃度检测配置已正确加载
    assert!(app_config.activity.enabled);
    assert_eq!(app_config.activity.idle_timeout, 300);
    assert_eq!(app_config.activity.check_interval, 1000);
    assert!(!app_config.activity.video_apps.is_empty());
    assert!(!app_config.activity.video_sites.is_empty());

    // 2. 测试配置转换
    println!("  ✓ 测试配置转换");
    let activity_config = app_config.activity.to_activity_config();
    assert_eq!(activity_config.enabled, app_config.activity.enabled);
    assert_eq!(
        activity_config.idle_timeout,
        app_config.activity.idle_timeout
    );
    assert_eq!(activity_config.video_apps, app_config.activity.video_apps);

    // 3. 测试监控器集成
    println!("  ✓ 测试监控器集成");
    let mut monitor = HybridWindowMonitor::with_activity_config(activity_config);

    // 验证活跃度检测器已正确集成
    let detector = monitor.activity_detector();
    assert_eq!(detector.current_status(), &ActivityStatus::Unknown);

    // 4. 测试活跃度检测功能
    println!("  ✓ 测试活跃度检测功能");

    // 强制设置为活跃状态
    monitor.force_active();
    assert_eq!(
        monitor.activity_detector().current_status(),
        &ActivityStatus::Active
    );

    // 测试获取窗口信息（可能返回None，这是正常的）
    let window_result = monitor.get_active_window();
    match window_result {
        Ok(Some(window_info)) => {
            println!(
                "    检测到窗口: {} - {}",
                window_info.app_name, window_info.window_title
            );
        }
        Ok(None) => {
            println!("    未检测到活动窗口（正常情况）");
        }
        Err(e) => {
            println!("    窗口检测错误: {} （可能是权限问题）", e);
        }
    }

    // 5. 测试配置验证和修复
    println!("  ✓ 测试配置验证和修复");
    let mut test_config = ActivityDetectionConfig::default();

    // 设置无效值
    test_config.idle_timeout = 0;
    test_config.check_interval = 50;

    // 验证检测到错误
    assert!(test_config.validate().is_err());

    // 执行修复
    let fixes = test_config.fix();
    assert!(!fixes.is_empty());
    assert!(test_config.validate().is_ok());

    // 6. 测试视频检测逻辑
    println!("  ✓ 测试视频检测逻辑");
    let mut detector = ActivityDetector::with_default_config();
    detector.force_active();

    // 测试视频应用检测
    let test_cases = vec![
        ("VLC", "Movie.mp4", true),
        ("YouTube", "Video Title", true),
        ("Safari", "YouTube - Video", true),
        ("Chrome", "正在播放电影", true),
        ("TextEdit", "Document.txt", false),
        ("Terminal", "bash", false),
    ];

    for (app, window, should_be_video) in test_cases {
        let status = detector.detect_activity(Some(app), Some(window))?;
        if should_be_video {
            // 应该是视频或活跃状态（取决于系统闲置检测）
            assert!(matches!(
                status,
                ActivityStatus::WatchingVideo | ActivityStatus::Active
            ));
        } else {
            // 应该是活跃状态（因为我们强制设置为活跃）
            assert!(matches!(
                status,
                ActivityStatus::Active | ActivityStatus::Unknown
            ));
        }
    }

    println!("✅ 活跃度检测集成测试完成");
    Ok(())
}

#[test]
fn test_activity_detection_config_integration() {
    println!("🧪 测试活跃度检测配置集成");

    // 测试默认应用配置包含活跃度检测
    let app_config = AppConfig::default();
    assert!(app_config.activity.enabled);

    // 测试配置验证
    assert!(app_config.validate().is_ok());

    // 测试配置修复
    let mut broken_config = app_config.clone();
    broken_config.activity.idle_timeout = 0;
    broken_config.monitor_interval = 0;

    let fixes = broken_config.fix();
    assert!(!fixes.is_empty());
    assert!(broken_config.validate().is_ok());

    println!("✅ 活跃度检测配置集成测试完成");
}

#[test]
fn test_activity_status_comprehensive() {
    println!("🧪 测试活跃度状态综合功能");

    // 测试所有状态的方法
    let statuses = vec![
        ActivityStatus::Active,
        ActivityStatus::Idle,
        ActivityStatus::WatchingVideo,
        ActivityStatus::Unknown,
    ];

    for status in statuses {
        // 测试基本方法
        let _ = status.description();
        let _ = status.icon();
        let should_record = status.should_record();

        // 验证记录逻辑
        match status {
            ActivityStatus::Active | ActivityStatus::WatchingVideo => {
                assert!(should_record, "活跃和观看视频状态应该记录");
            }
            ActivityStatus::Idle | ActivityStatus::Unknown => {
                assert!(!should_record, "闲置和未知状态不应该记录");
            }
        }
    }

    println!("✅ 活跃度状态综合测试完成");
}

#[test]
fn test_activity_detector_lifecycle() {
    println!("🧪 测试活跃度检测器生命周期");

    // 创建检测器
    let mut detector = ActivityDetector::with_default_config();

    // 初始状态应该是未知
    assert_eq!(detector.current_status(), &ActivityStatus::Unknown);

    // 强制设置为活跃
    detector.force_active();
    assert_eq!(detector.current_status(), &ActivityStatus::Active);

    // 测试配置更新
    let mut new_config = ActivityConfig::default();
    new_config.idle_timeout = 600;
    new_config.enabled = false;

    detector.update_config(new_config.clone());
    assert_eq!(detector.config(), &new_config);

    // 测试统计信息
    let stats = detector.get_stats();
    assert_eq!(stats.current_status, ActivityStatus::Active);
    assert!(!stats.detection_enabled); // 因为我们禁用了检测
    assert_eq!(stats.idle_timeout, Duration::from_secs(600));

    println!("✅ 活跃度检测器生命周期测试完成");
}

#[test]
fn test_video_detection_edge_cases() {
    println!("🧪 测试视频检测边界情况");

    let mut detector = ActivityDetector::with_default_config();
    detector.force_active();

    // 测试边界情况
    let edge_cases = vec![
        // 空值测试
        (None, None),
        (Some(""), Some("")),
        (Some("App"), None),
        (None, Some("Window")),
        // 大小写测试
        (Some("vlc"), Some("movie.mp4")),
        (Some("VLC"), Some("MOVIE.MP4")),
        (Some("Vlc"), Some("Movie.Mp4")),
        // 特殊字符测试
        (Some("VLC"), Some("电影-2024.mp4")),
        (Some("Safari"), Some("YouTube - 视频标题 [4K]")),
        // 长字符串测试
        (
            Some("VLC Media Player"),
            Some("Very Long Movie Title That Contains Many Words And Special Characters.mp4"),
        ),
    ];

    for (app, window) in edge_cases {
        // 这些调用不应该崩溃
        let result = detector.detect_activity(app, window);
        assert!(result.is_ok(), "检测活跃度不应该失败");

        let status = result.unwrap();
        // 状态应该是有效的枚举值
        assert!(matches!(
            status,
            ActivityStatus::Active
                | ActivityStatus::Idle
                | ActivityStatus::WatchingVideo
                | ActivityStatus::Unknown
        ));
    }

    println!("✅ 视频检测边界情况测试完成");
}

#[tokio::test]
async fn test_monitor_activity_integration() -> Result<()> {
    println!("🧪 测试监控器活跃度集成");

    // 创建带有自定义活跃度配置的监控器
    let mut activity_config = ActivityConfig::default();
    activity_config.idle_timeout = 60; // 1分钟
    activity_config.check_interval = 500; // 0.5秒

    let mut monitor = HybridWindowMonitor::with_activity_config(activity_config);

    // 测试活跃度检测器访问
    let detector = monitor.activity_detector();
    assert_eq!(detector.config().idle_timeout, 60);
    assert_eq!(detector.config().check_interval, 500);

    // 测试强制活跃
    monitor.force_active();
    assert_eq!(
        monitor.activity_detector().current_status(),
        &ActivityStatus::Active
    );

    // 测试配置更新
    let mut new_config = ActivityConfig::default();
    new_config.idle_timeout = 120;
    monitor.update_activity_config(new_config);
    assert_eq!(monitor.activity_detector().config().idle_timeout, 120);

    println!("✅ 监控器活跃度集成测试完成");
    Ok(())
}
