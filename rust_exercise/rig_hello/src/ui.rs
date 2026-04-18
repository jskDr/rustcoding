use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Role};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Chat history
            Constraint::Length(1), // Status bar
            Constraint::Length(3), // Input box
        ])
        .split(f.area());

    render_chat_history(f, app, chunks[0]);
    render_status_bar(f, app, chunks[1]);
    render_input(f, app, chunks[2]);
}

fn render_chat_history(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();

    for msg in &app.messages {
        let (role_name, style) = match msg.role {
            Role::User => ("You: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Role::Teacher => ("Teacher: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Role::Reviewer => ("Reviewer: ", Style::default().fg(Color::Yellow)),
            Role::System => ("System: ", Style::default().fg(Color::Green)),
        };

        // We split content by lines so we can color "thinking" sections slightly differently if needed.
        // For simplicity, we just use dark gray for thinking sections (if it's enclosed in [thinking][/thinking])
        // or just apply basic styling.
        // (we previously mapped msg_text here but we recreate it later)
        
        let header = Line::from(vec![Span::styled(role_name, style)]);
        lines.push(header);

        for line in msg.content.lines() {
            let line_style = if line.contains("[thinking]") || line.contains("[/thinking]") || line.contains("Thinking Process:") {
                Style::default().fg(Color::DarkGray)
            } else if line.starts_with(">") { // simplistic heuristic for thinking/reasoning parts sometimes
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            lines.push(Line::from(Span::styled(line.to_string(), line_style)));
        }

        // Add an empty line between messages
        lines.push(Line::from(""));
    }

    let chat_text = Text::from(lines);
    let inner_height = area.height.saturating_sub(2); // account for borders
    let total_lines = chat_text.lines.len() as u16;

    let scroll = if app.auto_scroll {
        total_lines.saturating_sub(inner_height)
    } else {
        // Clamp manual scroll to valid range
        app.scroll_offset.min(total_lines.saturating_sub(inner_height))
    };

    let paragraph = Paragraph::new(chat_text)
        .block(Block::default().borders(Borders::ALL).title(" Chat History "))
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));

    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let text = format!(
        " Status: {} | Review Mode: {} | History: {} ",
        app.status,
        if app.review_mode { "ON (F2 to toggle)" } else { "OFF (F2 to toggle)" },
        app.history.len() / 2 // Rough pair count
    );
    let paragraph = Paragraph::new(text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(paragraph, area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let input_text = if app.input.is_empty() && !app.is_processing {
        Line::from(Span::styled("Type a message...", Style::default().fg(Color::DarkGray)))
    } else {
        Line::from(app.input.clone())
    };

    let paragraph = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title(" Input "));

    f.render_widget(paragraph, area);

    // Set cursor position
    if !app.is_processing {
        f.set_cursor_position((
            area.x + 1 + app.cursor_pos as u16,
            area.y + 1,
        ));
    }
}
