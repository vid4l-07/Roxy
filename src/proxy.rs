use tokio::{
    net::TcpListener,
    sync::mpsc
};

use std::io;

use crate::{connections, events, http};


async fn send_event(sender: &mpsc::Sender<events::ProxyEvents>, event: events::ProxyEvents) -> io::Result<()> {
    sender.send(event).await.map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "TUI channel closed",
            )
        })
}

pub async fn start(sender: &mpsc::Sender<events::ProxyEvents>, mut receiver: mpsc::Receiver<events::TuiEvents>) -> io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let mut intercept = false;

    loop {
        tokio::select! {
            // Connected client
            connection = listener.accept() => {
                let (mut client, _) = match connection {
                    Ok(connection) => connection,
                    Err(e) => {
                        send_event(
                            &sender,
                            events::ProxyEvents::Error(e.to_string())
                        ).await?;
                        continue;
                    }
                };

                let request = match connections::get_request(&mut client).await {
                    Ok(request) => request,
                    Err(e) => {
                        send_event(&sender, events::ProxyEvents::Error(e.to_string())).await?;
                        continue;
                    }
                };

                if intercept {
                    send_event(&sender, events::ProxyEvents::ReceivedRequest(request.clone())).await?;

                    loop {
                        match receiver.recv().await {
                            Some(events::TuiEvents::Forward(request)) => {
                                if let Err(e) = connections::forward(&mut client, &request).await {
                                    send_event(&sender, events::ProxyEvents::Error(e.to_string())).await?;
                                }
                                break;
                            }

                            Some(events::TuiEvents::SendRepeater { index, request }) => {
                                send_repeater(sender, request, index).await;
                            }

                            Some(events::TuiEvents::SetIntercept(value)) => {
                                intercept = value;
                            }

                            None => {
                                return Err(io::Error::new(
                                        io::ErrorKind::BrokenPipe,
                                        "TUI channel closed",
                                ));
                            }

                        }
                    }

                } else {
                    if let Err(e) = connections::forward(&mut client, &request).await {
                        send_event(&sender, events::ProxyEvents::Error(e.to_string())).await?;
                    }
                }
            }

            // Tui event received
            config = receiver.recv() => {
                match config {
                    Some(events::TuiEvents::SetIntercept(value)) => {
                        intercept = value;
                    }

                    Some(events::TuiEvents::SendRepeater { index, request }) => {
                        send_repeater(sender, request, index).await;
                    }

                    None => {
                        return Err(io::Error::new(
                                io::ErrorKind::BrokenPipe,
                                "TUI channel closed",
                        ));
                    }

                    _ => {}
                }
            }
        }
    }
}

async fn send_repeater(sender: &mpsc::Sender<events::ProxyEvents>, request: http::Request, index: usize) {
    let sender = sender.clone();

    tokio::spawn(async move {
        match connections::send_request(&request).await {
            Ok(response) => {
                let _ = sender.send(
                    events::ProxyEvents::RepeaterResponse {
                        index,
                        response,
                    }
                ).await;
            }

            Err(error) => {
                let _ = sender.send(
                    events::ProxyEvents::Error(
                        error.to_string()
                    )
                ).await;
            }
        }
    });
}
