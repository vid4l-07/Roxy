use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Clear, Block, BorderType, Borders, Paragraph},
    style::Style,
    Frame,
};

use crossterm::event::KeyCode;


pub enum PopupAction {
    Close,
    None
}

pub enum Popup {
    Text(TextPopup),
}

impl Popup {
    pub fn render(&self, frame: &mut ratatui::Frame) {
        match self {
            Popup::Text(popup) => popup.render(frame),
        }
    }
    pub fn handle_input(&self, key: KeyCode) -> PopupAction{
        match self {
            Popup::Text(popup) => popup.handle_input(key),
        }
    }
}



pub struct TextPopup {
    pub message: String,
    pub title: String,
    pub color: ratatui::style::Color,
}

impl TextPopup {
    pub fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        let text_width = self
            .message
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);

        let text_height = self.message.lines().count().max(1);

        let popup_width = text_width as u16 + 4 + 2;

        let popup_height = text_height as u16 + 2 + 2;

        let popup_width = popup_width.min(area.width);
        let popup_height = popup_height.min(area.height);

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

        let popup = Paragraph::new(self.message.clone())
            .block(
                Block::default()
                .title(self.title.clone())
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(self.color))
                .title_style(Style::default().fg(self.color))
                .padding(ratatui::widgets::Padding::new(2, 2, 1, 1)),
            );

        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);

    }
    
    pub fn handle_input(&self, key: KeyCode) -> PopupAction {
        match key {
            KeyCode::Enter => {
                PopupAction::Close
            }
            _ => { PopupAction::None }
        }
    }

}
