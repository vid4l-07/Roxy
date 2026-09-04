use std::io;

use crate::http;
use crate::editor;

pub enum RepeaterFocus {
    Request,
    Response,
}

pub struct Repeater {
    pub name: String,
    pub request: http::Request,
    pub response: Option<http::Response>,

    pub request_scroll: u16,
    pub response_scroll: u16
}

impl Repeater {
    pub fn new(request: http::Request, name: String) -> Self {
        Self {
            name,
            request,
            response: None,
            request_scroll: 0,
            response_scroll: 0
        }
    }
    pub fn edit(&mut self) -> io::Result<()>{
        let edited_data = editor::edit(&self.request.to_str())?;

        self.request = http::Request::from_edited(&edited_data, self.request.host.clone(), self.request.port);

        Ok(())
    }
}


