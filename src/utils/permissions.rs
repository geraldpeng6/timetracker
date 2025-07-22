use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
    NotRequired,
    Unknown,
}

impl PermissionStatus {
    /// 权限是否可用
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Granted | Self::NotRequired)
    }

    /// 权限是否需要用户操作
    pub fn needs_user_action(&self) -> bool {
        matches!(self, Self::Denied | Self::NotDetermined | Self::Restricted)
    }

    /// 获取状态描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Granted => "已授予",
            Self::Denied => "被拒绝",
            Self::NotDetermined => "未确定",
            Self::Restricted => "受限制",
            Self::NotRequired => "不需要",
            Self::Unknown => "未知",
        }
    }

    /// 获取状态图标
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Granted => "✅",
            Self::Denied => "❌",
            Self::NotDetermined => "⚠️",
            Self::Restricted => "🚫",
            Self::NotRequired => "➖",
            Self::Unknown => "❓",
        }
    }
}

pub struct PermissionManager;

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

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
            let status_str = format!("{} {}", status.icon(), status.description());
            println!("  {}: {}", name, status_str);
        }
        println!();

        Ok(())
    }

    /// 验证权限是否足够运行应用
    pub fn validate_permissions(&self) -> Result<PermissionValidationResult> {
        let permissions = self.check_all_permissions()?;
        let mut missing_permissions = Vec::new();
        let mut available_permissions = Vec::new();
        let mut warnings = Vec::new();

        for (name, status) in permissions {
            match status {
                PermissionStatus::Granted | PermissionStatus::NotRequired => {
                    available_permissions.push((name, status));
                }
                PermissionStatus::Denied | PermissionStatus::Restricted => {
                    missing_permissions.push((name, status));
                }
                PermissionStatus::NotDetermined | PermissionStatus::Unknown => {
                    warnings.push((name, status));
                }
            }
        }

        Ok(PermissionValidationResult {
            available_permissions,
            missing_permissions,
            warnings,
        })
    }

    /// 生成权限报告
    pub fn generate_permission_report(&self) -> Result<String> {
        let validation = self.validate_permissions()?;
        let mut report = String::new();

        report.push_str("=== 权限状态报告 ===\n\n");

        if !validation.available_permissions.is_empty() {
            report.push_str("✅ 可用权限:\n");
            for (name, status) in &validation.available_permissions {
                report.push_str(&format!(
                    "  {} {}: {}\n",
                    status.icon(),
                    name,
                    status.description()
                ));
            }
            report.push('\n');
        }

        if !validation.missing_permissions.is_empty() {
            report.push_str("❌ 缺失权限:\n");
            for (name, status) in &validation.missing_permissions {
                report.push_str(&format!(
                    "  {} {}: {}\n",
                    status.icon(),
                    name,
                    status.description()
                ));
            }
            report.push('\n');
        }

        if !validation.warnings.is_empty() {
            report.push_str("⚠️ 需要注意:\n");
            for (name, status) in &validation.warnings {
                report.push_str(&format!(
                    "  {} {}: {}\n",
                    status.icon(),
                    name,
                    status.description()
                ));
            }
            report.push('\n');
        }

        // 添加总结
        if validation.missing_permissions.is_empty() && validation.warnings.is_empty() {
            report.push_str("🎉 所有权限配置正确，应用可以正常运行！\n");
        } else if validation.missing_permissions.is_empty() {
            report.push_str("⚠️ 基本权限已配置，但建议检查警告项目\n");
        } else {
            report.push_str("❌ 存在权限问题，请按照指导进行配置\n");
        }

        Ok(report)
    }
}

/// 权限验证结果
#[derive(Debug)]
pub struct PermissionValidationResult {
    pub available_permissions: Vec<(String, PermissionStatus)>,
    pub missing_permissions: Vec<(String, PermissionStatus)>,
    pub warnings: Vec<(String, PermissionStatus)>,
}

impl PermissionValidationResult {
    /// 是否所有权限都可用
    pub fn all_available(&self) -> bool {
        self.missing_permissions.is_empty() && self.warnings.is_empty()
    }

    /// 是否有基本权限
    pub fn has_basic_permissions(&self) -> bool {
        !self.available_permissions.is_empty()
    }

    /// 获取需要用户操作的权限
    pub fn permissions_needing_action(&self) -> Vec<(String, PermissionStatus)> {
        let mut result = self.missing_permissions.clone();
        result.extend(self.warnings.clone());
        result
    }
}

#[cfg(target_os = "macos")]
impl PermissionManager {
    fn check_screen_recording_permission(&self) -> Result<PermissionStatus> {
        // 尝试多种方法检查屏幕录制权限

        // 方法1: 检查TCC数据库
        if let Ok(status) = self.check_tcc_permission("kTCCServiceScreenCapture") {
            return Ok(status);
        }

        // 方法2: 尝试简单的屏幕捕获测试
        if let Ok(status) = self.test_screen_capture() {
            return Ok(status);
        }

        // 如果都失败了，返回未知状态
        Ok(PermissionStatus::Unknown)
    }

    fn check_accessibility_permission(&self) -> Result<PermissionStatus> {
        // 尝试多种方法检查辅助功能权限

        // 方法1: 检查TCC数据库
        if let Ok(status) = self.check_tcc_permission("kTCCServiceAccessibility") {
            return Ok(status);
        }

        // 方法2: 尝试简单的AppleScript测试
        if let Ok(status) = self.test_accessibility() {
            return Ok(status);
        }

        // 如果都失败了，返回未知状态
        Ok(PermissionStatus::Unknown)
    }

    /// 检查TCC数据库中的权限状态
    fn check_tcc_permission(&self, service: &str) -> Result<PermissionStatus> {
        let output = Command::new("sqlite3")
            .arg("/Library/Application Support/com.apple.TCC/TCC.db")
            .arg(&format!(
                "SELECT allowed FROM access WHERE service='{}' AND client LIKE '%timetracker%' OR client LIKE '%Terminal%' OR client LIKE '%iTerm%';",
                service
            ))
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let output_str = String::from_utf8_lossy(&result.stdout);
                if output_str.trim() == "1" {
                    Ok(PermissionStatus::Granted)
                } else if output_str.trim() == "0" {
                    Ok(PermissionStatus::Denied)
                } else {
                    Ok(PermissionStatus::NotDetermined)
                }
            }
            _ => Err(anyhow::anyhow!("无法检查TCC数据库")),
        }
    }

    /// 测试屏幕捕获功能
    fn test_screen_capture(&self) -> Result<PermissionStatus> {
        // 尝试使用screencapture命令测试
        let output = Command::new("screencapture")
            .arg("-t")
            .arg("png")
            .arg("-x")
            .arg("/tmp/timetracker_test.png")
            .output();

        // 清理测试文件
        let _ = std::fs::remove_file("/tmp/timetracker_test.png");

        match output {
            Ok(result) if result.status.success() => Ok(PermissionStatus::Granted),
            _ => Ok(PermissionStatus::Denied),
        }
    }

    /// 测试辅助功能权限
    fn test_accessibility(&self) -> Result<PermissionStatus> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of first process")
            .output();

        match output {
            Ok(result) if result.status.success() => Ok(PermissionStatus::Granted),
            Ok(_) => Ok(PermissionStatus::Denied),
            Err(_) => Ok(PermissionStatus::Unknown),
        }
    }

    fn request_macos_permissions(&self) -> Result<()> {
        println!("🍎 macOS 权限配置向导");
        println!("{}", "=".repeat(50));

        let mut needs_restart = false;

        // 检查屏幕录制权限
        println!("\n📺 检查屏幕录制权限...");
        match self.check_screen_recording_permission()? {
            PermissionStatus::Granted => {
                println!("✅ 屏幕录制权限已授权");
            }
            status => {
                println!("❌ 屏幕录制权限状态: {}", status.description());
                self.show_screen_recording_guide()?;
                needs_restart = true;
            }
        }

        // 检查辅助功能权限
        println!("\n🔧 检查辅助功能权限...");
        match self.check_accessibility_permission()? {
            PermissionStatus::Granted => {
                println!("✅ 辅助功能权限已授权");
            }
            status => {
                println!("❌ 辅助功能权限状态: {}", status.description());
                self.show_accessibility_guide()?;
                needs_restart = true;
            }
        }

        if needs_restart {
            println!("\n🔄 重要提示:");
            println!("权限更改后，请重启 TimeTracker 应用以使更改生效。");
            println!("您可以运行 'timetracker permissions check' 来验证权限状态。");
        } else {
            println!("\n🎉 所有权限已正确配置！");
        }

        Ok(())
    }

    /// 显示屏幕录制权限配置指南
    fn show_screen_recording_guide(&self) -> Result<()> {
        println!("\n📋 屏幕录制权限配置步骤:");
        println!("1. 点击下方链接或手动打开 系统偏好设置");
        println!("2. 导航到 安全性与隐私 > 隐私 > 屏幕录制");
        println!("3. 点击左下角的锁图标并输入管理员密码");
        println!("4. 找到并勾选您的终端应用 (Terminal, iTerm2, 或 timetracker)");
        println!("5. 如果没有看到应用，点击 '+' 按钮手动添加");

        // 尝试打开系统偏好设置
        match Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn()
        {
            Ok(_) => println!("\n🔗 正在打开系统偏好设置..."),
            Err(_) => {
                println!("\n⚠️ 无法自动打开系统偏好设置，请手动打开:");
                println!("   系统偏好设置 > 安全性与隐私 > 隐私 > 屏幕录制");
            }
        }

        Ok(())
    }

    /// 显示辅助功能权限配置指南
    fn show_accessibility_guide(&self) -> Result<()> {
        println!("\n📋 辅助功能权限配置步骤:");
        println!("1. 点击下方链接或手动打开 系统偏好设置");
        println!("2. 导航到 安全性与隐私 > 隐私 > 辅助功能");
        println!("3. 点击左下角的锁图标并输入管理员密码");
        println!("4. 找到并勾选您的终端应用 (Terminal, iTerm2, 或 timetracker)");
        println!("5. 如果没有看到应用，点击 '+' 按钮手动添加");

        // 尝试打开系统偏好设置
        match Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
        {
            Ok(_) => println!("\n🔗 正在打开系统偏好设置..."),
            Err(_) => {
                println!("\n⚠️ 无法自动打开系统偏好设置，请手动打开:");
                println!("   系统偏好设置 > 安全性与隐私 > 隐私 > 辅助功能");
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

    // 显示详细的权限报告
    let report = manager.generate_permission_report()?;
    println!("{}", report);

    // 获取验证结果
    let validation = manager.validate_permissions()?;

    if validation.all_available() {
        println!("🎉 所有权限配置正确，应用可以正常运行！");
    } else if validation.has_basic_permissions() {
        println!("⚠️ 基本权限已配置，但建议完善所有权限");
        if !validation.permissions_needing_action().is_empty() {
            println!("运行 'timetracker permissions request' 来配置缺失的权限");
        }
    } else {
        println!("❌ 权限配置不完整，请运行权限请求流程");
        manager.request_permissions()?;
    }

    Ok(())
}
