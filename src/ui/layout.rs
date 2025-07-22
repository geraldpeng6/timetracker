use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 屏幕尺寸类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenSize {
    /// 小屏幕 (宽度 < 80, 高度 < 24)
    Small,
    /// 中等屏幕 (宽度 < 120, 高度 < 40)
    Medium,
    /// 大屏幕 (宽度 >= 120, 高度 >= 40)
    Large,
}

impl ScreenSize {
    /// 根据终端尺寸确定屏幕大小
    pub fn from_rect(rect: Rect) -> Self {
        match (rect.width, rect.height) {
            (w, h) if w < 80 || h < 24 => ScreenSize::Small,
            (w, h) if w < 120 || h < 40 => ScreenSize::Medium,
            _ => ScreenSize::Large,
        }
    }

    /// 是否为小屏幕
    pub fn is_small(self) -> bool {
        matches!(self, ScreenSize::Small)
    }

    /// 是否为中等屏幕
    pub fn is_medium(self) -> bool {
        matches!(self, ScreenSize::Medium)
    }

    /// 是否为大屏幕
    pub fn is_large(self) -> bool {
        matches!(self, ScreenSize::Large)
    }
}

/// 响应式布局管理器
pub struct ResponsiveLayout;

impl ResponsiveLayout {
    /// 获取主布局约束
    pub fn main_constraints(screen_size: ScreenSize) -> Vec<Constraint> {
        match screen_size {
            ScreenSize::Small => vec![
                Constraint::Length(3), // 标签页
                Constraint::Min(0),    // 主要内容
                Constraint::Length(2), // 简化的帮助信息
            ],
            ScreenSize::Medium => vec![
                Constraint::Length(3), // 标签页
                Constraint::Min(0),    // 主要内容
                Constraint::Length(3), // 帮助信息
            ],
            ScreenSize::Large => vec![
                Constraint::Length(3), // 标签页
                Constraint::Min(0),    // 主要内容
                Constraint::Length(4), // 详细的帮助信息
            ],
        }
    }

    /// 获取概览页面布局约束
    pub fn overview_constraints(screen_size: ScreenSize) -> Vec<Constraint> {
        match screen_size {
            ScreenSize::Small => vec![
                Constraint::Length(3), // 时间头部
                Constraint::Length(6), // 简化的图表
                Constraint::Min(0),    // 应用列表
            ],
            ScreenSize::Medium => vec![
                Constraint::Length(4), // 时间头部
                Constraint::Length(8), // 周视图图表
                Constraint::Length(8), // 24小时图表和分类统计
                Constraint::Min(0),    // 应用列表
            ],
            ScreenSize::Large => vec![
                Constraint::Length(4),  // 时间头部
                Constraint::Length(10), // 周视图图表
                Constraint::Length(12), // 24小时图表和分类统计
                Constraint::Length(8),  // 扇形图区域
                Constraint::Min(0),     // 应用列表
            ],
        }
    }

    /// 获取图表区域的水平分割约束
    pub fn chart_horizontal_constraints(screen_size: ScreenSize) -> Vec<Constraint> {
        match screen_size {
            ScreenSize::Small => vec![
                Constraint::Percentage(100), // 只显示一个图表
            ],
            ScreenSize::Medium => vec![
                Constraint::Percentage(65), // 24小时图表
                Constraint::Percentage(35), // 分类统计
            ],
            ScreenSize::Large => vec![
                Constraint::Percentage(60), // 24小时图表
                Constraint::Percentage(40), // 分类统计
            ],
        }
    }

    /// 获取活动表格的列约束
    pub fn activity_table_constraints(screen_size: ScreenSize) -> Vec<Constraint> {
        match screen_size {
            ScreenSize::Small => vec![
                Constraint::Percentage(40), // 应用名称
                Constraint::Percentage(35), // 窗口标题（截断）
                Constraint::Percentage(25), // 时长
            ],
            ScreenSize::Medium => vec![
                Constraint::Percentage(20), // 应用名称
                Constraint::Percentage(35), // 窗口标题
                Constraint::Percentage(15), // 时长
                Constraint::Percentage(10), // 活动次数
                Constraint::Percentage(20), // 最后活动
            ],
            ScreenSize::Large => vec![
                Constraint::Percentage(15), // 应用名称
                Constraint::Percentage(30), // 窗口标题
                Constraint::Percentage(12), // 时长
                Constraint::Percentage(10), // 活动次数
                Constraint::Percentage(13), // 最后活动
                Constraint::Percentage(10), // 状态
                Constraint::Percentage(10), // 类型
            ],
        }
    }

    /// 获取应用表格的列约束
    pub fn app_table_constraints(screen_size: ScreenSize) -> Vec<Constraint> {
        match screen_size {
            ScreenSize::Small => vec![
                Constraint::Percentage(60), // App名称
                Constraint::Percentage(40), // 时间
            ],
            ScreenSize::Medium => vec![
                Constraint::Percentage(50), // App名称
                Constraint::Percentage(30), // 时间
                Constraint::Percentage(20), // 限额
            ],
            ScreenSize::Large => vec![
                Constraint::Percentage(50), // App名称
                Constraint::Percentage(30), // 时间
                Constraint::Percentage(20), // 限额
            ],
        }
    }

    /// 获取柱状图的柱宽
    pub fn bar_chart_width(screen_size: ScreenSize) -> u16 {
        match screen_size {
            ScreenSize::Small => 4,
            ScreenSize::Medium => 6,
            ScreenSize::Large => 8,
        }
    }

    /// 获取表格的每页项目数
    pub fn items_per_page(screen_size: ScreenSize) -> usize {
        match screen_size {
            ScreenSize::Small => 10,
            ScreenSize::Medium => 15,
            ScreenSize::Large => 20,
        }
    }

    /// 创建自适应布局
    pub fn create_layout(direction: Direction, constraints: Vec<Constraint>) -> Layout {
        Layout::default()
            .direction(direction)
            .constraints(constraints)
    }

    /// 截断文本以适应屏幕
    pub fn truncate_text(text: &str, max_length: usize, screen_size: ScreenSize) -> String {
        let adjusted_length = match screen_size {
            ScreenSize::Small => max_length.saturating_sub(5),
            ScreenSize::Medium => max_length,
            ScreenSize::Large => max_length + 5,
        };

        if text.chars().count() > adjusted_length {
            let truncated: String = text
                .chars()
                .take(adjusted_length.saturating_sub(3))
                .collect();
            format!("{}...", truncated)
        } else {
            text.to_string()
        }
    }
}

/// 图表配置
pub struct ChartConfig;

impl ChartConfig {
    /// 获取扇形图的配置
    pub fn pie_chart_config(screen_size: ScreenSize) -> (u16, u16) {
        match screen_size {
            ScreenSize::Small => (8, 4),   // 半径, 高度
            ScreenSize::Medium => (12, 6), // 半径, 高度
            ScreenSize::Large => (16, 8),  // 半径, 高度
        }
    }

    /// 获取迷你图的数据点数量
    pub fn sparkline_data_points(screen_size: ScreenSize) -> usize {
        match screen_size {
            ScreenSize::Small => 12,
            ScreenSize::Medium => 24,
            ScreenSize::Large => 48,
        }
    }
}
