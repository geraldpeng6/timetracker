// UI 组件和状态定义
// 提供可重用的 UI 组件和状态管理

use serde::{Deserialize, Serialize};

/// 标签页索引
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabIndex {
    Dashboard,  // 概览
    Activities, // 活动（统一的应用和窗口）
}

impl TabIndex {
    pub fn all() -> Vec<Self> {
        vec![Self::Dashboard, Self::Activities]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Dashboard => "概览",
            Self::Activities => "活动",
        }
    }

    pub fn next(&self) -> Self {
        let tabs = Self::all();
        let current_index = tabs.iter().position(|&t| t == *self).unwrap_or(0);
        tabs[(current_index + 1) % tabs.len()]
    }

    pub fn previous(&self) -> Self {
        let tabs = Self::all();
        let current_index = tabs.iter().position(|&t| t == *self).unwrap_or(0);
        tabs[(current_index + tabs.len() - 1) % tabs.len()]
    }
}

/// 视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Unified,   // 统一的可展开表格
    Separated, // 分离的应用和窗口视图
    Timeline,  // 时间线视图
}

impl ViewMode {
    pub fn all() -> Vec<Self> {
        vec![Self::Unified, Self::Separated, Self::Timeline]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Unified => "统一视图",
            Self::Separated => "分离视图",
            Self::Timeline => "时间线",
        }
    }
}

/// 排序字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    Duration,
    AppName,
    WindowTitle,
    StartTime,
    EndTime,
    ActivityCount,
}

impl SortBy {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Duration,
            Self::AppName,
            Self::WindowTitle,
            Self::StartTime,
            Self::EndTime,
            Self::ActivityCount,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Duration => "持续时间",
            Self::AppName => "应用名称",
            Self::WindowTitle => "窗口标题",
            Self::StartTime => "开始时间",
            Self::EndTime => "结束时间",
            Self::ActivityCount => "活动次数",
        }
    }
}

/// 排序顺序
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Ascending => "升序",
            Self::Descending => "降序",
        }
    }
}

/// 图表模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartMode {
    BarChart,
    PieChart,
    Gauge,
    Sparkline,
    Timeline,
}

impl ChartMode {
    pub fn all() -> Vec<Self> {
        vec![
            Self::BarChart,
            Self::PieChart,
            Self::Gauge,
            Self::Sparkline,
            Self::Timeline,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::BarChart => "柱状图",
            Self::PieChart => "饼图",
            Self::Gauge => "仪表盘",
            Self::Sparkline => "迷你图",
            Self::Timeline => "时间线",
        }
    }
}

/// 输入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    EditingApiKey,
    EditingModel,
    EditingEndpoint,
    EditingTemperature,
    EditingMaxTokens,
    Search,
}

impl InputMode {
    pub fn is_editing(&self) -> bool {
        !matches!(self, Self::Normal)
    }
}

/// 时间范围选择
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeRangeFilter {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    All,
}

impl TimeRangeFilter {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Today,
            Self::Yesterday,
            Self::ThisWeek,
            Self::LastWeek,
            Self::ThisMonth,
            Self::LastMonth,
            Self::All,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Today => "今天",
            Self::Yesterday => "昨天",
            Self::ThisWeek => "本周",
            Self::LastWeek => "上周",
            Self::ThisMonth => "本月",
            Self::LastMonth => "上月",
            Self::All => "全部",
        }
    }
}

/// 应用程序表格项
#[derive(Debug, Clone)]
pub struct AppTableItem {
    pub app_name: String,
    pub total_duration: u64,
    pub window_count: usize,
    pub windows: Vec<WindowItem>,
    pub is_expanded: bool,
    pub last_active: chrono::DateTime<chrono::Utc>,
}

/// 窗口项
#[derive(Debug, Clone)]
pub struct WindowItem {
    pub window_title: String,
    pub duration: u64,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub activity_count: usize,
}

/// 最近活动项
#[derive(Debug, Clone)]
pub struct RecentActivityItem {
    pub app_name: String,
    pub window_title: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration: u64,
}

/// 综合活动项 - 统一显示应用和窗口信息
#[derive(Debug, Clone)]
pub struct UnifiedActivityItem {
    pub app_name: String,
    pub window_title: String,
    pub total_duration: u64,
    pub recent_duration: u64, // 最近一次的使用时长
    pub activity_count: usize,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub first_active: chrono::DateTime<chrono::Utc>,
    pub is_currently_active: bool,
    pub productivity_category: ProductivityCategory,
}

/// 生产力分类
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductivityCategory {
    Productive,   // 生产力应用
    Neutral,      // 中性应用
    Unproductive, // 娱乐/分心应用
}

/// 图表配置
#[derive(Debug, Clone)]
pub struct ChartConfiguration {
    pub show_bar_chart: bool,
    pub show_sparkline: bool,
    pub show_pie_chart: bool,
    pub show_timeline: bool,
    pub chart_position: ChartPosition,
    pub chart_size: ChartSize,
}

/// 图表位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartPosition {
    Top,
    Bottom,
    Left,
    Right,
    Overlay,
}

/// 图表大小
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartSize {
    Small,
    Medium,
    Large,
    FullWidth,
}

impl Default for ChartConfiguration {
    fn default() -> Self {
        Self {
            show_bar_chart: true,
            show_sparkline: false,
            show_pie_chart: false,
            show_timeline: false,
            chart_position: ChartPosition::Bottom,
            chart_size: ChartSize::Medium,
        }
    }
}

/// 分页状态
#[derive(Debug, Clone)]
pub struct PaginationState {
    pub current_page: usize,
    pub items_per_page: usize,
    pub total_items: usize,
}

impl PaginationState {
    pub fn new(items_per_page: usize) -> Self {
        Self {
            current_page: 0,
            items_per_page,
            total_items: 0,
        }
    }

    pub fn total_pages(&self) -> usize {
        if self.total_items == 0 {
            0
        } else {
            (self.total_items + self.items_per_page - 1) / self.items_per_page
        }
    }

    pub fn start_index(&self) -> usize {
        self.current_page * self.items_per_page
    }

    pub fn end_index(&self) -> usize {
        std::cmp::min(self.start_index() + self.items_per_page, self.total_items)
    }

    pub fn can_go_next(&self) -> bool {
        self.current_page < self.total_pages().saturating_sub(1)
    }

    pub fn can_go_prev(&self) -> bool {
        self.current_page > 0
    }

    pub fn next_page(&mut self) {
        if self.can_go_next() {
            self.current_page += 1;
        }
    }

    pub fn prev_page(&mut self) {
        if self.can_go_prev() {
            self.current_page -= 1;
        }
    }

    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        // 确保当前页面在有效范围内
        if self.total_pages() > 0 && self.current_page >= self.total_pages() {
            self.current_page = self.total_pages() - 1;
        }
    }
}

impl Default for PaginationState {
    fn default() -> Self {
        Self::new(20) // 默认每页20项
    }
}

/// 对话框状态
#[derive(Debug, Clone)]
pub struct DialogState {
    pub dialog_type: DialogType,
    pub is_visible: bool,
    pub title: String,
    pub message: String,
    pub selected_option: usize,
}

/// 对话框类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogType {
    None,
    Confirmation,
    Information,
    Warning,
    Error,
    Input,
    QuitTui,     // 退出TUI确认
    QuitProgram, // 退出程序确认
}

impl DialogState {
    pub fn new() -> Self {
        Self {
            dialog_type: DialogType::None,
            is_visible: false,
            title: String::new(),
            message: String::new(),
            selected_option: 0,
        }
    }

    pub fn show_confirmation(&mut self, title: &str, message: &str) {
        self.dialog_type = DialogType::Confirmation;
        self.is_visible = true;
        self.title = title.to_string();
        self.message = message.to_string();
        self.selected_option = 0;
    }

    pub fn show_info(&mut self, title: &str, message: &str) {
        self.dialog_type = DialogType::Information;
        self.is_visible = true;
        self.title = title.to_string();
        self.message = message.to_string();
        self.selected_option = 0;
    }

    pub fn show_warning(&mut self, title: &str, message: &str) {
        self.dialog_type = DialogType::Warning;
        self.is_visible = true;
        self.title = title.to_string();
        self.message = message.to_string();
        self.selected_option = 0;
    }

    pub fn show_error(&mut self, title: &str, message: &str) {
        self.dialog_type = DialogType::Error;
        self.is_visible = true;
        self.title = title.to_string();
        self.message = message.to_string();
        self.selected_option = 0;
    }

    pub fn show_quit_tui_confirmation(&mut self) {
        self.dialog_type = DialogType::QuitTui;
        self.is_visible = true;
        self.title = "退出TUI界面".to_string();
        self.message = "确定要退出TUI界面吗？程序将继续在后台运行。".to_string();
        self.selected_option = 0;
    }

    pub fn show_quit_program_confirmation(&mut self) {
        self.dialog_type = DialogType::QuitProgram;
        self.is_visible = true;
        self.title = "退出程序".to_string();
        self.message = "确定要完全退出TimeTracker程序吗？".to_string();
        self.selected_option = 0;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.dialog_type = DialogType::None;
        self.title.clear();
        self.message.clear();
        self.selected_option = 0;
    }

    pub fn toggle_option(&mut self) {
        if matches!(
            self.dialog_type,
            DialogType::Confirmation | DialogType::QuitTui | DialogType::QuitProgram
        ) {
            self.selected_option = if self.selected_option == 0 { 1 } else { 0 };
        }
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self::new()
    }
}

/// UI 状态
#[derive(Debug, Clone)]
pub struct UiState {
    pub current_tab: TabIndex,
    pub view_mode: ViewMode,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub chart_mode: ChartMode,
    pub time_range: TimeRangeFilter,
    pub input_mode: InputMode,
    pub selected_row: usize,
    pub selected_column: usize,
    pub show_help: bool,
    pub mouse_enabled: bool,
    pub search_query: String,
    pub input_buffer: String, // 用于AI配置等输入
    pub chart_config: ChartConfiguration,
    pub pagination: PaginationState,
    pub dialog_state: DialogState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_tab: TabIndex::Dashboard,
            view_mode: ViewMode::Unified,
            sort_by: SortBy::EndTime,
            sort_order: SortOrder::Descending, // 降序表示最新的在前面
            chart_mode: ChartMode::BarChart,
            time_range: TimeRangeFilter::Today,
            input_mode: InputMode::Normal,
            selected_row: 0,
            selected_column: 0,
            show_help: false,
            mouse_enabled: true,
            search_query: String::new(),
            input_buffer: String::new(),
            chart_config: ChartConfiguration::default(),
            pagination: PaginationState::default(),
            dialog_state: DialogState::default(),
        }
    }
}

impl UiState {
    /// 切换视图模式
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Unified => ViewMode::Separated,
            ViewMode::Separated => ViewMode::Timeline,
            ViewMode::Timeline => ViewMode::Unified,
        };
    }

    /// 切换排序字段
    pub fn toggle_sort_by(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Duration => SortBy::AppName,
            SortBy::AppName => SortBy::WindowTitle,
            SortBy::WindowTitle => SortBy::StartTime,
            SortBy::StartTime => SortBy::EndTime,
            SortBy::EndTime => SortBy::ActivityCount,
            SortBy::ActivityCount => SortBy::Duration,
        };
    }

    /// 切换排序顺序
    pub fn toggle_sort_order(&mut self) {
        self.sort_order = self.sort_order.toggle();
    }

    /// 切换图表模式
    pub fn toggle_chart_mode(&mut self) {
        let modes = ChartMode::all();
        let current_index = modes
            .iter()
            .position(|&m| m == self.chart_mode)
            .unwrap_or(0);
        let next_index = (current_index + 1) % modes.len();
        self.chart_mode = modes[next_index];
    }

    /// 切换时间范围
    pub fn toggle_time_range(&mut self) {
        let ranges = TimeRangeFilter::all();
        let current_index = ranges
            .iter()
            .position(|&r| r == self.time_range)
            .unwrap_or(0);
        let next_index = (current_index + 1) % ranges.len();
        self.time_range = ranges[next_index];
    }
}
