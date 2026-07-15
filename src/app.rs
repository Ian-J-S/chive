use anyhow::{anyhow, Result};
use flate2::Compression;
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use std::collections::HashSet;
use std::env;
use std::io::{self, Seek, SeekFrom};
use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};
use path_clean::PathClean;
use ratatui::widgets::{ScrollbarState, ListState};
use tar::{Archive, Builder};
use tempfile::tempfile;

use crate::util::strip_all_extensions;

#[derive(PartialEq, Clone, Copy)]
pub enum CurrentPane {
    Browser,
    Archive,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    SaveWindow,
    CompressionStrength,
}

pub struct App {
    pub browser_path: PathBuf,
    pub browser_files: Vec<PathBuf>,
    pub browser_idx: usize,
    pub browser_list_state: ListState,
    pub browser_scrollbar: ScrollbarState,
    pub current_pane: CurrentPane,
    pub show_hidden: bool,
    pub archive_names: HashSet<PathBuf>,  // Stores unique file names, not full paths
    pub current_archive: Option<Builder<GzEncoder<File>>>,
    pub archive_idx: usize,
    pub show_footer: bool,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub save_filename: String,
    pub compression_strength: u32,
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
            archive_names: HashSet::new(),
            current_archive: None,
            archive_idx: 0, 
            show_footer: false,
            should_quit: false,
            input_mode: InputMode::CompressionStrength,
            save_filename: String::from("archive"),
            compression_strength: 6, // Default gzip compression level
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

    /// Get current file under the cursor in the browser pane.
    fn get_selected_browser(&self) -> PathBuf {
        self.browser_files[self.browser_idx].clone()
    }

    /// Get current file under cursor in the archive pane.
    fn get_selected_archive(&self) -> Option<PathBuf> {
        self.archive_names
            .iter()
            .nth(self.archive_idx)
            .cloned()
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

    /// Try to change directory to the one under the cursor.
    /// If the selected file isn't a directory, do nothing.
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

    /// Refresh files in file browser.
    pub fn refresh(&mut self) -> io::Result<()> {
        self.browser_files = self.get_browser_files()?;

        // Prevent incorrect index when last file in directory is removed
        self.browser_idx = self.browser_idx.clamp(0, self.browser_files.len() - 1);

        Ok(())
    }

    /// Toggle the current pane between browser and archive
    pub fn toggle_pane(&mut self) {
        self.current_pane = match self.current_pane {
            CurrentPane::Browser if !self.archive_names.is_empty() => CurrentPane::Archive,
            CurrentPane::Archive => CurrentPane::Browser,
            _ => CurrentPane::Browser,
        }
    }

    /// Create a compressed archive from the selected paths
    pub fn create_archive(&mut self) -> Result<()> {
        let file = tempfile()?; 
        let gz = GzEncoder::new(file, Compression::new(self.compression_strength));
        let ar = Builder::new(gz);
        self.current_archive = Some(ar);

        Ok(())
    }

    /// Helper function for adding a file to the current archive.
    fn append_to_archive(&mut self, full_path: &Path, path: &Path) -> Result<()> {
        self.archive_names.insert(path.to_path_buf());
        self.current_archive
            .as_mut()
            .expect("Current archive does not exist")
            .append_path_with_name(full_path, path)?;
        Ok(())
    }

    /// Append the current selected browser-pane file to the archive.
    pub fn add_file_to_archive(&mut self) -> Result<()> {
        let full_path = self.get_selected_browser();
        let path = full_path.strip_prefix(self.browser_path.clone())?;

        if self.current_archive.is_none() {
            self.create_archive()?;
        }

        if self.archive_names.contains(path) {
            self.remove_from_archive()?;
            return Ok(());
        }

        if path.is_dir() {
            let dir = path.read_dir()?;
            self.append_to_archive(&full_path, path)?;
            for f in dir {
                let f = f?;
                let p = f.path();
                self.archive_names.insert(p.to_path_buf());
                self.current_archive
                    .as_mut()
                    .expect("Current archive does not exist")
                    .append_path(p)?;
            }
        } else {
            self.append_to_archive(&full_path, path)?;
        }

        Ok(())
    }

    /// Save the currently built/loaded archive.
    pub fn save_archive(&mut self) -> Result<()> {
        let builder = self
            .current_archive
            .take()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?;

        let gz = builder.into_inner()?;

        let mut temp_file = gz.finish()?;

        temp_file.seek(SeekFrom::Start(0))?;

        let filename = PathBuf::from(format!("{}.tar.gz", self.save_filename));
        let mut out = File::create(filename)?;
        io::copy(&mut temp_file, &mut out)?;

        Ok(())
    }

    /// Increase or decrease the selected index in the archive pane.
    pub fn update_archive_idx(&mut self, step: isize) {
        let new_idx = if !self.archive_names.is_empty() {
            if step >= 0 {
                self.archive_idx.wrapping_add(step as usize) % self.archive_names.len()
            } else {
                ((self.archive_idx as isize + step)
                    .rem_euclid(self.archive_names.len() as isize)) as usize
            }
        } else {
            0
        };

        self.archive_idx = new_idx;
    }

    /// Extract an archive.
    /// When an archive is selected in the browser window, extract that.
    /// If the archive pane is active, extract the currently loaded archive.
    pub fn extract_archive(&mut self) -> Result<()> {
        match self.current_pane {
            CurrentPane::Browser => {
                // Build path to extract archive to
                let path = self.get_selected_browser();
                let file = File::open(&path)?;
                let file_stem = strip_all_extensions(&path);
                let unpack_path = self.browser_path.join(file_stem);

                let decoder = MultiGzDecoder::new(&file);
                let mut archive = Archive::new(decoder);
                archive.unpack(unpack_path)?;
            }
            // Current archive has been created / loaded,
            // so we can extract it.
            CurrentPane::Archive => {
                let b = self.current_archive.take().unwrap();
                self.archive_names = HashSet::new();

                let gz = b.into_inner()?;
                let mut temp = gz.finish()?;
                temp.seek(SeekFrom::Start(0))?;

                let decoder = MultiGzDecoder::new(temp);
                let mut archive = Archive::new(decoder);

                // TODO - get destination from a save input box
                let unpack_dst = self.browser_path.join("unpack/");
                archive.unpack(unpack_dst)?;
            }
        }

        Ok(())
    }

    /// Removes selected file from the archive
    pub fn remove_from_archive(&mut self) -> Result<()> {
        // Get selected file
        let file_to_remove = match self.current_pane {
            CurrentPane::Browser => {
                self.get_selected_browser()
                    .strip_prefix(self.browser_path.clone())?
                    .to_path_buf()
                    .clean()
            }
            CurrentPane::Archive => {
                self.get_selected_archive().ok_or_else(|| anyhow!("Unable to get current archive file"))?
            }
        };

        // Get current archive file
        let current_builder = self.current_archive.take()
            .ok_or_else(|| anyhow!("Current archive does not exist, cannot remove file"))?;

        // Get inner file from encoder
        let gz = current_builder.into_inner()?;
        let mut temp_file = gz.finish()?;
        // Rewind file position after .finish()
        temp_file.seek(SeekFrom::Start(0))?;

        let decoder = MultiGzDecoder::new(temp_file);
        let mut archive = Archive::new(decoder);

        let new_file = tempfile()?;
        let gz = GzEncoder::new(new_file, Compression::new(self.compression_strength));
        let mut new_builder = Builder::new(gz);

        for entry_res in archive.entries()? {
            let entry = entry_res?;
            let entry_path = entry.path()?;

            // Skip entries that match the file to remove
            if entry_path.as_ref().starts_with(&file_to_remove) {
                continue;
            }

            let header = entry.header().clone();
            new_builder.append(&header, entry)?;
        }

        self.archive_names.retain(|n| n != &file_to_remove);
        self.current_archive = Some(new_builder);

        Ok(())
    }

    /// Load an existing archive for editing
    pub fn load_archive(&mut self) -> Result<()> {
        // Get selected file
        let archive_name = self.get_selected_browser()
            .strip_prefix(self.browser_path.clone())?
            .to_path_buf()
            .clean();

        // Get decoder and archive object
        let f = File::open(&archive_name)?;
        let decoder = flate2::read::GzDecoder::new(f);
        let mut archive = Archive::new(decoder);

        // Create a tempfile and encoder for the new archive
        let new_file = tempfile()?;
        let gz = GzEncoder::new(new_file, Compression::new(self.compression_strength));
        let mut new_builder = Builder::new(gz);

        // Copy entries from archive into new builder
        self.archive_names = HashSet::new();
        for entry_res in archive.entries()? {
            let entry = entry_res?;
            let entry_path = entry.path()?;
            self.archive_names.insert(entry_path.into());
            let header = entry.header().clone();
            new_builder.append(&header, entry)?;
        }

        self.current_archive = Some(new_builder);

        Ok(())
    }

    /// Discard the current archive and reset.
    pub fn clear_archive(&mut self) {
        self.current_archive = None;
        self.archive_names = HashSet::new();
        self.current_pane = CurrentPane::Browser;
    }

    pub fn toggle_footer(&mut self) {
        self.show_footer = !self.show_footer;
    }

    pub fn enter_save_char(&mut self, to_insert: char) {
        self.save_filename.push(to_insert);
    }

    pub fn increase_comp_strength(&mut self) {
        self.compression_strength = (self.compression_strength + 1).min(9);
    }

    pub fn decrease_comp_strength(&mut self) {
        self.compression_strength = self.compression_strength.saturating_sub(1);
    }
}
