use std::{error::Error, io};

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
use crate::{
    app::{App, CurrentPane},
    ui::ui,
};

fn main() -> Result<(), Box<dyn Error>> {
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    app.browser_files = app.get_browser_files().unwrap(); // TODO - should replace with some app.init function
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            // TODO - remove this once you get the other pane working
            #[allow(clippy::single_match)]
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
                        app.toggle_selected_path();
                    }
                    _ => {}
                }
                _ => {}
            }
        }
    }
}
