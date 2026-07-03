use anyhow::Result;
use std::io;

use ratatui::{
    Terminal, backend::{Backend, CrosstermBackend}, crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    }
};

mod app;
mod ui;
mod util;
use crate::{
    app::{App, CurrentPane, InputMode},
    ui::ui,
};

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    app.browser_files = app.get_browser_files().unwrap(); // TODO - should replace with some app.init function
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.input_mode {
                InputMode::Normal => {
                    match app.current_pane {
                        CurrentPane::Browser => handle_browser_key(app, key)?,
                        CurrentPane::Archive => handle_archive_key(app, key)?,
                    }
                }
                InputMode::SaveWindow => handle_save_window_keys(app, key)?,
            }
        }
    }
}

fn handle_browser_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.update_browser_idx(1),
        KeyCode::Char('k') | KeyCode::Up => app.update_browser_idx(-1),
        KeyCode::Char('r') => app.refresh()?,
        KeyCode::Char(' ') => app.change_browser_dir()?,
        KeyCode::Char('.') => app.toggle_hidden_files()?,
        KeyCode::Char('a') => app.add_file_to_archive()?,
        KeyCode::Char('l') => app.load_archive()?,
        KeyCode::Tab => app.toggle_pane(),
        KeyCode::Char('?') => app.toggle_footer(),
        KeyCode::Char('e') => {
            app.extract_archive()?;
            app.refresh()?;
        }
        _ => {}
    };
    Ok(())
}

fn handle_archive_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Tab => app.toggle_pane(),
        KeyCode::Char('j') | KeyCode::Down => app.update_archive_idx(1),
        KeyCode::Char('k') | KeyCode::Up => app.update_archive_idx(-1),
        KeyCode::Char('?') => app.toggle_footer(),
        KeyCode::Char('e') => {
            app.extract_archive()?;
            app.refresh()?;
        }
        KeyCode::Char('c') => {
            app.clear_archive();
            app.refresh()?;
        }
        KeyCode::Char('s') => {
            app.input_mode = InputMode::SaveWindow;
        }
        KeyCode::Char('a') => {
            app.remove_from_archive()?;
            app.refresh()?;
        }
        _ => {}
    };
    Ok(())
}

fn handle_save_window_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Enter => {
            app.save_archive()?;
            app.refresh()?;
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Backspace => {
            let _ = app.save_filename.pop();
        }
        // Use other characters to build filename
        KeyCode::Char(to_insert) => app.enter_save_char(to_insert),
        _ => {}
    };

    Ok(())
}
