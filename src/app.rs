
use crate::http;


pub struct App {
    pub intercepted_request: Option<http::Request>,
}
