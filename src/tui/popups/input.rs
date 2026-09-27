use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Clear, Block, BorderType, Borders, Paragraph},
    style::Style,
};

use crossterm::event::KeyCode;

use super::{PopupWidget, PopupAction};

pub struct InputPopup {
    pub input: String,
    pub title: String,
    pub color: ratatui::style::Color,
}

impl PopupWidget for InputPopup {
    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        let popup_width = 30.min(area.width);
        let popup_height = 3.min(area.height);

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

        let text_size = popup_area.width
            .saturating_sub(2) as usize;

        let mut text = self.input.clone();

        while text.chars().count() >= text_size {
            text.remove(0);
        }

        let popup = Paragraph::new(format!("{}|", text))
            .block(
                Block::default()
                    .title(self.title.as_str())
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.color))
                    .title_style(Style::default().fg(self.color))
            );

        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
    }

    fn handle_input(&mut self, key: KeyCode) -> PopupAction {
        match key {
            KeyCode::Char(c) => {
                self.input.push(c);
                PopupAction::None
            }
            KeyCode::Backspace => {
                self.input.pop();
                PopupAction::None
            }

            KeyCode::Enter => PopupAction::Input(self.input.clone()),

            KeyCode::Esc => PopupAction::Close,

            _ => PopupAction::None 
        }
    }
}

