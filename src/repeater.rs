use std::io;


use crate::http;
use crate::connections;

pub struct Repeater {
    pub request: http::Request,
    pub response: Option<http::Response>,
}

impl Repeater {
    pub fn new(request: http::Request) -> Self {
        Self {
            request,
            response: None,
        }
    }
    pub async fn send(&mut self) -> io::Result<()> {
        let response = connections::send_request(&self.request).await?;
        self.response = Some(response);

        Ok(())
    }
}


