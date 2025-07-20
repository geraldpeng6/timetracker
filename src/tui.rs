use crate::tracker::TimeTracker;
use anyhow::Result;
use chrono::{Local, Timelike, Utc};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        BarChart, Block, Borders, Cell, Clear, Gauge, ListState, Paragraph, Row, Sparkline, Table,
        TableState, Tabs, Wrap,
    },
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    Duration,
    AppName,
    WindowTitle,
    StartTime,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabIndex {
    Dashboard,
    Analytics,
    Settings,
}

/// 稳定排序的辅助结构，包含原始索引以确保排序稳定性
#[derive(Debug, Clone)]
struct SortableItem<T> {
    item: T,
    original_index: usize,
}

impl<T> SortableItem<T> {
    fn new(item: T, index: usize) -> Self {
        Self {
            item,
            original_index: index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartMode {
    BarChart,
    PieChart,
    Gauge,
    Sparkline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Applications,
    Windows,
    Recent,
}

pub struct TuiApp {
    pub should_quit: bool,
    pub current_tab: TabIndex,
    pub view_mode: ViewMode,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub table_state: TableState,
    pub list_state: ListState,
    pub show_help: bool,
    pub tracker: TimeTracker,
    /// 鼠标支持
    pub mouse_enabled: bool,
    /// 当前选中的列（用于排序）
    pub selected_column: usize,
    /// 鼠标悬浮位置
    pub mouse_position: (u16, u16),
    /// AI 配置相关状态
    pub ai_api_key: String,
    pub ai_model: String,
    pub ai_models: Vec<String>,
    pub ai_model_index: usize,
    /// 图表显示模式
    pub chart_mode: ChartMode,
    pub chart_modes: Vec<ChartMode>,
    pub active_charts: usize,
    /// 输入状态
    pub input_mode: InputMode,
    pub input_buffer: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    EditingApiKey,
    EditingModel,
}

impl TuiApp {
    pub fn new(mut tracker: TimeTracker) -> Result<Self> {
        tracker.load_data()?;

        Ok(Self {
            should_quit: false,
            current_tab: TabIndex::Dashboard,
            view_mode: ViewMode::Applications,
            sort_by: SortBy::Duration,
            sort_order: SortOrder::Descending,
            table_state: TableState::default(),
            list_state: ListState::default(),
            show_help: false,
            tracker,
            mouse_enabled: true,
            selected_column: 0,
            mouse_position: (0, 0),
            ai_api_key: String::new(),
            ai_model: "gpt-3.5-turbo".to_string(),
            ai_models: vec![
                "gpt-3.5-turbo".to_string(),
                "gpt-4".to_string(),
                "gpt-4-turbo".to_string(),
                "claude-3-sonnet".to_string(),
                "claude-3-opus".to_string(),
            ],
            ai_model_index: 0,
            chart_mode: ChartMode::BarChart,
            chart_modes: vec![
                ChartMode::BarChart,
                ChartMode::PieChart,
                ChartMode::Gauge,
                ChartMode::Sparkline,
            ],
            active_charts: 2,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // 设置终端
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        // 恢复终端
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            match self.input_mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        self.should_quit = true;
                                    }
                                    KeyCode::Char('h') | KeyCode::F(1) => {
                                        self.show_help = !self.show_help;
                                    }
                                    KeyCode::Char('1') => self.current_tab = TabIndex::Dashboard,
                                    KeyCode::Char('2') => self.current_tab = TabIndex::Analytics,
                                    KeyCode::Char('3') => self.current_tab = TabIndex::Settings,
                                    KeyCode::Tab => self.next_tab(),
                                    KeyCode::BackTab => self.previous_tab(),
                                    KeyCode::Char('v') => self.cycle_view_mode(),
                                    KeyCode::Char('s') => self.cycle_sort(),
                                    KeyCode::Char('r') => self.reverse_sort(),
                                    KeyCode::Up => self.previous_item(),
                                    KeyCode::Down => self.next_item(),
                                    KeyCode::Left => self.previous_column(),
                                    KeyCode::Right => self.next_column(),
                                    KeyCode::Enter => self.handle_enter(),
                                    KeyCode::Char('c') => self.cycle_chart_mode(),
                                    KeyCode::Char('m') => self.toggle_mouse(),
                                    KeyCode::Char('+') => self.increase_chart_count(),
                                    KeyCode::Char('-') => self.decrease_chart_count(),
                                    _ => {}
                                },
                                InputMode::EditingApiKey => match key.code {
                                    KeyCode::Enter => {
                                        self.ai_api_key = self.input_buffer.clone();
                                        self.input_buffer.clear();
                                        self.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Esc => {
                                        self.input_buffer.clear();
                                        self.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Char(c) => {
                                        self.input_buffer.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        self.input_buffer.pop();
                                    }
                                    _ => {}
                                },
                                InputMode::EditingModel => match key.code {
                                    KeyCode::Enter => {
                                        if self.ai_model_index < self.ai_models.len() {
                                            self.ai_model =
                                                self.ai_models[self.ai_model_index].clone();
                                        }
                                        self.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Esc => {
                                        self.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Up => {
                                        if self.ai_model_index > 0 {
                                            self.ai_model_index -= 1;
                                        }
                                    }
                                    KeyCode::Down => {
                                        if self.ai_model_index < self.ai_models.len() - 1 {
                                            self.ai_model_index += 1;
                                        }
                                    }
                                    _ => {}
                                },
                            }
                        }
                    }
                    Event::Mouse(mouse) => {
                        if self.mouse_enabled {
                            self.handle_mouse_event(mouse);
                        }
                    }
                    _ => {}
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let size = f.area();

        // 创建主布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题和标签页
                Constraint::Min(0),    // 主内容
                Constraint::Length(3), // 状态栏
            ])
            .split(size);

        // 渲染标题和标签页
        self.render_header(f, chunks[0]);

        // 渲染主内容
        if self.show_help {
            self.render_help(f, chunks[1]);
        } else {
            match self.current_tab {
                TabIndex::Dashboard => self.render_dashboard(f, chunks[1]),
                TabIndex::Analytics => self.render_analytics(f, chunks[1]),
                TabIndex::Settings => self.render_settings(f, chunks[1]),
            }
        }

        // 渲染状态栏
        self.render_status_bar(f, chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let titles = vec!["仪表盘", "数据分析", "设置"];
        let index = match self.current_tab {
            TabIndex::Dashboard => 0,
            TabIndex::Analytics => 1,
            TabIndex::Settings => 2,
        };

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("TimeTracker 统计"),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .select(index);

        f.render_widget(tabs, area);
    }

    fn render_dashboard(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // 总体统计
                Constraint::Length(8), // 当前活动
                Constraint::Min(0),    // 图表区域
            ])
            .split(area);

        // 总体统计
        self.render_total_stats(f, chunks[0]);

        // 当前活动
        self.render_current_activity(f, chunks[1]);

        // 多图表显示
        self.render_multi_charts(f, chunks[2]);
    }

    fn render_analytics(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 视图模式选择
                Constraint::Min(0),    // 数据表格
            ])
            .split(area);

        // 视图模式选择
        self.render_view_mode_selector(f, chunks[0]);

        // 根据视图模式显示不同的数据
        match self.view_mode {
            ViewMode::Applications => self.render_applications_table(f, chunks[1]),
            ViewMode::Windows => self.render_windows_table(f, chunks[1]),
            ViewMode::Recent => self.render_recent_table(f, chunks[1]),
        }
    }

    fn render_view_mode_selector(&self, f: &mut Frame, area: Rect) {
        let titles = vec!["应用程序", "窗口", "最近活动"];
        let index = match self.view_mode {
            ViewMode::Applications => 0,
            ViewMode::Windows => 1,
            ViewMode::Recent => 2,
        };

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("数据视图 (按 v 切换)"),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .select(index);

        f.render_widget(tabs, area);
    }

    fn render_multi_charts(&mut self, f: &mut Frame, area: Rect) {
        let chart_count = self.active_charts.clamp(1, 4);

        match chart_count {
            1 => {
                self.render_single_chart(f, area, 0);
            }
            2 => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

                self.render_single_chart(f, chunks[0], 0);
                self.render_single_chart(f, chunks[1], 1);
            }
            3 => {
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

                self.render_single_chart(f, main_chunks[0], 0);

                let bottom_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main_chunks[1]);

                self.render_single_chart(f, bottom_chunks[0], 1);
                self.render_single_chart(f, bottom_chunks[1], 2);
            }
            4 => {
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

                let top_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main_chunks[0]);

                let bottom_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main_chunks[1]);

                self.render_single_chart(f, top_chunks[0], 0);
                self.render_single_chart(f, top_chunks[1], 1);
                self.render_single_chart(f, bottom_chunks[0], 2);
                self.render_single_chart(f, bottom_chunks[1], 3);
            }
            _ => {}
        }
    }

    fn render_single_chart(&mut self, f: &mut Frame, area: Rect, chart_index: usize) {
        if chart_index >= self.chart_modes.len() {
            return;
        }

        match self.chart_modes[chart_index] {
            ChartMode::BarChart => self.render_bar_chart(f, area),
            ChartMode::PieChart => self.render_pie_chart(f, area, "应用使用分布", "应用程序"),
            ChartMode::Gauge => self.render_gauge_chart(f, area),
            ChartMode::Sparkline => self.render_sparkline_chart(f, area),
        }
    }

    fn render_total_stats(&self, f: &mut Frame, area: Rect) {
        let total_time = self.tracker.get_total_time();
        let total_activities = self.tracker.get_activities().len();
        let apps_count = self.tracker.get_activities_by_app().len();

        let hours = total_time / 3600;
        let minutes = (total_time % 3600) / 60;
        let seconds = total_time % 60;

        let text = vec![
            Line::from(vec![
                Span::styled("总追踪时间: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}h {}m {}s", hours, minutes, seconds),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("总活动数: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    total_activities.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("应用程序数: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    apps_count.to_string(),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("总体统计"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_current_activity(&self, f: &mut Frame, area: Rect) {
        let text = if let Some(current) = &self.tracker.current_activity {
            let duration = (Utc::now() - current.start_time).num_seconds() as u64;
            let hours = duration / 3600;
            let minutes = (duration % 3600) / 60;
            let seconds = duration % 60;

            vec![
                Line::from(vec![
                    Span::styled("应用: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        &current.app_name,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("窗口: ", Style::default().fg(Color::Cyan)),
                    Span::styled(&current.window_title, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("持续时间: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{}h {}m {}s", hours, minutes, seconds),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("开始时间: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        current
                            .start_time
                            .with_timezone(&Local)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("进程ID: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        current.process_id.to_string(),
                        Style::default().fg(Color::Gray),
                    ),
                ]),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "当前没有活动",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "请确保 TimeTracker 守护进程正在运行",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(vec![Span::styled(
                    "使用 'timetracker start' 启动监控",
                    Style::default().fg(Color::Cyan),
                )]),
            ]
        };

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("当前活动"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_applications_table(&mut self, f: &mut Frame, area: Rect) {
        let stats = self.tracker.get_statistics();
        let mut app_stats: HashMap<String, u64> = HashMap::new();

        // 按应用程序聚合统计
        for (key, duration) in stats {
            let app_name = key.split(" - ").next().unwrap_or("Unknown").to_string();
            *app_stats.entry(app_name).or_insert(0) += duration;
        }

        // 创建可排序的项目列表
        let mut sortable_apps: Vec<SortableItem<(&String, &u64)>> = app_stats
            .iter()
            .enumerate()
            .map(|(i, item)| SortableItem::new(item, i))
            .collect();

        // 排序逻辑
        match (self.sort_by, self.sort_order) {
            (SortBy::Duration, SortOrder::Descending) => {
                sortable_apps.sort_by(|a, b| {
                    b.item
                        .1
                        .cmp(a.item.1)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
            (SortBy::Duration, SortOrder::Ascending) => {
                sortable_apps.sort_by(|a, b| {
                    a.item
                        .1
                        .cmp(b.item.1)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
            (SortBy::AppName, SortOrder::Descending) => {
                sortable_apps.sort_by(|a, b| {
                    b.item
                        .0
                        .cmp(a.item.0)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
            (SortBy::AppName, SortOrder::Ascending) => {
                sortable_apps.sort_by(|a, b| {
                    a.item
                        .0
                        .cmp(b.item.0)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
            _ => {
                sortable_apps.sort_by(|a, b| {
                    b.item
                        .1
                        .cmp(a.item.1)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
        }

        let total_time = self.tracker.get_total_time().max(1);
        let rows: Vec<Row> = sortable_apps
            .iter()
            .map(|sortable_item| {
                let (app_name, duration) = sortable_item.item;
                let hours = *duration / 3600;
                let minutes = (*duration % 3600) / 60;
                let seconds = *duration % 60;

                Row::new(vec![
                    Cell::from(app_name.as_str()),
                    Cell::from(format!("{}h {}m {}s", hours, minutes, seconds)),
                    Cell::from(format!(
                        "{:.1}%",
                        (*duration * 100) as f64 / total_time as f64
                    )),
                ])
            })
            .collect();

        let sort_indicator = match self.sort_order {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        };

        // 创建表头，高亮当前排序列
        let header_cells = ["应用程序", "使用时间", "占比"];
        let header_row = Row::new(
            header_cells
                .iter()
                .enumerate()
                .map(|(i, &text)| {
                    let style = if i == self.selected_column {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                    } else {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    };
                    Cell::from(text).style(style)
                })
                .collect::<Vec<_>>(),
        )
        .bottom_margin(1);

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ],
        )
        .header(header_row)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "应用程序统计 (按{} {} 排序)",
            match self.sort_by {
                SortBy::Duration => "使用时间",
                SortBy::AppName => "应用名称",
                _ => "使用时间",
            },
            sort_indicator
        )))
        .column_spacing(1)
        .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_windows_table(&mut self, f: &mut Frame, area: Rect) {
        let stats = self.tracker.get_statistics();
        let mut window_stats: HashMap<String, u64> = HashMap::new();

        // 按窗口聚合统计
        for (key, duration) in stats {
            if let Some(window_name) = key.split(" - ").nth(1) {
                *window_stats.entry(window_name.to_string()).or_insert(0) += duration;
            }
        }

        // 创建可排序的项目列表
        let mut sortable_windows: Vec<SortableItem<(&String, &u64)>> = window_stats
            .iter()
            .enumerate()
            .map(|(i, item)| SortableItem::new(item, i))
            .collect();

        // 排序逻辑
        match (self.sort_by, self.sort_order) {
            (SortBy::Duration, SortOrder::Descending) => {
                sortable_windows.sort_by(|a, b| {
                    b.item
                        .1
                        .cmp(a.item.1)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
            (SortBy::Duration, SortOrder::Ascending) => {
                sortable_windows.sort_by(|a, b| {
                    a.item
                        .1
                        .cmp(b.item.1)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
            (SortBy::AppName, SortOrder::Descending) => {
                sortable_windows.sort_by(|a, b| {
                    b.item
                        .0
                        .cmp(a.item.0)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
            (SortBy::AppName, SortOrder::Ascending) => {
                sortable_windows.sort_by(|a, b| {
                    a.item
                        .0
                        .cmp(b.item.0)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
            _ => {
                sortable_windows.sort_by(|a, b| {
                    b.item
                        .1
                        .cmp(a.item.1)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                });
            }
        }

        let total_time = self.tracker.get_total_time().max(1);
        let rows: Vec<Row> = sortable_windows
            .iter()
            .map(|sortable_item| {
                let (window_name, duration) = sortable_item.item;
                let hours = *duration / 3600;
                let minutes = (*duration % 3600) / 60;
                let seconds = *duration % 60;

                Row::new(vec![
                    Cell::from(window_name.as_str()),
                    Cell::from(format!("{}h {}m {}s", hours, minutes, seconds)),
                    Cell::from(format!(
                        "{:.1}%",
                        (*duration * 100) as f64 / total_time as f64
                    )),
                ])
            })
            .collect();

        let sort_indicator = match self.sort_order {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        };

        // 创建表头，高亮当前排序列
        let header_cells = ["窗口", "使用时间", "占比"];
        let header_row = Row::new(
            header_cells
                .iter()
                .enumerate()
                .map(|(i, &text)| {
                    let style = if i == self.selected_column {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                    } else {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    };
                    Cell::from(text).style(style)
                })
                .collect::<Vec<_>>(),
        )
        .bottom_margin(1);

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ],
        )
        .header(header_row)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "窗口统计 (按{} {} 排序)",
            match self.sort_by {
                SortBy::Duration => "使用时间",
                SortBy::AppName => "窗口名称",
                _ => "使用时间",
            },
            sort_indicator
        )))
        .column_spacing(1)
        .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_pie_chart(&mut self, f: &mut Frame, area: Rect, title: &str, data_type: &str) {
        let stats = self.tracker.get_statistics();
        let mut data: HashMap<String, u64> = HashMap::new();

        // 根据数据类型聚合统计
        match data_type {
            "applications" => {
                for (key, duration) in stats {
                    let app_name = key.split(" - ").next().unwrap_or("Unknown").to_string();
                    *data.entry(app_name).or_insert(0) += duration;
                }
            }
            "windows" => {
                for (key, duration) in stats {
                    if let Some(window_name) = key.split(" - ").nth(1) {
                        *data.entry(window_name.to_string()).or_insert(0) += duration;
                    }
                }
            }
            _ => {
                for (key, duration) in stats {
                    *data.entry(key).or_insert(0) += duration;
                }
            }
        }

        // 获取前10个最大的数据项
        let mut sorted_data: Vec<_> = data.iter().collect();
        sorted_data.sort_by(|a, b| b.1.cmp(a.1));
        sorted_data.truncate(10);

        let total_time = self.tracker.get_total_time().max(1);

        let mut lines = vec![
            Line::from(vec![Span::styled(
                format!("{} 分布", title),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        for (i, (name, duration)) in sorted_data.iter().enumerate() {
            let percentage = (**duration * 100) as f64 / total_time as f64;
            let color = match i % 8 {
                0 => Color::Red,
                1 => Color::Green,
                2 => Color::Blue,
                3 => Color::Yellow,
                4 => Color::Magenta,
                5 => Color::Cyan,
                6 => Color::White,
                _ => Color::Gray,
            };

            let hours = **duration / 3600;
            let minutes = (**duration % 3600) / 60;

            let bar_length = (percentage / 100.0 * 30.0) as usize;
            let bar = "█".repeat(bar_length);

            lines.push(Line::from(vec![
                Span::styled(format!("{:2}. ", i + 1), Style::default().fg(Color::Gray)),
                Span::styled(format!("{:<20}", name), Style::default().fg(Color::White)),
                Span::styled(format!("{:<30}", bar), Style::default().fg(color)),
                Span::styled(
                    format!(" {}h{}m ({:.1}%)", hours, minutes, percentage),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_bar_chart(&mut self, f: &mut Frame, area: Rect) {
        let activities = self.tracker.get_activities_by_app();
        let mut app_durations: Vec<(String, u64)> = activities
            .iter()
            .map(|(name, records)| {
                let total_duration: u64 = records.iter().map(|r| r.duration).sum();
                (name.clone(), total_duration)
            })
            .collect();

        app_durations.sort_by(|a, b| b.1.cmp(&a.1));
        app_durations.truncate(10);

        let data: Vec<(&str, u64)> = app_durations
            .iter()
            .map(|(name, duration)| (name.as_str(), *duration))
            .collect();

        let chart = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("应用使用时间"))
            .data(&data)
            .bar_width(3)
            .bar_gap(1)
            .value_style(Style::default().fg(Color::Yellow))
            .label_style(Style::default().fg(Color::White));

        f.render_widget(chart, area);
    }

    fn render_gauge_chart(&mut self, f: &mut Frame, area: Rect) {
        let activities = self.tracker.get_activities_by_app();
        if let Some((app_name, records)) = activities.iter().next() {
            let app_duration: u64 = records.iter().map(|r| r.duration).sum();
            let total_time: u64 = activities
                .iter()
                .flat_map(|(_, records)| records.iter())
                .map(|r| r.duration)
                .sum();

            let ratio = if total_time > 0 {
                (app_duration as f64 / total_time as f64) * 100.0
            } else {
                0.0
            };

            let hours = app_duration / 3600;
            let minutes = (app_duration % 3600) / 60;

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("最常用应用: {} ({}h{}m)", app_name, hours, minutes)),
                )
                .gauge_style(Style::default().fg(Color::Green))
                .percent(ratio as u16)
                .label(format!("{:.1}%", ratio));

            f.render_widget(gauge, area);
        }
    }

    fn render_sparkline_chart(&mut self, f: &mut Frame, area: Rect) {
        // 生成24小时的活动数据
        let mut hourly_data = vec![0u64; 24];
        let activities = self.tracker.get_recent_activities(1000);

        for activity in activities {
            let hour = activity.start_time.with_timezone(&Local).hour() as usize;
            hourly_data[hour] += activity.duration;
        }

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("24小时活动趋势"),
            )
            .data(&hourly_data)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(sparkline, area);
    }

    fn render_recent_table(&mut self, f: &mut Frame, area: Rect) {
        let recent_activities = self.tracker.get_recent_activities(20);
        let mut sortable_activities: Vec<SortableItem<&crate::tracker::ActivityRecord>> =
            recent_activities
                .into_iter()
                .enumerate()
                .map(|(index, item)| SortableItem {
                    item,
                    original_index: index,
                })
                .collect();

        match (self.sort_by, self.sort_order) {
            (SortBy::StartTime, SortOrder::Descending) => sortable_activities.sort_by(|a, b| {
                b.item
                    .start_time
                    .cmp(&a.item.start_time)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            (SortBy::StartTime, SortOrder::Ascending) => sortable_activities.sort_by(|a, b| {
                a.item
                    .start_time
                    .cmp(&b.item.start_time)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            (SortBy::Duration, SortOrder::Descending) => sortable_activities.sort_by(|a, b| {
                b.item
                    .duration
                    .cmp(&a.item.duration)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            (SortBy::Duration, SortOrder::Ascending) => sortable_activities.sort_by(|a, b| {
                a.item
                    .duration
                    .cmp(&b.item.duration)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            (SortBy::AppName, SortOrder::Descending) => sortable_activities.sort_by(|a, b| {
                b.item
                    .app_name
                    .cmp(&a.item.app_name)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            (SortBy::AppName, SortOrder::Ascending) => sortable_activities.sort_by(|a, b| {
                a.item
                    .app_name
                    .cmp(&b.item.app_name)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            _ => sortable_activities.sort_by(|a, b| {
                b.item
                    .start_time
                    .cmp(&a.item.start_time)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
        }

        let rows: Vec<Row> = sortable_activities
            .iter()
            .map(|sortable_item| {
                let activity = sortable_item.item;
                let hours = activity.duration / 3600;
                let minutes = (activity.duration % 3600) / 60;
                let seconds = activity.duration % 60;

                Row::new(vec![
                    Cell::from(
                        activity
                            .start_time
                            .with_timezone(&Local)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string(),
                    ),
                    Cell::from(activity.app_name.as_str()),
                    Cell::from(activity.window_title.as_str()),
                    Cell::from(format!("{}h {}m {}s", hours, minutes, seconds)),
                ])
            })
            .collect();

        let sort_indicator = match self.sort_order {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        };

        let header_cells = ["开始时间", "应用程序", "窗口标题", "持续时间"];
        let header_row = Row::new(
            header_cells
                .iter()
                .enumerate()
                .map(|(i, &text)| {
                    let style = if i == self.selected_column {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                    } else {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    };
                    Cell::from(text).style(style)
                })
                .collect::<Vec<_>>(),
        )
        .bottom_margin(1);

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30),
                Constraint::Percentage(25),
                Constraint::Percentage(30),
                Constraint::Percentage(15),
            ],
        )
        .header(header_row)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "最近活动 (按{} {} 排序)",
            match self.sort_by {
                SortBy::StartTime => "开始时间",
                SortBy::Duration => "持续时间",
                SortBy::AppName => "应用名称",
                _ => "开始时间",
            },
            sort_indicator
        )))
        .column_spacing(1)
        .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "快捷键:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  q/Esc",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("     - 退出程序", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  h/F1",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("      - 显示/隐藏帮助", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  1-4",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("        - 切换标签页", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Tab",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("        - 下一个标签页", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Shift+Tab",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  - 上一个标签页", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  s",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "          - 切换排序方式",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  r",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "          - 反转排序顺序",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  ↑/↓",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("        - 上下移动", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("      - 选择项目", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "标签页:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  概览",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "        - 总体统计和当前活动",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  应用程序",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "    - 按应用程序分组的统计",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  窗口",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "        - 详细的窗口使用统计",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  最近活动",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("    - 最近的活动记录", Style::default().fg(Color::White)),
            ]),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("帮助"))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    fn render_settings(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Length(7),
                Constraint::Min(0),
            ])
            .split(area);

        // 鼠标设置
        let mouse_text = vec![Line::from(vec![
            Span::styled("鼠标支持: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if self.mouse_enabled {
                    "启用"
                } else {
                    "禁用"
                },
                Style::default()
                    .fg(if self.mouse_enabled {
                        Color::Green
                    } else {
                        Color::Red
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (按 m 切换)", Style::default().fg(Color::Gray)),
        ])];

        let mouse_paragraph = Paragraph::new(mouse_text)
            .block(Block::default().borders(Borders::ALL).title("鼠标设置"))
            .alignment(Alignment::Left);

        f.render_widget(mouse_paragraph, chunks[0]);

        // AI 配置
        let ai_title = match self.input_mode {
            InputMode::EditingApiKey => "AI 配置 (编辑API Key - 按 ESC 退出)",
            InputMode::EditingModel => "AI 配置 (选择模型 - 按 ESC 退出)",
            _ => "AI 配置 (按 Enter 编辑API Key, 按 Tab 选择模型)",
        };

        let mut ai_text = vec![
            Line::from(vec![
                Span::styled("API Key: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if self.input_mode == InputMode::EditingApiKey {
                        &self.input_buffer
                    } else if self.ai_api_key.is_empty() {
                        "未设置"
                    } else {
                        "已设置"
                    },
                    Style::default().fg(if self.ai_api_key.is_empty() {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("模型: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    &self.ai_models[self.ai_model_index],
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        if self.input_mode == InputMode::EditingModel {
            ai_text.push(Line::from(""));
            ai_text.push(Line::from(vec![Span::styled(
                "可用模型 (↑↓选择, Enter确认):",
                Style::default().fg(Color::Gray),
            )]));
            for (i, model) in self.ai_models.iter().enumerate() {
                let style = if i == self.ai_model_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };
                ai_text.push(Line::from(vec![Span::styled(
                    format!("  {}", model),
                    style,
                )]));
            }
        }

        let ai_paragraph = Paragraph::new(ai_text)
            .block(Block::default().borders(Borders::ALL).title(ai_title))
            .alignment(Alignment::Left);

        f.render_widget(ai_paragraph, chunks[1]);

        // 快捷键说明
        let shortcuts_text = vec![
            Line::from(vec![Span::styled(
                "快捷键:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled(
                    "  m",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" - 切换鼠标支持", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  v",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" - 切换视图模式", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  +/-",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" - 增加/减少图表数量", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  c",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" - 循环图表类型", Style::default().fg(Color::White)),
            ]),
        ];

        let shortcuts_paragraph = Paragraph::new(shortcuts_text)
            .block(Block::default().borders(Borders::ALL).title("快捷键"))
            .alignment(Alignment::Left);

        f.render_widget(shortcuts_paragraph, chunks[2]);
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = vec![Line::from(vec![
            Span::styled("按 ", Style::default().fg(Color::Gray)),
            Span::styled(
                "h",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 查看帮助 | ", Style::default().fg(Color::Gray)),
            Span::styled(
                "q",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 退出 | ", Style::default().fg(Color::Gray)),
            Span::styled(
                "s",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 排序 | ", Style::default().fg(Color::Gray)),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 反转", Style::default().fg(Color::Gray)),
        ])];

        let paragraph = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabIndex::Dashboard => TabIndex::Analytics,
            TabIndex::Analytics => TabIndex::Settings,
            TabIndex::Settings => TabIndex::Dashboard,
        };
        self.reset_selection();
    }

    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabIndex::Dashboard => TabIndex::Settings,
            TabIndex::Analytics => TabIndex::Dashboard,
            TabIndex::Settings => TabIndex::Analytics,
        };
        self.reset_selection();
    }

    fn cycle_chart_mode(&mut self) {
        let current_index = self
            .chart_modes
            .iter()
            .position(|&mode| mode == self.chart_mode)
            .unwrap_or(0);
        let next_index = (current_index + 1) % self.chart_modes.len();
        self.chart_mode = self.chart_modes[next_index];
    }

    fn cycle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Applications => ViewMode::Windows,
            ViewMode::Windows => ViewMode::Recent,
            ViewMode::Recent => ViewMode::Applications,
        };
        self.reset_selection();
    }

    fn increase_chart_count(&mut self) {
        if self.active_charts < 4 {
            self.active_charts += 1;
        }
    }

    fn decrease_chart_count(&mut self) {
        if self.active_charts > 1 {
            self.active_charts -= 1;
        }
    }

    fn toggle_mouse(&mut self) {
        self.mouse_enabled = !self.mouse_enabled;
    }

    #[allow(dead_code)]
    fn select_column(&mut self, column: usize) {
        self.selected_column = column;
    }

    fn previous_column(&mut self) {
        if self.selected_column > 0 {
            self.selected_column -= 1;
        }
    }

    fn next_column(&mut self) {
        let max_columns = match self.view_mode {
            ViewMode::Applications => 2, // 应用程序、使用时间、占比
            ViewMode::Windows => 3,      // 应用程序、窗口标题、使用时间、占比
            ViewMode::Recent => 3,       // 开始时间、应用程序、窗口标题、持续时间
        };
        if self.selected_column < max_columns {
            self.selected_column += 1;
        }
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        self.mouse_position = (mouse.column, mouse.row);

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // 处理标签页点击
                if mouse.row == 0 {
                    // 标签页区域
                    if mouse.column < 10 {
                        self.current_tab = TabIndex::Dashboard;
                    } else if mouse.column < 20 {
                        self.current_tab = TabIndex::Analytics;
                    } else if mouse.column < 30 {
                        self.current_tab = TabIndex::Settings;
                    }
                    self.reset_selection();
                }

                // 处理表格头部点击进行排序
                if mouse.row == 2
                    && (self.current_tab == TabIndex::Dashboard
                        || self.current_tab == TabIndex::Analytics)
                {
                    // 表格头部点击排序
                    let column = mouse.column / 20; // 假设每列宽度为20
                    match self.view_mode {
                        ViewMode::Applications => {
                            self.sort_by = match column {
                                0 => SortBy::AppName,
                                1 => SortBy::Duration,
                                _ => self.sort_by,
                            };
                        }
                        ViewMode::Windows => {
                            self.sort_by = match column {
                                0 => SortBy::AppName,
                                1 => SortBy::WindowTitle,
                                2 => SortBy::Duration,
                                _ => self.sort_by,
                            };
                        }
                        ViewMode::Recent => {
                            self.sort_by = match column {
                                0 => SortBy::StartTime,
                                1 => SortBy::AppName,
                                2 => SortBy::Duration,
                                _ => self.sort_by,
                            };
                        }
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                self.next_item();
            }
            MouseEventKind::ScrollUp => {
                self.previous_item();
            }
            _ => {}
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_by = match self.view_mode {
            ViewMode::Applications => match self.sort_by {
                SortBy::Duration => SortBy::AppName,
                SortBy::AppName => SortBy::Duration,
                _ => SortBy::Duration,
            },
            ViewMode::Windows => match self.sort_by {
                SortBy::Duration => SortBy::WindowTitle,
                SortBy::WindowTitle => SortBy::Duration,
                _ => SortBy::Duration,
            },
            ViewMode::Recent => match self.sort_by {
                SortBy::StartTime => SortBy::Duration,
                SortBy::Duration => SortBy::AppName,
                SortBy::AppName => SortBy::StartTime,
                _ => SortBy::StartTime,
            },
        };
    }

    fn reverse_sort(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
    }

    fn next_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                let max_items = match self.view_mode {
                    ViewMode::Applications => self.tracker.get_activities_by_app().len(),
                    ViewMode::Windows => self.tracker.get_statistics().len(),
                    ViewMode::Recent => self.tracker.get_recent_activities(20).len(),
                };
                if i >= max_items.saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    let max_items = match self.view_mode {
                        ViewMode::Applications => self.tracker.get_activities_by_app().len(),
                        ViewMode::Windows => self.tracker.get_statistics().len(),
                        ViewMode::Recent => self.tracker.get_recent_activities(20).len(),
                    };
                    max_items.saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn select_item(&mut self) {
        // 这里可以添加选择项目的逻辑，比如显示详细信息
    }

    fn handle_enter(&mut self) {
        match self.input_mode {
            InputMode::Normal => {
                // 在正常模式下，Enter键可以选择当前项目
                self.select_item();
            }
            InputMode::EditingApiKey => {
                // 在编辑API密钥模式下，Enter键确认输入
                self.input_mode = InputMode::Normal;
            }
            InputMode::EditingModel => {
                // 在编辑模型模式下，Enter键确认选择
                self.input_mode = InputMode::Normal;
            }
        }
    }

    fn reset_selection(&mut self) {
        self.table_state.select(None);
        self.list_state.select(None);
    }
}
