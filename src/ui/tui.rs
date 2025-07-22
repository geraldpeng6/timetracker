use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::ai::manager::AIConfigManager;
use crate::config::manager::ConfigManager;
use crate::ui::components::{InputMode, RecentActivityItem, UiState};
use crate::ui::data::DataManager;
use crate::ui::events::{EventHandler, EventResult};
use crate::ui::renderer::Renderer;
use crate::ui::themes::Theme;

/// TUI 应用程序
pub struct TuiApp {
    ui_state: UiState,
    config_manager: ConfigManager,
    ai_manager: AIConfigManager,
    theme: Theme,
    event_handler: EventHandler,
    data_manager: DataManager,
    should_quit: bool,
    should_quit_program: bool, // 是否退出整个程序
    last_tick: Instant,
    tick_counter: u32,
    data_initialized: bool,
}

impl TuiApp {
    /// 创建新的 TUI 应用程序
    pub fn new(data_file: String) -> anyhow::Result<Self> {
        let config_manager = ConfigManager::new()?;
        let ai_manager = AIConfigManager::new()?;
        let theme = Theme::default();
        let event_handler = EventHandler::new();
        let data_manager = DataManager::new(data_file)?;

        Ok(Self {
            ui_state: UiState::default(),
            config_manager,
            ai_manager,
            theme,
            event_handler,
            data_manager,
            should_quit: false,
            should_quit_program: false,
            last_tick: Instant::now(),
            tick_counter: 0,
            data_initialized: false,
        })
    }

    /// 运行 TUI 应用程序
    pub fn run(&mut self) -> anyhow::Result<()> {
        // 设置终端
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // 清屏并隐藏光标
        terminal.clear()?;
        terminal.hide_cursor()?;

        // 运行主循环
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

    /// 运行应用程序主循环
    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        let tick_rate = Duration::from_millis(50); // 减少到50ms，提高响应性

        loop {
            // 渲染界面
            terminal.draw(|f| {
                self.render::<B>(f);
            })?;

            // 处理事件
            let timeout = tick_rate
                .checked_sub(self.last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_event(key),
                    Event::Mouse(mouse) => self.handle_mouse_event(mouse),
                    _ => {}
                }
            }

            // 定时更新
            if self.last_tick.elapsed() >= tick_rate {
                self.on_tick();
                self.last_tick = Instant::now();
            }

            // 检查是否退出
            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// 渲染界面
    fn render<B: Backend>(&mut self, f: &mut ratatui::Frame) {
        // 延迟初始化数据，避免在TUI启动时阻塞
        if !self.data_initialized {
            if let Err(e) = self.data_manager.initialize_data() {
                // 如果数据初始化失败，继续运行但显示空数据
                log::warn!("数据初始化失败: {}", e);
                // 不要因为数据初始化失败就退出TUI
            }
            self.data_initialized = true;
        }

        // 获取数据
        let app_items = self
            .data_manager
            .get_app_table_data(self.ui_state.time_range);
        let window_items = self.data_manager.get_window_data(self.ui_state.time_range);
        let mut recent_activities = self.data_manager.get_recent_activities(50);

        // 如果有当前活动，将其添加到最近活动列表的开头
        if let Some(current_activity) = self.data_manager.get_current_activity() {
            let current_duration =
                (chrono::Utc::now() - current_activity.start_time).num_seconds() as u64;
            let current_item = RecentActivityItem {
                app_name: current_activity.app_name.clone(),
                window_title: current_activity.window_title.clone(),
                start_time: current_activity.start_time,
                end_time: None, // 当前活动还没有结束
                duration: current_duration,
            };
            recent_activities.insert(0, current_item);
        }

        // 使用新的排序功能获取统一活动数据（已经包含当前活动）
        let unified_activities = self.data_manager.get_unified_activities_sorted(
            self.ui_state.time_range,
            self.ui_state.sort_by,
            self.ui_state.sort_order,
        );

        // 更新分页信息
        self.ui_state
            .pagination
            .set_total_items(unified_activities.len());

        let statistics = (); // 已删除统计功能

        // 创建渲染器并渲染
        let renderer = Renderer::new(&self.theme);
        renderer.render::<B>(
            f,
            &self.ui_state,
            &app_items,
            &window_items,
            &recent_activities,
            &unified_activities,
            &statistics,
        );
    }

    /// 处理鼠标事件
    fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        if !self.ui_state.mouse_enabled {
            return;
        }

        use crossterm::event::{MouseButton, MouseEventKind};

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // 处理左键点击
                self.handle_mouse_click(mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Right) => {
                // 处理右键点击（显示上下文菜单或帮助）
                self.handle_right_click(mouse.column, mouse.row);
            }
            MouseEventKind::ScrollUp => {
                // 向上滚动
                self.handle_scroll_up();
            }
            MouseEventKind::ScrollDown => {
                // 向下滚动
                self.handle_scroll_down();
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                // 处理拖拽（可用于选择范围）
                self.handle_mouse_drag(mouse.column, mouse.row);
            }
            _ => {}
        }

        // 鼠标事件处理后，确保界面状态正确
        // 如果不在编辑模式，确保输入模式为正常模式，避免键盘功能失效
        if !matches!(
            self.ui_state.input_mode,
            crate::ui::components::InputMode::EditingApiKey
                | crate::ui::components::InputMode::EditingEndpoint
                | crate::ui::components::InputMode::EditingTemperature
                | crate::ui::components::InputMode::EditingMaxTokens
                | crate::ui::components::InputMode::EditingModel
                | crate::ui::components::InputMode::Search
        ) {
            self.ui_state.input_mode = crate::ui::components::InputMode::Normal;
        }
    }

    /// 处理鼠标点击
    fn handle_mouse_click(&mut self, x: u16, y: u16) {
        // 实现基于坐标的点击处理

        // 标签页区域 (y = 0-2)
        if y <= 2 {
            self.handle_tab_click(x);
            return;
        }

        // 根据当前标签页处理点击
        match self.ui_state.current_tab {
            crate::ui::components::TabIndex::Dashboard => {
                self.handle_dashboard_click(x, y);
            }
            crate::ui::components::TabIndex::Activities => {
                self.handle_activities_click(x, y);
            }
        }
    }

    /// 处理标签页点击
    fn handle_tab_click(&mut self, x: u16) {
        // 更精确的标签页点击检测
        // 标签页标题: "概览", "活动", "统计", "AI配置"
        // 根据实际的标签页布局计算

        // 标签页的大致位置（基于ratatui的Tabs组件布局）
        // 每个标签页包含标题文字加上一些间距
        let tab_positions = [
            (0, 8),  // "概览" (0-8)
            (8, 16), // "活动" (8-16)
        ];

        for (i, (start, end)) in tab_positions.iter().enumerate() {
            if x >= *start && x < *end {
                match i {
                    0 => self.ui_state.current_tab = crate::ui::components::TabIndex::Dashboard,
                    1 => self.ui_state.current_tab = crate::ui::components::TabIndex::Activities,
                    _ => {}
                }
                break;
            }
        }

        // 重置选中行，因为切换了标签页
        self.ui_state.selected_row = 0;
        self.ui_state.selected_column = 0;

        // 确保输入模式保持正常
        self.ui_state.input_mode = crate::ui::components::InputMode::Normal;

        log::debug!(
            "鼠标点击切换标签页: x={}, 当前标签: {:?}",
            x,
            self.ui_state.current_tab
        );
    }

    /// 处理概览页面点击
    fn handle_dashboard_click(&mut self, _x: u16, _y: u16) {
        // 概览页面的点击处理
        // 可以实现图表切换、时间范围选择等
    }

    /// 处理活动页面点击
    fn handle_activities_click(&mut self, _x: u16, y: u16) {
        // 活动列表的点击处理
        // 布局：标签页(3行) + 控制栏(3行) + 表格标题(1行) + 数据行
        // 根据实际渲染器布局计算正确的偏移量
        const TAB_HEIGHT: u16 = 3; // 标签页高度
        const CONTROL_HEIGHT: u16 = 3; // 控制栏高度
        const TABLE_HEADER_HEIGHT: u16 = 2; // 表格标题高度（包括边框）
        const TOTAL_HEADER_HEIGHT: u16 = TAB_HEIGHT + CONTROL_HEIGHT + TABLE_HEADER_HEIGHT;

        if y > TOTAL_HEADER_HEIGHT {
            let clicked_row = (y - TOTAL_HEADER_HEIGHT - 1) as usize;

            // 获取当前页面的活动数据
            let unified_activities = self.data_manager.get_unified_activities_sorted(
                self.ui_state.time_range,
                self.ui_state.sort_by,
                self.ui_state.sort_order,
            );

            // 计算当前页面的项目数量
            let page_start = self.ui_state.pagination.start_index();
            let page_end = self
                .ui_state
                .pagination
                .end_index()
                .min(unified_activities.len());
            let items_in_page = if page_end > page_start {
                page_end - page_start
            } else {
                0
            };

            // 检查点击的行是否在有效范围内
            if clicked_row < items_in_page && (page_start + clicked_row) < unified_activities.len()
            {
                self.ui_state.selected_row = clicked_row;

                // 确保输入模式保持正常，避免键盘功能失效
                self.ui_state.input_mode = crate::ui::components::InputMode::Normal;

                log::debug!(
                    "鼠标点击选择行: {} (页面内第{}行, 全局第{}行)",
                    clicked_row,
                    clicked_row,
                    page_start + clicked_row
                );
            }
        }
    }

    /// 处理对话框键盘事件
    fn handle_dialog_key_event(&mut self, key: crossterm::event::KeyEvent) {
        use crate::ui::components::DialogType;
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Char('q') => {
                // 在对话框模式下，q键先关闭对话框，再次按q才退出应用
                if self.ui_state.dialog_state.is_visible {
                    self.ui_state.dialog_state.hide();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Esc => {
                self.cancel_delete();
            }
            KeyCode::Enter => match self.ui_state.dialog_state.dialog_type {
                DialogType::Confirmation => {
                    self.confirm_delete();
                }
                DialogType::QuitTui => {
                    if self.ui_state.dialog_state.selected_option == 0 {
                        // 确认退出TUI
                        self.should_quit = true;
                    }
                    self.ui_state.dialog_state.hide();
                }
                DialogType::QuitProgram => {
                    if self.ui_state.dialog_state.selected_option == 0 {
                        // 确认退出程序
                        self.should_quit_program = true;
                        self.should_quit = true;
                    }
                    self.ui_state.dialog_state.hide();
                }
                DialogType::Information | DialogType::Warning | DialogType::Error => {
                    self.ui_state.dialog_state.hide();
                }
                _ => {}
            },
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                if matches!(
                    self.ui_state.dialog_state.dialog_type,
                    DialogType::Confirmation | DialogType::QuitTui | DialogType::QuitProgram
                ) {
                    self.ui_state.dialog_state.toggle_option();
                }
            }
            _ => {}
        }
    }

    /// 处理键盘事件
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        // 如果有对话框显示，优先处理对话框事件
        if self.ui_state.dialog_state.is_visible {
            self.handle_dialog_key_event(key);
            return;
        }

        // 在编辑模式下处理字符输入
        if self.ui_state.input_mode.is_editing() {
            match key.code {
                crossterm::event::KeyCode::Char(c) => {
                    // 添加字符到输入缓冲区
                    self.ui_state.input_buffer.push(c);
                    return;
                }
                crossterm::event::KeyCode::Backspace => {
                    // 删除最后一个字符
                    self.ui_state.input_buffer.pop();
                    return;
                }
                _ => {
                    // 其他键交给事件处理器处理
                }
            }
        }

        let result = self.event_handler.handle_key_event(
            key,
            self.ui_state.current_tab,
            self.ui_state.input_mode,
            self.ui_state.selected_row,
        );

        match result {
            EventResult::Quit => self.ui_state.dialog_state.show_quit_tui_confirmation(),
            EventResult::QuitProgram => self.ui_state.dialog_state.show_quit_program_confirmation(),
            EventResult::SwitchTab(tab) => self.ui_state.current_tab = tab,
            EventResult::ToggleViewMode => self.ui_state.toggle_view_mode(),
            EventResult::ToggleSortBy => self.ui_state.toggle_sort_by(),
            EventResult::ToggleSortOrder => self.ui_state.toggle_sort_order(),
            EventResult::ToggleChartMode => self.ui_state.toggle_chart_mode(),
            EventResult::ToggleTimeRange => self.ui_state.toggle_time_range(),
            EventResult::StartEditing(mode) => self.ui_state.input_mode = mode,
            EventResult::StopEditing => self.ui_state.input_mode = InputMode::Normal,
            EventResult::SaveInput => {}      // 已删除AI配置功能
            EventResult::TestConnection => {} // 已删除AI配置功能
            EventResult::SaveConfig => self.save_config(),
            EventResult::RefreshData => self.refresh_data(),
            EventResult::ShowHelp => self.show_help(),
            EventResult::NavigateUp => self.navigate_up(),
            EventResult::NavigateDown => self.navigate_down(),
            EventResult::NavigateLeft => self.navigate_left(),
            EventResult::NavigateRight => self.navigate_right(),
            EventResult::SelectProvider(_index) => {} // 已删除AI配置功能
            EventResult::SelectModel(_index) => {}    // 已删除AI配置功能
            EventResult::ToggleAdvanced => {}         // 已删除高级设置功能
            EventResult::DeleteActivity(index) => self.delete_activity(index),
            EventResult::ConfirmDelete => self.confirm_delete(),
            EventResult::CancelDelete => self.cancel_delete(),
            EventResult::NextPage => self.next_page(),
            EventResult::PrevPage => self.prev_page(),
            EventResult::ToggleBarChart => self.toggle_bar_chart(),
            EventResult::ToggleSparkline => self.toggle_sparkline(),
            EventResult::TogglePieChart => self.toggle_pie_chart(),
            EventResult::ToggleTimeline => self.toggle_timeline(),
            EventResult::Continue => {}
        }
    }

    /// 保存配置
    fn save_config(&mut self) {
        if let Err(e) = self.config_manager.save() {
            eprintln!("保存配置失败: {}", e);
        }
        if let Err(e) = self.ai_manager.save() {
            eprintln!("保存 AI 配置失败: {}", e);
        }
    }

    /// 刷新数据
    fn refresh_data(&mut self) {
        if let Err(e) = self.data_manager.refresh() {
            eprintln!("刷新数据失败: {}", e);
        }
    }

    /// 显示帮助
    fn show_help(&mut self) {
        self.ui_state.show_help = !self.ui_state.show_help;
    }

    /// 定时更新
    fn on_tick(&mut self) {
        // 降低刷新频率，避免界面闪动
        self.tick_counter += 1;
        if self.tick_counter >= 20 {
            // 每1秒刷新一次 (20 * 50ms = 1000ms)，平衡实时性和稳定性
            self.tick_counter = 0;
            if let Err(e) = self.data_manager.refresh() {
                // 静默处理错误，避免在TUI中显示错误信息
                log::debug!("定时刷新数据失败: {}", e);
            }
        }
    }

    /// 向上导航
    fn navigate_up(&mut self) {
        // 常规导航
        if self.ui_state.selected_row > 0 {
            self.ui_state.selected_row -= 1;
        } else if self.ui_state.pagination.can_go_prev() {
            // 如果在第一行且可以上一页，则跳到上一页的最后一行
            self.prev_page();
            self.ui_state.selected_row = self.ui_state.pagination.items_per_page - 1;
        }
    }

    /// 向下导航
    fn navigate_down(&mut self) {
        // 常规导航
        let items_in_current_page =
            self.ui_state.pagination.end_index() - self.ui_state.pagination.start_index();
        if self.ui_state.selected_row + 1 < items_in_current_page {
            self.ui_state.selected_row += 1;
        } else if self.ui_state.pagination.can_go_next() {
            // 如果在最后一行且可以下一页，则跳到下一页的第一行
            self.next_page();
            self.ui_state.selected_row = 0;
        }
    }

    /// 向左导航
    fn navigate_left(&mut self) {
        // 常规导航
        if self.ui_state.selected_column > 0 {
            self.ui_state.selected_column -= 1;
        }
    }

    /// 向右导航
    fn navigate_right(&mut self) {
        // 常规导航
        self.ui_state.selected_column += 1;
    }

    /// 删除活动
    fn delete_activity(&mut self, _index: usize) {
        // 计算实际的活动索引（考虑分页）
        let actual_index = self.ui_state.pagination.start_index() + self.ui_state.selected_row;

        // 获取统一活动数据以找到对应的原始活动
        let unified_activities = self.data_manager.get_unified_activities_sorted(
            self.ui_state.time_range,
            self.ui_state.sort_by,
            self.ui_state.sort_order,
        );

        if actual_index < unified_activities.len() {
            let activity = &unified_activities[actual_index];
            // 显示确认对话框 - 只删除这一条记录
            self.ui_state.dialog_state.show_confirmation(
                "确认删除",
                &format!(
                    "确定要删除这条记录吗？\n应用: {}\n窗口: {}\n持续时间: {}秒\n\n此操作不可撤销。",
                    activity.app_name,
                    activity.window_title,
                    activity.total_duration
                ),
            );
        }
    }

    /// 确认删除
    fn confirm_delete(&mut self) {
        if self.ui_state.dialog_state.selected_option == 0 {
            // 用户选择确认删除
            let actual_index = self.ui_state.pagination.start_index() + self.ui_state.selected_row;
            let unified_activities = self.data_manager.get_unified_activities_sorted(
                self.ui_state.time_range,
                self.ui_state.sort_by,
                self.ui_state.sort_order,
            );

            if actual_index < unified_activities.len() {
                let activity = &unified_activities[actual_index];

                // 实际删除逻辑：删除该应用和窗口的最近一条活动记录
                match self.data_manager.delete_recent_activity_by_app_window(
                    &activity.app_name,
                    &activity.window_title,
                ) {
                    Ok(deleted) => {
                        if deleted {
                            log::info!(
                                "已删除活动记录: {} - {}",
                                activity.app_name,
                                activity.window_title
                            );

                            // 显示成功消息
                            self.ui_state.dialog_state.show_info(
                                "删除成功",
                                &format!(
                                    "已成功删除活动记录:\n应用: {}\n窗口: {}",
                                    activity.app_name, activity.window_title
                                ),
                            );

                            // 刷新数据以更新界面
                            if let Err(e) = self.data_manager.refresh() {
                                log::error!("删除后刷新数据失败: {}", e);
                            }
                        } else {
                            // 没有找到匹配的记录
                            self.ui_state
                                .dialog_state
                                .show_error("删除失败", "未找到匹配的活动记录。");
                        }
                    }
                    Err(e) => {
                        log::error!("删除活动失败: {}", e);
                        self.ui_state
                            .dialog_state
                            .show_error("删除失败", &format!("删除活动时发生错误: {}", e));
                    }
                }
            }
        } else {
            // 用户选择取消
            self.cancel_delete();
        }
    }

    /// 取消删除
    fn cancel_delete(&mut self) {
        self.ui_state.dialog_state.hide();
    }

    /// 下一页
    fn next_page(&mut self) {
        self.ui_state.pagination.next_page();
        // 重置选中行到页面开始
        self.ui_state.selected_row = 0;
    }

    /// 上一页
    fn prev_page(&mut self) {
        self.ui_state.pagination.prev_page();
        // 重置选中行到页面开始
        self.ui_state.selected_row = 0;
    }

    /// 切换柱状图显示
    fn toggle_bar_chart(&mut self) {
        self.ui_state.chart_config.show_bar_chart = !self.ui_state.chart_config.show_bar_chart;
    }

    /// 切换趋势图显示
    fn toggle_sparkline(&mut self) {
        self.ui_state.chart_config.show_sparkline = !self.ui_state.chart_config.show_sparkline;
    }

    /// 切换饼图显示
    fn toggle_pie_chart(&mut self) {
        self.ui_state.chart_config.show_pie_chart = !self.ui_state.chart_config.show_pie_chart;
    }

    /// 切换时间线显示
    fn toggle_timeline(&mut self) {
        self.ui_state.chart_config.show_timeline = !self.ui_state.chart_config.show_timeline;
    }

    /// 处理右键点击
    fn handle_right_click(&mut self, _x: u16, _y: u16) {
        // 显示帮助信息
        self.ui_state.show_help = !self.ui_state.show_help;
    }

    /// 处理向上滚动
    fn handle_scroll_up(&mut self) {
        match self.ui_state.current_tab {
            crate::ui::components::TabIndex::Activities => {
                // 在列表中向上滚动
                if self.ui_state.selected_row > 0 {
                    self.ui_state.selected_row -= 1;
                } else {
                    // 到达顶部，切换到上一页
                    self.prev_page();
                }
            }
            _ => {
                // 在其他标签页中，向上滚动可以切换到上一个标签页
                let result = self.event_handler.handle_key_event(
                    crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::BackTab,
                        crossterm::event::KeyModifiers::empty(),
                    ),
                    self.ui_state.current_tab,
                    self.ui_state.input_mode,
                    self.ui_state.selected_row,
                );
                match result {
                    crate::ui::events::EventResult::SwitchTab(tab) => {
                        self.ui_state.current_tab = tab
                    }
                    _ => {}
                }
            }
        }
    }

    /// 处理向下滚动
    fn handle_scroll_down(&mut self) {
        match self.ui_state.current_tab {
            crate::ui::components::TabIndex::Activities => {
                // 在列表中向下滚动
                let max_items = self.ui_state.pagination.items_per_page;
                if self.ui_state.selected_row < max_items.saturating_sub(1) {
                    self.ui_state.selected_row += 1;
                } else {
                    // 到达底部，切换到下一页
                    self.next_page();
                }
            }
            _ => {
                // 在其他标签页中，向下滚动可以切换到下一个标签页
                let result = self.event_handler.handle_key_event(
                    crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Tab,
                        crossterm::event::KeyModifiers::empty(),
                    ),
                    self.ui_state.current_tab,
                    self.ui_state.input_mode,
                    self.ui_state.selected_row,
                );
                match result {
                    crate::ui::events::EventResult::SwitchTab(tab) => {
                        self.ui_state.current_tab = tab
                    }
                    _ => {}
                }
            }
        }
    }

    /// 处理鼠标拖拽
    fn handle_mouse_drag(&mut self, _x: u16, _y: u16) {
        // 目前暂不实现拖拽功能，可以在未来添加
        // 可能的用途：选择时间范围、调整列宽等
    }

    /// 检查是否应该退出整个程序
    pub fn should_quit_program(&self) -> bool {
        self.should_quit_program
    }
}
