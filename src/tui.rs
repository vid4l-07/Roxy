use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;

use ratatui::widgets::Paragraph;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use tokio::sync::mpsc;

use std::io;

use crate::app::{self, App};
use crate::events;

// General

pub async fn run(app: &mut App, sender: mpsc::Sender<events::TuiEvents>, mut receiver: mpsc::Receiver<events::ProxyEvents>) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut events = EventStream::new();

    loop {
        terminal.draw(|frame| render(frame, app))?;

        tokio::select! {
            event = events.next() => {
                if let Some(Ok(Event::Key(key))) = event {
                    if handle_input(app, key.code, &mut terminal, &sender).await? {
                        ratatui::restore();
                        return Ok(());
                    }
                }
            }

            event = receiver.recv() => {
                match event {
                    Some(events::ProxyEvents::ReceivedRequest(request)) => {
                        app.intercepted_request = Some(request);
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

fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        app::Screen::Proxy => render_proxy(frame, app),
        app::Screen::Repeater => render_repeater(frame, app),
    }
}

async fn handle_input(app: &mut App, key: KeyCode, terminal: &mut ratatui::DefaultTerminal, sender: &mpsc::Sender<events::TuiEvents>) -> io::Result<bool>{
    if key == KeyCode::Char('q') {
        return Ok(true);
    }
    
    match app.screen {
        app::Screen::Proxy => handle_proxy_input(app, key, terminal, sender).await?,
        app::Screen::Repeater => handle_repeater_input(app, key, terminal).await?,
    }
    Ok(false)
}

// Proxy

fn render_proxy(frame: &mut Frame, app: &App) {
    let request = app
        .intercepted_request
        .as_ref()
        .map(|request| request.to_str())
        .unwrap_or_default();
    frame.render_widget(
        Paragraph::new(format!(
            "PROXY\n\nIntercept: {}\n\n{}", 
            if app.intercept { "ON" } else { "OFF" }, request
        )),
        frame.area(),
    );
}

async fn handle_proxy_input(app: &mut App, key: KeyCode, terminal: &mut ratatui::DefaultTerminal, sender: &mpsc::Sender<events::TuiEvents>) -> io::Result<()> {
    match key {
        KeyCode::Char('i') => {
            app.intercept = !app.intercept;
            sender.send(events::TuiEvents::SetIntercept(app.intercept)).await.map_err(|_| 
                io::Error::new(io::ErrorKind::BrokenPipe, "TUI channel closed")
            )?;
        }

        KeyCode::Enter => {
            match &app.intercepted_request {
                Some(request) => {
                    sender.send(events::TuiEvents::Forward(request.clone())).await.map_err(|_| 
                        io::Error::new(io::ErrorKind::BrokenPipe, "TUI channel closed")
                    )?;
                    app.intercepted_request = None;
                }
                None => {}
            }
        }

        KeyCode::Char('e') => {
            ratatui::restore();
            match &mut app.intercepted_request {
                Some(request) => {
                    let edited = crate::editor::edit(&request.to_str())?;
                    request.raw = edited.into_bytes();
                }
                None => {
                    let edited = crate::editor::edit("")?;
                    app.intercepted_request = Some(crate::http::Request { raw: edited.into_bytes() });
                }
            }
            *terminal = ratatui::init();
        }

        KeyCode::Char('r') => {
            if let Some(request) = &app.intercepted_request {
                let repeater = crate::repeater::Repeater::new(request.clone());

                app.repeaters.push(repeater);
                app.selected_repeater = app.repeaters.len() - 1;
                app.screen = app::Screen::Repeater;
            }
        }

        KeyCode::Tab => {
            app.screen = app::Screen::Repeater;
        }

        _ => {}
    }

    Ok(())
}

// Repeater
fn render_repeater(frame: &mut Frame, app: &App) {
    let (request, response) = match app.repeaters.get(app.selected_repeater) {
        Some(repeater) => {
            let request = repeater.request.to_str();

            let response = repeater
                .response
                .as_ref()
                .map(|response| response.to_str())
                .unwrap_or_default();

            (request, response)
        }
        None => (String::new(), String::new()),
    };

    frame.render_widget(
        Paragraph::new(format!("REPEATER\n{}\n\n{}", request, response)),
        frame.area(),
    );
}

async fn handle_repeater_input(app: &mut App, key: KeyCode, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    match key {
        KeyCode::Enter => {
            if let Some(repeater) = app.repeaters.get_mut(app.selected_repeater) {
                repeater.send().await?;
            }
        }

        KeyCode::Char('e') => {
            if let Some(repeater) = app.repeaters.get_mut(app.selected_repeater) {
                ratatui::restore();

                repeater.edit()?;

                *terminal = ratatui::init();
            }
        }


        KeyCode::Char('n') => {
            if !app.repeaters.is_empty() {
                app.selected_repeater = (app.selected_repeater + 1) % app.repeaters.len();
            }
        }

        KeyCode::Char('p') => {
            if !app.repeaters.is_empty() {
                app.selected_repeater = (app.selected_repeater + app.repeaters.len() - 1) % app.repeaters.len();
            }
        }

        KeyCode::Tab => {
            app.screen = app::Screen::Proxy;
        }

        _ => {}
    }

    Ok(())
}

