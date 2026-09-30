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
    pub previous_requests_stack: Vec<http::Request>,
    pub next_requests_stack: Vec<http::Request>,
    pub response: Option<http::Response>,

    pub request_scroll: u16,
    pub response_scroll: u16,
    
    pub thinking: bool,
}

impl Repeater {
    pub fn new(request: http::Request, name: String) -> Self {
        Self {
            name,
            request,
            response: None,
            previous_requests_stack: Vec::new(),
            next_requests_stack: Vec::new(),
            request_scroll: 0,
            response_scroll: 0,
            thinking: false,
        }
    }
    pub fn edit(&mut self) -> io::Result<()>{
        let edited_data = editor::edit(&self.request.to_str())?;

        self.next_requests_stack.clear();
        self.previous_requests_stack.push(self.request.clone());
        self.request = http::Request::from_edited(&edited_data, &self.request);

        Ok(())
    }

    pub fn undo(&mut self) {
        if let Some(previous) = self.previous_requests_stack.pop() {
            self.next_requests_stack.push(self.request.clone());
            self.request = previous;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.next_requests_stack.pop() {
            self.previous_requests_stack.push(self.request.clone());
            self.request = next;
        }
    }
}


