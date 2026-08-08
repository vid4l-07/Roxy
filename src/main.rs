use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening in 127.0.0.1:8080");
    loop {
        let (mut socket, _) = listener.accept().await?;

        let mut buff = [0u8; 4096];

        loop {
            let n = socket.read(&mut buff).await?;
            if n == 0{
                break;
            }

            println!("{}", String::from_utf8_lossy(&buff[..n]));

            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 18\r\n\r\nHola desde server!").await?;
        }
    }
}
