use crate::tracker::TimeTracker;
use anyhow::Result;
use chrono::{Local, Timelike, Utc};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEvent,
        MouseEventKind,
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
        BarChart, Block, Borders, Cell, Clear, Gauge, List, ListItem, ListState, Paragraph, Row,
        Sparkline, Table, TableState, Tabs, Wrap,
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
    Overview,
    Applications,
    Windows,
    Recent,
    Charts,
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

pub struct TuiApp {
    pub should_quit: bool,
    pub current_tab: TabIndex,
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
    /// AI 配置相关状态
    pub ai_config_editing: bool,
    pub ai_api_key: String,
    pub ai_model: String,
    /// 图表显示模式
    pub chart_mode: ChartMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartMode {
    BarChart,
    Gauge,
    Sparkline,
}

impl TuiApp {
    pub fn new(mut tracker: TimeTracker) -> Result<Self> {
        tracker.load_data()?;

        Ok(Self {
            should_quit: false,
            current_tab: TabIndex::Overview,
            sort_by: SortBy::Duration,
            sort_order: SortOrder::Descending,
            table_state: TableState::default(),
            list_state: ListState::default(),
            show_help: false,
            tracker,
            mouse_enabled: true,
            selected_column: 0,
            ai_config_editing: false,
            ai_api_key: String::new(),
            ai_model: "gpt-3.5-turbo".to_string(),
            chart_mode: ChartMode::BarChart,
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
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    self.should_quit = true;
                                }
                                KeyCode::Char('h') | KeyCode::F(1) => {
                                    self.show_help = !self.show_help;
                                }
                                KeyCode::Char('1') => self.current_tab = TabIndex::Overview,
                                KeyCode::Char('2') => self.current_tab = TabIndex::Applications,
                                KeyCode::Char('3') => self.current_tab = TabIndex::Windows,
                                KeyCode::Char('4') => self.current_tab = TabIndex::Recent,
                                KeyCode::Char('5') => self.current_tab = TabIndex::Charts,
                                KeyCode::Char('6') => self.current_tab = TabIndex::Settings,
                                KeyCode::Tab => self.next_tab(),
                                KeyCode::BackTab => self.previous_tab(),
                                KeyCode::Char('s') => self.cycle_sort(),
                                KeyCode::Char('r') => self.reverse_sort(),
                                KeyCode::Up => self.previous_item(),
                                KeyCode::Down => self.next_item(),
                                KeyCode::Left => self.previous_column(),
                                KeyCode::Right => self.next_column(),
                                KeyCode::Enter => self.select_item(),
                                KeyCode::Char('c') => self.cycle_chart_mode(),
                                KeyCode::Char('m') => self.toggle_mouse(),
                                _ => {}
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
                TabIndex::Overview => self.render_overview(f, chunks[1]),
                TabIndex::Applications => self.render_applications(f, chunks[1]),
                TabIndex::Windows => self.render_windows(f, chunks[1]),
                TabIndex::Recent => self.render_recent(f, chunks[1]),
                TabIndex::Charts => self.render_charts(f, chunks[1]),
                TabIndex::Settings => self.render_settings(f, chunks[1]),
            }
        }

        // 渲染状态栏
        self.render_status_bar(f, chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let titles = vec!["概览", "应用程序", "窗口", "最近活动", "图表", "设置"];
        let index = match self.current_tab {
            TabIndex::Overview => 0,
            TabIndex::Applications => 1,
            TabIndex::Windows => 2,
            TabIndex::Recent => 3,
            TabIndex::Charts => 4,
            TabIndex::Settings => 5,
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

    fn render_overview(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // 总体统计
                Constraint::Length(8), // 当前活动
                Constraint::Min(0),    // 最近使用的应用
            ])
            .split(area);

        // 总体统计
        self.render_total_stats(f, chunks[0]);

        // 当前活动
        self.render_current_activity(f, chunks[1]);

        // 最近使用的应用（前5个）
        self.render_top_apps(f, chunks[2], 5);
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

    fn render_top_apps(&mut self, f: &mut Frame, area: Rect, limit: usize) {
        let stats = self.tracker.get_statistics();
        let mut app_stats: HashMap<String, u64> = HashMap::new();

        // 按应用程序聚合统计
        for (key, duration) in stats {
            let app_name = key.split(" - ").next().unwrap_or("Unknown").to_string();
            *app_stats.entry(app_name).or_insert(0) += duration;
        }

        let mut sorted_apps: Vec<_> = app_stats.iter().collect();
        sorted_apps.sort_by(|a, b| b.1.cmp(a.1));

        let items: Vec<ListItem> = sorted_apps
            .iter()
            .take(limit)
            .enumerate()
            .map(|(i, (app_name, duration))| {
                let hours = *duration / 3600;
                let minutes = (*duration % 3600) / 60;
                let seconds = *duration % 60;

                let color = match i {
                    0 => Color::Yellow,
                    1 => Color::Green,
                    2 => Color::Cyan,
                    _ => Color::White,
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Gray)),
                    Span::styled(
                        (*app_name).clone(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" - {}h {}m {}s", hours, minutes, seconds),
                        Style::default().fg(Color::Gray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("最常用应用 (前{})", limit)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_applications(&mut self, f: &mut Frame, area: Rect) {
        let stats = self.tracker.get_statistics();
        let mut app_stats: HashMap<String, u64> = HashMap::new();

        // 按应用程序聚合统计
        for (key, duration) in stats {
            let app_name = key.split(" - ").next().unwrap_or("Unknown").to_string();
            *app_stats.entry(app_name).or_insert(0) += duration;
        }

        // 创建可排序的项目列表，包含原始索引以确保稳定排序
        let mut sortable_apps: Vec<SortableItem<(&String, &u64)>> = app_stats
            .iter()
            .enumerate()
            .map(|(i, item)| SortableItem::new(item, i))
            .collect();

        // 稳定排序：首先按主要条件排序，相同时按原始索引排序
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

    fn render_windows(&mut self, f: &mut Frame, area: Rect) {
        let stats = self.tracker.get_statistics();
        let mut sortable_windows: Vec<SortableItem<(&String, &u64)>> = stats
            .iter()
            .enumerate()
            .map(|(index, item)| SortableItem {
                item,
                original_index: index,
            })
            .collect();

        match (self.sort_by, self.sort_order) {
            (SortBy::Duration, SortOrder::Descending) => sortable_windows.sort_by(|a, b| {
                b.item
                    .1
                    .cmp(a.item.1)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            (SortBy::Duration, SortOrder::Ascending) => sortable_windows.sort_by(|a, b| {
                a.item
                    .1
                    .cmp(b.item.1)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            (SortBy::WindowTitle, SortOrder::Descending) => sortable_windows.sort_by(|a, b| {
                b.item
                    .0
                    .cmp(a.item.0)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            (SortBy::WindowTitle, SortOrder::Ascending) => sortable_windows.sort_by(|a, b| {
                a.item
                    .0
                    .cmp(b.item.0)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
            _ => sortable_windows.sort_by(|a, b| {
                b.item
                    .1
                    .cmp(a.item.1)
                    .then_with(|| a.original_index.cmp(&b.original_index))
            }),
        }

        let total_time = self.tracker.get_total_time().max(1);
        let rows: Vec<Row> = sortable_windows
            .iter()
            .map(|sortable_item| {
                let (window_name, duration) = sortable_item.item;
                let hours = *duration / 3600;
                let minutes = (*duration % 3600) / 60;
                let seconds = *duration % 60;

                let parts: Vec<&str> = window_name.split(" - ").collect();
                let app_name = parts.first().unwrap_or(&"Unknown");
                let window_title = parts.get(1).unwrap_or(&"Unknown");

                Row::new(vec![
                    Cell::from(*app_name),
                    Cell::from(*window_title),
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
        let header_cells = ["应用程序", "窗口标题", "使用时间", "占比"];
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
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
                Constraint::Percentage(15),
            ],
        )
        .header(header_row)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "窗口统计 (按{} {} 排序)",
            match self.sort_by {
                SortBy::Duration => "使用时间",
                SortBy::WindowTitle => "窗口标题",
                _ => "使用时间",
            },
            sort_indicator
        )))
        .column_spacing(1)
        .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_recent(&mut self, f: &mut Frame, area: Rect) {
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
            _ => {
                // 默认按开始时间降序排序
                sortable_activities.sort_by(|a, b| {
                    b.item
                        .start_time
                        .cmp(&a.item.start_time)
                        .then_with(|| a.original_index.cmp(&b.original_index))
                })
            }
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

        // 创建表头，高亮当前排序列
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
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
            ],
        )
        .header(header_row)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "最近活动 (按{} {} 排序)",
            match self.sort_by {
                SortBy::StartTime => "开始时间",
                SortBy::Duration => "持续时间",
                SortBy::AppName => "应用名称",
                SortBy::WindowTitle => "窗口标题",
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

    fn render_charts(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        match self.chart_mode {
            ChartMode::BarChart => self.render_bar_chart(f, chunks[0]),
            ChartMode::Gauge => self.render_gauge_chart(f, chunks[0]),
            ChartMode::Sparkline => self.render_sparkline_chart(f, chunks[0]),
        }

        // 图表模式切换说明
        let help_text = vec![
            Line::from(vec![
                Span::styled("图表模式: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    match self.chart_mode {
                        ChartMode::BarChart => "柱状图",
                        ChartMode::Gauge => "仪表盘",
                        ChartMode::Sparkline => "趋势图",
                    },
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("按 ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "c",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" 切换图表类型", Style::default().fg(Color::Gray)),
            ]),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("图表控制"))
            .alignment(Alignment::Left);

        f.render_widget(paragraph, chunks[1]);
    }

    fn render_bar_chart(&self, f: &mut Frame, area: Rect) {
        let apps = self.tracker.get_activities_by_app();

        // 计算每个应用的总时间
        let mut app_durations: Vec<(String, u64)> = apps
            .iter()
            .map(|(app_name, activities)| {
                let total_duration: u64 = activities.iter().map(|a| a.duration).sum();
                (app_name.clone(), total_duration)
            })
            .collect();

        // 按持续时间排序
        app_durations.sort_by(|a, b| b.1.cmp(&a.1));
        app_durations.truncate(10); // 只显示前10个应用

        let data: Vec<(&str, u64)> = app_durations
            .iter()
            .map(|(name, duration)| (name.as_str(), duration / 60)) // 转换为分钟
            .collect();

        let barchart = BarChart::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("应用使用时间 (分钟)"),
            )
            .data(&data)
            .bar_width(9)
            .bar_style(Style::default().fg(Color::Cyan))
            .value_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(barchart, area);
    }

    fn render_gauge_chart(&self, f: &mut Frame, area: Rect) {
        let total_time = self.tracker.get_total_time();
        let apps = self.tracker.get_activities_by_app();

        // 计算每个应用的总时间
        let mut app_durations: Vec<(String, u64)> = apps
            .iter()
            .map(|(app_name, activities)| {
                let total_duration: u64 = activities.iter().map(|a| a.duration).sum();
                (app_name.clone(), total_duration)
            })
            .collect();

        // 按持续时间排序，获取最常用的应用
        app_durations.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((top_app, top_duration)) = app_durations.first() {
            let ratio = if total_time > 0 {
                (*top_duration as f64 / total_time as f64) * 100.0
            } else {
                0.0
            };

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("最常用应用: {}", top_app)),
                )
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(ratio as u16)
                .label(format!("{:.1}%", ratio));

            f.render_widget(gauge, area);
        }
    }

    fn render_sparkline_chart(&self, f: &mut Frame, area: Rect) {
        // 获取最近24小时的活动数据
        let recent_activities = self.tracker.get_recent_activities(100);
        let mut hourly_data = vec![0u64; 24];

        for activity in recent_activities {
            let hour = activity.start_time.with_timezone(&Local).hour() as usize;
            hourly_data[hour] += activity.duration / 60; // 转换为分钟
        }

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("24小时活动趋势 (分钟)"),
            )
            .data(&hourly_data)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(sparkline, area);
    }

    fn render_settings(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
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
        let ai_title = if self.ai_config_editing {
            "AI 配置 (编辑模式 - 按 ESC 退出)"
        } else {
            "AI 配置 (按 Enter 编辑)"
        };

        let ai_text = vec![
            Line::from(vec![
                Span::styled("API Key: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if self.ai_api_key.is_empty() {
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
                Span::styled(&self.ai_model, Style::default().fg(Color::Cyan)),
            ]),
        ];

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
                    "  c",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" - 切换图表类型", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  1-3",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" - 选择排序列", Style::default().fg(Color::White)),
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
            TabIndex::Overview => TabIndex::Applications,
            TabIndex::Applications => TabIndex::Windows,
            TabIndex::Windows => TabIndex::Recent,
            TabIndex::Recent => TabIndex::Charts,
            TabIndex::Charts => TabIndex::Settings,
            TabIndex::Settings => TabIndex::Overview,
        };
        self.reset_selection();
    }

    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabIndex::Overview => TabIndex::Settings,
            TabIndex::Applications => TabIndex::Overview,
            TabIndex::Windows => TabIndex::Applications,
            TabIndex::Recent => TabIndex::Windows,
            TabIndex::Charts => TabIndex::Recent,
            TabIndex::Settings => TabIndex::Charts,
        };
        self.reset_selection();
    }

    fn cycle_chart_mode(&mut self) {
        self.chart_mode = match self.chart_mode {
            ChartMode::BarChart => ChartMode::Gauge,
            ChartMode::Gauge => ChartMode::Sparkline,
            ChartMode::Sparkline => ChartMode::BarChart,
        };
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
        let max_columns = match self.current_tab {
            TabIndex::Applications => 2, // 应用程序、使用时间、占比
            TabIndex::Windows => 3,      // 应用程序、窗口标题、使用时间、占比
            TabIndex::Recent => 3,       // 开始时间、应用程序、窗口标题、持续时间
            _ => 0,
        };
        if self.selected_column < max_columns {
            self.selected_column += 1;
        }
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(_) => {
                // 处理鼠标点击事件
                // 这里可以添加更复杂的鼠标交互逻辑
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
        self.sort_by = match self.current_tab {
            TabIndex::Applications => match self.sort_by {
                SortBy::Duration => SortBy::AppName,
                SortBy::AppName => SortBy::Duration,
                _ => SortBy::Duration,
            },
            TabIndex::Windows => match self.sort_by {
                SortBy::Duration => SortBy::WindowTitle,
                SortBy::WindowTitle => SortBy::Duration,
                _ => SortBy::Duration,
            },
            TabIndex::Recent => match self.sort_by {
                SortBy::StartTime => SortBy::Duration,
                SortBy::Duration => SortBy::AppName,
                SortBy::AppName => SortBy::StartTime,
                _ => SortBy::StartTime,
            },
            _ => self.sort_by,
        };
    }

    fn reverse_sort(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
    }

    fn next_item(&mut self) {
        match self.current_tab {
            TabIndex::Overview => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i >= 4 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            _ => {
                let i = match self.table_state.selected() {
                    Some(i) => {
                        let max_items = match self.current_tab {
                            TabIndex::Applications => self.tracker.get_activities_by_app().len(),
                            TabIndex::Windows => self.tracker.get_statistics().len(),
                            TabIndex::Recent => self.tracker.get_recent_activities(20).len(),
                            _ => 0,
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
        }
    }

    fn previous_item(&mut self) {
        match self.current_tab {
            TabIndex::Overview => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            4
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            _ => {
                let i = match self.table_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            let max_items = match self.current_tab {
                                TabIndex::Applications => {
                                    self.tracker.get_activities_by_app().len()
                                }
                                TabIndex::Windows => self.tracker.get_statistics().len(),
                                TabIndex::Recent => self.tracker.get_recent_activities(20).len(),
                                _ => 0,
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
        }
    }

    fn select_item(&mut self) {
        // 这里可以添加选择项目的逻辑，比如显示详细信息
    }

    fn reset_selection(&mut self) {
        self.table_state.select(None);
        self.list_state.select(None);
    }
}
