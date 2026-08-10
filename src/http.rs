use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt};
use std::io;


pub struct Request {
    pub raw: Vec<u8>,
}

impl Request {
    pub fn to_str(&self) -> String {
        String::from_utf8_lossy(&self.raw).to_string()
    }

    fn find_header(&self, header: &str) -> Option<String> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut request = httparse::Request::new(&mut headers);
        match request.parse(&self.raw) {
            Ok(httparse::Status::Complete(_)) => {
                for i in request.headers {
                    if i.name.eq_ignore_ascii_case(header) {
                        return Some(
                            String::from_utf8_lossy(i.value)
                            .trim()
                            .to_string(),
                        );
                    }
                }

                None
            }

            _ => None,
        }

    }

    pub fn host(&self) -> Option<String> {
        self.find_header("Host")
    }

    pub fn method(&self) -> Option<String> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut request = httparse::Request::new(&mut headers);

        match request.parse(&self.raw) {
            Ok(httparse::Status::Complete(_)) => {
                request.method.map(|method| method.to_string())
            }
            _ => None,
        }
    }

}

pub struct Response {
    pub raw: Vec<u8>,
}

impl Response {
    pub fn to_str(&self) -> String {
        String::from_utf8_lossy(&self.raw).to_string()
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
        Ok(Request { raw: vec })
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

