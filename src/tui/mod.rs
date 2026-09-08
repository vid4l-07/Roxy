use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use ratatui::Frame;
use tokio::sync::mpsc;
use std::io;

use crate::{app, events};

mod render;
mod input;
pub mod popups;

use render::{render_proxy, render_repeater};
use input::{handle_input, error_popup};

// General
pub async fn run(app: &mut app::App, sender: mpsc::Sender<events::TuiEvents>, receiver: mpsc::Receiver<events::ProxyEvents>) -> io::Result<()> {
    let mut terminal = ratatui::init();

    let result = main_loop(app, sender, receiver, &mut terminal).await;

    ratatui::restore();

    if let Err(ref e) = result {
        eprintln!("Fatal error: {}", e);
    }

    result
}

async fn main_loop(app: &mut app::App, sender: mpsc::Sender<events::TuiEvents>, mut receiver: mpsc::Receiver<events::ProxyEvents>, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut events = EventStream::new();

    loop {
        terminal.draw(|frame| render(frame, app))?;

        tokio::select! {
            event = events.next() => {
                if let Some(Ok(Event::Key(key))) = event {
                    if handle_input(app, key.code, terminal, &sender).await? {
                        return Ok(());
                    }
                }
            }

            event = receiver.recv() => {
                match event {
                    Some(events::ProxyEvents::ReceivedRequest{ id, request }) => {
                        app.request_queue.push(app::InterceptedRequest { id, request });
                    }

                    Some(events::ProxyEvents::Error(error)) => {
                        error_popup(app, error);
                    }

                    Some(events::ProxyEvents::FatalError(error)) => {
                        return Err(io::Error::new(
                                io::ErrorKind::Other,
                                error,
                        ));
                    }

                    Some(events::ProxyEvents::RepeaterResponse { index, response }) => {
                        if let Some(repeater) = app.repeaters.get_mut(index) {
                            repeater.response = Some(response);
                            repeater.thinking = false;
                        }
                    }

                    None => {
                        return Err(io::Error::new(
                                io::ErrorKind::BrokenPipe,
                                "Proxy channel closed",
                        ));
                    }
                }
            }
        }
    }
}

fn render(frame: &mut Frame, app: &app::App) {
    match app.screen {
        app::Screen::Proxy => render_proxy(frame, app),
        app::Screen::Repeater => render_repeater(frame, app),
    }

    if let Some(popup) = app.popups.last() {
        popup.render(frame);
    }
}
