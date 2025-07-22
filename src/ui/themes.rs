// UI 主题配置
// 提供不同的颜色主题和样式

use ratatui::style::{Color, Modifier, Style};

/// 主题配置
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub border: Color,
    pub selected: Color,
    pub inactive: Color,
}

impl Theme {
    /// 默认主题
    pub fn default() -> Self {
        Self {
            name: "Default".to_string(),
            background: Color::Reset,
            foreground: Color::White,
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            border: Color::Gray,
            selected: Color::LightBlue,
            inactive: Color::DarkGray,
        }
    }

    /// 暗色主题
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Rgb(100, 149, 237),  // 蓝色
            secondary: Color::Rgb(72, 209, 204), // 青色
            accent: Color::Rgb(255, 105, 180),   // 粉色
            success: Color::Rgb(144, 238, 144),  // 浅绿色
            warning: Color::Rgb(255, 215, 0),    // 金色
            error: Color::Rgb(255, 99, 71),      // 番茄红
            border: Color::Rgb(105, 105, 105),   // 暗灰色
            selected: Color::Rgb(70, 130, 180),  // 钢蓝色
            inactive: Color::Rgb(169, 169, 169), // 暗灰色
        }
    }

    /// 浅色主题
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            background: Color::White,
            foreground: Color::Black,
            primary: Color::Rgb(25, 25, 112),    // 深蓝色
            secondary: Color::Rgb(0, 139, 139),  // 深青色
            accent: Color::Rgb(199, 21, 133),    // 深粉色
            success: Color::Rgb(34, 139, 34),    // 森林绿
            warning: Color::Rgb(255, 140, 0),    // 深橙色
            error: Color::Rgb(220, 20, 60),      // 深红色
            border: Color::Rgb(169, 169, 169),   // 暗灰色
            selected: Color::Rgb(135, 206, 235), // 天蓝色
            inactive: Color::Rgb(211, 211, 211), // 浅灰色
        }
    }

    /// 高对比度主题
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Rgb(255, 255, 0),   // 亮黄色
            secondary: Color::Rgb(0, 255, 255), // 亮青色
            accent: Color::Rgb(255, 0, 255),    // 亮紫色
            success: Color::Rgb(0, 255, 0),     // 亮绿色
            warning: Color::Rgb(255, 165, 0),   // 橙色
            error: Color::Rgb(255, 0, 0),       // 亮红色
            border: Color::White,
            selected: Color::Rgb(255, 255, 0),   // 亮黄色
            inactive: Color::Rgb(128, 128, 128), // 中灰色
        }
    }

    /// 护眼主题（暖色调）
    pub fn eye_care() -> Self {
        Self {
            name: "Eye Care".to_string(),
            background: Color::Rgb(32, 32, 24),    // 暖黑色
            foreground: Color::Rgb(255, 248, 220), // 米色
            primary: Color::Rgb(255, 215, 0),      // 金色
            secondary: Color::Rgb(255, 165, 0),    // 橙色
            accent: Color::Rgb(255, 140, 0),       // 深橙色
            success: Color::Rgb(154, 205, 50),     // 黄绿色
            warning: Color::Rgb(255, 215, 0),      // 金色
            error: Color::Rgb(255, 99, 71),        // 番茄红
            border: Color::Rgb(139, 69, 19),       // 棕色
            selected: Color::Rgb(255, 215, 0),     // 金色
            inactive: Color::Rgb(105, 105, 105),   // 暗灰色
        }
    }

    /// 蓝色主题
    pub fn blue() -> Self {
        Self {
            name: "Blue".to_string(),
            background: Color::Rgb(15, 23, 42),    // 深蓝色背景
            foreground: Color::Rgb(226, 232, 240), // 浅灰色文字
            primary: Color::Rgb(59, 130, 246),     // 蓝色
            secondary: Color::Rgb(14, 165, 233),   // 天蓝色
            accent: Color::Rgb(168, 85, 247),      // 紫色
            success: Color::Rgb(34, 197, 94),      // 绿色
            warning: Color::Rgb(251, 191, 36),     // 黄色
            error: Color::Rgb(239, 68, 68),        // 红色
            border: Color::Rgb(71, 85, 105),       // 蓝灰色
            selected: Color::Rgb(59, 130, 246),    // 蓝色
            inactive: Color::Rgb(100, 116, 139),   // 灰蓝色
        }
    }

    /// 获取所有可用主题
    pub fn all() -> Vec<Self> {
        vec![
            Self::default(),
            Self::dark(),
            Self::light(),
            Self::high_contrast(),
            Self::eye_care(),
            Self::blue(),
        ]
    }

    /// 根据名称获取主题
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dark" => Self::dark(),
            "light" => Self::light(),
            "high_contrast" | "high-contrast" => Self::high_contrast(),
            "eye_care" | "eye-care" => Self::eye_care(),
            "blue" => Self::blue(),
            _ => Self::default(),
        }
    }

    /// 获取主题名称列表
    pub fn names() -> Vec<String> {
        Self::all().into_iter().map(|t| t.name).collect()
    }

    /// 获取标题样式
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// 获取边框样式
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// 获取选中项样式
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.background)
            .bg(self.selected)
            .add_modifier(Modifier::BOLD)
    }

    /// 获取普通文本样式
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    /// 获取成功样式
    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// 获取警告样式
    pub fn warning_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    /// 获取错误样式
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    /// 获取非活跃样式
    pub fn inactive_style(&self) -> Style {
        Style::default().fg(self.inactive)
    }

    /// 高亮样式
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// 表格头部样式
    pub fn table_header_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .bg(self.background)
            .add_modifier(Modifier::BOLD)
    }

    /// 表格行样式
    pub fn table_row_style(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    /// 图表样式
    pub fn chart_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    /// 输入框样式
    pub fn input_style(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }

    /// 输入框焦点样式
    pub fn input_focus_style(&self) -> Style {
        Style::default().fg(self.background).bg(self.primary)
    }
}
