use tokio::{
    net::TcpListener,
    net::TcpStream,
    sync::{mpsc,oneshot}
};
use std::collections::HashMap;

use std::io;

use crate::{events, http};
use crate::web_page;

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
                        if let Err(e) = send_repeater(sender, request, index).await{
                            let _ = send_event(&sender, events::ProxyEvents::Error(e.to_string())).await; 
                        }
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

    if matches!((request.host.as_str(), request.port), ("roxy", _) | ("127.0.0.1" | "localhost", 8080)) {
        let response = web_page::generate_response();
        return connections::send_response(&mut client, &response).await;
    }

    // HTTPS
    if request.method.eq_ignore_ascii_case("CONNECT") {

        let (mut client, request) = connections::handle_https(client, &request).await?;

        let request = if intercept{
            let (tx, rx) = oneshot::channel();

            proxy_sender.send((id, tx)).await.map_err(|e| {
                io::Error::new(io::ErrorKind::Other, e)
            })?;


            send_event(&tui_sender, events::ProxyEvents::ReceivedRequest{ id, request }).await?;

            rx.await.map_err(|e| {      // Wait for the proxy to send the forward command with the edited request
                io::Error::new(io::ErrorKind::Other, e)
            })?

        } else {
            request
        };

        let mut server = connections::connect_tls(&request).await?;
        connections::forward(&mut client, &mut server, &request).await?;

    // HTTP
    } else {
        let request = if intercept {
            let (tx, rx) = oneshot::channel();

            proxy_sender.send((id, tx)).await.map_err(|e| {
                io::Error::new(io::ErrorKind::Other, e)
            })?;


            send_event(&tui_sender, events::ProxyEvents::ReceivedRequest{ id, request }).await?;

            rx.await.map_err(|e| {      // Wait for the proxy to send the forward command with the edited request
                io::Error::new(io::ErrorKind::Other, e)
            })?

        } else {
            request
        };

        let mut server = connections::connect_to_server(&request).await?;
        connections::forward(&mut client, &mut server, &request).await?;

    }

    Ok(())
}

async fn send_repeater(sender: &mpsc::Sender<events::ProxyEvents>, request: http::Request, index: usize) -> io::Result<()>{
    let sender = sender.clone();

    tokio::spawn(async move {

        let result = if let http::Protocol::HTTPS = request.protocol {
            match connections::connect_tls(&request).await {
                Ok(mut server) => connections::send_request(&mut server, &request).await,
                Err(e) => Err(e)
            }
        } else {
            match connections::connect_to_server(&request).await {
                Ok(mut server) => connections::send_request(&mut server, &request).await,
                Err(e) => Err(e)
            }

        };


        match result {
            Ok(response) => {
                let _ = sender.send(events::ProxyEvents::RepeaterResponse { index, response: Some(response) }).await;
            }

            Err(e) => {
                let _ = sender.send(events::ProxyEvents::RepeaterResponse { index, response: None }).await;
                let _ = sender.send(events::ProxyEvents::Error(e.to_string())).await;
            }
        }
    });
    Ok(())
}
