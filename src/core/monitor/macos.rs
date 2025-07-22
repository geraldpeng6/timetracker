// macOS平台窗口监控器实现
// 使用Accessibility API和AppleScript获取窗口信息

use super::*;
use crate::core::platform::correct_app_name;
use anyhow::Result;
use std::process::Command;
use std::time::{Duration, SystemTime};

#[cfg(target_os = "macos")]
use core_foundation::{
    base::{CFTypeRef, TCFType},
    string::{CFString, CFStringRef},
};

/// macOS平台窗口监控器
pub struct MacOSMonitor {
    system: sysinfo::System,
    cache: Option<EnhancedWindowInfo>,
    cache_timestamp: SystemTime,
    cache_duration: Duration,
    accessibility_checked: bool,
    accessibility_available: bool,
    last_error: Option<String>,
    retry_count: u32,
    max_retries: u32,
}

impl MacOSMonitor {
    pub fn new() -> Self {
        Self {
            system: sysinfo::System::new(),
            cache: None,
            cache_timestamp: SystemTime::UNIX_EPOCH,
            cache_duration: Duration::from_millis(200), // 200ms缓存，macOS检测较慢
            accessibility_checked: false,
            accessibility_available: false,
            last_error: None,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// 检查辅助功能权限
    fn check_accessibility_permission(&mut self) -> bool {
        if self.accessibility_checked {
            return self.accessibility_available;
        }

        let output = Command::new("osascript")
            .arg("-e")
            .arg(
                r#"
                tell application "System Events"
                    try
                        set frontApp to first application process whose frontmost is true
                        return "true"
                    on error
                        return "false"
                    end try
                end tell
            "#,
            )
            .output();

        self.accessibility_available = match output {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout);
                result.trim() == "true"
            }
            Err(_) => false,
        };

        self.accessibility_checked = true;
        self.accessibility_available
    }

    /// 使用AppleScript获取窗口信息
    fn get_window_info_applescript(&self) -> Result<EnhancedWindowInfo> {
        let script = r#"
            global frontApp, frontAppName, windowTitle, bundleId, processId
            
            set windowTitle to ""
            set bundleId to ""
            set processId to 0
            set appPath to ""
            
            try
                tell application "System Events"
                    set frontApp to first application process whose frontmost is true
                    set frontAppName to name of frontApp
                    set processId to unix id of frontApp
                    
                    try
                        set bundleId to bundle identifier of frontApp
                    on error
                        set bundleId to frontAppName
                    end try
                    
                    try
                        set appPath to POSIX path of (file of frontApp)
                    on error
                        set appPath to ""
                    end try
                    
                    tell process frontAppName
                        try
                            tell (1st window whose value of attribute "AXMain" is true)
                                set windowTitle to value of attribute "AXTitle"
                            end tell
                        on error
                            try
                                if (count of windows) > 0 then
                                    set windowTitle to name of window 1
                                end if
                            on error
                                set windowTitle to frontAppName
                            end try
                        end try
                    end tell
                end tell
                
                if windowTitle is "" then
                    set windowTitle to frontAppName
                end if
                
                return "{\"app_name\":\"" & frontAppName & "\",\"window_title\":\"" & windowTitle & "\",\"bundle_id\":\"" & bundleId & "\",\"process_id\":" & processId & ",\"app_path\":\"" & appPath & "\"}"
                
            on error errorMessage
                return "{\"error\":\"" & errorMessage & "\"}"
            end try
        "#;

        let output = Command::new("osascript").arg("-e").arg(script).output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("AppleScript execution failed"));
        }

        let result = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&result.trim())?;

        if let Some(error) = json.get("error") {
            return Err(anyhow::anyhow!("AppleScript error: {}", error));
        }

        let app_name = json
            .get("app_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Application")
            .to_string();

        let window_title = json
            .get("window_title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Window")
            .to_string();

        let process_id = json.get("process_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let bundle_id = json
            .get("bundle_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let app_path = json
            .get("app_path")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // 修正应用名称，提取真实的应用名称
        let corrected_app_name = correct_app_name(&app_name, &window_title, process_id);

        // 计算置信度
        let confidence = if !corrected_app_name.is_empty() && !window_title.is_empty() && bundle_id.is_some()
        {
            0.95
        } else if !corrected_app_name.is_empty() && !window_title.is_empty() {
            0.85
        } else if !corrected_app_name.is_empty() {
            0.7
        } else {
            0.5
        };

        Ok(EnhancedWindowInfo {
            app_name: corrected_app_name,
            window_title,
            process_id,
            app_path,
            bundle_id,
            geometry: None, // macOS几何信息需要额外的API调用
            timestamp: SystemTime::now(),
            confidence,
        })
    }

    /// 使用Accessibility API获取窗口信息（备用方法）
    #[cfg(target_os = "macos")]
    #[allow(dead_code)] // 保留作为备用实现
    fn get_window_info_accessibility(&mut self, pid: u32) -> Option<String> {
        use core_foundation::base::kCFAllocatorDefault;
        use core_foundation::string::CFStringCreateWithCString;
        use std::ffi::CString;

        unsafe {
            let ax_app = AXUIElementCreateApplication(pid as i32);
            if ax_app.is_null() {
                return None;
            }

            let focused_window_attr = {
                let c_str = CString::new("AXFocusedWindow").ok()?;
                CFStringCreateWithCString(kCFAllocatorDefault, c_str.as_ptr(), 0x08000100)
            };

            let title_attr = {
                let c_str = CString::new("AXTitle").ok()?;
                CFStringCreateWithCString(kCFAllocatorDefault, c_str.as_ptr(), 0x08000100)
            };

            let mut frontmost_window: CFTypeRef = std::ptr::null();
            let result =
                AXUIElementCopyAttributeValue(ax_app, focused_window_attr, &mut frontmost_window);

            let title = if result == 0 && !frontmost_window.is_null() {
                let mut title_ref: CFTypeRef = std::ptr::null();
                let title_result = AXUIElementCopyAttributeValue(
                    frontmost_window as *const _,
                    title_attr,
                    &mut title_ref,
                );

                if title_result == 0 && !title_ref.is_null() {
                    let cf_string = CFString::wrap_under_create_rule(title_ref as CFStringRef);
                    let title_str = cf_string.to_string();
                    CFRelease(frontmost_window);
                    Some(title_str)
                } else {
                    CFRelease(frontmost_window);
                    None
                }
            } else {
                None
            };

            CFRelease(ax_app as CFTypeRef);
            CFRelease(focused_window_attr as CFTypeRef);
            CFRelease(title_attr as CFTypeRef);

            title
        }
    }

    /// 检查缓存是否有效
    fn is_cache_valid(&self) -> bool {
        self.cache.is_some()
            && self.cache_timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration
    }

    /// 获取进程信息
    #[allow(dead_code)] // 保留作为工具方法
    fn get_process_info(&mut self, pid: u32) -> (String, Option<String>) {
        self.system
            .refresh_processes(sysinfo::ProcessesToUpdate::All);

        if let Some(process) = self.system.process(sysinfo::Pid::from_u32(pid)) {
            let name = process.name().to_string_lossy().to_string();
            let path = process.exe().map(|p| p.to_string_lossy().to_string());
            (name, path)
        } else {
            ("Unknown".to_string(), None)
        }
    }
}

impl EnhancedWindowMonitor for MacOSMonitor {
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        // 检查缓存
        if self.is_cache_valid() {
            return Ok(self.cache.clone());
        }

        // 检查辅助功能权限
        if !self.check_accessibility_permission() {
            return Err(anyhow::anyhow!(
                "Accessibility permission required. Please grant permission in System Preferences > Security & Privacy > Privacy > Accessibility"
            ));
        }

        // 尝试使用AppleScript获取窗口信息
        match self.get_window_info_applescript() {
            Ok(window_info) => {
                // 成功，重置错误状态
                self.retry_count = 0;
                self.last_error = None;

                // 更新缓存
                self.cache = Some(window_info.clone());
                self.cache_timestamp = SystemTime::now();
                Ok(Some(window_info))
            }
            Err(e) => {
                self.retry_count += 1;
                let error_msg = format!(
                    "AppleScript failed (attempt {}/{}): {}",
                    self.retry_count, self.max_retries, e
                );
                self.last_error = Some(error_msg.clone());

                log::warn!("{}", error_msg);

                // 如果重试次数未达到上限，返回错误让上层重试
                if self.retry_count < self.max_retries {
                    return Err(anyhow::anyhow!("{}", error_msg));
                }

                // 达到重试上限，尝试回退方案
                log::info!(
                    "Trying fallback method after {} failed attempts",
                    self.max_retries
                );

                // 回退到基本的进程信息
                self.system.refresh_all();

                // 查找前台进程（简化实现）
                if let Some(process) = self.system.processes().values().find(|p| {
                    let name = p.name().to_string_lossy();
                    // 常见的用户应用
                    name.contains("Safari")
                        || name.contains("Chrome")
                        || name.contains("Code")
                        || name.contains("Terminal")
                        || name.contains("Finder")
                        || name.contains("System Settings")
                }) {
                    let window_info = EnhancedWindowInfo {
                        app_name: process.name().to_string_lossy().to_string(),
                        window_title: "Unknown Window".to_string(),
                        process_id: process.pid().as_u32(),
                        app_path: process.exe().map(|p| p.to_string_lossy().to_string()),
                        bundle_id: None,
                        geometry: None,
                        timestamp: SystemTime::now(),
                        confidence: 0.6, // 较低的置信度
                    };

                    Ok(Some(window_info))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn check_permissions(&self) -> Vec<(String, PermissionStatus)> {
        let accessibility_status = if self.accessibility_available {
            PermissionStatus::Granted
        } else if self.accessibility_checked {
            PermissionStatus::Denied
        } else {
            PermissionStatus::Unknown
        };

        vec![
            ("Accessibility".to_string(), accessibility_status),
            ("AppleScript".to_string(), PermissionStatus::Granted),
        ]
    }

    fn request_permissions(&self) -> Result<()> {
        // 打开系统偏好设置的隐私页面
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()?;

        println!("请在系统偏好设置中授予辅助功能权限，然后重启应用程序。");
        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        let mut capabilities = vec![
            "AppleScript integration".to_string(),
            "Bundle ID detection".to_string(),
            "Process information".to_string(),
        ];

        if self.accessibility_available {
            capabilities.push("Accessibility API".to_string());
        }

        capabilities
    }
}

// Accessibility API 外部函数声明
#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    #[allow(dead_code)] // 保留作为备用API
    fn AXUIElementCreateApplication(pid: i32) -> *const std::ffi::c_void;
    #[allow(dead_code)] // 保留作为备用API
    fn AXUIElementCopyAttributeValue(
        element: *const std::ffi::c_void,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    #[allow(dead_code)] // 保留作为备用API
    fn CFRelease(cf: CFTypeRef);
}

// 非macOS平台的空实现
#[cfg(not(target_os = "macos"))]
impl EnhancedWindowMonitor for MacOSMonitor {
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        Err(anyhow::anyhow!(
            "macOS monitor not available on this platform"
        ))
    }

    fn check_permissions(&self) -> Vec<(String, PermissionStatus)> {
        vec![("Accessibility".to_string(), PermissionStatus::NotRequired)]
    }

    fn request_permissions(&self) -> Result<()> {
        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![]
    }
}
