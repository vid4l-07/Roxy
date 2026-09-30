use std::fs;
use crate::http;
use crate::certs;

const WEB: &str = include_str!("index.html");

pub fn send_web_page() -> http::Response {
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

pub fn error(e: String) -> http::Response {
    let response = format!("\
                    HTTP/1.1 500 Internal Server Error\r\n\
                    Content-Type: text/plain\r\n\
                    Content-Length: {}\r\n\
                    \r\n\
                    {}", e.len(), e);
    http::Response { raw: response.to_owned().into_bytes() }

}

pub fn download_ca() -> std::io::Result<http::Response> {
    certs::get_or_create_ca()?;
    let path = certs::ca_folder()?.join("ca.crt");
    let cert = fs::read(path)?;

    let mut raw = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/x-x509-ca-cert\r\n\
         Content-Disposition: attachment; filename=\"ca.crt\"\r\n\
         Content-Length: {}\r\n\
         \r\n",
        cert.len()
    ).into_bytes();

    raw.extend_from_slice(&cert);

    Ok(http::Response{ raw })

}
