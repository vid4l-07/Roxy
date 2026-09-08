use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt};
use std::io;

use crate::http;

async fn connect_to_server(request: &http::Request) -> io::Result<TcpStream> {
    let address = format!("{}:{}", &request.host, &request.port);

    TcpStream::connect(address).await
}



pub async fn get_request(client: &mut TcpStream) -> io::Result<http::Request> {
    http::read_request(client).await
}

pub async fn send_request(request: &http::Request) -> io::Result<http::Response> {
    let mut server = connect_to_server(&request).await?;

    server.write_all(&request.to_bytes()).await?;

    http::read_response(&mut server).await
}

pub async fn send_response(client: &mut TcpStream, response: &http::Response) -> io::Result<()> {
    client.write_all(&response.raw).await
}

pub async fn forward(client: &mut TcpStream, request: &http::Request) -> io::Result<http::Response>{
    let response = send_request(request).await?;
    send_response(client, &response).await?;
    Ok(response)
}

// HTTPS

pub async fn handle_https(client: &mut TcpStream, request: &http::Request) -> io::Result<()> {
    let host = format!("{}:{}", request.host, request.port);

    let mut server = TcpStream::connect(&host).await?;

    client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;

    tokio::io::copy_bidirectional(client, &mut server).await?;

    Ok(())
}

