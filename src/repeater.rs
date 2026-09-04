use std::io;

use crate::http;
use crate::editor;

pub struct Repeater {
    pub name: String,
    pub request: http::Request,
    pub response: Option<http::Response>,
}

impl Repeater {
    pub fn new(request: http::Request, name: String) -> Self {
        Self {
            name,
            request,
            response: None,
        }
    }
    pub fn edit(&mut self) -> io::Result<()>{
        let edited_data = editor::edit(&self.request.to_str())?;

        self.request = http::Request::from_edited(&edited_data, self.request.host.clone(), self.request.port);

        Ok(())
    }
}


