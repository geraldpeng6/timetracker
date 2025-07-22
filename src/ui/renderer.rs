use crate::ui::components::{
    AppTableItem, ProductivityCategory, RecentActivityItem, SortBy, SortOrder, TabIndex,
    TimeRangeFilter, UiState, UnifiedActivityItem, ViewMode, WindowItem,
};
use crate::ui::layout::{ResponsiveLayout, ScreenSize};
use crate::ui::themes::Theme;
use crate::ui::widgets::{ContextHelpWidget, DialogWidget};
use crate::utils::time::format_duration;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{
        BarChart, Block, Borders, Cell, List, ListItem, Paragraph, Row, Sparkline, Table, Tabs,
        Wrap,
    },
    Frame,
};

/// 渲染器
pub struct Renderer<'a> {
    theme: &'a Theme,
}

impl<'a> Renderer<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    /// 渲染主界面
    pub fn render<B: Backend>(
        &self,
        f: &mut ratatui::Frame,
        ui_state: &UiState,
        _app_items: &[AppTableItem],
        _window_items: &[WindowItem],
        _recent_activities: &[RecentActivityItem],
        unified_activities: &[UnifiedActivityItem],
        _statistics: &(),
    ) {
        // 检测屏幕尺寸
        let screen_size = ScreenSize::from_rect(f.area());

        // 使用响应式布局约束
        let constraints = ResponsiveLayout::main_constraints(screen_size);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(f.area());

        self.render_tabs::<B>(f, chunks[0], ui_state.current_tab);
        self.render_content::<B>(
            f,
            chunks[1],
            ui_state,
            _app_items,
            _window_items,
            _recent_activities,
            unified_activities,
            _statistics,
            screen_size,
        );

        // 渲染上下文敏感的帮助信息
        if ui_state.show_help && chunks.len() > 2 {
            self.render_context_help::<B>(f, chunks[2], ui_state);
        }

        // 渲染对话框（如果有）
        if ui_state.dialog_state.is_visible {
            self.render_dialog::<B>(f, f.area(), ui_state);
        }
    }

    /// 渲染标签页
    fn render_tabs<B: Backend>(&self, f: &mut Frame, area: Rect, current_tab: TabIndex) {
        let titles: Vec<_> = TabIndex::all()
            .iter()
            .map(|tab| Line::from(tab.title()))
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("时间追踪器"))
            .style(self.theme.title_style())
            .highlight_style(self.theme.selected_style())
            .select(current_tab as usize);

        f.render_widget(tabs, area);
    }

    /// 渲染内容区域
    fn render_content<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        ui_state: &UiState,
        app_items: &[AppTableItem],
        _window_items: &[WindowItem],
        _recent_activities: &[RecentActivityItem],
        unified_activities: &[UnifiedActivityItem],
        _statistics: &(),
        screen_size: ScreenSize,
    ) {
        match ui_state.current_tab {
            TabIndex::Dashboard => self.render_overview::<B>(
                f,
                area,
                _statistics,
                app_items,
                ui_state,
                unified_activities,
                screen_size,
            ),
            TabIndex::Activities => self.render_unified_activities::<B>(
                f,
                area,
                ui_state,
                unified_activities,
                screen_size,
            ),
        }
    }

    /// 渲染概览页面 - 全新的现代化设计
    fn render_overview<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        _statistics: &(),
        app_items: &[AppTableItem],
        ui_state: &UiState,
        unified_activities: &[UnifiedActivityItem],
        screen_size: ScreenSize,
    ) {
        // 使用响应式布局约束
        let constraints = ResponsiveLayout::overview_constraints(screen_size);
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // 渲染各个部分
        self.render_time_header::<B>(f, main_chunks[0], _statistics, ui_state, screen_size);

        if main_chunks.len() > 1 {
            self.render_week_chart::<B>(f, main_chunks[1], _statistics, ui_state, screen_size);
        }

        if main_chunks.len() > 2 {
            self.render_daily_chart_and_categories::<B>(
                f,
                main_chunks[2],
                _statistics,
                ui_state,
                screen_size,
            );
        }

        // 大屏幕上显示扇形图（使用文本表示）
        if screen_size.is_large() && main_chunks.len() > 3 {
            self.render_pie_charts::<B>(f, main_chunks[3], unified_activities, screen_size);
        }

        // 应用列表
        let app_list_index = if screen_size.is_large() {
            4
        } else {
            main_chunks.len() - 1
        };
        if app_list_index < main_chunks.len() {
            self.render_app_list::<B>(
                f,
                main_chunks[app_list_index],
                app_items,
                unified_activities,
                ui_state,
                screen_size,
            );
        }
    }

    /// 渲染时间头部 - 显示日期选择器
    fn render_time_header<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        _statistics: &(),
        ui_state: &UiState,
        _screen_size: ScreenSize,
    ) {
        // 日期选择器
        let date_text = match ui_state.time_range {
            TimeRangeFilter::Today => "今天",
            TimeRangeFilter::Yesterday => "昨天",
            TimeRangeFilter::ThisWeek => "本周",
            TimeRangeFilter::LastWeek => "上周",
            TimeRangeFilter::ThisMonth => "本月",
            TimeRangeFilter::LastMonth => "上月",
            TimeRangeFilter::All => "全部",
        };

        let date_paragraph = Paragraph::new(format!(
            "{} ◀ {} ▶",
            chrono::Local::now().format("%m月%d日"),
            date_text
        ))
        .style(self.theme.table_row_style())
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);
        f.render_widget(date_paragraph, area);
    }

    /// 渲染周视图图表
    fn render_week_chart<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        _statistics: &(),
        _ui_state: &UiState,
        screen_size: ScreenSize,
    ) {
        // 模拟一周的数据（实际应该从统计数据中获取）
        let week_data = vec![
            ("一", 6),
            ("二", 8),
            ("三", 7),
            ("四", 9),
            ("五", 5),
            ("六", 3),
            ("日", 4),
        ];

        let bar_width = ResponsiveLayout::bar_chart_width(screen_size);
        let title = if screen_size.is_small() {
            "本周"
        } else {
            "本周使用情况"
        };

        let chart = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title(title))
            .data(&week_data)
            .bar_width(bar_width)
            .bar_style(self.theme.chart_style())
            .value_style(self.theme.table_row_style());

        f.render_widget(chart, area);
    }

    /// 渲染24小时图表和分类统计
    fn render_daily_chart_and_categories<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        _statistics: &(),
        _ui_state: &UiState,
        screen_size: ScreenSize,
    ) {
        // 根据屏幕大小调整布局
        let constraints = ResponsiveLayout::chart_horizontal_constraints(screen_size);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // 左侧：24小时使用情况图表
        self.render_24hour_chart::<B>(f, chunks[0], screen_size);

        // 右侧：分类统计（仅在中等和大屏幕上显示）
        if chunks.len() > 1 && !screen_size.is_small() {
            self.render_category_stats::<B>(f, chunks[1], _statistics, screen_size);
        }
    }

    /// 渲染24小时图表
    fn render_24hour_chart<B: Backend>(&self, f: &mut Frame, area: Rect, screen_size: ScreenSize) {
        // 模拟24小时数据
        let data_points = match screen_size {
            ScreenSize::Small => 12, // 每2小时一个点
            _ => 24,                 // 每小时一个点
        };

        let hour_data: Vec<u64> = (0..data_points)
            .map(|i| {
                let h = if data_points == 12 { i * 2 } else { i };
                match h {
                    6..=8 => 2,   // 早上
                    9..=11 => 4,  // 上午
                    12..=13 => 2, // 午休
                    14..=17 => 5, // 下午
                    18..=20 => 3, // 晚上
                    21..=23 => 2, // 夜晚
                    _ => 0,       // 深夜
                }
            })
            .collect();

        let title = if screen_size.is_small() {
            "24小时分布"
        } else {
            "24小时使用分布"
        };

        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title(title))
            .data(&hour_data)
            .style(self.theme.chart_style());

        f.render_widget(sparkline, area);
    }

    /// 渲染分类统计
    fn render_category_stats<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        _statistics: &(),
        screen_size: ScreenSize,
    ) {
        // 计算各类别的时间（简化版本）
        let productive_time = 0; // 简化版本，不显示统计
        let social_time = 0;
        let other_time = 0;

        let categories = if screen_size.is_small() {
            vec![
                format!("🔵 效率\n{}", format_duration(productive_time)),
                format!("🟢 社交\n{}", format_duration(social_time)),
                format!("🟠 其他\n{}", format_duration(other_time)),
            ]
        } else {
            vec![
                format!("🔵 效率与财务\n{}", format_duration(productive_time)),
                format!("🟢 社交\n{}", format_duration(social_time)),
                format!("🟠 其他\n{}", format_duration(other_time)),
            ]
        };

        let items: Vec<ListItem> = categories
            .iter()
            .map(|cat| ListItem::new(cat.as_str()))
            .collect();

        let title = if screen_size.is_small() {
            "分类"
        } else {
            "分类统计"
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(self.theme.table_row_style());

        f.render_widget(list, area);
    }

    /// 渲染扇形图区域（仅在大屏幕上显示）
    fn render_pie_charts<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        unified_activities: &[UnifiedActivityItem],
        screen_size: ScreenSize,
    ) {
        // 水平分割：左侧应用分布扇形图，右侧生产力分布扇形图
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // 左侧：应用使用时间分布
        self.render_app_pie_chart::<B>(f, chunks[0], unified_activities, screen_size);

        // 右侧：生产力分布
        self.render_productivity_pie_chart::<B>(f, chunks[1], unified_activities, screen_size);
    }

    /// 渲染今日应用时间分布饼状图
    fn render_app_pie_chart<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        unified_activities: &[UnifiedActivityItem],
        _screen_size: ScreenSize,
    ) {
        // 获取今日的开始时间（0点为分割点，可配置）
        // TODO: 从配置中读取 day_split_hour
        let split_hour = 0u32; // 默认0点分割
        let today_start = chrono::Local::now()
            .date_naive()
            .and_hms_opt(split_hour, 0, 0)
            .unwrap()
            .and_local_timezone(chrono::Local)
            .unwrap()
            .with_timezone(&chrono::Utc);

        // 过滤今日的活动数据
        let mut app_data: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        for activity in unified_activities.iter() {
            // 检查活动是否在今日
            if activity.last_active >= today_start {
                *app_data.entry(activity.app_name.clone()).or_insert(0) += activity.total_duration;
            }
        }

        // 按时长排序，取前5个应用
        let mut sorted_apps: Vec<_> = app_data.iter().collect();
        sorted_apps.sort_by(|a, b| b.1.cmp(a.1));
        let top_apps: std::collections::HashMap<String, u64> = sorted_apps
            .into_iter()
            .take(5)
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        if top_apps.is_empty() {
            let empty_text = Paragraph::new("今日暂无应用数据")
                .style(self.theme.inactive_style())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("今日应用分布"));
            f.render_widget(empty_text, area);
            return;
        }

        // 简化的饼状图显示（使用文本表示）
        let total: u64 = top_apps.values().sum();
        let mut items = Vec::new();

        // 按时长排序显示
        let mut sorted_display: Vec<_> = top_apps.iter().collect();
        sorted_display.sort_by(|a, b| b.1.cmp(a.1));

        for (app, duration) in sorted_display {
            let percentage = if total > 0 {
                (*duration as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            items.push(ListItem::new(format!(
                "{}: {:.1}% ({})",
                ResponsiveLayout::truncate_text(app, 10, _screen_size),
                percentage,
                format_duration(*duration)
            )));
        }

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("今日应用分布"))
            .style(self.theme.table_row_style());

        f.render_widget(list, area);
    }

    /// 渲染生产力分布扇形图
    fn render_productivity_pie_chart<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        unified_activities: &[UnifiedActivityItem],
        _screen_size: ScreenSize,
    ) {
        // 计算生产力分布
        let mut productive_time = 0u64;
        let mut neutral_time = 0u64;
        let mut unproductive_time = 0u64;

        for activity in unified_activities {
            match activity.productivity_category {
                ProductivityCategory::Productive => productive_time += activity.total_duration,
                ProductivityCategory::Neutral => neutral_time += activity.total_duration,
                ProductivityCategory::Unproductive => unproductive_time += activity.total_duration,
            }
        }

        let total = productive_time + neutral_time + unproductive_time;

        if total == 0 {
            let empty_text = Paragraph::new("暂无生产力数据")
                .style(self.theme.inactive_style())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("生产力分布"));
            f.render_widget(empty_text, area);
            return;
        }

        let items = vec![
            ListItem::new(format!(
                "🟢 生产: {:.1}% ({})",
                (productive_time as f64 / total as f64) * 100.0,
                format_duration(productive_time)
            )),
            ListItem::new(format!(
                "🟡 中性: {:.1}% ({})",
                (neutral_time as f64 / total as f64) * 100.0,
                format_duration(neutral_time)
            )),
            ListItem::new(format!(
                "🔴 娱乐: {:.1}% ({})",
                (unproductive_time as f64 / total as f64) * 100.0,
                format_duration(unproductive_time)
            )),
        ];

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("生产力分布"))
            .style(self.theme.table_row_style());

        f.render_widget(list, area);
    }

    /// 渲染应用列表
    fn render_app_list<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_items: &[AppTableItem],
        _unified_activities: &[UnifiedActivityItem],
        _ui_state: &UiState,
        screen_size: ScreenSize,
    ) {
        // 根据屏幕大小调整布局
        let header_height = if screen_size.is_small() { 0 } else { 3 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
            .split(area);

        // 搜索栏（仅在中等和大屏幕上显示）
        if !screen_size.is_small() {
            let search_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0]);

            let app_title = Paragraph::new("显示 App")
                .style(self.theme.title_style())
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Left);
            f.render_widget(app_title, search_chunks[0]);

            let search_box = Paragraph::new("🔍 搜索")
                .style(self.theme.table_row_style())
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Left);
            f.render_widget(search_box, search_chunks[1]);
        }

        // 应用表格
        let table_area = if screen_size.is_small() {
            area
        } else {
            chunks[1]
        };

        // 根据屏幕大小调整表格列
        let (header_cells, constraints) = match screen_size {
            ScreenSize::Small => {
                let headers = ["App", "时间"];
                let constraints = ResponsiveLayout::app_table_constraints(screen_size);
                (headers.to_vec(), constraints)
            }
            _ => {
                let headers = ["App", "时间", "限额"];
                let constraints = ResponsiveLayout::app_table_constraints(screen_size);
                (headers.to_vec(), constraints)
            }
        };

        let header = Row::new(
            header_cells
                .iter()
                .map(|h| Cell::from(*h).style(self.theme.table_header_style())),
        )
        .height(1)
        .bottom_margin(1);

        let max_items = ResponsiveLayout::items_per_page(screen_size).min(10);
        let rows: Vec<Row> = app_items
            .iter()
            .take(max_items)
            .map(|item| {
                let app_name = ResponsiveLayout::truncate_text(&item.app_name, 20, screen_size);
                let mut cells = vec![
                    Cell::from(app_name),
                    Cell::from(format_duration(item.total_duration)),
                ];

                // 只在非小屏幕上显示限额列
                if !screen_size.is_small() {
                    cells.push(Cell::from("")); // 限额列暂时为空
                }

                Row::new(cells).height(1)
            })
            .collect();

        let table = Table::new(rows, constraints)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("应用程序"))
            .style(self.theme.table_row_style())
            .column_spacing(1);

        f.render_widget(table, table_area);
    }

    /// 渲染热门应用概览
    #[allow(dead_code)] // 保留为将来功能
    fn render_top_apps_overview<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_items: &[AppTableItem],
    ) {
        // 计算总时间用于百分比计算
        let total_time: u64 = app_items.iter().map(|item| item.total_duration).sum();

        let items: Vec<ListItem> = app_items
            .iter()
            .take(10)
            .map(|item| {
                let percentage = if total_time > 0 {
                    (item.total_duration as f64 / total_time as f64) * 100.0
                } else {
                    0.0
                };
                ListItem::new(format!(
                    "{} - {} ({:.1}%)",
                    item.app_name,
                    format_duration(item.total_duration),
                    percentage
                ))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("热门应用").borders(Borders::ALL))
            .style(self.theme.table_row_style());

        f.render_widget(list, area);
    }

    /// 渲染统一活动页面
    fn render_unified_activities<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        ui_state: &UiState,
        unified_activities: &[UnifiedActivityItem],
        screen_size: ScreenSize,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 控制栏
                Constraint::Min(0),    // 活动表格
                Constraint::Length(3), // 分页信息
            ])
            .split(area);

        // 控制栏
        self.render_control_bar::<B>(f, chunks[0], ui_state, screen_size);

        // 统一活动表格（带分页）
        self.render_paginated_activity_table::<B>(
            f,
            chunks[1],
            unified_activities,
            ui_state,
            screen_size,
        );

        // 分页信息
        self.render_pagination_info::<B>(
            f,
            chunks[2],
            ui_state,
            unified_activities.len(),
            screen_size,
        );
    }

    /// 渲染带分页的活动表格
    fn render_paginated_activity_table<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        unified_activities: &[UnifiedActivityItem],
        ui_state: &UiState,
        screen_size: ScreenSize,
    ) {
        // 根据屏幕大小调整表格列
        let (header_cells, _constraints) = match screen_size {
            ScreenSize::Small => {
                let headers = vec!["应用名称", "窗口标题", "时长"];
                let constraints = ResponsiveLayout::activity_table_constraints(screen_size);
                (headers, constraints)
            }
            ScreenSize::Medium => {
                let headers = vec!["应用名称", "窗口标题", "时长", "次数", "最后活动"];
                let constraints = ResponsiveLayout::activity_table_constraints(screen_size);
                (headers, constraints)
            }
            ScreenSize::Large => {
                let headers = vec![
                    "应用名称",
                    "窗口标题",
                    "最近时长",
                    "活动次数",
                    "最后活动",
                    "状态",
                    "类型",
                ];
                let constraints = ResponsiveLayout::activity_table_constraints(screen_size);
                (headers, constraints)
            }
        };

        let header = Row::new(
            header_cells
                .iter()
                .map(|h| Cell::from(*h).style(self.theme.table_header_style())),
        )
        .height(1)
        .bottom_margin(1);

        // 计算分页范围
        let start_index = ui_state.pagination.start_index();
        let end_index = ui_state
            .pagination
            .end_index()
            .min(unified_activities.len());

        let paginated_activities = if start_index < unified_activities.len() {
            &unified_activities[start_index..end_index]
        } else {
            &[]
        };

        let rows = paginated_activities
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let last_active = chrono::DateTime::<chrono::Local>::from(item.last_active)
                    .format("%m-%d %H:%M:%S") // 添加秒数显示
                    .to_string();

                let status = if item.is_currently_active {
                    "🟢 活跃"
                } else {
                    "⚪ 空闲"
                };

                let category = match item.productivity_category {
                    ProductivityCategory::Productive => "🟢 生产",
                    ProductivityCategory::Neutral => "🟡 中性",
                    ProductivityCategory::Unproductive => "🔴 娱乐",
                };

                // 根据屏幕大小截断文本
                let app_name = ResponsiveLayout::truncate_text(&item.app_name, 15, screen_size);
                let window_title =
                    ResponsiveLayout::truncate_text(&item.window_title, 35, screen_size);

                // 根据屏幕大小构建单元格
                let mut cells = vec![
                    Cell::from(app_name),
                    Cell::from(window_title),
                    Cell::from(format_duration(item.recent_duration)),
                ];

                // 根据屏幕大小添加额外的列
                match screen_size {
                    ScreenSize::Small => {
                        // 小屏幕只显示基本信息
                    }
                    ScreenSize::Medium => {
                        cells.push(Cell::from(item.activity_count.to_string()));
                        cells.push(Cell::from(last_active));
                    }
                    ScreenSize::Large => {
                        cells.push(Cell::from(item.activity_count.to_string()));
                        cells.push(Cell::from(last_active));
                        cells.push(Cell::from(status));
                        cells.push(Cell::from(category));
                    }
                }

                let row = Row::new(cells).height(1);
                if index == ui_state.selected_row {
                    row.style(self.theme.selected_style())
                } else {
                    row
                }
            });

        let constraints = ResponsiveLayout::activity_table_constraints(screen_size);
        let table = Table::new(rows, constraints)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "活动记录 (第 {}/{} 页)",
                ui_state.pagination.current_page + 1,
                ui_state.pagination.total_pages().max(1)
            )))
            .column_spacing(1)
            .style(self.theme.table_row_style())
            .highlight_style(self.theme.selected_style());

        f.render_widget(table, area);
    }

    /// 渲染分页信息
    fn render_pagination_info<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        ui_state: &UiState,
        total_items: usize,
        screen_size: ScreenSize,
    ) {
        let pagination_text = if screen_size.is_small() {
            format!(
                "共 {} 项 | 第 {}/{} 页",
                total_items,
                ui_state.pagination.current_page + 1,
                ui_state.pagination.total_pages().max(1)
            )
        } else {
            format!(
                "共 {} 项 | 第 {}/{} 页 | 每页 {} 项 | PgUp/PgDn:翻页 ↑/↓:选择 Enter:详情 Del:删除",
                total_items,
                ui_state.pagination.current_page + 1,
                ui_state.pagination.total_pages().max(1),
                ui_state.pagination.items_per_page
            )
        };

        let pagination = Paragraph::new(pagination_text)
            .block(Block::default().borders(Borders::ALL).title("分页控制"))
            .wrap(Wrap { trim: true })
            .style(self.theme.table_row_style());

        f.render_widget(pagination, area);
    }

    #[allow(dead_code)] // 保留为将来功能
    fn render_applications<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        _ui_state: &UiState,
        app_items: &[AppTableItem],
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // 控制栏 (这些方法暂时不需要screen_size参数)
        // self.render_control_bar::<B>(f, chunks[0], ui_state, screen_size);

        // 应用表格
        self.render_app_table::<B>(f, chunks[1], app_items);
    }

    /// 渲染控制栏
    fn render_control_bar<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        ui_state: &UiState,
        screen_size: ScreenSize,
    ) {
        let view_mode_text = match ui_state.view_mode {
            ViewMode::Unified => "统一",
            ViewMode::Separated => "分离",
            ViewMode::Timeline => "时间线",
        };

        let sort_by_text = match ui_state.sort_by {
            SortBy::Duration => "时长",
            SortBy::AppName => "应用名称",
            SortBy::WindowTitle => "窗口标题",
            SortBy::StartTime => "开始时间",
            SortBy::EndTime => "结束时间",
            SortBy::ActivityCount => "活动次数",
        };

        let sort_order_text = match ui_state.sort_order {
            SortOrder::Ascending => "升序",
            SortOrder::Descending => "降序",
        };

        let time_range_text = match ui_state.time_range {
            TimeRangeFilter::Today => "今天",
            TimeRangeFilter::Yesterday => "昨天",
            TimeRangeFilter::ThisWeek => "本周",
            TimeRangeFilter::LastWeek => "上周",
            TimeRangeFilter::ThisMonth => "本月",
            TimeRangeFilter::LastMonth => "上月",
            TimeRangeFilter::All => "全部",
        };

        let control_text = if screen_size.is_small() {
            format!(
                "视图: {} | 排序: {} | 时间: {}",
                view_mode_text, sort_by_text, time_range_text
            )
        } else {
            format!(
                "视图: {} | 排序: {} ({}) | 时间: {} | v:切换视图 s:排序 o:顺序 f:时间范围",
                view_mode_text, sort_by_text, sort_order_text, time_range_text
            )
        };

        let control = Paragraph::new(control_text)
            .block(Block::default().borders(Borders::ALL).title("控制"))
            .wrap(Wrap { trim: true });

        f.render_widget(control, area);
    }

    /// 渲染应用表格
    #[allow(dead_code)] // 保留为将来功能
    fn render_app_table<B: Backend>(&self, f: &mut Frame, area: Rect, app_items: &[AppTableItem]) {
        let header_cells = ["应用名称", "总时长", "窗口数"]
            .iter()
            .map(|h| Cell::from(*h).style(self.theme.table_header_style()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = app_items.iter().map(|item| {
            let cells = vec![
                Cell::from(item.app_name.clone()),
                Cell::from(format_duration(item.total_duration)),
                Cell::from(item.window_count.to_string()),
            ];
            Row::new(cells).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("应用程序"))
        .column_spacing(1)
        .style(self.theme.table_row_style())
        .highlight_style(self.theme.selected_style());

        f.render_widget(table, area);
    }

    /// 渲染窗口页面
    #[allow(dead_code)] // 保留为将来功能
    fn render_windows<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        _ui_state: &UiState,
        window_items: &[WindowItem],
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // 控制栏 (这些方法暂时不需要screen_size参数)
        // self.render_control_bar::<B>(f, chunks[0], ui_state, screen_size);

        // 窗口表格
        self.render_window_table::<B>(f, chunks[1], window_items);
    }

    /// 渲染窗口表格
    #[allow(dead_code)] // 保留为将来功能
    fn render_window_table<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        window_items: &[WindowItem],
    ) {
        let header_cells = ["窗口标题", "时长", "活动次数"]
            .iter()
            .map(|h| Cell::from(*h).style(self.theme.table_header_style()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = window_items.iter().map(|item| {
            let cells = vec![
                Cell::from(item.window_title.clone()),
                Cell::from(format_duration(item.duration)),
                Cell::from(item.activity_count.to_string()),
            ];
            Row::new(cells).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("窗口"))
        .column_spacing(1)
        .style(self.theme.table_row_style())
        .highlight_style(self.theme.selected_style());

        f.render_widget(table, area);
    }

    /// 渲染最近活动页面
    #[allow(dead_code)] // 保留为将来功能
    fn render_recent_activity<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        recent_activities: &[RecentActivityItem],
    ) {
        let header_cells = ["应用名称", "窗口标题", "开始时间", "时长"]
            .iter()
            .map(|h| Cell::from(*h).style(self.theme.table_header_style()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = recent_activities.iter().map(|item| {
            let start_time = chrono::DateTime::<chrono::Local>::from(item.start_time)
                .format("%H:%M:%S")
                .to_string();
            let cells = vec![
                Cell::from(item.app_name.clone()),
                Cell::from(item.window_title.clone()),
                Cell::from(start_time),
                Cell::from(format_duration(item.duration)),
            ];
            Row::new(cells).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(40),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("最近活动"))
        .column_spacing(1)
        .style(self.theme.table_row_style())
        .highlight_style(self.theme.selected_style());

        f.render_widget(table, area);
    }

    /// 渲染上下文敏感的帮助信息
    fn render_context_help<B: Backend>(&self, f: &mut Frame, area: Rect, ui_state: &UiState) {
        let help_widget =
            ContextHelpWidget::new(self.theme, ui_state.current_tab, ui_state.input_mode);
        help_widget.render::<B>(f, area);
    }

    /// 渲染对话框
    fn render_dialog<B: Backend>(&self, f: &mut Frame, area: Rect, ui_state: &UiState) {
        let dialog_widget = DialogWidget::new(
            self.theme,
            ui_state.dialog_state.title.clone(),
            ui_state.dialog_state.message.clone(),
            match ui_state.dialog_state.dialog_type {
                crate::ui::components::DialogType::Confirmation => {
                    crate::ui::widgets::DialogType::Confirm
                }
                crate::ui::components::DialogType::QuitTui => {
                    crate::ui::widgets::DialogType::Confirm
                }
                crate::ui::components::DialogType::QuitProgram => {
                    crate::ui::widgets::DialogType::Confirm
                }
                crate::ui::components::DialogType::Information => {
                    crate::ui::widgets::DialogType::Info
                }
                crate::ui::components::DialogType::Error => crate::ui::widgets::DialogType::Error,
                crate::ui::components::DialogType::Warning => {
                    crate::ui::widgets::DialogType::Warning
                }
                crate::ui::components::DialogType::None => crate::ui::widgets::DialogType::Info,
                crate::ui::components::DialogType::Input => crate::ui::widgets::DialogType::Info,
            },
        );
        dialog_widget.render::<B>(f, area);
    }
}
