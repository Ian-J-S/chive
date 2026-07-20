use anyhow::{Result, anyhow};
use core::fmt;
use path_clean::PathClean;
use ratatui::widgets::{ListState, ScrollbarState};
use std::collections::HashSet;
use std::env;
use std::fmt::Display;
use std::fs::read_dir;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::archive::zip::Zip;
use crate::archive::{tar_gz::TarGz, traits::AppArchive};

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
    ArchiveType,
}

pub struct InfoMsg {
    pub msg: String,
    pub timeout: Instant,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ArchiveType {
    Zip,
    TarGz,
}

impl ArchiveType {
    pub const ALL: [Self; 2] = [Self::Zip, Self::TarGz];

    pub fn from_path(path: &Path) -> Result<Self> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Archive path has no valid file name"))?;

        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Ok(Self::TarGz)
        } else if name.ends_with(".zip") {
            Ok(Self::Zip)
        } else {
            Err(anyhow!("Unsupported archive type: {name}"))
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|kind| *kind == self).unwrap_or(0)
    }

    pub fn next(self) -> Self {
        let next = (self.index() + 1) % Self::ALL.len();
        Self::ALL[next]
    }

    pub fn previous(self) -> Self {
        let previous = self.index().checked_sub(1).unwrap_or(Self::ALL.len() - 1);

        Self::ALL[previous]
    }

    // For now, zip and tar_gz both support 0 compression
    pub fn min_compression(&self) -> u32 {
        0
    }

    // Some common compression methods have a max above 9,
    // so I am leaving this function to make it easier to
    // add those later if I want to.
    pub fn max_compression(&self) -> u32 {
        9
    }
}

impl Display for ArchiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let st = match self {
            Self::TarGz => ".tar.gz",
            Self::Zip => ".zip",
        };
        write!(f, "{st}")
    }
}

pub fn load_archive_from_file(
    path: &Path,
    compression_strength: u32,
) -> Result<(Box<dyn AppArchive>, Vec<PathBuf>)> {
    match ArchiveType::from_path(path)? {
        ArchiveType::Zip => {
            let (archive, names) = Zip::load_from_file(path, compression_strength)?;

            Ok((Box::new(archive), names))
        }

        ArchiveType::TarGz => {
            let (archive, names) = TarGz::load_from_file(path, compression_strength)?;

            Ok((Box::new(archive), names))
        }
    }
}

pub struct App {
    pub browser_path: PathBuf,
    pub browser_files: Vec<PathBuf>,
    pub browser_idx: usize,
    pub browser_list_state: ListState,
    pub browser_scrollbar: ScrollbarState,
    pub current_pane: CurrentPane,
    pub show_hidden: bool,
    pub archive_names: HashSet<PathBuf>, // Stores unique file names, not full paths
    pub current_archive: Option<Box<dyn AppArchive>>,
    pub archive_idx: usize,
    pub show_footer: bool,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub save_filename: String,
    pub compression_strength: u32,
    pub info_message: Option<InfoMsg>,
    pub archive_type: ArchiveType,
}

pub fn create_archive(kind: ArchiveType, compression_strength: u32) -> Result<Box<dyn AppArchive>> {
    match kind {
        ArchiveType::Zip => Ok(Box::new(Zip::new(compression_strength)?)),
        ArchiveType::TarGz => Ok(Box::new(TarGz::new(compression_strength)?)),
    }
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
            input_mode: InputMode::ArchiveType,
            save_filename: String::from("archive"),
            compression_strength: 6, // Default compression level
            info_message: None,
            archive_type: ArchiveType::Zip, // Default to Zip
        }
    }

    /// Add files to list displayed in left pane.
    pub fn get_browser_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut entries = read_dir(self.browser_path.clone())?
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
    fn get_selected_browser(&self) -> PathBuf {
        self.browser_files[self.browser_idx].clone()
    }

    /// Get current file under cursor in the archive pane.
    fn get_selected_archive(&self) -> Option<PathBuf> {
        self.archive_names.iter().nth(self.archive_idx).cloned()
    }

    /// Increase or decrease the selected index in the file browser.
    pub fn update_browser_idx(&mut self, step: isize) {
        let new_idx = if step >= 0 {
            self.browser_idx.wrapping_add(step as usize) % self.browser_files.len()
        } else {
            ((self.browser_idx as isize + step).rem_euclid(self.browser_files.len() as isize))
                as usize
        };

        self.browser_idx = new_idx;
    }

    /// Try to change directory to the one under the cursor.
    /// If the selected file isn't a directory, do nothing.
    pub fn change_browser_dir(&mut self) -> io::Result<()> {
        let selected_path = self.browser_files[self.browser_idx].clone();
        let new_path = if selected_path
            .to_str()
            .expect("Unable to convert path to string")
            == ".."
        {
            self.browser_path
                .parent()
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

    pub fn add_file_to_archive(&mut self) -> Result<()> {
        let full_path = self.get_selected_browser();
        let archive_path = full_path
            .strip_prefix(self.browser_path.clone())?
            .to_path_buf()
            .clean();

        if self.current_archive.is_none() {
            self.current_archive = Some(create_archive(
                self.archive_type,
                self.compression_strength,
            )?);
        }

        if self.archive_names.contains(&archive_path) {
            self.remove_from_archive()?;
            return Ok(());
        }

        self.set_info_msg(&format!("adding {full_path:?}"));
        if let Some(archive) = self.current_archive.as_mut() {
            archive.add_file_to_archive(&full_path, &archive_path)?;
        }

        self.archive_names.insert(archive_path);

        Ok(())
    }

    /// Save the currently built/loaded archive.
    pub fn save_archive(&mut self) -> Result<()> {
        let mut archive = self
            .current_archive
            .take()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?;
        let msg = archive.save_msg(&self.save_filename);

        archive.save_archive(&self.save_filename)?;

        self.set_info_msg(&msg);

        Ok(())
    }

    pub fn extract_archive(&mut self) -> Result<()> {
        match self.current_pane {
            CurrentPane::Browser => {
                let archive_path = self.get_selected_browser();
                let destination = self.browser_path.join(strip_all_extensions(&archive_path));
                self.current_archive
                    .as_mut()
                    .unwrap()
                    .extract_archive_file(&archive_path, &destination)?
            }
            CurrentPane::Archive => {
                if let Some(archive) = self.current_archive.as_mut() {
                    let destination = self.browser_path.join(PathBuf::from("extract"));
                    archive.extract_current(&destination)?;
                }
            }
        }

        Ok(())
    }

    pub fn remove_from_archive(&mut self) -> Result<()> {
        let file_to_remove = match self.current_pane {
            CurrentPane::Browser => self
                .get_selected_browser()
                .strip_prefix(self.browser_path.clone())?
                .to_path_buf()
                .clean(),
            CurrentPane::Archive => self
                .get_selected_archive()
                .ok_or_else(|| anyhow!("Unable to get current archive file"))?,
        };
        if let Some(archive) = self.current_archive.as_mut() {
            archive.remove_from_archive(&file_to_remove)?;
        }
        self.set_info_msg(&format!(
            "file to remove: {}",
            file_to_remove.to_string_lossy()
        ));
        self.archive_names.remove(&file_to_remove);
        self.archive_idx = self
            .archive_idx
            .clamp(0, self.archive_names.len().saturating_sub(1));

        Ok(())
    }

    pub fn load_archive(&mut self) -> Result<()> {
        let archive_path = self.get_selected_browser();
        let (archive, entry_names) =
            load_archive_from_file(&archive_path, self.compression_strength)?;

        self.current_archive = Some(archive);
        self.archive_names = entry_names.into_iter().collect();
        self.current_pane = CurrentPane::Archive;
        self.archive_idx = 0;

        self.set_info_msg("Loaded archive");
        Ok(())
    }

    /// Increase or decrease the selected index in the archive pane.
    pub fn update_archive_idx(&mut self, step: isize) {
        let new_idx = if !self.archive_names.is_empty() {
            if step >= 0 {
                self.archive_idx.wrapping_add(step as usize) % self.archive_names.len()
            } else {
                ((self.archive_idx as isize + step).rem_euclid(self.archive_names.len() as isize))
                    as usize
            }
        } else {
            0
        };

        self.archive_idx = new_idx;
    }

    /// Discard the current archive and reset.
    pub fn clear_archive(&mut self) {
        self.current_archive = None;
        self.archive_names = HashSet::new();
        self.current_pane = CurrentPane::Browser;
        self.set_info_msg("Cleared current archive");
    }

    pub fn toggle_footer(&mut self) {
        self.show_footer = !self.show_footer;
    }

    pub fn enter_save_char(&mut self, to_insert: char) {
        self.save_filename.push(to_insert);
    }

    pub fn increase_comp_strength(&mut self) {
        self.compression_strength =
            (self.compression_strength + 1).min(self.archive_type.max_compression());
    }

    pub fn decrease_comp_strength(&mut self) {
        self.compression_strength = self
            .compression_strength
            .saturating_sub(1)
            .max(self.archive_type.min_compression())
    }

    pub fn confirm_compression(&mut self) {
        self.input_mode = InputMode::Normal;
        self.set_info_msg(&format!(
            "Compression strength set to {}",
            self.compression_strength
        ));
    }

    pub fn set_info_msg(&mut self, msg: &str) {
        self.info_message = Some(InfoMsg {
            msg: msg.to_string(),
            timeout: Instant::now() + Duration::from_secs(3),
        })
    }

    pub fn select_next_archive_type(&mut self) {
        self.archive_type = self.archive_type.next();
    }

    pub fn select_previous_archive_type(&mut self) {
        self.archive_type = self.archive_type.previous();
    }

    pub fn confirm_archive_type(&mut self) {
        self.input_mode = InputMode::CompressionStrength;
        self.set_info_msg(&format!("Chose {}", self.archive_type));
    }
}
