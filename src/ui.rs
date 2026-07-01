use ratatui::{
    layout::{Constraint, Layout, Rect}, 
    prelude::Margin,
    style::{palette::tailwind, Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame
};

use std::path::{Path, PathBuf};
use crate::app::{App, CurrentPane};

pub fn ui(frame: &mut Frame, app: &mut App) {
    let title_layout = if app.show_footer {
        Layout::vertical([
            Constraint::Percentage(90),
            Constraint::Percentage(10),
        ])
        .split(frame.area())
    } else {
        Layout::vertical([
            Constraint::Percentage(100),
        ]).split(frame.area())
    };

    let main_layout = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(title_layout[0]);

    if app.show_footer {
        render_footer(frame, title_layout[1], &app.current_pane);
    }
    render_browser(frame, app, main_layout[0]);
    render_archive(frame, app, main_layout[1]);
}

fn render_footer(frame: &mut Frame, area: Rect, current_pane: &CurrentPane) {
    let browser_style = Style::default().fg(tailwind::BLUE.c400).add_modifier(Modifier::BOLD);
    let archive_style = Style::default().fg(tailwind::ORANGE.c400).add_modifier(Modifier::BOLD);

    // I kind of hate this but it works for now
    let hints = match current_pane {
        CurrentPane::Browser => {
            Line::from(vec![
                Span::styled("<q>", browser_style),
                Span::raw(": Quit, "),
                Span::styled("<j/k>", browser_style),
                Span::raw(": U/D, "),
                Span::styled("<tab>", browser_style),
                Span::raw(": Toggle Pane, "),
                Span::styled("<space>", browser_style),
                Span::raw(": Change dir, "),
                Span::styled("<.>", browser_style),
                Span::raw(": Toggle hidden, "),
                Span::styled("<a>", browser_style),
                Span::raw(": add, "),
                Span::styled("<l>", browser_style),
                Span::raw(": load archive, "),
                Span::styled("<e>", browser_style),
                Span::raw(": extract archive, "),
                Span::styled("<r>", browser_style),
                Span::raw(": refresh files"),
            ])
        }
        CurrentPane::Archive => {
            Line::from(vec![
                Span::styled("<q>", archive_style),
                Span::raw(": Quit, "),
                Span::styled("<j/k>", archive_style),
                Span::raw(": U/D, "),
                Span::styled("<tab>", archive_style),
                Span::raw(": Toggle Pane, "),
                Span::styled("<c>", archive_style),
                Span::raw(": clear, "),
                Span::styled("<s>", archive_style),
                Span::raw(": save, "),
                Span::styled("<a>", archive_style),
                Span::raw(": remove, "),
            ])
        }
    };

    let paragraph = Paragraph::new(hints)
        .block(Block::default().borders(Borders::ALL).title("Keybinds"));

    frame.render_widget(
        paragraph,
        area,
    );
}

fn render_browser(frame: &mut Frame, app: &mut App, area: Rect) {
    let browser_files = app.browser_files.clone();
    let browser_path = app.browser_path.clone();
    let archive_names = app.archive_names.clone();
    let browser_idx = app.browser_idx;
    let current_pane = app.current_pane;
    let show_hidden = app.show_hidden;

    let browser_list = List::new(build_browser_items(
        &browser_files,
        &archive_names,
        &browser_path,
        browser_idx,
        current_pane,
        show_hidden,
    ));

    let hint_style = Style::default().fg(Color::Gray).add_modifier(Modifier::DIM);
    let hint_text = if !app.show_footer {
        Line::from(vec![
            Span::styled("Press ", hint_style),
            Span::styled("?", Style::default().fg(tailwind::BLUE.c400).add_modifier(Modifier::BOLD)),
            Span::styled(" for help", hint_style),
        ])
    } else {
        Line::from(vec![])
    };

    let browser_block = browser_block(current_pane)
        .title_bottom(hint_text);

    app.browser_list_state.select(Some(browser_idx));

    frame.render_stateful_widget(
        browser_list.block(browser_block),
        area,
        &mut app.browser_list_state,
    );

    if browser_files.len() > area.height as usize {
        app.browser_scrollbar = app
            .browser_scrollbar
            .content_length(browser_files.len())
            .position(browser_idx);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("k"))
            .end_symbol(Some("j"));

        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut app.browser_scrollbar,
        );
    }
}

fn render_archive(frame: &mut Frame, app: &mut App, area: Rect) {
    let archive_list = List::new(build_archive_items(app));

    frame.render_widget(archive_list.block(archive_block(app)), area);
}

fn build_browser_items(
    browser_files: &[PathBuf],
    archive_names: &std::collections::HashSet<PathBuf>,
    browser_path: &Path,
    browser_idx: usize,
    current_pane: CurrentPane,
    show_hidden: bool,
) -> Vec<ListItem<'static>> {
    let mut browser_items = Vec::new();

    for (i, path) in browser_files.iter().enumerate() {
        let style = browser_item_style(
            archive_names,
            browser_path,
            browser_idx,
            current_pane,
            i,
            path,
        );
        let label = browser_item_label(path);

        browser_items.push(ListItem::new(Line::from(Span::styled(label, style))));
    }

    if cfg!(debug_assertions) {
        append_browser_debug_info(
            &mut browser_items,
            browser_idx,
            browser_files.len(),
            browser_path,
            show_hidden,
        );
    }

    browser_items
}

fn browser_item_style(
    archive_names: &std::collections::HashSet<PathBuf>,
    browser_path: &Path,
    browser_idx: usize,
    current_pane: CurrentPane,
    index: usize,
    path: &Path,
) -> Style {
    let relative_path = if path.to_str().expect("Unable to convert path to string") == ".." {
        PathBuf::from("..")
    } else {
        path.strip_prefix(browser_path)
            .unwrap_or(path)
            .to_path_buf()
    };

    if archive_names.contains(&relative_path) {
        if index == browser_idx {
            Style::default()
                .fg(tailwind::ORANGE.c400)
                .bg(tailwind::SLATE.c900)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC)
        } else {
            Style::default()
                .fg(tailwind::ORANGE.c400)
                .add_modifier(Modifier::ITALIC)
        }
    } else if index == browser_idx && current_pane == CurrentPane::Browser {
        Style::default()
            .fg(tailwind::GREEN.c400)
            .bg(tailwind::SLATE.c900)
            .add_modifier(Modifier::BOLD)
    } else if index == browser_idx && current_pane == CurrentPane::Archive {
        Style::default().fg(tailwind::GREEN.c400)
    } else if path.is_dir() {
        Style::default().fg(tailwind::BLUE.c400)
    } else {
        Style::default().fg(Color::White)
    }
}

fn browser_item_label(path: &Path) -> String {
    if path.to_str().expect("Unable to convert path to string") == ".." {
        path.to_string_lossy().into_owned()
    } else {
        path.file_name()
            .expect("Path has no file name")
            .to_string_lossy()
            .into_owned()
    }
}

fn append_browser_debug_info(
    items: &mut Vec<ListItem>,
    browser_idx: usize,
    browser_files_len: usize,
    browser_path: &Path,
    show_hidden: bool,
) {
    items.push(ListItem::new(Line::from(Span::styled(
        "Debug Info:",
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
    ))));

    items.push(ListItem::new(Line::from(Span::styled(
        format!("idx: {} / {}", browser_idx, browser_files_len.saturating_sub(1)),
        Style::default().fg(Color::White),
    ))));

    items.push(ListItem::new(Line::from(Span::styled(
        format!("cwd: {}", browser_path.to_string_lossy()),
        Style::default().fg(Color::White),
    ))));

    items.push(ListItem::new(Line::from(Span::styled(
        format!("Showing hidden? {}", show_hidden),
        Style::default().fg(Color::White),
    ))));
}

fn build_archive_items(app: &App) -> Vec<ListItem<'static>> {
    let mut archive_items = Vec::new();

    for (i, path) in app.archive_names.iter().enumerate() {
        let style = archive_item_style(app, i);
        archive_items.push(ListItem::new(Line::from(Span::styled(
            archive_item_label(path),
            style,
        ))));
    }

    if cfg!(debug_assertions) {
        append_archive_debug_info(&mut archive_items, app);
    }

    archive_items
}

fn archive_item_style(app: &App, index: usize) -> Style {
    if index == app.archive_idx && app.current_pane == CurrentPane::Archive {
        Style::default()
            .fg(tailwind::ORANGE.c300)
            .bg(tailwind::SLATE.c900)
            .add_modifier(Modifier::BOLD)
    } else if index == app.archive_idx && app.current_pane == CurrentPane::Browser {
        Style::default().fg(tailwind::ORANGE.c300)
    } else {
        Style::default().fg(Color::White)
    }
}

fn archive_item_label(path: &Path) -> String {
    path.file_name()
        .expect("Path has no file name")
        .to_string_lossy()
        .into_owned()
}

fn append_archive_debug_info(items: &mut Vec<ListItem>, app: &App) {
    items.push(ListItem::new(Line::from(Span::styled(
        "Debug Info:",
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
    ))));

    let idx_info = if app.archive_names.is_empty() {
        "0 / 0".to_string()
    } else {
        format!("idx: {} / {}", app.archive_idx, app.archive_names.len().saturating_sub(1))
    };

    items.push(ListItem::new(Line::from(Span::styled(
        idx_info,
        Style::default().fg(Color::White),
    ))));

    items.push(ListItem::new(Line::from(Span::styled(
        format!("cwd: {}", app.browser_path.to_string_lossy()),
        Style::default().fg(Color::White),
    ))));

    items.push(ListItem::new(Line::from(Span::styled(
        format!("Showing hidden? {}", app.show_hidden),
        Style::default().fg(Color::White),
    ))));
}

fn browser_block(current_pane: CurrentPane) -> Block<'static> {
    let block = Block::default()
        .title("Browser")
        .border_style(Style::default())
        .borders(Borders::all());

    match current_pane {
        CurrentPane::Browser => block.border_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(tailwind::BLUE.c400),
        ),
        CurrentPane::Archive => block,
    }
}

fn archive_block(app: &App) -> Block<'static> {
    let border_style = match app.current_pane {
        CurrentPane::Browser => {
            if app.archive_names.is_empty() {
                Style::default().add_modifier(Modifier::DIM)
            } else {
                Style::default()
            }
        }
        CurrentPane::Archive => Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(tailwind::ORANGE.c400),
    };

    Block::default()
        .title("Archive")
        .border_style(border_style)
        .borders(Borders::all())
}
