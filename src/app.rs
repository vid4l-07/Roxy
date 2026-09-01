
use crate::{http, repeater, popups};

pub enum Screen {
    Proxy,
    Repeater
}

pub struct App {
    pub screen: Screen,

    pub intercepted_request: Option<http::Request>,
    pub intercept: bool,

    pub repeaters: Vec<repeater::Repeater>,
    pub repeaters_names: Vec<String>,
    pub selected_repeater: usize,

    pub popups: Vec<popups::Popup>,
}

impl App {
    pub fn new() -> App{
        App { 
            screen: Screen::Proxy,

            intercepted_request: None,
            intercept: false,

            repeaters: Vec::new(),
            repeaters_names: Vec::new(),
            selected_repeater: 0,

            popups: Vec::new(),
        }
    }
}
