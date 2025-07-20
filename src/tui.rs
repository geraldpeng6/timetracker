use crate::tracker::TimeTracker;
use anyhow::Result;
use chrono::{Local, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState,
        Tabs, Wrap,
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
                if let Event::Key(key) = event::read()? {
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
                            KeyCode::Tab => self.next_tab(),
                            KeyCode::BackTab => self.previous_tab(),
                            KeyCode::Char('s') => self.cycle_sort(),
                            KeyCode::Char('r') => self.reverse_sort(),
                            KeyCode::Up => self.previous_item(),
                            KeyCode::Down => self.next_item(),
                            KeyCode::Enter => self.select_item(),
                            _ => {}
                        }
                    }
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
            }
        }

        // 渲染状态栏
        self.render_status_bar(f, chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let titles = vec!["概览", "应用程序", "窗口", "最近活动"];
        let index = match self.current_tab {
            TabIndex::Overview => 0,
            TabIndex::Applications => 1,
            TabIndex::Windows => 2,
            TabIndex::Recent => 3,
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
                            .format("%H:%M:%S")
                            .to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
            ]
        } else {
            vec![Line::from(Span::styled(
                "当前没有活动",
                Style::default().fg(Color::Gray),
            ))]
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

        let mut sorted_apps: Vec<_> = app_stats.iter().collect();
        match (self.sort_by, self.sort_order) {
            (SortBy::Duration, SortOrder::Descending) => sorted_apps.sort_by(|a, b| b.1.cmp(a.1)),
            (SortBy::Duration, SortOrder::Ascending) => sorted_apps.sort_by(|a, b| a.1.cmp(b.1)),
            (SortBy::AppName, SortOrder::Descending) => sorted_apps.sort_by(|a, b| b.0.cmp(a.0)),
            (SortBy::AppName, SortOrder::Ascending) => sorted_apps.sort_by(|a, b| a.0.cmp(b.0)),
            _ => sorted_apps.sort_by(|a, b| b.1.cmp(a.1)),
        }

        let rows: Vec<Row> = sorted_apps
            .iter()
            .map(|(app_name, duration)| {
                let hours = *duration / 3600;
                let minutes = (*duration % 3600) / 60;
                let seconds = *duration % 60;

                Row::new(vec![
                    Cell::from(app_name.as_str()),
                    Cell::from(format!("{}h {}m {}s", hours, minutes, seconds)),
                    Cell::from(format!(
                        "{}%",
                        (*duration * 100) / self.tracker.get_total_time().max(1)
                    )),
                ])
            })
            .collect();

        let sort_indicator = match self.sort_order {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        };

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(20),
            ],
        )
        .header(
            Row::new(vec!["应用程序", "使用时间", "占比"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
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
        let mut sorted_windows: Vec<_> = stats.iter().collect();

        match (self.sort_by, self.sort_order) {
            (SortBy::Duration, SortOrder::Descending) => {
                sorted_windows.sort_by(|a, b| b.1.cmp(a.1))
            }
            (SortBy::Duration, SortOrder::Ascending) => sorted_windows.sort_by(|a, b| a.1.cmp(b.1)),
            (SortBy::WindowTitle, SortOrder::Descending) => {
                sorted_windows.sort_by(|a, b| b.0.cmp(a.0))
            }
            (SortBy::WindowTitle, SortOrder::Ascending) => {
                sorted_windows.sort_by(|a, b| a.0.cmp(b.0))
            }
            _ => sorted_windows.sort_by(|a, b| b.1.cmp(a.1)),
        }

        let rows: Vec<Row> = sorted_windows
            .iter()
            .map(|(window_name, duration)| {
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
                        "{}%",
                        (*duration * 100) / self.tracker.get_total_time().max(1)
                    )),
                ])
            })
            .collect();

        let sort_indicator = match self.sort_order {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        };

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
                Constraint::Percentage(15),
            ],
        )
        .header(
            Row::new(vec!["应用程序", "窗口标题", "使用时间", "占比"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
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

        let rows: Vec<Row> = recent_activities
            .iter()
            .map(|activity| {
                let hours = activity.duration / 3600;
                let minutes = (activity.duration % 3600) / 60;
                let seconds = activity.duration % 60;

                Row::new(vec![
                    Cell::from(
                        activity
                            .start_time
                            .with_timezone(&Local)
                            .format("%m-%d %H:%M")
                            .to_string(),
                    ),
                    Cell::from(activity.app_name.as_str()),
                    Cell::from(activity.window_title.as_str()),
                    Cell::from(format!("{}h {}m {}s", hours, minutes, seconds)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(20),
            ],
        )
        .header(
            Row::new(vec!["开始时间", "应用程序", "窗口标题", "持续时间"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(Block::default().borders(Borders::ALL).title("最近活动"))
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
            TabIndex::Recent => TabIndex::Overview,
        };
        self.reset_selection();
    }

    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabIndex::Overview => TabIndex::Recent,
            TabIndex::Applications => TabIndex::Overview,
            TabIndex::Windows => TabIndex::Applications,
            TabIndex::Recent => TabIndex::Windows,
        };
        self.reset_selection();
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