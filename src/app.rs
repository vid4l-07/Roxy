
use crate::{http, repeater, popups};

pub enum Screen {
    Proxy,
    Repeater
}

pub struct App {
    pub screen: Screen,
    pub proxy_scroll: u16,

    pub intercepted_request: Option<http::Request>,
    pub intercept: bool,

    pub repeaters: Vec<repeater::Repeater>,
    pub selected_repeater: usize,
    pub repeater_focus: repeater::RepeaterFocus,
    pub request_size: u16,

    pub popups: Vec<popups::Popup>,
}

impl App {
    pub fn new() -> App{
        App { 
            screen: Screen::Proxy,
            proxy_scroll: 0,

            intercepted_request: None,
            intercept: false,

            repeaters: Vec::new(),
            selected_repeater: 0,
            repeater_focus: repeater::RepeaterFocus::Request,
            request_size: 50,

            popups: Vec::new(),
        }
    }
}
