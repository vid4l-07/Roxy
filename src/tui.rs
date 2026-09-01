use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;

use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Paragraph, Tabs, Borders, BorderType, Padding},
    layout::{Constraint, Direction, Layout},
    Frame
};

use tokio::sync::mpsc;

use std::io;

use crate::{app::{Screen, App}, events, repeater, popups};

// Secondary

pub fn error_popup(app: &mut App, error: impl Into<String>){
    app.popups.push(
        popups::Popup::Text(
            popups::TextPopup {
                message: error.into(),
                title: " Error ".into(),
                color: Color::Red,
            }
        )
    );

}

// General
pub async fn run(app: &mut App, sender: mpsc::Sender<events::TuiEvents>, mut receiver: mpsc::Receiver<events::ProxyEvents>) -> io::Result<()> {
    let mut terminal = ratatui::init();

    let result = main_loop(app, sender, receiver, &mut terminal).await;

    ratatui::restore();

    if let Err(ref e) = result {
        eprintln!("Fatal error: {}", e);
    }

    result
}

async fn main_loop(app: &mut App, sender: mpsc::Sender<events::TuiEvents>, mut receiver: mpsc::Receiver<events::ProxyEvents>, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
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
                    Some(events::ProxyEvents::ReceivedRequest(request)) => {
                        app.intercepted_request = Some(request);
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
        Screen::Proxy => render_proxy(frame, app),
        Screen::Repeater => render_repeater(frame, app),
    }

    if let Some(popup) = app.popups.last() {
        popup.render(frame);
    }
}

async fn handle_input(app: &mut App, key: KeyCode, terminal: &mut ratatui::DefaultTerminal, sender: &mpsc::Sender<events::TuiEvents>) -> io::Result<bool>{
    if key == KeyCode::Char('q') {
        return Ok(true);
    }

    if let Some(popup) = app.popups.last() {
        match popup.handle_input(key) {
            popups::PopupAction::Close => {
                app.popups.pop();
            }
            popups::PopupAction::None => {}
        }

        return Ok(false);
    }
    
    match app.screen {
        Screen::Proxy => handle_proxy_input(app, key, terminal, sender).await?,
        Screen::Repeater => handle_repeater_input(app, key, terminal).await?,
    }
    Ok(false)
}

// Proxy

fn render_proxy(frame: &mut Frame, app: &App) {
    let request = app.intercepted_request.as_ref().map(|request| request.to_str()).unwrap_or_default();
    let vertical = Layout::vertical([
        Constraint::Length(3), 
        Constraint::Min(0),   
        Constraint::Length(1),
    ]).split(frame.area());

    let title = Paragraph::new("Proxy").block(
        Block::default()
            .padding(Padding::new(2, 0, 1, 1))
    );

    frame.render_widget(title, vertical[0]);

    let request_scroll: u16 = 0;
    let request_info = Paragraph::new(request).block(
        Block::bordered().border_type(BorderType::Rounded).title(Line::from(vec![
        Span::raw(" Intercept: "),
        Span::styled(
            if app.intercept { "ON " } else { "OFF " },
            if app.intercept {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            },
        ),
        ]))).scroll((request_scroll, 0));

    frame.render_widget(request_info, vertical[1]);

    let help = Paragraph::new(
        "[↑↓] Scroll  [Tab] Switch  [e] Edit  [Enter] Send  [q] Quit"
    ).style(Style::default().fg(Color::Black));

    frame.render_widget(help, vertical[2]);

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
            let request = app.intercepted_request
                .as_ref()
                .map(|request| request.to_str())
                .unwrap_or_default();

            match crate::editor::edit(&request) {
                Ok(edited) => {
                    app.intercepted_request = Some(crate::http::Request {
                        raw: edited.into_bytes(),
                    });
                }

                Err(e) => {
                    error_popup(app, e.to_string());
                }
            }

            *terminal = ratatui::init();
        }

        KeyCode::Char('r') => {
            if let Some(request) = &app.intercepted_request {
                let repeater = crate::repeater::Repeater::new(request.clone());

                app.repeaters.push(repeater);
                app.repeaters_names.push(app.repeaters.len().to_string());
                app.selected_repeater = app.repeaters.len() - 1;
                app.screen = Screen::Repeater;
            }
        }

        KeyCode::Tab => {
            app.screen = Screen::Repeater;
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

            let response = repeater.response.as_ref().map(|response| response.to_str()).unwrap_or_default();

            (request, response)
        }
        None => (String::new(), String::new()),
    };

    let vertical = Layout::vertical([
        Constraint::Length(4), 
        Constraint::Min(0),   
        Constraint::Length(1),
    ]).split(frame.area());

    let horizontal = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]).split(vertical[1]);


    let tabs = Tabs::new(app.repeaters_names.iter().map(|title| Line::from(title.clone())).collect::<Vec<_>>())
        .block(
            Block::bordered().border_type(BorderType::Rounded).title(" Repeaters "),
        ).select(app.selected_repeater);
    frame.render_widget(tabs, vertical[0]);


    let request_scroll: u16 = 0;
    let request_info = Paragraph::new(request).block(
        Block::bordered().border_type(BorderType::Rounded).title(" Request ")
    ).scroll((request_scroll, 0));
    frame.render_widget(request_info, horizontal[0]);


    let response_scroll: u16 = 0;
    let response_info = Paragraph::new(response).block(
        Block::bordered().border_type(BorderType::Rounded).title(" Response ")
    ).scroll((response_scroll, 0));
    frame.render_widget(response_info, horizontal[1]);


    let help = Paragraph::new(
        "[↑↓] Scroll  [Tab] Switch  [e] Edit  [Enter] Send  [q] Quit"
    ).style(Style::default().fg(Color::Black));
    frame.render_widget(help, vertical[2]);

}

async fn handle_repeater_input(app: &mut App, key: KeyCode, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    match key {
        KeyCode::Enter => {
            if let Some(repeater) = app.repeaters.get_mut(app.selected_repeater) {
                if let Err(e) = repeater.send().await {
                    error_popup(app, e.to_string());
                }
            }
        }

        KeyCode::Char('e') => {
            if let Some(repeater) = app.repeaters.get_mut(app.selected_repeater) {
                ratatui::restore();

                let result = repeater.edit();

                *terminal = ratatui::init();

                if let Err(e) = result {
                    error_popup(app, e.to_string());
                }
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
            app.screen = Screen::Proxy;
        }

        _ => {}
    }

    Ok(())
}

