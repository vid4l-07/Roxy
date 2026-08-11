#![allow(unused)]
use tokio::net::TcpListener;
use std::io;

mod http;
mod editor;
mod connections;
mod repeater;
mod app;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Listening in 127.0.0.1:8080");

    let intercept = false;

    loop {
        let (client, _) = listener.accept().await?;
        connections::handle_connection(client, intercept).await?;

    }
}


