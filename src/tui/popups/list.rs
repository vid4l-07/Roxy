use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Clear, Block, BorderType, Borders, Paragraph, Padding, Wrap},
    style::Style,
};

use crossterm::event::KeyCode;

use super::{PopupWidget, PopupAction};

pub struct ListPopup {
    pub commands: Vec<String>,
    pub title: String,
    pub color: ratatui::style::Color,
    pub scroll: u16,
}

impl PopupWidget for ListPopup {
    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        let popup_width = 30.min(area.width);
        let popup_height = (self.commands.len() as u16 + 2).min(area.height - 4);

        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(popup_height),
                Constraint::Fill(1),
            ])
            .split(area)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(popup_width),
                Constraint::Fill(1),
            ])
            .split(popup_area)[1];

        let text = self.commands.join("\n");

        let popup = Paragraph::new(text)
            .block(
                Block::default()
                .title(self.title.as_str())
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(self.color))
                .title_style(Style::default().fg(self.color))
                .padding(Padding::new(1, 1, 0, 0))
            ).wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
    }

    fn handle_input(&mut self, key: KeyCode) -> PopupAction {
        match key {

            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
                PopupAction::None
            }

            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll = self.scroll.saturating_add(1);
                PopupAction::None
            }

            KeyCode::Esc | KeyCode::Enter => PopupAction::Close,
            _ => PopupAction::None,
        }
    }
}

