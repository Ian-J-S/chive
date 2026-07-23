use anyhow::Result;
use std::io;

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

mod app;
mod archive;
mod archive_state;
mod browser;
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

fn run_app<B>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> 
where 
    B: Backend,
    B::Error: Send + Sync + 'static,
{
    app.browser_state.files = app.browser_state.get_files()?;
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
                InputMode::Normal => match app.current_pane {
                    CurrentPane::Browser => handle_browser_key(app, key)?,
                    CurrentPane::Archive => handle_archive_key(app, key)?,
                },
                InputMode::CompressionStrength => handle_comp_strength_key(app, key)?,
                InputMode::SaveWindow => handle_save_window_keys(app, key)?,
                InputMode::ArchiveType => handle_archive_type_key(app, key)?,
            }
        }
    }
}

fn handle_browser_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.browser_state.update_idx(1),
        KeyCode::Char('k') | KeyCode::Up => app.browser_state.update_idx(-1),
        KeyCode::Char('r') => app.browser_state.refresh()?,
        KeyCode::Char(' ') => app.browser_state.change_browser_dir()?,
        KeyCode::Char('.') => app.browser_state.toggle_hidden_files()?,
        KeyCode::Char('l') => app.load_archive()?,
        KeyCode::Tab => app.toggle_pane(),
        KeyCode::Char('?') => app.toggle_footer(),
        KeyCode::Char('a') => {
            app.archive_state.add_file(
                &app.browser_state.get_selected_browser(),
                &app.browser_state.current_path,
            )?;
        },
        KeyCode::Char('e') => {
            app.extract_archive()?;
            app.browser_state.refresh()?;
        }
        KeyCode::Char('s') => {
            app.input_mode = InputMode::SaveWindow;
        }
        _ => {}
    };
    Ok(())
}

fn handle_archive_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Tab => app.toggle_pane(),
        KeyCode::Char('j') | KeyCode::Down => app.archive_state.update_archive_idx(1),
        KeyCode::Char('k') | KeyCode::Up => app.archive_state.update_archive_idx(-1),
        KeyCode::Char('?') => app.toggle_footer(),
        KeyCode::Char('e') => {
            app.extract_archive()?;
            app.browser_state.refresh()?;
        }
        KeyCode::Char('c') => {
            app.clear_archive();
            app.browser_state.refresh()?;
        }
        KeyCode::Char('C') => {
            app.clear_archive();
            app.input_mode = InputMode::ArchiveType;
        }
        KeyCode::Char('s') => {
            app.input_mode = InputMode::SaveWindow;
        }
        KeyCode::Char('a') => {
            app.remove_from_archive()?;
            app.browser_state.refresh()?;
        }
        _ => {}
    };
    Ok(())
}

fn handle_save_window_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Enter => {
            let msg = app.archive_state.save_archive()?;
            app.browser_state.refresh()?;
            app.input_mode = InputMode::Normal;
            app.set_info_msg(&msg);
        }
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Backspace => {
            let _ = app.archive_state.save_filename.pop();
        }
        // Use other characters to build filename
        KeyCode::Char(to_insert) => app.archive_state.enter_save_char(to_insert),
        _ => {}
    };

    Ok(())
}

fn handle_comp_strength_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('+') => app.archive_state.increase_comp_strength(),
        KeyCode::Char('-') => app.archive_state.decrease_comp_strength(),
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
            app.input_mode = InputMode::Normal;
            app.set_info_msg(&format!(
                "Compression strength set to {}",
                app.archive_state.compression_strength
            ));
        },
        _ => {}
    };

    Ok(())
}

fn handle_archive_type_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Left | KeyCode::Up | KeyCode::Char('h') | KeyCode::Char('k') => {
            app.archive_state.archive_type = app.archive_state.archive_type.previous()
        }
        KeyCode::Right | KeyCode::Down | KeyCode::Char('l') | KeyCode::Char('j') => {
            app.archive_state.archive_type = app.archive_state.archive_type.next()
        }
        KeyCode::Esc | KeyCode::Char('q') => app.input_mode = InputMode::Normal, // defaults to zip
        KeyCode::Enter => {
            app.input_mode = InputMode::CompressionStrength;
            app.set_info_msg(&format!("Chose {}", app.archive_state.archive_type));
        },
        _ => {}
    }
    Ok(())
}
