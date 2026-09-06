use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt};
use std::io;
use url::Url;


// Request
#[derive(Clone)]
pub struct Request {
    pub method: String,
    pub target: String,
    pub version: u8,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,

    pub host: String,
    pub port: u16,
}

impl Request {
    
    // Constructor
    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut request = httparse::Request::new(&mut headers);

        let header_size = match request.parse(data).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid HTTP request: {e}"),
            )
        })? {
            httparse::Status::Complete(size) => size,

            httparse::Status::Partial => {
                return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Incomplete HTTP request",
                ));
            }
        };

        let method = request.method.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing HTTP method",
                )
            })?.to_string();

        let target = request.path.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing request target",
                )
            })?.to_string();

        let version = request.version.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing HTTP version",
                )
            })?;

        let mut request_headers = Vec::new();

        for header in request.headers {
            let name = header.name.to_string();

            let value = String::from_utf8_lossy(header.value).trim().to_string();

            request_headers.push((name, value));
        }

        let body = data[header_size..].to_vec();

        if method.eq_ignore_ascii_case("CONNECT") {
            let (host, port) = Self::parse_connect_target(&target)?;

            return Ok(Self {
                method,
                target,
                version,
                headers: request_headers,
                body,
                host,
                port,
            });
        }


        let (target, host, port) = Self::normalize_target(&target, &request_headers)?;

        Ok(Self {
            method,
            target,
            version,
            headers: request_headers,
            body,
            host,
            port,
        })
    }

    // parsers
    fn normalize_target(target: &str, headers: &[(String, String)]) -> io::Result<(String, String, u16)> {
        if target.starts_with("http://") {
            let url = Url::parse(target).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid URL: {e}"),
                )
            })?;

            let host = url.host_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "URL has no host",
                )
            })?;

            let port = url.port().unwrap_or(80);

            let mut target = url.path().to_string();

            if target.is_empty() {
                target.push('/');
            }

            if let Some(query) = url.query() {
                target.push('?');
                target.push_str(query);
            }

            return Ok((
                target,
                host.to_string(),
                port,
            ));
        }

        let (host, port) = Self::parse_host(headers)?;

        Ok((
            target.to_string(),
            host,
            port,
        ))
    }

    fn parse_host(headers: &[(String, String)]) -> io::Result<(String, u16)> {
        let host = headers.iter().find(|(name, _)| name.eq_ignore_ascii_case("Host")).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing Host",
            )
        })?;

        if let Some((host, port)) = host.1.rsplit_once(':') {
            let port = port.parse::<u16>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid port",
                )
            })?;

            return Ok((host.to_string(), port));
        }

        Ok((host.1.clone(), 80))
    }

    fn parse_connect_target(target: &str) -> io::Result<(String, u16)> {
        let (host, port) = target.rsplit_once(':')
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid CONNECT target",
                )
            })?;

        let port = port.parse::<u16>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid CONNECT port",
            )
        })?;

        Ok((host.to_string(), port))
    }


    pub fn from_edited(data: &str, host: String, port: u16) -> Self {
        let (headers_part, body) = data
            .split_once("\r\n\r\n")
            .unwrap_or((data, ""));

        let mut lines = headers_part.split("\r\n");

        let request_line = lines.next().unwrap_or("");

        let mut parts = request_line.splitn(3, ' ');

        let method = parts.next().unwrap_or("").to_string();
        let target = parts.next().unwrap_or("").to_string();

        let version = parts
            .next()
            .and_then(|version| version.strip_prefix("HTTP/1."))
            .and_then(|version| version.parse::<u8>().ok())
            .unwrap_or(1);

        let mut headers = Vec::new();

        for line in lines {
            if let Some((name, value)) = line.split_once(':') {
                headers.push((
                        name.trim().to_string(),
                        value.trim().to_string(),
                ));
            }
        }

        if let Some((_, value)) = headers.iter_mut().find(|(name, _)| name.eq_ignore_ascii_case("Content-Length")) {
            *value = body.len().to_string();
        } else if !body.is_empty() {
            headers.push(("Content-Length".into(), body.len().to_string()));
        }

        Self {
            method,
            target,
            version,
            headers,
            body: body.as_bytes().to_vec(),
            host,
            port,
        }
    }

    // Transformations
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut request = format!(
            "{} {} HTTP/1.{}\r\n",
            self.method,
            self.target,
            self.version
        );

        for (name, value) in &self.headers {
            request.push_str(&format!("{}: {}\r\n", name, value));
        }

        request.push_str("\r\n");

        let mut bytes = request.into_bytes();
        bytes.extend_from_slice(&self.body);

        bytes
    }

    pub fn to_str(&self) -> String {
        String::from_utf8_lossy(&self.to_bytes()).to_string()
    }

}

pub async fn read_request(socket: &mut TcpStream) -> io::Result<Request> {
        let mut buff = [0u8; 4096];
        let mut vec: Vec<u8> = Vec::new();

        loop {
            let n = socket.read(&mut buff).await?;

            if n == 0{
                break;
            }

            vec.extend_from_slice(&buff[..n]);
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut request = httparse::Request::new(&mut headers);

            match request.parse(&vec) {
                Ok(httparse::Status::Complete(n)) => {
                    let mut content_length: Option<usize> = None;

                    for header in request.headers {
                        if header.name.eq_ignore_ascii_case("Content-Length"){
                            content_length = String::from_utf8_lossy(header.value).parse().ok();
                        }
                    }

                    match content_length {
                        Some(length) => {
                            if vec[n..].len() >= length {
                                break;
                            }
                        }

                        None => {
                            break;
                        }
                    }

                }

                Ok(httparse::Status::Partial) => {}
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e));
                }
            }
        }
        Request::from_bytes(&vec)
}


// Response
pub struct Response {
    pub raw: Vec<u8>,
}

impl Response {
    pub fn to_str(&self) -> String {
        String::from_utf8_lossy(&self.raw).to_string()
    }
}

pub async fn read_response(socket: &mut TcpStream) -> io::Result<Response> {
        let mut buff = [0u8; 4096];
        let mut vec: Vec<u8> = Vec::new();

        loop {
            let n = socket.read(&mut buff).await?;

            if n == 0{
                break;
            }

            vec.extend_from_slice(&buff[..n]);
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut request = httparse::Response::new(&mut headers);

            match request.parse(&vec) {
                Ok(httparse::Status::Complete(n)) => {
                    let mut content_length: Option<usize> = None;

                    for header in request.headers {
                        if header.name == "Content-Length"{
                            content_length = String::from_utf8_lossy(header.value).parse().ok();
                        }
                    }

                    match content_length {
                        Some(length) => {
                            if vec[n..].len() >= length {
                                break;
                            }
                        }

                        None => {
                            break;
                        }
                    }

                }

                Ok(httparse::Status::Partial) => {}
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e));
                }
            }
        }
        Ok(Response { raw: vec })
}

