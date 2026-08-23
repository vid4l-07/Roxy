use tokio::net::TcpListener;
use tokio::sync::mpsc;
use std::io;

use crate::connections;
use crate::events;


pub async fn start(sender: mpsc::Sender<events::ProxyEvents>, mut receiver: mpsc::Receiver<events::TuiEvents>) -> io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let mut intercept = false;

    loop {
        tokio::select! {
            connection = listener.accept() => {
                let (mut client, _) = connection?;

                let request = connections::get_request(&mut client).await?;

                if intercept {
                    sender.send(events::ProxyEvents::ReceivedRequest(request.clone())).await.map_err(|_| 
                        io::Error::new(io::ErrorKind::BrokenPipe, "TUI channel closed")
                    )?;

                    loop {
                        match receiver.recv().await {
                            Some(events::TuiEvents::Forward(request)) => {
                                connections::forward(&mut client, &request).await?;
                                break;
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
                    connections::forward(&mut client, &request).await?;
                }
            }

            config = receiver.recv() => {
                match config {
                    Some(events::TuiEvents::SetIntercept(value)) => {
                        intercept = value;
                    }

                    Some(events::TuiEvents::Forward(_)) => {}

                    None => {
                        return Err(io::Error::new(
                                io::ErrorKind::BrokenPipe,
                                "TUI channel closed",
                        ));
                    }
                }
            }
        }
    }
}
