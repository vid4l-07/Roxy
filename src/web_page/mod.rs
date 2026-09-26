use crate::http;


const WEB: &str = include_str!("index.html");

pub fn generate_response() -> http::Response {
    let response = http::Response { raw: format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
        WEB.len(),
        WEB 
    ).into_bytes() };

    return response;
}
