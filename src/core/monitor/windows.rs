// Windows平台窗口监控器实现
// 使用Win32 API获取准确的窗口信息

use super::*;
#[cfg(target_os = "windows")]
use crate::core::platform::correct_app_name;
use anyhow::Result;
#[cfg(target_os = "windows")]
use std::time::{Duration, SystemTime};

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{HWND, RECT},
    System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible, GW_OWNER,
    },
};

/// Windows平台窗口监控器
pub struct WindowsMonitor {
    #[cfg(target_os = "windows")]
    system: sysinfo::System,
    #[cfg(target_os = "windows")]
    last_hwnd: Option<isize>,
    #[cfg(target_os = "windows")]
    cache: Option<EnhancedWindowInfo>,
    #[cfg(target_os = "windows")]
    cache_timestamp: SystemTime,
    #[cfg(target_os = "windows")]
    cache_duration: Duration,
}

impl WindowsMonitor {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            system: sysinfo::System::new(),
            #[cfg(target_os = "windows")]
            last_hwnd: None,
            #[cfg(target_os = "windows")]
            cache: None,
            #[cfg(target_os = "windows")]
            cache_timestamp: SystemTime::UNIX_EPOCH,
            #[cfg(target_os = "windows")]
            cache_duration: Duration::from_millis(100), // 100ms缓存
        }
    }

    /// 检查是否为UWP应用
    #[cfg(target_os = "windows")]
    fn is_uwp_app(&self, process_name: &str, window_title: &str) -> bool {
        process_name == "ApplicationFrameHost.exe" && !window_title.is_empty()
    }

    /// 获取进程信息
    #[cfg(target_os = "windows")]
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

    /// 检查窗口是否为主窗口
    #[cfg(target_os = "windows")]
    fn is_main_window(&self, hwnd: HWND) -> bool {
        unsafe {
            // 检查窗口是否可见
            if !IsWindowVisible(hwnd).as_bool() {
                return false;
            }

            // 检查是否有所有者窗口（如果有，通常不是主窗口）
            let owner = GetWindow(hwnd, GW_OWNER);
            owner.0 == 0
        }
    }

    /// 获取窗口几何信息
    #[cfg(target_os = "windows")]
    fn get_window_geometry(&self, hwnd: HWND) -> Option<WindowGeometry> {
        unsafe {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                Some(WindowGeometry {
                    x: rect.left,
                    y: rect.top,
                    width: (rect.right - rect.left) as u32,
                    height: (rect.bottom - rect.top) as u32,
                })
            } else {
                None
            }
        }
    }

    /// 检查缓存是否有效
    #[cfg(target_os = "windows")]
    fn is_cache_valid(&self) -> bool {
        self.cache.is_some()
            && self.cache_timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration
    }
}

#[cfg(target_os = "windows")]
impl EnhancedWindowMonitor for WindowsMonitor {
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        unsafe {
            let hwnd = GetForegroundWindow();

            // 如果窗口句柄没有变化且缓存有效，直接返回缓存
            if Some(hwnd.0) == self.last_hwnd && self.is_cache_valid() {
                return Ok(self.cache.clone());
            }

            if hwnd.0 == 0 {
                self.cache = None;
                return Ok(None);
            }

            // 检查是否为主窗口
            if !self.is_main_window(hwnd) {
                return Ok(None);
            }

            // 获取窗口标题
            let mut buffer = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut buffer);
            let window_title = if len > 0 {
                String::from_utf16_lossy(&buffer[..len as usize])
            } else {
                String::new()
            };

            // 获取进程ID
            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            let (process_name, app_path) = self.get_process_info(process_id);

            // 处理UWP应用
            let app_name = if self.is_uwp_app(&process_name, &window_title) {
                window_title.clone()
            } else {
                process_name
            };

            // 修正应用名称，提取真实的应用名称
            let corrected_app_name = correct_app_name(&app_name, &window_title, process_id);

            // 获取窗口几何信息
            let geometry = self.get_window_geometry(hwnd);

            // 计算置信度
            let confidence = if !corrected_app_name.is_empty() && !window_title.is_empty() {
                0.95
            } else if !corrected_app_name.is_empty() {
                0.8
            } else {
                0.5
            };

            let window_info = EnhancedWindowInfo {
                app_name: corrected_app_name,
                window_title,
                process_id,
                app_path,
                bundle_id: None,
                geometry,
                timestamp: SystemTime::now(),
                confidence,
            };

            // 更新缓存
            self.last_hwnd = Some(hwnd.0);
            self.cache = Some(window_info.clone());
            self.cache_timestamp = SystemTime::now();

            Ok(Some(window_info))
        }
    }

    fn check_permissions(&self) -> Vec<(String, PermissionStatus)> {
        vec![
            ("Windows API".to_string(), PermissionStatus::Granted),
            ("Process Information".to_string(), PermissionStatus::Granted),
        ]
    }

    fn request_permissions(&self) -> Result<()> {
        // Windows通常不需要特殊权限请求
        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "Real-time monitoring".to_string(),
            "Window geometry".to_string(),
            "Process information".to_string(),
            "UWP app support".to_string(),
        ]
    }

    fn supports_geometry(&self) -> bool {
        true
    }
}

// 非Windows平台的空实现
#[cfg(not(target_os = "windows"))]
impl EnhancedWindowMonitor for WindowsMonitor {
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        Err(anyhow::anyhow!(
            "Windows monitor not available on this platform"
        ))
    }

    fn check_permissions(&self) -> Vec<(String, PermissionStatus)> {
        vec![("Windows API".to_string(), PermissionStatus::NotRequired)]
    }

    fn request_permissions(&self) -> Result<()> {
        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![]
    }
}
