use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 窗口信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub app_name: String,
    pub window_title: String,
    pub process_id: u32,
}

impl WindowInfo {
    pub fn new(app_name: String, window_title: String, process_id: u32) -> Self {
        // 修正应用名称
        let corrected_app_name = correct_app_name(&app_name, &window_title, process_id);

        Self {
            app_name: corrected_app_name,
            window_title,
            process_id,
        }
    }
}

/// 修正应用名称，将通用进程名映射为实际应用名
pub fn correct_app_name(app_name: &str, window_title: &str, process_id: u32) -> String {
    // 首先尝试通过进程路径提取真实应用名称
    if let Some(extracted_name) = identify_app_by_process(process_id) {
        // 如果提取出的名称不是通用进程名，直接返回
        if !is_generic_process_name(&extracted_name) {
            return extracted_name;
        }
    }

    // 如果原始应用名称看起来像路径，尝试提取应用名称
    if app_name.contains('/') || app_name.contains('\\') {
        if let Some(extracted_name) = extract_app_name_from_path(app_name) {
            let cleaned_name = extracted_name
                .replace(".exe", "")
                .replace("-bin", "")
                .replace("_bin", "");

            if !is_generic_process_name(&cleaned_name) {
                return cleaned_name;
            }
        }
    }

    // 创建应用名称映射表
    let mut app_mappings = HashMap::new();

    // Electron应用映射
    app_mappings.insert(
        "electron",
        vec![
            ("Visual Studio Code", "VSCode"),
            ("Code", "VSCode"),
            ("Discord", "Discord"),
            ("Slack", "Slack"),
            ("WhatsApp", "WhatsApp"),
            ("Figma", "Figma"),
            ("Notion", "Notion"),
            ("Obsidian", "Obsidian"),
            ("Spotify", "Spotify"),
        ],
    );

    // 其他通用进程名映射
    app_mappings.insert("stable", vec![("Warp", "Warp")]);

    app_mappings.insert(
        "chrome",
        vec![("Google Chrome", "Chrome"), ("Chromium", "Chromium")],
    );

    app_mappings.insert(
        "firefox",
        vec![("Mozilla Firefox", "Firefox"), ("Firefox", "Firefox")],
    );

    app_mappings.insert(
        "java",
        vec![
            ("IntelliJ IDEA", "IntelliJ IDEA"),
            ("PyCharm", "PyCharm"),
            ("WebStorm", "WebStorm"),
            ("Android Studio", "Android Studio"),
            ("Eclipse", "Eclipse"),
        ],
    );

    // 检查是否需要映射
    let app_lower = app_name.to_lowercase();

    // 首先检查直接映射
    if let Some(mappings) = app_mappings.get(app_lower.as_str()) {
        for (title_pattern, mapped_name) in mappings {
            if window_title.contains(title_pattern) {
                return mapped_name.to_string();
            }
        }
    }

    // 通过窗口标题进行智能识别
    let title_lower = window_title.to_lowercase();

    // VSCode相关
    if (app_lower.contains("electron") || app_lower.contains("code"))
        && (title_lower.contains("visual studio code")
            || title_lower.contains("vscode")
            || window_title.contains(" - Visual Studio Code"))
    {
        return "VSCode".to_string();
    }

    // Warp终端
    if (app_lower == "stable" || app_lower.contains("warp"))
        && (title_lower.contains("warp") || window_title.contains("Warp"))
    {
        return "Warp".to_string();
    }

    // 通过进程路径进行识别
    if let Some(real_name) = identify_app_by_process(process_id) {
        return real_name;
    }

    // 如果没有找到映射，返回原始名称
    app_name.to_string()
}

/// 判断是否是通用进程名（需要进一步识别的进程名）
pub fn is_generic_process_name(name: &str) -> bool {
    let generic_names = [
        "electron", "java", "python", "node", "chrome", "firefox", "stable", "main", "app", "bin",
        "exec", "launcher",
    ];

    let name_lower = name.to_lowercase();
    generic_names
        .iter()
        .any(|&generic| name_lower.contains(generic))
}

/// 从应用路径中提取应用名称
pub fn extract_app_name_from_path(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }

    // macOS应用路径处理
    #[cfg(target_os = "macos")]
    {
        // 匹配 .app 包格式: /Applications/AppName.app/Contents/MacOS/executable
        if let Some(app_match) = path.rfind(".app/") {
            if let Some(start) = path[..app_match].rfind('/') {
                let app_name = &path[start + 1..app_match];
                return Some(app_name.to_string());
            } else {
                // 如果没有找到前面的斜杠，说明整个路径就是应用名
                let app_name = &path[..app_match];
                return Some(app_name.to_string());
            }
        }

        // 处理 /Applications/AppName 格式
        if path.starts_with("/Applications/") {
            let app_part = &path[14..]; // 去掉 "/Applications/"
            if let Some(slash_pos) = app_part.find('/') {
                return Some(app_part[..slash_pos].to_string());
            } else {
                return Some(app_part.to_string());
            }
        }
    }

    // Windows应用路径处理
    #[cfg(target_os = "windows")]
    {
        // 处理 C:\Program Files\AppName\executable.exe 格式
        if let Some(exe_pos) = path.rfind(".exe") {
            let path_without_exe = &path[..exe_pos];
            if let Some(slash_pos) = path_without_exe.rfind('\\') {
                return Some(path_without_exe[slash_pos + 1..].to_string());
            }
        }

        // 处理其他Windows路径格式
        if let Some(slash_pos) = path.rfind('\\') {
            return Some(path[slash_pos + 1..].to_string());
        }
    }

    // Linux应用路径处理
    #[cfg(target_os = "linux")]
    {
        // 处理 /usr/bin/appname 或 /opt/appname/bin/executable 格式
        if let Some(slash_pos) = path.rfind('/') {
            return Some(path[slash_pos + 1..].to_string());
        }
    }

    // 通用处理：如果上述都不匹配，返回最后一个路径组件
    if let Some(slash_pos) = path.rfind('/') {
        Some(path[slash_pos + 1..].to_string())
    } else if let Some(slash_pos) = path.rfind('\\') {
        Some(path[slash_pos + 1..].to_string())
    } else {
        Some(path.to_string())
    }
}

/// 通过进程ID识别真实应用名称
pub fn identify_app_by_process(process_id: u32) -> Option<String> {
    use sysinfo::{Pid, ProcessesToUpdate, System};

    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All);

    if let Some(process) = system.process(Pid::from_u32(process_id)) {
        let exe_path = process
            .exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // 首先尝试从路径中提取应用名称
        if let Some(extracted_name) = extract_app_name_from_path(&exe_path) {
            // 对提取出的名称进行进一步处理，去除常见的后缀
            let cleaned_name = extracted_name
                .replace(".exe", "")
                .replace("-bin", "")
                .replace("_bin", "");

            // 如果提取出的名称不是通用的进程名，直接返回
            if !is_generic_process_name(&cleaned_name) {
                return Some(cleaned_name);
            }
        }

        // macOS应用路径识别
        #[cfg(target_os = "macos")]
        {
            if exe_path.contains("/Visual Studio Code.app/") {
                return Some("VSCode".to_string());
            }
            if exe_path.contains("/Warp.app/") {
                return Some("Warp".to_string());
            }
            if exe_path.contains("/Discord.app/") {
                return Some("Discord".to_string());
            }
            if exe_path.contains("/Slack.app/") {
                return Some("Slack".to_string());
            }
            if exe_path.contains("/Figma.app/") {
                return Some("Figma".to_string());
            }
            if exe_path.contains("/Notion.app/") {
                return Some("Notion".to_string());
            }
            if exe_path.contains("/Obsidian.app/") {
                return Some("Obsidian".to_string());
            }
            if exe_path.contains("/Spotify.app/") {
                return Some("Spotify".to_string());
            }
            if exe_path.contains("/IntelliJ IDEA") {
                return Some("IntelliJ IDEA".to_string());
            }
            if exe_path.contains("/PyCharm") {
                return Some("PyCharm".to_string());
            }
            if exe_path.contains("/WebStorm") {
                return Some("WebStorm".to_string());
            }
        }

        // Windows应用路径识别
        #[cfg(target_os = "windows")]
        {
            if exe_path.to_lowercase().contains("code.exe") {
                return Some("VSCode".to_string());
            }
            if exe_path.to_lowercase().contains("warp.exe") {
                return Some("Warp".to_string());
            }
            if exe_path.to_lowercase().contains("discord.exe") {
                return Some("Discord".to_string());
            }
            if exe_path.to_lowercase().contains("slack.exe") {
                return Some("Slack".to_string());
            }
        }

        // Linux应用路径识别
        #[cfg(target_os = "linux")]
        {
            if exe_path.contains("code") || exe_path.contains("vscode") {
                return Some("VSCode".to_string());
            }
            if exe_path.contains("warp") {
                return Some("Warp".to_string());
            }
            if exe_path.contains("discord") {
                return Some("Discord".to_string());
            }
            if exe_path.contains("slack") {
                return Some("Slack".to_string());
            }
        }
    }

    None
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

/// 获取活动窗口的统一接口（兼容旧接口）
pub fn get_active_window() -> Result<WindowInfo> {
    // 优先使用新的增强监控系统
    use crate::core::enhanced_platform::get_best_monitor;

    let mut monitor = get_best_monitor();
    match monitor.get_active_window() {
        Ok(Some(enhanced_info)) => {
            // 转换为旧的 WindowInfo 格式
            Ok(WindowInfo::new(
                enhanced_info.app_name,
                enhanced_info.window_title,
                enhanced_info.process_id,
            ))
        }
        Ok(None) => {
            // 如果增强监控器没有检测到窗口，回退到原始实现
            get_active_window_fallback()
        }
        Err(_) => {
            // 如果增强监控器失败，回退到原始实现
            get_active_window_fallback()
        }
    }
}

/// 原始的获取活动窗口实现（作为回退方案）
#[allow(dead_code)] // 保留作为备用实现
fn get_active_window_legacy() -> Result<WindowInfo> {
    // 首先尝试使用AppleScript获取更准确的窗口信息（仅限macOS）
    #[cfg(target_os = "macos")]
    {
        if let Ok(window_info) = get_active_window_applescript() {
            // 过滤掉后台系统进程和服务
            if !is_background_process(&window_info.app_name, &window_info.window_title)
                && is_user_application(&window_info.app_name, window_info.process_id)
            {
                return Ok(window_info);
            }
        }
    }

    // 使用 active-win-pos-rs 库获取窗口信息
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
            if is_background_process(&app_name, &window_title) {
                return Err(anyhow::anyhow!("Background process filtered: {}", app_name));
            }

            // 验证这是一个真正的用户应用程序
            if !is_user_application(&app_name, process_id) {
                return Err(anyhow::anyhow!(
                    "Non-user application filtered: {}",
                    app_name
                ));
            }

            // 修正应用名称，提取真实的应用名称
            let corrected_app_name = correct_app_name(&app_name, &window_title, process_id);

            Ok(WindowInfo::new(corrected_app_name, window_title, process_id))
        }
        Err(_) => {
            // 如果 active-win-pos-rs 失败，回退到平台特定的实现
            get_active_window_fallback()
        }
    }
}

/// 使用AppleScript获取活动窗口信息（macOS专用）
#[cfg(target_os = "macos")]
#[allow(dead_code)] // 保留作为备用实现
fn get_active_window_applescript() -> Result<WindowInfo> {
    use std::process::Command;

    // 执行AppleScript获取窗口信息
    let script = r#"
        global frontApp, frontAppName, windowTitle
        
        set windowTitle to ""
        set bundleId to ""
        set processId to 0
        
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
                            set windowTitle to "Unknown Window"
                        end try
                    end try
                end tell
            end tell
            
            if windowTitle is "" then
                set windowTitle to frontAppName
            end if
            
            return "{\"app_name\":\"" & frontAppName & "\",\"window_title\":\"" & windowTitle & "\",\"bundle_id\":\"" & bundleId & "\",\"process_id\":" & processId & "}"
            
        on error errorMessage
            return "{\"error\":\"" & errorMessage & "\"}"
        end try
    "#;

    let output = Command::new("osascript").arg("-e").arg(script).output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("AppleScript execution failed"));
    }

    let result = String::from_utf8(output.stdout)?;
    let json: serde_json::Value = serde_json::from_str(result.trim())?;

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

    Ok(WindowInfo::new(app_name, window_title, process_id))
}

/// 检查是否为后台进程
#[allow(dead_code)] // 保留作为备用实现
fn is_background_process(app_name: &str, window_title: &str) -> bool {
    // 常见的后台系统进程和服务
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
    ];

    // 检查应用名称
    for bg_process in &background_processes {
        if app_name.contains(bg_process) {
            return true;
        }
    }

    // 检查窗口标题是否为空或包含系统相关内容
    if window_title.is_empty()
        || window_title.contains("System Preferences")
        || window_title.contains("Activity Monitor")
        || window_title.len() < 3
    {
        return true;
    }

    false
}

/// 检查是否为用户应用程序
#[allow(dead_code)] // 保留作为备用实现
fn is_user_application(app_name: &str, process_id: u32) -> bool {
    use sysinfo::{Pid, ProcessesToUpdate, System};

    // 创建系统信息实例
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All);

    if let Some(process) = system.process(Pid::from_u32(process_id)) {
        let exe_path = process
            .exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // 检查是否在 /Applications 目录下（用户应用）
        if exe_path.starts_with("/Applications/") {
            return true;
        }

        // 检查是否在用户目录下的应用
        if exe_path.contains("/Users/") && exe_path.contains("/Applications/") {
            return true;
        }

        // 检查是否为常见的用户应用程序
        let user_apps = [
            "Safari",
            "Chrome",
            "Firefox",
            "Edge",
            "Code",
            "Xcode",
            "Terminal",
            "iTerm",
            "Finder",
            "TextEdit",
            "Preview",
            "Mail",
            "Messages",
            "FaceTime",
            "Music",
            "TV",
            "Photos",
            "Notes",
            "Reminders",
            "Calendar",
            "Slack",
            "Discord",
            "Zoom",
            "Teams",
            "Photoshop",
            "Illustrator",
            "Sketch",
            "Word",
            "Excel",
            "PowerPoint",
            "IntelliJ",
            "PyCharm",
            "WebStorm",
        ];

        for user_app in &user_apps {
            if app_name.contains(user_app) {
                return true;
            }
        }

        // 如果进程有可见窗口且不在系统目录，可能是用户应用
        if !exe_path.starts_with("/System/")
            && !exe_path.starts_with("/usr/")
            && !exe_path.starts_with("/Library/")
        {
            return true;
        }
    }

    false
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
    use core_foundation::base::{CFTypeRef, TCFType};
    use core_foundation::string::{CFString, CFStringRef};
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

        /// 获取前台应用信息 - 使用简化的实现
        fn get_frontmost_application(&self) -> Option<(String, u32)> {
            // 暂时使用 sysinfo 获取进程信息作为回退
            // 这是一个简化的实现，后续可以优化
            if let Some(process) = self.system.processes().values().find(|p| {
                let name = p.name().to_string_lossy();
                name.contains("Finder") || name.contains("Safari") || name.contains("Chrome")
            }) {
                return Some((
                    process.name().to_string_lossy().to_string(),
                    process.pid().as_u32(),
                ));
            }
            None
        }

        /// 使用 Accessibility API 获取窗口标题
        fn get_window_title_via_accessibility(&self, pid: u32) -> String {
            use core_foundation::base::kCFAllocatorDefault;
            use core_foundation::string::CFStringCreateWithCString;
            use std::ffi::CString;

            unsafe {
                // 创建 AXUIElementRef 用于指定进程
                let ax_app = AXUIElementCreateApplication(pid as i32);
                if ax_app.is_null() {
                    return "Unknown Window".to_string();
                }

                // 创建属性名称字符串
                let focused_window_attr = {
                    let c_str = CString::new("AXFocusedWindow").unwrap();
                    CFStringCreateWithCString(kCFAllocatorDefault, c_str.as_ptr(), 0x08000100)
                };

                let title_attr = {
                    let c_str = CString::new("AXTitle").unwrap();
                    CFStringCreateWithCString(kCFAllocatorDefault, c_str.as_ptr(), 0x08000100)
                };

                // 获取前台窗口
                let mut frontmost_window: CFTypeRef = std::ptr::null();
                let result = AXUIElementCopyAttributeValue(
                    ax_app,
                    focused_window_attr,
                    &mut frontmost_window,
                );

                if result == 0 && !frontmost_window.is_null() {
                    // 获取窗口标题
                    let mut title: CFTypeRef = std::ptr::null();
                    let title_result = AXUIElementCopyAttributeValue(
                        frontmost_window as *const _,
                        title_attr,
                        &mut title,
                    );

                    if title_result == 0 && !title.is_null() {
                        let cf_string = CFString::wrap_under_create_rule(title as CFStringRef);
                        let title_str = cf_string.to_string();

                        // 清理资源
                        CFRelease(frontmost_window);
                        CFRelease(ax_app as CFTypeRef);
                        CFRelease(focused_window_attr as CFTypeRef);
                        CFRelease(title_attr as CFTypeRef);

                        return if title_str.is_empty() {
                            "Unknown Window".to_string()
                        } else {
                            title_str
                        };
                    }
                    CFRelease(frontmost_window);
                }
                CFRelease(ax_app as CFTypeRef);
                CFRelease(focused_window_attr as CFTypeRef);
                CFRelease(title_attr as CFTypeRef);
            }

            "Unknown Window".to_string()
        }
    }

    impl super::WindowMonitor for MacOSMonitor {
        fn get_active_window(&mut self) -> anyhow::Result<Option<super::WindowInfo>> {
            // 刷新系统信息
            self.system.refresh_all();

            // 获取前台应用信息
            if let Some((app_name, process_id)) = self.get_frontmost_application() {
                // 尝试获取窗口标题
                let window_title = self.get_window_title_via_accessibility(process_id);

                Ok(Some(super::WindowInfo {
                    app_name,
                    window_title,
                    process_id,
                }))
            } else {
                // 如果无法获取前台应用，返回基本信息
                Ok(Some(super::WindowInfo {
                    app_name: "Unknown Application".to_string(),
                    window_title: "Unknown Window".to_string(),
                    process_id: 0,
                }))
            }
        }
    }

    // Accessibility API 外部函数声明
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> *const std::ffi::c_void;
        fn AXUIElementCopyAttributeValue(
            element: *const std::ffi::c_void,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;
        fn CFRelease(cf: CFTypeRef);
        #[allow(dead_code)]
        fn CFStringCreateWithCString(
            alloc: *const std::ffi::c_void,
            cstr: *const std::ffi::c_char,
            encoding: u32,
        ) -> CFStringRef;
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
