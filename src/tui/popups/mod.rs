use crossterm::event::KeyCode;
pub mod text;
pub mod input;
pub mod list;

pub enum PopupAction {
    Close,
    Input(String),
    None
}

pub enum Popup {
    Text(text::TextPopup),
    Input(input::InputPopup),
    List(list::ListPopup),
}

impl Popup {
    pub fn render(&self, frame: &mut ratatui::Frame) {
        match self {
            Popup::Text(popup) => popup.render(frame),
            Popup::Input(popup) => popup.render(frame),
            Popup::List(popup) => popup.render(frame),
        }
    }
    pub fn handle_input(&mut self, key: KeyCode) -> PopupAction{
        match self {
            Popup::Text(popup) => popup.handle_input(key),
            Popup::Input(popup) => popup.handle_input(key),
            Popup::List(popup) => popup.handle_input(key),
        }
    }
}

pub trait PopupWidget {
    fn render(&self, frame: &mut ratatui::Frame);
    fn handle_input(&mut self, key: KeyCode) -> PopupAction;
}

