#![allow(unused)]
use tokio::net::TcpListener;
use std::io;

mod http;
mod editor;
mod connections;
mod repeater;
mod app;
mod tui;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut app = app::App::new();

    tui::run(&mut app).await;

    Ok(())

    // Flujo
    // let listener = TcpListener::bind("127.0.0.1:8080").await?;
    // let (mut client, _) = listener.accept().await?;
    // let request = connections::get_request(&mut client).await?;
    // println!("{}", request.to_str());
    // let response = connections::forward(&mut client, &request).await?;
    // println!("{}", response.to_str());
    // Ok(())
}


