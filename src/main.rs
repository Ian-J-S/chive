use anyhow::Result;
use std::io;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app;
mod ui;
mod util;
use crate::{
    app::{App, CurrentPane},
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
    app.browser_files = app.get_browser_files().unwrap(); // TODO - should replace with some app.init function
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_pane {
                CurrentPane::Browser => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(false);
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.update_browser_idx(1);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.update_browser_idx(-1);
                    }
                    KeyCode::Char('r') => {
                        app.refresh()?;
                    }
                    KeyCode::Char(' ') => {
                        app.change_browser_dir()?;
                    }
                    KeyCode::Char('.') => {
                        app.toggle_hidden_files()?;
                    }
                    KeyCode::Char('a') => {
                        app.add_file_to_archive()?;
                    }
                    KeyCode::Char('l') => {
                        app.load_archive()?;
                    }
                    KeyCode::Char('e') => {
                        // Attemp to extract an existing archive
                        app.extract_archive()?;
                        app.refresh()?;
                    }
                    KeyCode::Tab => {
                        app.toggle_pane();
                    }
                    KeyCode::Char('?') => {
                        app.toggle_footer();
                    }
                    _ => {}
                }
                CurrentPane::Archive => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(false);
                    }
                    KeyCode::Tab => {
                        app.toggle_pane();
                    }
                    KeyCode::Char('c') => {
                        app.clear_archive();
                        app.refresh()?;
                    }
                    KeyCode::Char('s') => {
                        app.save_archive()?;
                        app.refresh()?;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.update_archive_idx(1);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.update_archive_idx(-1);
                    }
                    KeyCode::Char('e') => {
                        app.extract_archive()?;
                        app.refresh()?;
                    }
                    KeyCode::Char('a') => {
                        app.remove_from_archive()?;
                        app.refresh()?;
                    }
                    KeyCode::Char('?') => {
                        app.toggle_footer();
                    }
                    _ => {}
                }
            }
        }
    }
}
