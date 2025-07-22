// UI 小部件
// 提供可重用的 UI 小部件组件

use crate::ui::{components::*, themes::Theme};
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
    Frame,
};

// 已删除AIConfigWidget和ConnectionStatus

// 已删除StatsDashboardWidget

/// 上下文敏感的帮助小部件
pub struct ContextHelpWidget<'a> {
    pub theme: &'a Theme,
    pub current_tab: TabIndex,
    pub input_mode: InputMode,
    pub show_detailed: bool,
}

impl<'a> Widget for ContextHelpWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 简化的帮助小部件渲染
        let help_block = Block::default().title("帮助").borders(Borders::ALL);
        help_block.render(area, buf);
    }
}

impl<'a> ContextHelpWidget<'a> {
    pub fn new(theme: &'a Theme, current_tab: TabIndex, input_mode: InputMode) -> Self {
        Self {
            theme,
            current_tab,
            input_mode,
            show_detailed: false,
        }
    }

    pub fn render<B: Backend>(&self, f: &mut Frame, area: Rect) {
        let help_block = Block::default().title("帮助").borders(Borders::ALL);
        f.render_widget(help_block, area);
    }
}

/// 对话框小部件
pub struct DialogWidget<'a> {
    pub theme: &'a Theme,
    pub title: String,
    pub message: String,
    pub dialog_type: DialogType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogType {
    Info,
    Warning,
    Error,
    Confirm,
}

impl<'a> DialogWidget<'a> {
    pub fn new(theme: &'a Theme, title: String, message: String, dialog_type: DialogType) -> Self {
        Self {
            theme,
            title,
            message,
            dialog_type,
        }
    }

    pub fn render<B: Backend>(&self, f: &mut Frame, area: Rect) {
        // 创建居中的对话框区域
        let popup_area = self.centered_rect(60, 20, area);

        // 清除背景
        let clear = Block::default().style(self.theme.inactive_style());
        f.render_widget(clear, area);

        // 渲染对话框
        match self.dialog_type {
            DialogType::Confirm => self.render_confirmation_dialog::<B>(f, popup_area),
            DialogType::Info => self.render_info_dialog::<B>(f, popup_area),
            DialogType::Warning => self.render_warning_dialog::<B>(f, popup_area),
            DialogType::Error => self.render_error_dialog::<B>(f, popup_area),
        }
    }

    fn render_confirmation_dialog<B: Backend>(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Min(3),    // 消息
                Constraint::Length(3), // 按钮
            ])
            .split(area);

        // 标题
        let title = Paragraph::new(self.title.clone())
            .style(self.theme.warning_style())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // 消息
        let message = Paragraph::new(self.message.clone())
            .style(self.theme.table_row_style())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT));
        f.render_widget(message, chunks[1]);

        // 按钮
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        let confirm_button = Paragraph::new("确认 (Enter)")
            .style(self.theme.success_style())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(confirm_button, button_chunks[0]);

        let cancel_button = Paragraph::new("取消 (Esc)")
            .style(self.theme.table_row_style())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(cancel_button, button_chunks[1]);
    }

    fn render_info_dialog<B: Backend>(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Min(3),    // 消息
                Constraint::Length(3), // 按钮
            ])
            .split(area);

        // 标题
        let title = Paragraph::new(self.title.clone())
            .style(self.theme.success_style())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // 消息
        let message = Paragraph::new(self.message.clone())
            .style(self.theme.table_row_style())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT));
        f.render_widget(message, chunks[1]);

        // 确认按钮
        let ok_button = Paragraph::new("确定 (Enter)")
            .style(self.theme.selected_style())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(ok_button, chunks[2]);
    }

    fn render_warning_dialog<B: Backend>(&self, f: &mut Frame, area: Rect) {
        // 类似于信息对话框，但使用警告样式
        self.render_info_dialog::<B>(f, area);
    }

    fn render_error_dialog<B: Backend>(&self, f: &mut Frame, area: Rect) {
        // 类似于信息对话框，但使用错误样式
        self.render_info_dialog::<B>(f, area);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
