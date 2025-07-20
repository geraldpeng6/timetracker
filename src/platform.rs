use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 窗口信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub app_name: String,
    pub window_title: String,
    pub process_id: u32,
}

impl WindowInfo {
    pub fn new(app_name: String, window_title: String, process_id: u32) -> Self {
        Self {
            app_name,
            window_title,
            process_id,
        }
    }
}

// 回退函数，使用平台特定的实现
fn get_active_window_fallback() -> anyhow::Result<WindowInfo> {
    #[cfg(target_os = "windows")]
    {
        let mut monitor = windows::WindowsMonitor::new();
        monitor
            .get_active_window()?
            .ok_or_else(|| anyhow::anyhow!("No active window found"))
    }

    #[cfg(target_os = "macos")]
    {
        let mut monitor = macos::MacOSMonitor::new();
        monitor
            .get_active_window()?
            .ok_or_else(|| anyhow::anyhow!("No active window found"))
    }

    #[cfg(target_os = "linux")]
    {
        let mut monitor = linux::LinuxMonitor::new();
        monitor
            .get_active_window()?
            .ok_or_else(|| anyhow::anyhow!("No active window found"))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}

/// 窗口监控器trait
pub trait WindowMonitor {
    fn get_active_window(&mut self) -> Result<Option<WindowInfo>>;
}

/// 获取活动窗口的统一接口
pub fn get_active_window() -> Result<WindowInfo> {
    // 使用 active-win-pos-rs 库获取更准确的窗口信息
    match active_win_pos_rs::get_active_window() {
        Ok(active_window) => {
            let app_name = if active_window.app_name.is_empty() {
                "Unknown Application".to_string()
            } else {
                active_window.app_name
            };

            let window_title = if active_window.title.is_empty() {
                "Unknown Window".to_string()
            } else {
                active_window.title
            };

            Ok(WindowInfo::new(
                app_name,
                window_title,
                active_window.process_id.try_into().unwrap_or(0),
            ))
        }
        Err(_) => {
            // 如果 active-win-pos-rs 失败，回退到平台特定的实现
            get_active_window_fallback()
        }
    }
}

// Windows平台实现
#[cfg(target_os = "windows")]
mod windows {
    use sysinfo::System;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    pub struct WindowsMonitor {
        system: System,
    }

    impl WindowsMonitor {
        pub fn new() -> Self {
            Self {
                system: System::new_all(),
            }
        }

        fn get_window_text(hwnd: HWND) -> String {
            let mut buffer = [0u16; 512];
            let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };
            if len > 0 {
                String::from_utf16_lossy(&buffer[..len as usize])
            } else {
                String::new()
            }
        }
    }

    impl super::WindowMonitor for WindowsMonitor {
        fn get_active_window(&mut self) -> anyhow::Result<Option<super::WindowInfo>> {
            let hwnd = unsafe { GetForegroundWindow() };
            if hwnd.0 == 0 {
                return Ok(None);
            }

            let window_title = Self::get_window_text(hwnd);

            let mut process_id = 0u32;
            unsafe {
                GetWindowThreadProcessId(hwnd, Some(&mut process_id));
            }

            // 刷新系统信息
            self.system.refresh_all();

            let app_name =
                if let Some(process) = self.system.process(sysinfo::Pid::from_u32(process_id)) {
                    let name = process.name().to_string();
                    // 处理UWP应用
                    if name == "ApplicationFrameHost.exe" && !window_title.is_empty() {
                        window_title.clone()
                    } else {
                        name
                    }
                } else {
                    "Unknown".to_string()
                };

            Ok(Some(super::WindowInfo {
                app_name,
                window_title,
                process_id,
            }))
        }
    }
}

// macOS平台实现
#[cfg(target_os = "macos")]
mod macos {
    use std::process::Command;
    use sysinfo::System;

    pub struct MacOSMonitor {
        system: System,
    }

    impl MacOSMonitor {
        pub fn new() -> Self {
            Self {
                system: System::new_all(),
            }
        }
    }

    impl super::WindowMonitor for MacOSMonitor {
        fn get_active_window(&mut self) -> anyhow::Result<Option<super::WindowInfo>> {
            // 刷新系统信息
            self.system.refresh_all();

            // 使用AppleScript获取前台应用信息
            let app_script = r#"
                tell application "System Events"
                    set frontApp to first application process whose frontmost is true
                    set appName to name of frontApp
                    try
                        set windowTitle to name of front window of frontApp
                    on error
                        set windowTitle to ""
                    end try
                    return appName & "|" & windowTitle
                end tell
            "#;

            let output = Command::new("osascript").arg("-e").arg(app_script).output();

            match output {
                Ok(output) if output.status.success() => {
                    let result_string = String::from_utf8_lossy(&output.stdout);
                    let result = result_string.trim();
                    let parts: Vec<&str> = result.split('|').collect();

                    let app_name = if !parts.is_empty() && !parts[0].is_empty() {
                        parts[0].to_string()
                    } else {
                        "Unknown Application".to_string()
                    };

                    let window_title = if parts.len() > 1 && !parts[1].is_empty() {
                        parts[1].to_string()
                    } else {
                        "Unknown Window".to_string()
                    };

                    // 尝试获取进程ID
                    let pid_script = r#"
                        tell application "System Events"
                            set frontApp to first application process whose frontmost is true
                            return unix id of frontApp
                        end tell
                    "#;

                    let pid_output = Command::new("osascript").arg("-e").arg(pid_script).output();

                    let process_id = match pid_output {
                        Ok(output) if output.status.success() => {
                            String::from_utf8_lossy(&output.stdout)
                                .trim()
                                .parse::<u32>()
                                .unwrap_or(0)
                        }
                        _ => 0,
                    };

                    Ok(Some(super::WindowInfo {
                        app_name,
                        window_title,
                        process_id,
                    }))
                }
                _ => {
                    // 如果AppleScript失败，返回基本信息
                    Ok(Some(super::WindowInfo {
                        app_name: "Unknown Application".to_string(),
                        window_title: "Unknown Window".to_string(),
                        process_id: 0,
                    }))
                }
            }
        }
    }
}

// Linux平台实现
#[cfg(target_os = "linux")]
mod linux {
    use std::process::Command;
    use sysinfo::System;

    pub struct LinuxMonitor {
        system: System,
    }

    impl LinuxMonitor {
        pub fn new() -> Self {
            Self {
                system: System::new_all(),
            }
        }
    }

    impl super::WindowMonitor for LinuxMonitor {
        fn get_active_window(&mut self) -> anyhow::Result<Option<super::WindowInfo>> {
            // 尝试使用xdotool获取活动窗口
            let output = Command::new("xdotool")
                .args(&["getactivewindow", "getwindowname"])
                .output();

            let window_title = match output {
                Ok(output) if output.status.success() => {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                }
                _ => return Ok(None),
            };

            // 获取窗口PID
            let pid_output = Command::new("xdotool")
                .args(&["getactivewindow", "getwindowpid"])
                .output();

            let process_id = match pid_output {
                Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<u32>()
                    .unwrap_or(0),
                _ => 0,
            };

            // 刷新系统信息
            self.system.refresh_all();

            let app_name = if process_id > 0 {
                if let Some(process) = self.system.process(sysinfo::Pid::from_u32(process_id)) {
                    process.name().to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            Ok(Some(super::WindowInfo {
                app_name,
                window_title,
                process_id,
            }))
        }
    }
}

/// 创建平台特定的窗口监控器
#[allow(dead_code)]
pub fn create_monitor() -> Box<dyn WindowMonitor> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsMonitor::new())
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSMonitor::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxMonitor::new())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        compile_error!("Unsupported platform")
    }
}
