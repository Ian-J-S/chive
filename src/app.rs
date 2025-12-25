use std::collections::HashSet;
use std::{env, io};
use std::path::PathBuf;
use std::fs::read_dir;
use path_clean::PathClean;
use ratatui::widgets::{ScrollbarState, ListState};

pub enum CurrentPane {
    Browser,
    Archive,
}

pub struct App {
    pub browser_path: PathBuf,
    pub browser_files: Vec<PathBuf>,
    pub browser_idx: usize,
    pub browser_list_state: ListState,
    pub browser_scrollbar: ScrollbarState,
    pub current_pane: CurrentPane,
    pub show_hidden: bool,
    pub selected_files: HashSet<PathBuf>,
}

impl App {
    pub fn new() -> Self {
        App {
            browser_path: std::env::current_dir().unwrap_or(PathBuf::from(".")),
            browser_files: Vec::new(),
            browser_idx: 0,
            browser_list_state: ListState::default(),
            browser_scrollbar: ScrollbarState::new(0).position(0),
            current_pane: CurrentPane::Browser,
            show_hidden: false,
            selected_files: HashSet::new(),
        }
    }

    /// Add files to list displayed in left pane.
    pub fn get_browser_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut entries = read_dir(self.browser_path.clone())?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
     
        if !self.show_hidden {
            entries.retain(|p| {
                !p.file_name().expect("No file name")
                .to_string_lossy().starts_with(".")
            });
        }
        entries.sort();
        entries.insert(0, PathBuf::from(".."));

        Ok(entries)
    }

    /// Get current file under the cursor.
    fn get_selected(&self) -> PathBuf {
        self.browser_files[self.browser_idx].clone()
    }

    /// Increase or decrease the selected index in the file browser.
    pub fn update_browser_idx(&mut self, step: isize) {
        let new_idx = if step >= 0 {
            self.browser_idx.wrapping_add(step as usize) % self.browser_files.len()
        } else {
            ((self.browser_idx as isize + step)
                .rem_euclid(self.browser_files.len() as isize)) as usize
        };

        self.browser_idx = new_idx;
    }

    pub fn change_browser_dir(&mut self) -> io::Result<()> {
        let selected_path = self.browser_files[self.browser_idx].clone();
        let new_path = if selected_path.to_str()
            .expect("Unable to convert path to string") == ".." {

            self.browser_path.parent()
                .expect("No parent dir")
                .to_path_buf()
        } else {
            env::current_dir()?.join(selected_path).clean()
        };

        if !new_path.is_dir() {
            return Ok(());
        }

        self.browser_path = new_path;
        self.browser_files = self.get_browser_files()?;
        self.browser_idx = 0;

        Ok(())
    }

    /// Show or hide hidden files.
    pub fn toggle_hidden_files(&mut self) -> io::Result<()> {
        self.show_hidden = !self.show_hidden;
        self.browser_files = self.get_browser_files()?;
        Ok(())
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        self.browser_files = self.get_browser_files()?;
        Ok(())
    }

    pub fn toggle_selected_path(&mut self) {
        let selected = self.get_selected();

        // Don't allow selection of parent directory.
        if let Some(name) = selected.to_str() && name == ".." {
            return;
        }

        if self.selected_files.contains(&selected) {
            self.selected_files.remove(&selected);
        } else {
            self.selected_files.insert(selected);
        }
    }
}
