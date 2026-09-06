use crate::{http, repeater, popups};

pub enum Screen {
    Proxy,
    Repeater
}

pub struct InterceptedRequest {
    pub id: usize,
    pub request: http::Request
}

pub struct App {
    pub screen: Screen,
    pub proxy_scroll: u16,

    pub request_queue: Vec<InterceptedRequest>,
    pub intercept: bool,

    pub repeaters: Vec<repeater::Repeater>,
    pub selected_repeater: usize,
    pub repeater_focus: repeater::RepeaterFocus,
    pub request_size: u16,

    pub renaming_repeater: Option<usize>,

    pub popups: Vec<popups::Popup>,
}

impl App {
    pub fn new() -> App{
        App { 
            screen: Screen::Proxy,
            proxy_scroll: 0,

            request_queue: Vec::new(),
            intercept: false,

            repeaters: Vec::new(),
            selected_repeater: 0,
            repeater_focus: repeater::RepeaterFocus::Request,
            request_size: 50,

            renaming_repeater: None,

            popups: Vec::new(),
        }
    }
}
