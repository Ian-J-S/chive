use std::{env, io};
use std::path::{Path, PathBuf};
use std::fs::read_dir;
use path_clean::PathClean;

pub enum CurrentPane {
    Local,
    Remote,
}

pub struct App {
    pub local_path: PathBuf,
    pub local_files: Vec<PathBuf>,
    pub local_idx: usize,
    pub current_pane: CurrentPane,
}

impl App {
    pub fn new() -> Self {
        App {
            local_path: std::env::current_dir().unwrap_or(PathBuf::from(".")),
            local_files: Vec::new(),
            local_idx: 0,
            current_pane: CurrentPane::Local,
            show_hidden: false,
        }
    }

    pub fn get_local_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut entries = read_dir(self.local_path.clone())?
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

    pub fn update_local_idx(&mut self, step: isize) {
        let new_idx = if step >= 0 {
            self.local_idx.wrapping_add(step as usize) % self.local_files.len()
        } else {
            ((self.local_idx as isize + step)
                .rem_euclid(self.local_files.len() as isize)) as usize
        };

        self.local_idx = new_idx;
    }

    pub fn change_local_dir(&mut self) -> io::Result<()> {
        let selected_path = self.local_files[self.local_idx].clone();
        let new_path = if selected_path.to_str()
            .expect("Unable to convert path to string") == ".." {

            self.local_path.parent()
                .expect("No parent dir")
                .to_path_buf()
        } else {
            env::current_dir()?.join(selected_path).clean()
        };

        if !new_path.is_dir() {
            return Ok(());
        }

        self.local_path = new_path;
        self.local_files = self.get_local_files()?;
        self.local_idx = 0;

        Ok(())
    }

    pub fn toggle_hidden_files(&mut self) -> io::Result<()> {
        self.show_hidden = !self.show_hidden;
        self.local_files = self.get_local_files()?;
        Ok(())
    }
}
