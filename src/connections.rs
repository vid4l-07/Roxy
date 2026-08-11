use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt};
use std::io;

use crate::http;
use crate::editor;

async fn connect_to_server(request: &http::Request) -> io::Result<TcpStream> {
    let host = request.host().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing Host header"))?;

    let address = if host.contains(':') {
        host
    } else {
        format!("{}:80", host)
    };

    TcpStream::connect(address).await
}

pub async fn send_request(request: &http::Request) -> io::Result<http::Response> {
    let mut server = connect_to_server(&request).await?;

    server.write_all(&request.raw).await?;

    http::read_response(&mut server).await
}

pub async fn handle_connection(mut client: TcpStream, intercept: bool) -> io::Result<()> {
    let request = http::read_request(&mut client).await?;

    match request.method().as_deref() {
        Some("CONNECT") => {
            handle_https(client, &request).await?;
        }

        _ => {
            let response = send_request(&request).await?;

            println!("{}", response.to_str());
            client.write_all(&response.raw).await?;
        }
    }

    Ok(())
}

async fn handle_https(mut client: TcpStream, request: &http::Request) -> io::Result<()> {
    let host = request.host().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing Host header"))?;

    let mut server = TcpStream::connect(&host).await?;

    client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;

    tokio::io::copy_bidirectional(&mut client, &mut server).await?;

    Ok(())
}


