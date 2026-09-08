use ratatui::{
    Frame, 
    layout::{Constraint, Layout},
    prelude::*, 
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Tabs, Wrap}
};

use crate::{app, repeater};

pub(super) fn render_proxy(frame: &mut Frame, app: &app::App) {
    let request = app.request_queue.first().map(|request| request.request.to_str()).unwrap_or_default();
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

    if let Some(intercepted) = app.request_queue.first() {
        if !intercepted.request.host.is_empty() {
            block = block.title_bottom(
                Line::styled(
                    format!(" Host: {}:{} ", intercepted.request.host, intercepted.request.port),
                    Style::default().fg(Color::Yellow),
                )
            );
        }
    }

    let request_info = Paragraph::new(request).wrap(Wrap { trim: false })
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
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, vertical[2]);

}

pub(super) fn render_repeater(frame: &mut Frame, app: &app::App) {
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

    if let Some(repeater) = app.repeaters.get(app.selected_repeater) {
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

    let request_info = Paragraph::new(request).wrap(Wrap { trim: false })
        .block(block)
        .scroll((request_scroll, 0));
    frame.render_widget(request_info, horizontal[0]);


    let response_info = Paragraph::new(response).wrap(Wrap { trim: false }).block(
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
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, vertical[2]);

}
