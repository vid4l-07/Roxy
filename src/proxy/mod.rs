use tokio::{
    net::TcpListener,
    net::TcpStream,
    sync::{mpsc,oneshot}
};
use std::collections::HashMap;

use std::io;

use crate::{events, http};

mod connections;


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
    let mut next_id = 0;

    let mut pending: HashMap<usize, oneshot::Sender<http::Request>> = HashMap::new();

    let (pending_sender, mut pending_receiver) = mpsc::channel::<(usize, oneshot::Sender<http::Request>)>(100);

    loop {
        tokio::select! {
            // Connected client
            connection = listener.accept() => {
                let (client, _) = match connection {
                    Ok(connection) => connection,
                    Err(e) => {
                        send_event(&sender, events::ProxyEvents::Error(e.to_string())).await?;
                        continue;
                    }
                };

                let id = next_id;
                next_id += 1;

                let sender = sender.clone();
                let pending_sender = pending_sender.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_connection(client, id, intercept, sender.clone(), pending_sender).await {
                        let _ = send_event(&sender, events::ProxyEvents::Error(e.to_string())).await; 
                    }
                });
            }

            // Pending request event
            request = pending_receiver.recv() => {
                if let Some((id, sender)) = request {
                    pending.insert(id, sender);
                }
            }

            // Tui event received
            config = receiver.recv() => {
                match config {
                    Some(events::TuiEvents::SetIntercept(value)) => {
                        intercept = value;
                    }

                    Some(events::TuiEvents::Forward { id, request }) => {
                        if let Some(sender) = pending.remove(&id) {  // Send forward command to pendig request and remove it
                            let _ = sender.send(request);
                        }
                    }

                    Some(events::TuiEvents::SendRepeater { index, request }) => {
                        send_repeater(sender, request, index).await;
                    }

                    None => {
                        return Err(io::Error::new(io::ErrorKind::BrokenPipe, "TUI channel closed"));
                    }

                }
            }
        }
    }
}

async fn handle_connection(mut client: TcpStream, id: usize, intercept: bool,
    tui_sender: mpsc::Sender<events::ProxyEvents>, proxy_sender: mpsc::Sender<(usize, oneshot::Sender<http::Request>)>) -> io::Result<()> {

    let request = connections::get_request(&mut client).await?;

    if request.method.eq_ignore_ascii_case("CONNECT") {
        connections::handle_https(&mut client, &request).await?;
        return Ok(());
    }

    if intercept {
        let (tx, rx) = oneshot::channel();

        proxy_sender.send((id, tx)).await.map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e)
        })?;

        send_event(&tui_sender, events::ProxyEvents::ReceivedRequest{ id, request }).await?;

        let request = rx.await.map_err(|e| {      // Wait for the proxy to send the forward command with the edited request
            io::Error::new(io::ErrorKind::Other, e)
        })?;

        connections::forward(&mut client, &request).await?;

    } else {
        connections::forward(&mut client, &request).await?;
    }

    Ok(())
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
