// 回退窗口监控器实现
// 使用active-win-pos-rs作为跨平台回退方案

use super::*;
use anyhow::Result;
use std::time::{Duration, SystemTime};

/// 回退窗口监控器（使用active-win-pos-rs）
pub struct FallbackMonitor {
    system: sysinfo::System,
    cache: Option<EnhancedWindowInfo>,
    cache_timestamp: SystemTime,
    cache_duration: Duration,
}

impl Default for FallbackMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FallbackMonitor {
    pub fn new() -> Self {
        Self {
            system: sysinfo::System::new(),
            cache: None,
            cache_timestamp: SystemTime::UNIX_EPOCH,
            cache_duration: Duration::from_millis(100),
        }
    }

    /// 检查是否为后台进程
    fn is_background_process(&self, app_name: &str, window_title: &str) -> bool {
        let background_processes = [
            "SafariNotificationAgent",
            "com.apple.SafariPlatformSupport",
            "KekaFinderIntegration",
            "com.apple.Safari.SafeBrowsing.Service",
            "loginwindow",
            "WindowServer",
            "Dock",
            "SystemUIServer",
            "ControlCenter",
            "NotificationCenter",
            "UserEventAgent",
            "cfprefsd",
            "launchd",
            "kernel_task",
            "kextd",
            "mds",
            "mdworker",
            "spotlight",
            "coreaudiod",
            "bluetoothd",
            "WiFiAgent",
            "AirPlayXPCHelper",
            "com.apple.WebKit.Networking",
            "com.apple.WebKit.WebContent",
            "com.apple.WebKit.GPU",
            "nsurlsessiond",
            "trustd",
            "securityd",
            "keychain",
            // Windows后台进程
            "dwm.exe",
            "winlogon.exe",
            "csrss.exe",
            "smss.exe",
            "wininit.exe",
            "services.exe",
            "lsass.exe",
            "svchost.exe",
            "explorer.exe",
            // Linux后台进程
            "systemd",
            "kthreadd",
            "ksoftirqd",
            "migration",
            "rcu_",
            "watchdog",
        ];

        // 检查应用名称
        for bg_process in &background_processes {
            if app_name.to_lowercase().contains(&bg_process.to_lowercase()) {
                return true;
            }
        }

        // 检查窗口标题
        if window_title.is_empty()
            || window_title.len() < 3
            || window_title.to_lowercase().contains("system")
            || window_title.to_lowercase().contains("background")
        {
            return true;
        }

        false
    }

    /// 检查是否为用户应用程序
    fn is_user_application(&mut self, app_name: &str, process_id: u32) -> bool {
        self.system
            .refresh_processes(sysinfo::ProcessesToUpdate::All);

        if let Some(process) = self.system.process(sysinfo::Pid::from_u32(process_id)) {
            let exe_path = process
                .exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            // 平台特定的用户应用路径检查
            #[cfg(target_os = "macos")]
            {
                if exe_path.starts_with("/Applications/")
                    || (exe_path.contains("/Users/") && exe_path.contains("/Applications/"))
                {
                    return true;
                }
            }

            #[cfg(target_os = "windows")]
            {
                if exe_path.contains("Program Files")
                    || exe_path.contains("Program Files (x86)")
                    || exe_path.contains("Users")
                {
                    return true;
                }
            }

            #[cfg(target_os = "linux")]
            {
                if exe_path.starts_with("/usr/bin/")
                    || exe_path.starts_with("/usr/local/bin/")
                    || exe_path.starts_with("/opt/")
                    || exe_path.contains("/home/")
                {
                    return true;
                }
            }

            // 常见的用户应用程序名称
            let user_apps = [
                "Safari",
                "Chrome",
                "Firefox",
                "Edge",
                "Opera",
                "Code",
                "Xcode",
                "Terminal",
                "iTerm",
                "Alacritty",
                "Finder",
                "TextEdit",
                "Preview",
                "Notepad",
                "Mail",
                "Messages",
                "FaceTime",
                "Skype",
                "Music",
                "TV",
                "Photos",
                "VLC",
                "Notes",
                "Reminders",
                "Calendar",
                "Slack",
                "Discord",
                "Zoom",
                "Teams",
                "Telegram",
                "Photoshop",
                "Illustrator",
                "Sketch",
                "GIMP",
                "Word",
                "Excel",
                "PowerPoint",
                "LibreOffice",
                "IntelliJ",
                "PyCharm",
                "WebStorm",
                "Eclipse",
                "Steam",
                "Epic Games",
                "Spotify",
            ];

            for user_app in &user_apps {
                if app_name.to_lowercase().contains(&user_app.to_lowercase()) {
                    return true;
                }
            }

            // 如果进程有可见窗口且不在系统目录，可能是用户应用
            #[cfg(target_os = "macos")]
            {
                if !exe_path.starts_with("/System/")
                    && !exe_path.starts_with("/usr/")
                    && !exe_path.starts_with("/Library/System/")
                {
                    return true;
                }
            }

            #[cfg(target_os = "windows")]
            {
                if !exe_path.to_lowercase().starts_with("c:\\windows\\")
                    && !exe_path.to_lowercase().starts_with("c:\\system")
                {
                    return true;
                }
            }

            #[cfg(target_os = "linux")]
            {
                if !exe_path.starts_with("/usr/lib/")
                    && !exe_path.starts_with("/lib/")
                    && !exe_path.starts_with("/sbin/")
                {
                    return true;
                }
            }
        }

        false
    }

    /// 获取进程信息
    fn get_process_info(&mut self, pid: u32) -> Option<String> {
        self.system
            .refresh_processes(sysinfo::ProcessesToUpdate::All);

        self.system
            .process(sysinfo::Pid::from_u32(pid))
            .and_then(|process| process.exe())
            .map(|path| path.to_string_lossy().to_string())
    }

    /// 检查缓存是否有效
    fn is_cache_valid(&self) -> bool {
        self.cache.is_some()
            && self.cache_timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration
    }
}

impl EnhancedWindowMonitor for FallbackMonitor {
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        // 检查缓存
        if self.is_cache_valid() {
            return Ok(self.cache.clone());
        }

        // 使用 active-win-pos-rs 获取窗口信息
        match active_win_pos_rs::get_active_window() {
            Ok(active_window) => {
                let app_name = if active_window.app_name.is_empty() {
                    "Unknown Application".to_string()
                } else {
                    active_window.app_name.clone()
                };

                let window_title = if active_window.title.is_empty() {
                    "Unknown Window".to_string()
                } else {
                    active_window.title.clone()
                };

                let process_id = active_window.process_id.try_into().unwrap_or(0);

                // 过滤掉后台系统进程和服务
                if self.is_background_process(&app_name, &window_title) {
                    return Ok(None);
                }

                // 验证这是一个真正的用户应用程序
                if !self.is_user_application(&app_name, process_id) {
                    return Ok(None);
                }

                // 获取应用程序路径
                let app_path = self.get_process_info(process_id);

                // 创建窗口几何信息
                let geometry = if active_window.position.x != 0.0 || active_window.position.y != 0.0
                {
                    Some(WindowGeometry {
                        x: active_window.position.x as i32,
                        y: active_window.position.y as i32,
                        width: 800, // active-win-pos-rs 不提供窗口大小信息，使用默认值
                        height: 600,
                    })
                } else {
                    None
                };

                // 计算置信度
                let confidence =
                    if !app_name.is_empty() && !window_title.is_empty() && geometry.is_some() {
                        0.85
                    } else if !app_name.is_empty() && !window_title.is_empty() {
                        0.8
                    } else if !app_name.is_empty() {
                        0.7
                    } else {
                        0.5
                    };

                let window_info = EnhancedWindowInfo {
                    app_name,
                    window_title,
                    process_id,
                    app_path,
                    bundle_id: None,
                    geometry,
                    timestamp: SystemTime::now(),
                    confidence,
                };

                // 更新缓存
                self.cache = Some(window_info.clone());
                self.cache_timestamp = SystemTime::now();

                Ok(Some(window_info))
            }
            Err(e) => {
                log::warn!("active-win-pos-rs failed: {:?}", e);
                Err(anyhow::anyhow!("Failed to get active window: {:?}", e))
            }
        }
    }

    fn check_permissions(&self) -> Vec<(String, PermissionStatus)> {
        let mut permissions = vec![("active-win-pos-rs".to_string(), PermissionStatus::Granted)];

        // 平台特定的权限检查
        #[cfg(target_os = "macos")]
        {
            // macOS可能需要屏幕录制权限来获取窗口标题
            permissions.push(("Screen Recording".to_string(), PermissionStatus::Unknown));
        }

        #[cfg(target_os = "windows")]
        {
            permissions.push(("Windows API".to_string(), PermissionStatus::Granted));
        }

        #[cfg(target_os = "linux")]
        {
            permissions.push(("X11/Wayland".to_string(), PermissionStatus::Unknown));
        }

        permissions
    }

    fn request_permissions(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            println!("macOS用户注意：");
            println!("如果无法获取窗口标题，请在系统偏好设置 > 安全性与隐私 > 隐私 > 屏幕录制中");
            println!("添加此应用程序的权限。");
        }

        #[cfg(target_os = "linux")]
        {
            println!("Linux用户注意：");
            println!("确保运行在X11或Wayland环境中，某些功能可能需要额外的权限。");
        }

        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "Cross-platform support".to_string(),
            "Basic window information".to_string(),
            "Process filtering".to_string(),
            "Window geometry (limited)".to_string(),
        ]
    }

    fn supports_geometry(&self) -> bool {
        true // active-win-pos-rs 提供基本的几何信息
    }
}
