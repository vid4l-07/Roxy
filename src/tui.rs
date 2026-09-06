use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Clear, Block, Paragraph, Tabs, Borders, BorderType, Padding},
    layout::{Constraint, Layout},
    Frame
};
use tokio::sync::mpsc;
use std::io;

use crate::{app::{Screen, App}, http, editor, events, repeater, popups};

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
pub async fn run(app: &mut App, sender: mpsc::Sender<events::TuiEvents>, receiver: mpsc::Receiver<events::ProxyEvents>) -> io::Result<()> {
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
    if let Some(popup) = app.popups.last_mut() {
        match popup.handle_input(key) {
            popups::PopupAction::Close => {
                app.popups.pop();
            }

            popups::PopupAction::Input(name) => {
                let name = name.trim();

                if !name.is_empty() {
                    if let Some(index) = app.renaming_repeater {
                        if let Some(repeater) = app.repeaters.get_mut(index) {
                            repeater.name = name.to_owned();
                        }
                    }
                }

                app.renaming_repeater = None;
                app.popups.pop();
            }
            popups::PopupAction::None => {}
        }

        return Ok(false);
    }

    match app.screen {
        Screen::Proxy => handle_proxy_input(app, key, terminal, sender).await?,
        Screen::Repeater => handle_repeater_input(app, key, terminal, sender).await?,
    }

    if key == KeyCode::Char('q') {
        return Ok(true);
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

    let status_color = if app.intercept {
        Color::Green
    } else {
        Color::Red
    };
    let status_text = if app.intercept { "ON" } else { "OFF" };

    let line = Line::from(vec![
        Span::styled(
            " Proxy ",
            Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  Intercept: "),
        Span::styled(
            format!("[{}]", status_text),
            Style::default()
            .fg(status_color)
            .add_modifier(Modifier::BOLD),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let paragraph = Paragraph::new(line).block(block);

    frame.render_widget(paragraph, vertical[0]);

    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" Request ");

    if let Some(request) = &app.intercepted_request {
        if !request.host.is_empty() {
            block = block.title_bottom(
                Line::styled(
                    format!(" Host: {}:{} ", request.host, request.port),
                    Style::default().fg(Color::Yellow),
                )
            );
        }
    }

    let request_info = Paragraph::new(request)
        .block(block)
        .scroll((app.proxy_scroll, 0));
    frame.render_widget(request_info, vertical[1]);

    let help = Paragraph::new(
        "[↑↓/jk] Scroll  [Tab] Switch  [Enter] Send  [q] Quit  [i] Intercept  [e] Edit  [r] Repeater",
    )
        .alignment(Alignment::Right)
        .block(
            Block::default()
            .padding(Padding::new(0, 2, 0, 0)),
        )
        .style(Style::default().fg(Color::Black));

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
            if let Some(request) = &app.intercepted_request {
                match editor::edit(&request.to_str()) {
                    Ok(edited) => {
                        app.intercepted_request = Some(http::Request::from_edited(&edited, request.host.clone(), request.port));
                    }

                    Err(e) => {
                        error_popup(app, e.to_string());
                    }
                }
            }

            *terminal = ratatui::init();
        }

        KeyCode::Char('r') => {
            if let Some(request) = &app.intercepted_request {
                let repeater = repeater::Repeater::new(request.clone(), (app.repeaters.len() + 1).to_string());

                app.repeaters.push(repeater);
                app.selected_repeater = app.repeaters.len() - 1;
                app.screen = Screen::Repeater;
            }
        }

        KeyCode::Tab => {
            app.screen = Screen::Repeater;
        }

        KeyCode::Down | KeyCode::Char('j') => {
            scroll_down(&mut app.proxy_scroll);
        }

        KeyCode::Up | KeyCode::Char('k') => {
            scroll_up(&mut app.proxy_scroll);
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
        Constraint::Percentage(app.request_size),
        Constraint::Percentage(100 - app.request_size),
    ]).split(vertical[1]);


    let mut tab_width = 4;
    if !app.repeaters.is_empty(){
        tab_width = app.repeaters.iter().map(|repeater| repeater.name.len() + 4).sum::<usize>() / app.repeaters.len();
    }
    let available_width = vertical[0].width as usize;
    let max_tabs = (available_width / tab_width).max(1);
    
    let start = if app.selected_repeater >= max_tabs {
        app.selected_repeater - max_tabs + 1
    } else {
        0
    };

    let tabs = Tabs::new(
        app.repeaters[start..].iter().map(|repeater| Line::from(repeater.name.clone()))
        .collect::<Vec<_>>()
    )
        .block(
            Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Repeaters "),
        )
        .select(app.selected_repeater - start);
    frame.render_widget(Clear, vertical[0]);
    frame.render_widget(tabs, vertical[0]);


    let mut request_border_color = Color::Reset;
    let mut response_border_color = Color::Reset;

    match app.repeater_focus {
        repeater::RepeaterFocus::Request => {
            request_border_color = Color::Blue;
        }
        repeater::RepeaterFocus::Response => {
            response_border_color = Color::Blue;
        }
    }

    let mut block = Block::bordered()
    .border_type(BorderType::Rounded).border_style(Style::default().fg(request_border_color))
    .title(" Request ");

    if let Some(repeater) = app.repeaters.get(app.selected_repeater) {
        if repeater.thinking{
            block = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(request_border_color))
                .title(" Request ")
                .title(
                    Line::from(" Sending... ")
                    .alignment(Alignment::Right)
                );
        }
    }

    let mut request_scroll = 0;
    let mut response_scroll = 0;

    if let Some(repeater) = &app.repeaters.get(app.selected_repeater) {
        request_scroll = repeater.request_scroll;
        response_scroll = repeater.response_scroll;
        if !repeater.request.host.is_empty() {
            block = block.title_bottom(
                Line::styled(
                    format!(" Host: {}:{} ", repeater.request.host, repeater.request.port),
                    Style::default().fg(Color::Yellow),
                )
            );

        }
    }

    let request_info = Paragraph::new(request)
        .block(block)
        .scroll((request_scroll, 0));
    frame.render_widget(request_info, horizontal[0]);


    let response_info = Paragraph::new(response).block(
        Block::bordered().border_type(BorderType::Rounded)
        .border_type(BorderType::Rounded).border_style(Style::default().fg(response_border_color))
        .title(" Response ")
    ).scroll((response_scroll, 0));
    frame.render_widget(response_info, horizontal[1]);

    let help = Paragraph::new(
        "[↑↓/jk] Scroll  [←→/hl] Focus  [Tab] Switch  [Enter] Send  [q] Quit  [e] Edit  [n] Next  [p] Prev  [r] Rename  [x] Close  [H/L] Resize"
    )
        .alignment(Alignment::Right)
        .block(
            Block::default()
            .padding(Padding::new(0, 2, 0, 0)),
        )
        .style(Style::default().fg(Color::Black));

    frame.render_widget(help, vertical[2]);

}

async fn handle_repeater_input(app: &mut App, key: KeyCode, terminal: &mut ratatui::DefaultTerminal, sender: &mpsc::Sender<events::TuiEvents>) -> io::Result<()> {
    match key {
        KeyCode::Enter => {
            if let Some(repeater) = app.repeaters.get_mut(app.selected_repeater) {
                sender.send(
                    events::TuiEvents::SendRepeater{
                        index: app.selected_repeater,
                        request: repeater.request.clone()
                    }
                ).await.map_err(|_| 
                    io::Error::new(io::ErrorKind::BrokenPipe, "TUI channel closed")
                )?;
                repeater.thinking = true;
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

        KeyCode::Char('x') => {
            if app.repeaters.get(app.selected_repeater).is_some() {
                app.repeaters.remove(app.selected_repeater);
                app.selected_repeater = app.selected_repeater.saturating_sub(1);
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

        KeyCode::Char('r') => {
            if let Some(_) = app.repeaters.get(app.selected_repeater) {
                app.renaming_repeater = Some(app.selected_repeater);

                app.popups.push(popups::Popup::Input(
                        popups::InputPopup {
                            input: String::new(),
                            title: " Rename ".to_owned(),
                            color: Color::Reset,
                        }
                    )
                );

            }
        }

        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(repeater) = app.repeaters.get_mut(app.selected_repeater) {
                match app.repeater_focus {
                    repeater::RepeaterFocus::Request => {
                        scroll_down(&mut repeater.request_scroll);
                    }

                    repeater::RepeaterFocus::Response => {
                        scroll_down(&mut repeater.response_scroll);
                    }
                }
            }
        }

        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(repeater) = app.repeaters.get_mut(app.selected_repeater) {
                match app.repeater_focus {
                    repeater::RepeaterFocus::Request => {
                        scroll_up(&mut repeater.request_scroll);
                    }

                    repeater::RepeaterFocus::Response => {
                        scroll_up(&mut repeater.response_scroll);
                    }
                }
            }
        }

        KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
            match app.repeater_focus {
                repeater::RepeaterFocus::Request => {
                    app.repeater_focus = repeater::RepeaterFocus::Response;
                }
                repeater::RepeaterFocus::Response => {
                    app.repeater_focus = repeater::RepeaterFocus::Request;
                }
            }
        }

        KeyCode::Char('H') => {
            app.request_size = (app.request_size - 10).clamp(20,80);
        }
        KeyCode::Char('L') => {
            app.request_size = (app.request_size + 10).clamp(20,80);
        }

        KeyCode::Tab => {
            app.screen = Screen::Proxy;
        }

        _ => {}
    }

    Ok(())
}

fn scroll_down(scroll: &mut u16) {
    *scroll = scroll.saturating_add(2);
}

fn scroll_up(scroll: &mut u16) {
    *scroll = scroll.saturating_sub(2);
}
