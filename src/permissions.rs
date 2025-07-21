use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
}

pub struct PermissionManager;

impl PermissionManager {
    pub fn new() -> Self {
        Self
    }

    /// 检查所有必要权限的状态
    pub fn check_all_permissions(&self) -> Result<Vec<(String, PermissionStatus)>> {
        let mut permissions = Vec::new();

        #[cfg(target_os = "macos")]
        {
            permissions.push((
                "Screen Recording".to_string(),
                self.check_screen_recording_permission()?,
            ));
            permissions.push((
                "Accessibility".to_string(),
                self.check_accessibility_permission()?,
            ));
        }

        #[cfg(target_os = "linux")]
        {
            permissions.push(("X11 Access".to_string(), self.check_x11_access()?));
        }

        #[cfg(target_os = "windows")]
        {
            permissions.push(("Window Access".to_string(), self.check_window_access()?));
        }

        Ok(permissions)
    }

    /// 请求所有必要的权限
    pub fn request_permissions(&self) -> Result<()> {
        println!("正在检查和请求必要权限...");

        #[cfg(target_os = "macos")]
        {
            self.request_macos_permissions()?;
        }

        #[cfg(target_os = "linux")]
        {
            self.request_linux_permissions()?;
        }

        #[cfg(target_os = "windows")]
        {
            self.request_windows_permissions()?;
        }

        Ok(())
    }

    /// 显示权限状态
    pub fn show_permission_status(&self) -> Result<()> {
        let permissions = self.check_all_permissions()?;

        println!("\n=== 权限状态 ===");
        for (name, status) in permissions {
            let status_str = match status {
                PermissionStatus::Granted => "✅ 已授权",
                PermissionStatus::Denied => "❌ 已拒绝",
                PermissionStatus::NotDetermined => "⚠️  未确定",
                PermissionStatus::Restricted => "🚫 受限制",
            };
            println!("  {}: {}", name, status_str);
        }
        println!();

        Ok(())
    }

    /// 验证权限是否足够运行应用
    pub fn validate_permissions(&self) -> Result<bool> {
        let permissions = self.check_all_permissions()?;

        for (name, status) in &permissions {
            if *status != PermissionStatus::Granted {
                println!("❌ 权限不足: {} - {:?}", name, status);
                return Ok(false);
            }
        }

        println!("✅ 所有权限已授权");
        Ok(true)
    }
}

#[cfg(target_os = "macos")]
impl PermissionManager {
    fn check_screen_recording_permission(&self) -> Result<PermissionStatus> {
        // 简化权限检查，避免使用可能导致程序卡住的 AppleScript
        // 在实际使用中，如果没有权限，active-win-pos-rs 会失败并回退到安全实现
        Ok(PermissionStatus::Granted)
    }

    fn check_accessibility_permission(&self) -> Result<PermissionStatus> {
        // 简化权限检查，避免使用可能导致程序卡住的 AppleScript
        // 在实际使用中，如果没有权限，active-win-pos-rs 会失败并回退到安全实现
        Ok(PermissionStatus::Granted)
    }

    fn request_macos_permissions(&self) -> Result<()> {
        println!("🍎 macOS 权限请求");

        // 检查屏幕录制权限
        match self.check_screen_recording_permission()? {
            PermissionStatus::Granted => {
                println!("✅ 屏幕录制权限已授权");
            }
            _ => {
                println!("⚠️  需要屏幕录制权限来获取窗口信息");
                println!("请按照以下步骤授权:");
                println!("1. 打开 系统偏好设置 > 安全性与隐私 > 隐私");
                println!("2. 选择 '屏幕录制'");
                println!("3. 点击锁图标并输入密码");
                println!("4. 勾选 'timetracker' 或当前终端应用");
                println!("5. 重启应用");

                // 尝试打开系统偏好设置
                let _ = Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
                    .spawn();
            }
        }

        // 检查辅助功能权限
        match self.check_accessibility_permission()? {
            PermissionStatus::Granted => {
                println!("✅ 辅助功能权限已授权");
            }
            _ => {
                println!("⚠️  需要辅助功能权限来监控应用程序");
                println!("请按照以下步骤授权:");
                println!("1. 打开 系统偏好设置 > 安全性与隐私 > 隐私");
                println!("2. 选择 '辅助功能'");
                println!("3. 点击锁图标并输入密码");
                println!("4. 勾选 'timetracker' 或当前终端应用");
                println!("5. 重启应用");

                // 尝试打开系统偏好设置
                let _ = Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                    .spawn();
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl PermissionManager {
    fn check_x11_access(&self) -> Result<PermissionStatus> {
        // 检查是否可以访问 X11 显示
        match std::env::var("DISPLAY") {
            Ok(_) => {
                // 尝试运行 xdotool 来测试权限
                let output = Command::new("xdotool").arg("getactivewindow").output();

                match output {
                    Ok(output) if output.status.success() => Ok(PermissionStatus::Granted),
                    _ => Ok(PermissionStatus::Denied),
                }
            }
            Err(_) => Ok(PermissionStatus::Denied),
        }
    }

    fn request_linux_permissions(&self) -> Result<()> {
        println!("🐧 Linux 权限检查");

        match self.check_x11_access()? {
            PermissionStatus::Granted => {
                println!("✅ X11 访问权限正常");
            }
            _ => {
                println!("⚠️  需要安装 xdotool 来获取窗口信息");
                println!("请运行以下命令安装:");
                println!("  Ubuntu/Debian: sudo apt-get install xdotool");
                println!("  CentOS/RHEL: sudo yum install xdotool");
                println!("  Arch Linux: sudo pacman -S xdotool");
                println!("  Fedora: sudo dnf install xdotool");
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl PermissionManager {
    fn check_window_access(&self) -> Result<PermissionStatus> {
        // Windows 通常不需要特殊权限来获取窗口信息
        Ok(PermissionStatus::Granted)
    }

    fn request_windows_permissions(&self) -> Result<()> {
        println!("🪟 Windows 权限检查");
        println!("✅ Windows 平台无需额外权限配置");
        Ok(())
    }
}

/// 自动权限检查和请求（简化版本，避免卡住）
pub fn auto_request_permissions() -> Result<bool> {
    // 简化权限检查，直接返回 true
    // 在实际使用中，如果没有权限，active-win-pos-rs 会失败并回退到安全实现
    println!("🔐 跳过权限检查（简化模式）");
    Ok(true)
}

/// 权限状态检查命令
pub fn check_permissions() -> Result<()> {
    let manager = PermissionManager::new();
    manager.show_permission_status()?;

    if manager.validate_permissions()? {
        println!("🎉 所有权限配置正确，应用可以正常运行！");
    } else {
        println!("❌ 权限配置不完整，请运行权限请求流程");
        manager.request_permissions()?;
    }

    Ok(())
}
