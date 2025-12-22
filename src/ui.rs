use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, palette::tailwind},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, CurrentPane};

pub fn ui(frame: &mut Frame, app: &App) {
    // Define the layout: Top pane takes 1 row, bottom pane takes the rest
    let title_layout = Layout::vertical([
        Constraint::Percentage(10),
        Constraint::Percentage(90),
    ])
    .split(frame.area());

    let main_layout = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(title_layout[1]);

    // Render a title in the top pane
    frame.render_widget(
        Paragraph::new("Header")
        .block(Block::new().borders(Borders::ALL)),
        title_layout[0]);

    // Render local files
    let mut list_items = Vec::<ListItem>::new();
    for (i, path) in app.local_files.iter().enumerate() {
        let style = if i == app.local_idx {
            Style::default().fg(tailwind::GREEN.c400)
            .bg(tailwind::SLATE.c900)
            .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        if path.to_str().expect("Unable to convert path to string") == ".." {
            list_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}", path.to_string_lossy()),
                style,
            ))));
        } else {
            list_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}", path.file_name().expect("Path has no file name").to_string_lossy()),
                style,
            ))));
        }
    }
    
    #[cfg(debug_assertions)]
    list_items.push(ListItem::new(Line::from(Span::styled(
        format!("{} / {}", app.local_idx, app.local_files.len() - 1),
        Style::default().fg(Color::White)
    ))));

    #[cfg(debug_assertions)]
    list_items.push(ListItem::new(Line::from(Span::styled(
        format!("{}", app.local_path.to_string_lossy()),
        Style::default().fg(Color::White)
    ))));
    let list = List::new(list_items);
    frame.render_widget(
        list
            .block(Block::new().borders(Borders::ALL)),
        main_layout[0]);

    frame.render_widget(
        Paragraph::new("Right")
            .block(Block::new().borders(Borders::ALL)),
        main_layout[1]);
}
