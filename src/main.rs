#![allow(unused)]
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use std::io;

mod http;
mod editor;
mod connections;
mod repeater;
mod app;
mod tui;
mod proxy;
mod events;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (proxy_sender, proxy_receiver) = mpsc::channel::<events::ProxyEvents>(32);
    let (tui_sender, tui_receiver) = mpsc::channel::<events::TuiEvents>(32);

    let mut app = app::App::new();

    tokio::spawn(async move {
        proxy::start(proxy_sender, tui_receiver).await
    });

    tui::run(&mut app, tui_sender, proxy_receiver).await;

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


