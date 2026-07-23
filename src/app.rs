use anyhow::{Result, anyhow};
use path_clean::PathClean;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::archive_state::ArchiveState;
use crate::browser::BrowserState;
use crate::util::strip_all_extensions;

/// Represents the currently active app pane.
#[derive(PartialEq, Clone, Copy)]
pub enum CurrentPane {
    Browser,
    Archive,
}

/// Denotes normal operation (panes) or to
/// display one of the popups.
#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    SaveWindow,
    CompressionStrength,
    ArchiveType,
}

/// Informative messages that pop up after
/// certain actions are performed.
pub struct InfoMsg {
    pub msg: String,
    pub timeout: Instant,
}

/// Main app struct, holds state for both panes
/// as well as active pane, footer status, info
/// messages, and whether the app should quit.
pub struct App {
    pub browser_state: BrowserState,
    pub archive_state: ArchiveState,
    pub current_pane: CurrentPane,
    pub show_footer: bool,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub info_message: Option<InfoMsg>,
}

impl App {
    /// Create a new app with default state.
    pub fn new() -> Self {
        App {
            browser_state: BrowserState::new(),
            archive_state: ArchiveState::new(),
            current_pane: CurrentPane::Browser,
            show_footer: false,
            should_quit: false,
            input_mode: InputMode::ArchiveType,
            info_message: None,
        }
    }

    /// Toggle the current pane between browser and archive
    pub fn toggle_pane(&mut self) {
        self.current_pane = match self.current_pane {
            CurrentPane::Browser if !self.archive_state.archive_names.is_empty() => {
                CurrentPane::Archive
            }
            CurrentPane::Archive => CurrentPane::Browser,
            _ => CurrentPane::Browser,
        }
    }

    /// Helper function, depending on the current pane either
    /// extracts a selected archive file (in the browser) or
    /// extracts the build archive in the archive pane.
    pub fn extract_archive(&mut self) -> Result<()> {
        match self.current_pane {
            CurrentPane::Browser => {
                let archive_path = self.browser_state.get_selected_browser();
                let destination = self
                    .browser_state
                    .current_path
                    .join(strip_all_extensions(&archive_path));

                self.archive_state
                    .extract_archive_file(&archive_path, &destination)?;
            }
            CurrentPane::Archive => {
                if let Some(archive) = self.archive_state.current_archive.as_mut() {
                    let destination = self
                        .browser_state
                        .current_path
                        .join(PathBuf::from("extract"));
                    archive.extract_current(&destination)?;
                }
            }
        }

        Ok(())
    }

    /// Removes the selected file from the archive, in both
    /// the browser pane and the archive pane.
    pub fn remove_from_archive(&mut self) -> Result<()> {
        let file_to_remove = match self.current_pane {
            CurrentPane::Browser => self
                .browser_state
                .get_selected_browser()
                .strip_prefix(self.browser_state.current_path.clone())?
                .to_path_buf()
                .clean(),
            CurrentPane::Archive => self
                .archive_state
                .get_selected_archive()
                .ok_or_else(|| anyhow!("Unable to get current archive file"))?,
        };

        self.archive_state.remove_file(&file_to_remove)?;

        Ok(())
    }

    /// Loads archive file selected in the browser pane.
    pub fn load_archive(&mut self) -> Result<()> {
        let archive_path = self.browser_state.get_selected_browser();
        self.archive_state.load(&archive_path)?;
        self.current_pane = CurrentPane::Archive;
        self.set_info_msg("Loaded archive");
        Ok(())
    }

    /// Discard the current archive and reset.
    pub fn clear_archive(&mut self) {
        self.archive_state.clear();
        self.current_pane = CurrentPane::Browser;
        self.set_info_msg("Cleared current archive");
    }

    /// Show / hide footer.
    pub fn toggle_footer(&mut self) {
        self.show_footer = !self.show_footer;
    }

    /// Sets the info message to be displayed for 3 seconds.
    pub fn set_info_msg(&mut self, msg: &str) {
        self.info_message = Some(InfoMsg {
            msg: msg.to_string(),
            timeout: Instant::now() + Duration::from_secs(3),
        })
    }
}
