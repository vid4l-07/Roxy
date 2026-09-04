#![allow(unused)]
use tokio::{ net::TcpListener, sync::mpsc};
use std::io;

mod http;
mod editor;
mod connections;
mod repeater;
mod app;
mod tui;
mod proxy;
mod events;
mod popups;

fn wellcome_screen() {
    print!("
  ██████╗  ██████╗ ██╗  ██╗██╗   ██╗
  ██╔══██╗██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝
  ██████╔╝██║   ██║ ╚███╔╝  ╚████╔╝ 
  ██╔══██╗██║   ██║ ██╔██╗   ╚██╔╝  
  ██║  ██║╚██████╔╝██╔╝ ██╗   ██║   
  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   

  - You can contribute on github: \x1b[33mhttps://github.com/vid4l-07/Roxy\x1b[0m
");

}

#[tokio::main]
async fn main() {
    wellcome_screen();
    let (proxy_sender, proxy_receiver) = mpsc::channel::<events::ProxyEvents>(32);
    let (tui_sender, tui_receiver) = mpsc::channel::<events::TuiEvents>(32);

    let mut app = app::App::new();

    tokio::spawn(async move {
        if let Err(error) = proxy::start(&proxy_sender, tui_receiver).await {
            let _ = proxy_sender.send(events::ProxyEvents::FatalError(error.to_string())).await;
        }
    });

    if tui::run(&mut app, tui_sender, proxy_receiver).await.is_err() {
        std::process::exit(1);
    }

    // Workflow
    // let listener = TcpListener::bind("127.0.0.1:8080").await?;
    // let (client, _) = listener.accept().await?;
    // let request = connections::get_request(&mut client).await?;
    // println!("{}", request.to_str());
    // let response = connections::forward(&mut client, &request).await?;
    // println!("{}", response.to_str());
    // Ok(())
}


