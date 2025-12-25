use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, 
    prelude::Margin,
    style::{palette::tailwind, Color, Modifier, Style},
    symbols::scrollbar, text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap},
    Frame
};

use crate::app::{App, CurrentPane};

pub fn ui(frame: &mut Frame, app: &mut App) {
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

    // Render browser files
    let mut list_items = Vec::<ListItem>::new();
    for (i, path) in app.browser_files.iter().enumerate() {
        // Style based on cursor, selection, or directory.
        // TODO - There has got to be a more elegant way to do this lol.
        let style = if app.selected_files.contains(path) {
            if i == app.browser_idx {
                Style::default().fg(tailwind::ORANGE.c400)
                    .bg(tailwind::SLATE.c900)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC)
            }
            else {
                Style::default().fg(tailwind::ORANGE.c400)
                .add_modifier(Modifier::ITALIC)
            }
        } else if i == app.browser_idx {
            Style::default().fg(tailwind::GREEN.c400)
            .bg(tailwind::SLATE.c900)
            .add_modifier(Modifier::BOLD)
        } else if path.is_dir() {
            Style::default().fg(tailwind::BLUE.c400)
        } else {
            Style::default().fg(Color::White)
        };

        // Render parent as .. and skip getting file name
        if path.to_str().expect("Unable to convert path to string") == ".." {
            list_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}", path.to_string_lossy()),
                style,
            ))));
        } else {
            let file_name = path.file_name().expect("Path has no file name").to_string_lossy();
            list_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}", file_name),
                style,
            ))));
        }
    }
    
    #[cfg(debug_assertions)]
    {
        // Show index and total number of files
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!("{} / {}", app.browser_idx, app.browser_files.len() - 1),
            Style::default().fg(Color::White)
        ))));

        // Show current directory
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!("{}", app.browser_path.to_string_lossy()),
            Style::default().fg(Color::White)
        ))));

        // Show whether dotfiles are hidden
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!("Showing hidden? {}", app.show_hidden),
            Style::default().fg(Color::White)
        ))));
    }

    let list = List::new(list_items);
    app.browser_list_state.select(Some(app.browser_idx));
    frame.render_stateful_widget(
        list.block(Block::new().borders(Borders::ALL)),
        main_layout[0],
        &mut app.browser_list_state,
    );

    // Only show scrollbar if there are enough items
    if app.browser_files.len() > main_layout[0].height as usize {
        app.browser_scrollbar = app.browser_scrollbar
            .content_length(app.browser_files.len())
            .position(app.browser_idx);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("k")).end_symbol(Some("j")); // idk abut these lol

        // Render scrollbar
        frame.render_stateful_widget(
            scrollbar,
            main_layout[0].inner(Margin { vertical: 1, horizontal: 0 }),
            &mut app.browser_scrollbar,
        );
    }

    frame.render_widget(
        Paragraph::new("Right")
            .block(Block::new().borders(Borders::ALL)),
        main_layout[1]);
}
