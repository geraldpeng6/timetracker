// Linux平台窗口监控器实现
// 使用X11/Wayland协议获取窗口信息

use super::*;
#[cfg(target_os = "linux")]
use crate::core::platform::correct_app_name;
use anyhow::Result;
#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::time::{Duration, SystemTime};

#[cfg(all(target_os = "linux", feature = "x11"))]
use x11rb::{connection::Connection, protocol::xproto::*, rust_connection::RustConnection};

/// Linux平台窗口监控器
pub struct LinuxMonitor {
    #[cfg(target_os = "linux")]
    system: sysinfo::System,
    #[cfg(all(target_os = "linux", feature = "x11"))]
    x11_connection: Option<RustConnection>,
    #[cfg(target_os = "linux")]
    cache: Option<EnhancedWindowInfo>,
    #[cfg(target_os = "linux")]
    cache_timestamp: SystemTime,
    #[cfg(target_os = "linux")]
    cache_duration: Duration,
    #[cfg(target_os = "linux")]
    display_server: DisplayServer,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

impl LinuxMonitor {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        let display_server = Self::detect_display_server();

        #[cfg(all(target_os = "linux", feature = "x11"))]
        let x11_connection = if matches!(display_server, DisplayServer::X11) {
            x11rb::connect(None).ok().map(|(conn, _)| conn)
        } else {
            None
        };

        Self {
            #[cfg(target_os = "linux")]
            system: sysinfo::System::new(),
            #[cfg(all(target_os = "linux", feature = "x11"))]
            x11_connection,
            #[cfg(target_os = "linux")]
            cache: None,
            #[cfg(target_os = "linux")]
            cache_timestamp: SystemTime::UNIX_EPOCH,
            #[cfg(target_os = "linux")]
            cache_duration: Duration::from_millis(150),
            #[cfg(target_os = "linux")]
            display_server,
        }
    }

    /// 检测显示服务器类型
    #[cfg(target_os = "linux")]
    fn detect_display_server() -> DisplayServer {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            DisplayServer::Wayland
        } else if std::env::var("DISPLAY").is_ok() {
            DisplayServer::X11
        } else {
            DisplayServer::Unknown
        }
    }

    /// 使用X11获取活动窗口信息
    #[cfg(all(target_os = "linux", feature = "x11"))]
    fn get_active_window_x11(&self) -> Result<Option<(String, u32, Option<WindowGeometry>)>> {
        let conn = self
            .x11_connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No X11 connection"))?;

        let screen = &conn.setup().roots[0];

        // 获取 _NET_ACTIVE_WINDOW 属性
        let active_window_atom = conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
            .reply()?
            .atom;

        let reply = conn
            .get_property(
                false,
                screen.root,
                active_window_atom,
                AtomEnum::WINDOW,
                0,
                1,
            )?
            .reply()?;

        if reply.value.len() >= 4 {
            let window_id = u32::from_ne_bytes([
                reply.value[0],
                reply.value[1],
                reply.value[2],
                reply.value[3],
            ]);

            // 获取窗口标题
            let title = self.get_window_title_x11(conn, window_id)?;

            // 获取进程ID
            let pid = self.get_window_pid_x11(conn, window_id)?;

            // 获取窗口几何信息
            let geometry = self.get_window_geometry_x11(conn, window_id)?;

            Ok(Some((title, pid, geometry)))
        } else {
            Ok(None)
        }
    }

    #[cfg(all(target_os = "linux", feature = "x11"))]
    fn get_window_title_x11(&self, conn: &RustConnection, window_id: u32) -> Result<String> {
        // 尝试 _NET_WM_NAME (UTF-8)
        let net_wm_name_atom = conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom;
        let utf8_string_atom = conn.intern_atom(false, b"UTF8_STRING")?.reply()?.atom;

        let title_reply = conn
            .get_property(
                false,
                window_id,
                net_wm_name_atom,
                utf8_string_atom,
                0,
                1024,
            )?
            .reply()?;

        if !title_reply.value.is_empty() {
            return Ok(String::from_utf8_lossy(&title_reply.value).to_string());
        }

        // 回退到 WM_NAME
        let wm_name_atom = conn.intern_atom(false, b"WM_NAME")?.reply()?.atom;
        let title_reply = conn
            .get_property(false, window_id, wm_name_atom, AtomEnum::STRING, 0, 1024)?
            .reply()?;

        Ok(String::from_utf8_lossy(&title_reply.value).to_string())
    }

    #[cfg(all(target_os = "linux", feature = "x11"))]
    fn get_window_pid_x11(&self, conn: &RustConnection, window_id: u32) -> Result<u32> {
        let pid_atom = conn.intern_atom(false, b"_NET_WM_PID")?.reply()?.atom;

        let pid_reply = conn
            .get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1)?
            .reply()?;

        if pid_reply.value.len() >= 4 {
            Ok(u32::from_ne_bytes([
                pid_reply.value[0],
                pid_reply.value[1],
                pid_reply.value[2],
                pid_reply.value[3],
            ]))
        } else {
            Ok(0)
        }
    }

    #[cfg(all(target_os = "linux", feature = "x11"))]
    fn get_window_geometry_x11(
        &self,
        conn: &RustConnection,
        window_id: u32,
    ) -> Result<Option<WindowGeometry>> {
        let geometry = conn.get_geometry(window_id)?.reply()?;

        Ok(Some(WindowGeometry {
            x: geometry.x as i32,
            y: geometry.y as i32,
            width: geometry.width as u32,
            height: geometry.height as u32,
        }))
    }

    /// 使用命令行工具获取活动窗口信息（回退方法）
    #[cfg(target_os = "linux")]
    #[allow(dead_code)] // 保留作为备用实现
    fn get_active_window_fallback(&self) -> Result<Option<(String, u32)>> {
        // 尝试使用xdotool
        if let Ok(output) = Command::new("xdotool")
            .args(&["getactivewindow", "getwindowname"])
            .output()
        {
            if output.status.success() {
                let window_title = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // 获取窗口PID
                if let Ok(pid_output) = Command::new("xdotool")
                    .args(&["getactivewindow", "getwindowpid"])
                    .output()
                {
                    if pid_output.status.success() {
                        let process_id = String::from_utf8_lossy(&pid_output.stdout)
                            .trim()
                            .parse::<u32>()
                            .unwrap_or(0);

                        return Ok(Some((window_title, process_id)));
                    }
                }

                return Ok(Some((window_title, 0)));
            }
        }

        // 尝试使用wmctrl
        if let Ok(output) = Command::new("wmctrl").args(&["-a", ":ACTIVE:"]).output() {
            if output.status.success() {
                // wmctrl的输出需要解析
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 4 {
                        let title = parts[4..].join(" ");
                        return Ok(Some((title, 0)));
                    }
                }
            }
        }

        Ok(None)
    }

    /// 获取进程信息
    #[cfg(target_os = "linux")]
    fn get_process_info(&mut self, pid: u32) -> (String, Option<String>) {
        if pid == 0 {
            return ("Unknown".to_string(), None);
        }

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

    /// 检查缓存是否有效
    #[cfg(target_os = "linux")]
    fn is_cache_valid(&self) -> bool {
        self.cache.is_some()
            && self.cache_timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration
    }
}

// Linux平台的实现
#[cfg(target_os = "linux")]
impl EnhancedWindowMonitor for LinuxMonitor {
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        // 检查缓存
        if self.is_cache_valid() {
            return Ok(self.cache.clone());
        }

        let (title, pid, geometry) = match self.display_server {
            DisplayServer::X11 => {
                #[cfg(all(target_os = "linux", feature = "x11"))]
                {
                    if let Some((title, pid, geometry)) = self.get_active_window_x11()? {
                        (title, pid, geometry)
                    } else {
                        // X11失败，尝试命令行工具
                        if let Some((title, pid)) = self.get_active_window_fallback()? {
                            (title, pid, None)
                        } else {
                            return Ok(None);
                        }
                    }
                }
                #[cfg(not(all(target_os = "linux", feature = "x11")))]
                {
                    // 没有X11支持，使用命令行工具
                    if let Some((title, pid)) = self.get_active_window_fallback()? {
                        (title, pid, None)
                    } else {
                        return Ok(None);
                    }
                }
            }
            DisplayServer::Wayland => {
                // Wayland支持有限，使用命令行工具
                if let Some((title, pid)) = self.get_active_window_fallback()? {
                    (title, pid, None)
                } else {
                    return Ok(None);
                }
            }
            DisplayServer::Unknown => {
                return Err(anyhow::anyhow!("Unknown display server"));
            }
        };

        let (app_name, app_path) = self.get_process_info(pid);

        // 修正应用名称，提取真实的应用名称
        let corrected_app_name = correct_app_name(&app_name, &title, pid);

        // 计算置信度
        let confidence = match self.display_server {
            DisplayServer::X11 if geometry.is_some() => 0.9,
            DisplayServer::X11 => 0.85,
            DisplayServer::Wayland => 0.7,
            DisplayServer::Unknown => 0.5,
        };

        let window_info = EnhancedWindowInfo {
            app_name: corrected_app_name,
            window_title: title,
            process_id: pid,
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

    fn check_permissions(&self) -> Vec<(String, PermissionStatus)> {
        let mut permissions = vec![];

        match self.display_server {
            DisplayServer::X11 => {
                permissions.push(("X11 Access".to_string(), PermissionStatus::Granted));
                #[cfg(all(target_os = "linux", feature = "x11"))]
                {
                    if self.x11_connection.is_some() {
                        permissions.push(("X11 Connection".to_string(), PermissionStatus::Granted));
                    } else {
                        permissions.push(("X11 Connection".to_string(), PermissionStatus::Denied));
                    }
                }
            }
            DisplayServer::Wayland => {
                permissions.push(("Wayland Access".to_string(), PermissionStatus::Granted));
            }
            DisplayServer::Unknown => {
                permissions.push(("Display Server".to_string(), PermissionStatus::Unknown));
            }
        }

        // 检查命令行工具
        if Command::new("xdotool").arg("--version").output().is_ok() {
            permissions.push(("xdotool".to_string(), PermissionStatus::Granted));
        } else {
            permissions.push(("xdotool".to_string(), PermissionStatus::Denied));
        }

        permissions
    }

    fn request_permissions(&self) -> Result<()> {
        match self.display_server {
            DisplayServer::X11 => {
                println!("X11 display server detected. Make sure xdotool is installed:");
                println!("  Ubuntu/Debian: sudo apt install xdotool");
                println!("  Fedora: sudo dnf install xdotool");
                println!("  Arch: sudo pacman -S xdotool");
            }
            DisplayServer::Wayland => {
                println!(
                    "Wayland display server detected. Window monitoring capabilities are limited."
                );
                println!("Consider installing xdotool for better compatibility:");
                println!("  Ubuntu/Debian: sudo apt install xdotool");
            }
            DisplayServer::Unknown => {
                println!("Unknown display server. Please ensure X11 or Wayland is running.");
            }
        }
        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        let mut capabilities = vec!["Process information".to_string()];

        match self.display_server {
            DisplayServer::X11 => {
                capabilities.push("X11 protocol".to_string());
                #[cfg(all(target_os = "linux", feature = "x11"))]
                {
                    if self.x11_connection.is_some() {
                        capabilities.push("Native X11 API".to_string());
                        capabilities.push("Window geometry".to_string());
                    }
                }
            }
            DisplayServer::Wayland => {
                capabilities.push("Wayland protocol".to_string());
            }
            DisplayServer::Unknown => {}
        }

        // 检查可用的命令行工具
        if Command::new("xdotool").arg("--version").output().is_ok() {
            capabilities.push("xdotool support".to_string());
        }

        if Command::new("wmctrl").arg("--version").output().is_ok() {
            capabilities.push("wmctrl support".to_string());
        }

        capabilities
    }

    fn supports_geometry(&self) -> bool {
        matches!(self.display_server, DisplayServer::X11)
            && cfg!(all(target_os = "linux", feature = "x11"))
    }
}

// 非Linux平台的空实现
#[cfg(not(target_os = "linux"))]
impl EnhancedWindowMonitor for LinuxMonitor {
    fn get_active_window(&mut self) -> Result<Option<EnhancedWindowInfo>> {
        Err(anyhow::anyhow!(
            "Linux monitor not available on this platform"
        ))
    }

    fn check_permissions(&self) -> Vec<(String, PermissionStatus)> {
        vec![("X11/Wayland".to_string(), PermissionStatus::NotRequired)]
    }

    fn request_permissions(&self) -> Result<()> {
        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![]
    }
}
