#![allow(unused)]
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt};
use std::io;

mod http;
mod editor;

fn intercept_request(request: &http::Request) -> io::Result<http::Request>{
    println!("\n\n{}", request.to_str());

    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
    
    let edited = editor::edit(&request.to_str())?;

    Ok(http::Request { raw: edited.into_bytes() })
}

async fn connect_to_server(request: &http::Request) -> io::Result<TcpStream> {
    let host = request
        .host()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing Host header"))?;

    let address = if host.contains(':') {
        host
    } else {
        format!("{}:80", host)
    };

    TcpStream::connect(address).await
}

async fn handle_connection(mut client: TcpStream, intercept: bool) -> io::Result<()> {
    let request = http::read_request(&mut client).await?;

    let request = if intercept { 
        intercept_request(&request)? 
    } else { 
        request 
    };

    let mut server = connect_to_server(&request).await?;
    server.write_all(&request.raw).await?;

    let response = http::read_response(&mut server).await?;
    println!("{}", response.to_str());
    client.write_all(&response.raw).await?;

    Ok(())
}




#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening in 127.0.0.1:8080");
    let intercept = true;
    loop {
        let (client, _) = listener.accept().await?;
        handle_connection(client, intercept).await?;

    }
}
