use crate::ui::components::{InputMode, TabIndex};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// 事件处理结果
#[derive(Debug, Clone)]
pub enum EventResult {
    Continue,
    Quit,        // 退出TUI界面（返回到监控模式）
    QuitProgram, // 退出整个程序
    SwitchTab(TabIndex),
    ToggleViewMode,
    ToggleSortBy,
    ToggleSortOrder,
    ToggleChartMode,
    ToggleTimeRange,
    StartEditing(InputMode),
    StopEditing,
    SaveInput,
    TestConnection,
    SaveConfig,
    RefreshData,
    ShowHelp,
    // New AI configuration events
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    SelectProvider(usize),
    SelectModel(usize),
    ToggleAdvanced,
    DeleteActivity(usize),
    ConfirmDelete,
    CancelDelete,
    NextPage,
    PrevPage,
    ToggleBarChart,
    ToggleSparkline,
    TogglePieChart,
    ToggleTimeline,
}

/// 事件处理器
pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// 处理键盘事件
    pub fn handle_key_event(
        &self,
        key: KeyEvent,
        current_tab: TabIndex,
        input_mode: InputMode,
        selected_row: usize,
    ) -> EventResult {
        match input_mode {
            InputMode::Normal => self.handle_normal_mode(key, current_tab, selected_row),
            _ => self.handle_editing_mode(key, input_mode),
        }
    }

    /// 处理正常模式下的键盘事件
    fn handle_normal_mode(
        &self,
        key: KeyEvent,
        current_tab: TabIndex,
        selected_row: usize,
    ) -> EventResult {
        // 全局快捷键（在所有标签页都可用）
        match key.code {
            // 退出TUI界面快捷键
            KeyCode::Char('q') | KeyCode::Esc => EventResult::Quit,

            // 退出整个程序快捷键
            KeyCode::Char('Q') => EventResult::QuitProgram, // Shift+Q 退出整个程序
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EventResult::QuitProgram // Ctrl+C 退出整个程序
            }

            // 刷新数据快捷键
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EventResult::RefreshData
            }
            KeyCode::Char('r') => EventResult::RefreshData,
            KeyCode::F(5) => EventResult::RefreshData,

            // 帮助快捷键
            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EventResult::ShowHelp
            }
            KeyCode::Char('?') => EventResult::ShowHelp,
            KeyCode::F(1) => EventResult::ShowHelp,

            // 标签页切换快捷键
            KeyCode::Tab => self.handle_tab_switch(current_tab),
            KeyCode::BackTab => self.handle_tab_switch_reverse(current_tab),
            KeyCode::Char('1') => EventResult::SwitchTab(TabIndex::Dashboard),
            KeyCode::Char('2') => EventResult::SwitchTab(TabIndex::Activities),
            _ => self.handle_tab_specific_keys(key, current_tab, selected_row),
        }
    }

    /// 处理编辑模式下的键盘事件
    fn handle_editing_mode(&self, key: KeyEvent, _input_mode: InputMode) -> EventResult {
        match key.code {
            KeyCode::Char('q') => EventResult::Quit, // 在编辑模式下q退出TUI
            KeyCode::Char('Q') => EventResult::QuitProgram, // Shift+Q 退出整个程序
            KeyCode::Esc => EventResult::StopEditing,
            KeyCode::Enter => EventResult::SaveInput,
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EventResult::SaveConfig
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EventResult::TestConnection
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EventResult::QuitProgram // Ctrl+C 退出整个程序
            }
            _ => EventResult::Continue,
        }
    }

    /// 处理标签页切换
    fn handle_tab_switch(&self, current_tab: TabIndex) -> EventResult {
        let next_tab = match current_tab {
            TabIndex::Dashboard => TabIndex::Activities,
            TabIndex::Activities => TabIndex::Dashboard,
        };
        EventResult::SwitchTab(next_tab)
    }

    /// 处理反向标签页切换
    fn handle_tab_switch_reverse(&self, current_tab: TabIndex) -> EventResult {
        let prev_tab = match current_tab {
            TabIndex::Dashboard => TabIndex::Activities,
            TabIndex::Activities => TabIndex::Dashboard,
        };
        EventResult::SwitchTab(prev_tab)
    }

    /// 处理特定标签页的键盘事件
    fn handle_tab_specific_keys(
        &self,
        key: KeyEvent,
        current_tab: TabIndex,
        selected_row: usize,
    ) -> EventResult {
        match current_tab {
            TabIndex::Activities => {
                match key.code {
                    KeyCode::Char('v') => EventResult::ToggleViewMode,
                    KeyCode::Char('s') => EventResult::ToggleSortBy,
                    KeyCode::Char('o') => EventResult::ToggleSortOrder,
                    KeyCode::Char('f') => EventResult::ToggleTimeRange,
                    // 使用更通用的键位替代PgUp/PgDn，确保Mac兼容性
                    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        EventResult::PrevPage
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        EventResult::NextPage
                    }
                    // 保留原有的方向键和其他键位
                    KeyCode::Left => EventResult::PrevPage,
                    KeyCode::Right => EventResult::NextPage,
                    KeyCode::Up => EventResult::NavigateUp,
                    KeyCode::Down => EventResult::NavigateDown,
                    KeyCode::Delete | KeyCode::Backspace => {
                        EventResult::DeleteActivity(selected_row)
                    }
                    _ => EventResult::Continue,
                }
            }
            TabIndex::Dashboard => match key.code {
                KeyCode::Char('c') => EventResult::ToggleChartMode,
                KeyCode::Char('f') => EventResult::ToggleTimeRange,
                KeyCode::Char('b') => EventResult::ToggleBarChart,
                KeyCode::Char('l') => EventResult::ToggleSparkline,
                KeyCode::Char('p') => EventResult::TogglePieChart,
                KeyCode::Char('t') => EventResult::ToggleTimeline,
                _ => EventResult::Continue,
            },
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
