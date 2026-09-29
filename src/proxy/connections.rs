use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use std::io;

use std::sync::Arc;

use tokio_rustls::{rustls::{ClientConfig, RootCertStore}, TlsConnector, server::TlsStream};

use crate::http;


pub async fn connect_to_server(request: &http::Request) -> io::Result<TcpStream> {
    let address = format!("{}:{}", &request.host, &request.port);

    TcpStream::connect(address).await
}

pub async fn get_request<S>(client: &mut S) -> io::Result<http::Request> 
where S: AsyncRead + AsyncWrite + Unpin {
    http::read_request(client).await
}

pub async fn send_request<S>(server: &mut S, request: &http::Request) -> io::Result<http::Response> 
where S: AsyncRead + AsyncWrite + Unpin {
    server.write_all(&request.to_bytes()).await?;

    http::read_response(server).await
}

pub async fn send_response<S>(client: &mut S, response: &http::Response) -> io::Result<()> 
where S: tokio::io::AsyncWrite + Unpin {

    client.write_all(&response.raw).await
}

pub async fn forward<C, S>(client: &mut C, server: &mut S, request: &http::Request) -> io::Result<http::Response>
where C: AsyncWrite + Unpin, S: AsyncRead + AsyncWrite + Unpin{
    let response = send_request(server, request).await?;
    send_response(client, &response).await?;
    Ok(response)
}

// HTTPS

pub async fn handle_https(mut client: TcpStream, request: &http::Request) -> io::Result<(TlsStream<TcpStream>, http::Request)> {
    let (host,port) = (&request.host, &request.port);

    client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;

    let acceptor = crate::certs::create_tls_acceptor(&request.host)?;

    let mut client = acceptor.accept(client).await?;

    let mut request = http::read_request(&mut client).await?;
    request.host = host.clone();
    request.port = *port;
    request.protocol = http::Protocol::HTTPS;

    Ok((client, request))
}

pub async fn connect_tls(request: &http::Request) -> io::Result<tokio_rustls::client::TlsStream<TcpStream>> {

    let address = format!("{}:{}", &request.host, &request.port);

    let stream = TcpStream::connect(address).await?;

    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };

    let config = ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    let server_name = tokio_rustls::rustls::pki_types::ServerName::try_from(request.host.as_str()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid server name",
        )
    })?.to_owned();

    connector.connect(server_name, stream).await.map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            e,
        )
    })
}
