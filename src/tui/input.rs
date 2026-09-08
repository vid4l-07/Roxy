use crossterm::event::KeyCode;
use ratatui::{style::Color, DefaultTerminal};
use tokio::sync::mpsc;
use std::io;

use crate::{app, http, editor, events, repeater, tui::popups};

pub fn error_popup(app: &mut app::App, error: impl Into<String>){
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

pub(super) async fn handle_input(app: &mut app::App, key: KeyCode, terminal: &mut DefaultTerminal, sender: &mpsc::Sender<events::TuiEvents>) -> io::Result<bool>{
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
        app::Screen::Proxy => handle_proxy_input(app, key, terminal, sender).await?,
        app::Screen::Repeater => handle_repeater_input(app, key, terminal, sender).await?,
    }

    if key == KeyCode::Char('q') {
        return Ok(true);
    }
    
    Ok(false)
}

async fn handle_proxy_input(app: &mut app::App, key: KeyCode, terminal: &mut DefaultTerminal, sender: &mpsc::Sender<events::TuiEvents>) -> io::Result<()> {
    match key {
        KeyCode::Char('i') => {
            app.intercept = !app.intercept;
            sender.send(events::TuiEvents::SetIntercept(app.intercept)).await.map_err(|_| 
                io::Error::new(io::ErrorKind::BrokenPipe, "TUI channel closed")
            )?;
        }

        KeyCode::Enter => {
            match &app.request_queue.first() {
                Some(intercepted) => {
                    sender.send(events::TuiEvents::Forward{ id: intercepted.id, request: intercepted.request.clone() } ).await.map_err(|_| 
                        io::Error::new(io::ErrorKind::BrokenPipe, "TUI channel closed")
                    )?;
                    app.request_queue.remove(0) ;
                }
                None => {}
            }
        }

        KeyCode::Char('e') => {
            if let Some(intercepted) = app.request_queue.first_mut() {
                match editor::edit(&intercepted.request.to_str()) {
                    Ok(edited) => {
                        intercepted.request = http::Request::from_edited(
                            &edited, intercepted.request.host.clone(), intercepted.request.port
                        );
                    }

                    Err(e) => {
                        error_popup(app, e.to_string());
                    }
                }
            }

            *terminal = ratatui::init();
        }

        KeyCode::Char('r') => {
            if let Some(intecepted) = app.request_queue.first() {
                let repeater = repeater::Repeater::new(intecepted.request.clone(), (app.repeaters.len() + 1).to_string());

                app.repeaters.push(repeater);
                app.selected_repeater = app.repeaters.len() - 1;
                app.screen = app::Screen::Repeater;
            }
        }

        KeyCode::Tab => {
            app.screen = app::Screen::Repeater;
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

async fn handle_repeater_input(app: &mut app::App, key: KeyCode, terminal: &mut DefaultTerminal, sender: &mpsc::Sender<events::TuiEvents>) -> io::Result<()> {
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
            app.screen = app::Screen::Proxy;
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
