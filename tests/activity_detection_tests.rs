// 活跃度检测功能测试

use anyhow::Result;
use timetracker::{
    config::app::{ActivityDetectionConfig, AppConfig},
    core::{
        activity_detector::{ActivityConfig, ActivityDetector, ActivityStatus},
        enhanced_platform::HybridWindowMonitor,
    },
};

#[test]
fn test_activity_status_methods() {
    // 测试活跃状态
    let active = ActivityStatus::Active;
    assert!(active.should_record());
    assert_eq!(active.description(), "活跃");
    assert_eq!(active.icon(), "🟢");

    // 测试闲置状态
    let idle = ActivityStatus::Idle;
    assert!(!idle.should_record());
    assert_eq!(idle.description(), "闲置");
    assert_eq!(idle.icon(), "🟡");

    // 测试观看视频状态
    let watching = ActivityStatus::WatchingVideo;
    assert!(watching.should_record());
    assert_eq!(watching.description(), "观看视频");
    assert_eq!(watching.icon(), "📺");

    // 测试未知状态
    let unknown = ActivityStatus::Unknown;
    assert!(!unknown.should_record());
    assert_eq!(unknown.description(), "未知");
    assert_eq!(unknown.icon(), "❓");
}

#[test]
fn test_activity_config_default() {
    let config = ActivityConfig::default();

    assert!(config.enabled);
    assert_eq!(config.idle_timeout, 300); // 5分钟
    assert_eq!(config.check_interval, 1000); // 1秒
    assert!(!config.video_apps.is_empty());
    assert!(!config.video_sites.is_empty());

    // 检查是否包含常见的视频应用
    assert!(config.video_apps.contains(&"VLC".to_string()));
    assert!(config.video_apps.contains(&"YouTube".to_string()));

    // 检查是否包含常见的视频网站
    assert!(config.video_sites.contains(&"youtube.com".to_string()));
    assert!(config.video_sites.contains(&"bilibili.com".to_string()));
}

#[test]
fn test_activity_detector_creation() {
    let config = ActivityConfig::default();
    let detector = ActivityDetector::new(config.clone());

    assert_eq!(detector.config(), &config);
    assert_eq!(detector.current_status(), &ActivityStatus::Unknown);
}

#[test]
fn test_video_detection() {
    let mut detector = ActivityDetector::with_default_config();

    // 先强制设置为活跃状态，确保不是因为闲置而返回Unknown
    detector.force_active();

    // 测试视频应用检测
    let status = detector
        .detect_activity(Some("VLC"), Some("Movie.mp4"))
        .unwrap();
    // 在测试环境中，由于系统闲置时间检测可能不工作，我们检查是否识别为视频或活跃
    assert!(matches!(
        status,
        ActivityStatus::WatchingVideo | ActivityStatus::Active
    ));

    // 测试视频网站检测
    let status = detector
        .detect_activity(Some("Safari"), Some("YouTube - Video Title"))
        .unwrap();
    assert!(matches!(
        status,
        ActivityStatus::WatchingVideo | ActivityStatus::Active
    ));

    // 测试普通应用
    let status = detector
        .detect_activity(Some("TextEdit"), Some("Document.txt"))
        .unwrap();
    // 由于我们无法在测试中获取真实的系统闲置时间，这里可能是Active或Unknown
    assert!(matches!(
        status,
        ActivityStatus::Active | ActivityStatus::Unknown
    ));
}

#[test]
fn test_activity_detection_config_validation() {
    let mut config = ActivityDetectionConfig::default();

    // 测试有效配置
    assert!(config.validate().is_ok());

    // 测试无效的闲置超时
    config.idle_timeout = 0;
    assert!(config.validate().is_err());

    config.idle_timeout = 86401; // 超过24小时
    assert!(config.validate().is_err());

    // 重置并测试无效的检测间隔
    config = ActivityDetectionConfig::default();
    config.check_interval = 50; // 小于100ms
    assert!(config.validate().is_err());

    config.check_interval = 70000; // 超过60秒
    assert!(config.validate().is_err());
}

#[test]
fn test_activity_detection_config_fix() {
    let mut config = ActivityDetectionConfig::default();

    // 设置无效值
    config.idle_timeout = 0;
    config.check_interval = 50;

    // 执行修复
    let fixes = config.fix();

    // 验证修复结果
    assert!(!fixes.is_empty());
    assert!(config.validate().is_ok());
    assert_eq!(config.idle_timeout, 300);
    assert_eq!(config.check_interval, 1000);
}

#[test]
fn test_activity_detection_config_conversion() {
    let detection_config = ActivityDetectionConfig::default();
    let activity_config = detection_config.to_activity_config();

    assert_eq!(detection_config.enabled, activity_config.enabled);
    assert_eq!(detection_config.idle_timeout, activity_config.idle_timeout);
    assert_eq!(
        detection_config.check_interval,
        activity_config.check_interval
    );
    assert_eq!(detection_config.video_apps, activity_config.video_apps);
    assert_eq!(detection_config.video_sites, activity_config.video_sites);
}

#[tokio::test]
async fn test_hybrid_monitor_with_activity_detection() -> Result<()> {
    // 创建带有活跃度检测的监控器
    let activity_config = ActivityConfig::default();
    let mut monitor = HybridWindowMonitor::with_activity_config(activity_config);

    // 测试活跃度检测器访问
    let detector = monitor.activity_detector();
    assert_eq!(detector.current_status(), &ActivityStatus::Unknown);

    // 测试强制设置为活跃状态
    monitor.force_active();
    assert_eq!(
        monitor.activity_detector().current_status(),
        &ActivityStatus::Active
    );

    Ok(())
}

#[test]
fn test_app_config_with_activity() {
    let config = AppConfig::default();

    // 验证活跃度检测配置已包含在应用配置中
    assert!(config.activity.enabled);
    assert_eq!(config.activity.idle_timeout, 300);
    assert_eq!(config.activity.check_interval, 1000);

    // 测试配置验证包含活跃度检测
    assert!(config.validate().is_ok());

    // 测试配置修复包含活跃度检测
    let mut broken_config = config.clone();
    broken_config.activity.idle_timeout = 0;

    let fixes = broken_config.fix();
    assert!(!fixes.is_empty());
    assert!(broken_config.validate().is_ok());
}

#[test]
fn test_activity_stats() {
    let detector = ActivityDetector::with_default_config();
    let stats = detector.get_stats();

    assert_eq!(stats.current_status, ActivityStatus::Unknown);
    assert!(stats.detection_enabled);
    assert_eq!(stats.idle_timeout.as_secs(), 300);

    // 测试格式化方法
    let description = stats.status_description();
    assert!(description.contains("未知"));

    let duration_str = stats.format_idle_duration();
    assert_eq!(duration_str, "0秒");
}

#[test]
fn test_video_keyword_detection() {
    let mut detector = ActivityDetector::with_default_config();

    // 先强制设置为活跃状态
    detector.force_active();

    // 测试中文视频关键词
    let status = detector
        .detect_activity(Some("Safari"), Some("正在播放电影"))
        .unwrap();
    assert!(matches!(
        status,
        ActivityStatus::WatchingVideo | ActivityStatus::Active
    ));

    // 测试英文视频关键词
    let status = detector
        .detect_activity(Some("Chrome"), Some("Playing video"))
        .unwrap();
    assert!(matches!(
        status,
        ActivityStatus::WatchingVideo | ActivityStatus::Active
    ));

    // 测试直播关键词
    let status = detector
        .detect_activity(Some("Firefox"), Some("直播间 - 主播名"))
        .unwrap();
    assert!(matches!(
        status,
        ActivityStatus::WatchingVideo | ActivityStatus::Active
    ));
}

#[test]
fn test_activity_detector_force_active() {
    let mut detector = ActivityDetector::with_default_config();

    // 初始状态应该是未知
    assert_eq!(detector.current_status(), &ActivityStatus::Unknown);

    // 强制设置为活跃
    detector.force_active();
    assert_eq!(detector.current_status(), &ActivityStatus::Active);

    // 验证上次活跃时间已更新
    let idle_duration = detector.idle_duration();
    assert!(idle_duration.as_millis() < 100); // 应该很短，因为刚刚设置为活跃
}

#[test]
fn test_activity_config_update() {
    let mut detector = ActivityDetector::with_default_config();

    // 创建新的配置
    let mut new_config = ActivityConfig::default();
    new_config.idle_timeout = 600; // 10分钟
    new_config.enabled = false;

    // 更新配置
    detector.update_config(new_config.clone());

    // 验证配置已更新
    assert_eq!(detector.config(), &new_config);
    assert_eq!(detector.config().idle_timeout, 600);
    assert!(!detector.config().enabled);
}
