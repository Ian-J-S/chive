use std::{env, fs::read_dir, io, path::PathBuf};

use path_clean::PathClean;
use ratatui::widgets::{ListState, ScrollbarState};

pub struct BrowserState {
    pub current_path: PathBuf,
    pub files: Vec<PathBuf>,
    pub idx: usize,
    pub list_state: ListState,
    pub scrollbar_state: ScrollbarState,
    pub show_hidden: bool,
}

impl BrowserState {
    pub fn new() -> Self {
        BrowserState {
            current_path: std::env::current_dir().unwrap_or(PathBuf::from(".")),
            files: Vec::new(),
            idx: 0,
            list_state: ListState::default(),
            scrollbar_state: ScrollbarState::new(0).position(0),
            show_hidden: false,
        }
    }

    /// Add files to list displayed in left pane.
    pub fn get_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut entries = read_dir(self.current_path.clone())?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        if !self.show_hidden {
            entries.retain(|p| {
                !p.file_name()
                    .expect("No file name")
                    .to_string_lossy()
                    .starts_with(".")
            });
        }
        entries.sort();
        entries.insert(0, PathBuf::from(".."));

        Ok(entries)
    }

    /// Get current file under the cursor in the browser pane.
    pub fn get_selected_browser(&self) -> PathBuf {
        self.files[self.idx].clone()
    }

    /// Increase or decrease the selected index in the file browser.
    pub fn update_idx(&mut self, step: isize) {
        let new_idx = if step >= 0 {
            self.idx.wrapping_add(step as usize) % self.files.len()
        } else {
            ((self.idx as isize + step).rem_euclid(self.files.len() as isize)) as usize
        };

        self.idx = new_idx;
    }

    /// Try to change directory to the one under the cursor.
    /// If the selected file isn't a directory, do nothing.
    pub fn change_browser_dir(&mut self) -> io::Result<()> {
        let selected_path = self.files[self.idx].clone();
        let new_path = if selected_path
            .to_str()
            .expect("Unable to convert path to string")
            == ".."
        {
            self.current_path
                .parent()
                .expect("No parent dir")
                .to_path_buf()
        } else {
            env::current_dir()?.join(selected_path).clean()
        };

        if !new_path.is_dir() {
            return Ok(());
        }

        self.current_path = new_path;
        self.files = self.get_files()?;
        self.idx = 0;

        Ok(())
    }

    /// Show or hide hidden files.
    pub fn toggle_hidden_files(&mut self) -> io::Result<()> {
        self.show_hidden = !self.show_hidden;
        self.files = self.get_files()?;
        Ok(())
    }

    /// Refresh files in file browser.
    pub fn refresh(&mut self) -> io::Result<()> {
        self.files = self.get_files()?;

        // Prevent incorrect index when last file in directory is removed
        self.idx = self.idx.clamp(0, self.files.len() - 1);

        Ok(())
    }
}
