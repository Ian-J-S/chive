use ratatui::{
    layout::{Constraint, Layout}, 
    prelude::Margin,
    style::{palette::tailwind, Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame
};

use std::path::PathBuf;
use crate::app::{App, CurrentPane};

pub fn ui(frame: &mut Frame, app: &mut App) {
    // Define the layout: Top pane takes 1 row, bottom pane takes the rest
    let title_layout = Layout::vertical([
        Constraint::Percentage(90),
        Constraint::Percentage(10),
    ])
    .split(frame.area());

    let main_layout = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(title_layout[0]);

    // Render a title in the footer
    frame.render_widget(
        Paragraph::new("Footer")
        .block(Block::new().borders(Borders::ALL)),
        title_layout[1]);

    // Render browser files.
    // Style based on cursor position, selection, or directory.
    let mut browser_items = Vec::<ListItem>::new();
    for (i, path) in app.browser_files.iter().enumerate() {
        let mut style = Style::default().fg(Color::White);
        
        // Check if file is in archive by comparing relative path
        let relative_path = if path.to_str().expect("Unable to convert path to string") == ".." {
            PathBuf::from("..")
        } else {
            path.strip_prefix(app.browser_path.clone()).unwrap_or(path).to_path_buf()
        };
        
        style = if app.archive_names.contains(&relative_path) {
            if i == app.browser_idx {
                style.fg(tailwind::ORANGE.c400)
                    .bg(tailwind::SLATE.c900)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC)
            }
            else {
                Style::default().fg(tailwind::ORANGE.c400)
                .add_modifier(Modifier::ITALIC)
            }
        // Style by cursor position
        } else if i == app.browser_idx {
            Style::default().fg(tailwind::GREEN.c400)
            .bg(tailwind::SLATE.c900)
            .add_modifier(Modifier::BOLD)
        // Color blue if file is a directory 
        } else if path.is_dir() {
            Style::default().fg(tailwind::BLUE.c400)
        } else {
            Style::default().fg(Color::White)
        };

        // Render parent as ".." and skip getting file name
        if path.to_str().expect("Unable to convert path to string") == ".." {
            browser_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}", path.to_string_lossy()),
                style,
            ))));
        } else {
            let file_name = path.file_name().expect("Path has no file name").to_string_lossy();
            browser_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}", file_name),
                style,
            ))));
        }
    }
    
    #[cfg(debug_assertions)]
    {
        browser_items.push(ListItem::new(Line::from(Span::styled(
            "Debug Info:",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
        ))));
        // Show index and total number of files
        browser_items.push(ListItem::new(Line::from(Span::styled(
            format!("idx: {} / {}", app.browser_idx, app.browser_files.len() - 1),
            Style::default().fg(Color::White)
        ))));

        // Show current directory
        browser_items.push(ListItem::new(Line::from(Span::styled(
            format!("cwd: {}", app.browser_path.to_string_lossy()),
            Style::default().fg(Color::White)
        ))));

        // Show whether dotfiles are hidden
        browser_items.push(ListItem::new(Line::from(Span::styled(
            format!("Showing hidden? {}", app.show_hidden),
            Style::default().fg(Color::White)
        ))));
    }

    let browser_list = List::new(browser_items);
    app.browser_list_state.select(Some(app.browser_idx));

    // Change style based on which pane is selected
    let left_block = Block::default()
        .title("Browser")
        .border_style(Style::default())
        .borders(Borders::all());
    let left_block = match app.current_pane {
        CurrentPane::Browser => left_block.border_style(Style::default().add_modifier(Modifier::BOLD).fg(tailwind::BLUE.c400)),
        CurrentPane::Archive => left_block.border_style(Style::default()),
    };

    frame.render_stateful_widget(
        browser_list.block(left_block),
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

    // Create list of selected items
    let mut archive_items = Vec::<ListItem>::new();
    for (i, path) in app.archive_names.iter().enumerate() {
        let style = if i == app.archive_idx {
            Style::default().fg(tailwind::ORANGE.c300)
                .bg(tailwind::SLATE.c900)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let file_name = path.file_name().expect("Path has no file name");
        archive_items.push(ListItem::new(Line::from(Span::styled(
            format!("{}", file_name.to_string_lossy()),
            style,
        ))));
    }

    #[cfg(debug_assertions)]
    {
        archive_items.push(ListItem::new(Line::from(Span::styled(
            "Debug Info:",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
        ))));
        // Show index and total number of files
        archive_items.push(ListItem::new(Line::from(Span::styled(
            if app.archive_names.is_empty() {
                "0 / 0".to_string()
            } else {
                format!("idx: {} / {}", app.archive_idx, app.archive_names.len() - 1)
            },
            Style::default().fg(Color::White)
        ))));

        // Show current directory
        archive_items.push(ListItem::new(Line::from(Span::styled(
            format!("cwd: {}", app.browser_path.to_string_lossy()),
            Style::default().fg(Color::White)
        ))));

        // Show whether dotfiles are hidden
        archive_items.push(ListItem::new(Line::from(Span::styled(
            format!("Showing hidden? {}", app.show_hidden),
            Style::default().fg(Color::White)
        ))));
    }

    let archive_list = List::new(archive_items);

    let right_block_style = match app.current_pane {
        CurrentPane::Browser => {
            if app.archive_names.is_empty() {
                Style::default().add_modifier(Modifier::DIM)
            } else {
                Style::default()
            }
        }
        CurrentPane::Archive => Style::default().add_modifier(Modifier::BOLD).fg(tailwind::ORANGE.c400),
    };
    let right_block = Block::default()
        .title("Archive")
        .border_style(right_block_style)
        .borders(Borders::all());

    frame.render_widget(
        archive_list.block(right_block),
        main_layout[1]);
}
