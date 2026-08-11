
use crate::http;
use crate::repeater;

pub enum Screen {
    Proxy,
    Repeater
}

pub struct App {
    pub screen: Screen,
    pub intercepted_request: Option<http::Request>,
    pub repeaters: Vec<repeater::Repeater>,
    pub selected_repeater: usize,
    pub intercept: bool,
}

impl App {
    pub fn new() -> App{
        App { 
            screen: Screen::Proxy,
            intercepted_request: None,
            repeaters: Vec::new(),
            selected_repeater: 0,
            intercept: false,
        }
    }
}
