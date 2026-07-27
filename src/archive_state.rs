use std::{
    collections::HashSet,
    fmt::{self, Display},
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use path_clean::PathClean;

use crate::archive::{
    AppArchive, seven_z::SevenZ, tar_gz::TarGz, tar_xz::TarXz, tar_zst::TarZst, zip::Zip,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ArchiveType {
    Zip,
    SevenZ,
    TarGz,
    TarXz,
    TarZst,
}

impl ArchiveType {
    pub const ALL: [Self; 5] = [
        Self::Zip,
        Self::SevenZ,
        Self::TarGz,
        Self::TarXz,
        Self::TarZst,
    ];

    /// Determines the proper ArchiveType based on the
    /// extension of the path.
    pub fn from_path(path: &Path) -> Result<Self> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Archive path has no valid file name"))?;

        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Ok(Self::TarGz)
        } else if name.ends_with(".zip") {
            Ok(Self::Zip)
        } else if name.ends_with(".7z") {
            Ok(Self::SevenZ)
        } else if name.ends_with(".xz") {
            Ok(Self::TarXz)
        } else if name.ends_with(".zst") {
            Ok(Self::TarZst)
        } else {
            Err(anyhow!("Unsupported archive type: {name}"))
        }
    }

    /// Helper for archive type popup window.
    fn index(self) -> usize {
        Self::ALL.iter().position(|kind| *kind == self).unwrap_or(0)
    }

    /// Used for selecting next ArchiveType in the UI.
    pub fn next(self) -> Self {
        let next = (self.index() + 1) % Self::ALL.len();
        Self::ALL[next]
    }

    /// Used for selecting previous ArchiveType in the UI.
    pub fn previous(self) -> Self {
        let previous = self.index().checked_sub(1).unwrap_or(Self::ALL.len() - 1);

        Self::ALL[previous]
    }

    /// Returns the minimum possible compression level
    /// for the current ArchiveType.
    // For now, zip, 7z, tar.xz and tar.gz all support 0 compression
    pub fn min_compression(&self) -> u32 {
        0
    }

    /// Returns the maximum possible compression level
    /// for the current ArchiveType.
    // Some compression methods have a max above 9,
    // so I am leaving this function to make it easier to
    // add those later if I want to.
    pub fn max_compression(&self) -> u32 {
        match self {
            Self::TarZst => 19,
            _ => 9,
        }
    }
}

impl Display for ArchiveType {
    /// Formatter for getting the proper extension per ArchiveType.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let st = match self {
            Self::TarGz => ".tar.gz",
            Self::TarXz => ".tar.xz",
            Self::TarZst => ".tar.zst",
            Self::Zip => ".zip",
            Self::SevenZ => ".7z",
        };
        write!(f, "{st}")
    }
}

/// Given a path, checks the extension and loads the
/// returns a new archive of the proper type.
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

        ArchiveType::TarXz => {
            let (archive, names) = TarXz::load_from_file(path, compression_strength)?;

            Ok((Box::new(archive), names))
        }

        ArchiveType::TarZst => {
            let (archive, names) = TarZst::load_from_file(path, compression_strength)?;

            Ok((Box::new(archive), names))
        }

        ArchiveType::SevenZ => {
            let (archive, names) = SevenZ::load_from_file(path, compression_strength)?;

            Ok((Box::new(archive), names))
        }
    }
}

/// Creates a new blank archive.
pub fn create_archive(kind: ArchiveType, compression_strength: u32) -> Result<Box<dyn AppArchive>> {
    match kind {
        ArchiveType::Zip => Ok(Box::new(Zip::new(compression_strength)?)),
        ArchiveType::TarGz => Ok(Box::new(TarGz::new(compression_strength)?)),
        ArchiveType::SevenZ => Ok(Box::new(SevenZ::new(compression_strength)?)),
        ArchiveType::TarXz => Ok(Box::new(TarXz::new(compression_strength)?)),
        ArchiveType::TarZst => Ok(Box::new(TarZst::new(compression_strength)?)),
    }
}

/// Holds the state of the current archive for the
/// main app struct.
pub struct ArchiveState {
    pub archive_names: HashSet<PathBuf>, // Stores unique file names, not full paths
    pub current_archive: Option<Box<dyn AppArchive>>,
    pub archive_idx: usize,
    pub save_filename: String,
    pub compression_strength: u32,
    pub archive_type: ArchiveType,
}

impl ArchiveState {
    pub fn new() -> Self {
        ArchiveState {
            archive_names: HashSet::new(),
            current_archive: None,
            archive_idx: 0,
            save_filename: String::from("archive"),
            compression_strength: 6,        // Default compression level
            archive_type: ArchiveType::Zip, // Default to Zip
        }
    }

    /// Get current file under cursor in the archive pane.
    pub fn get_selected_archive(&self) -> Option<PathBuf> {
        self.archive_names.iter().nth(self.archive_idx).cloned()
    }

    /// Adds the given path to the current archive builder.
    pub fn add_file(&mut self, selected_path: &Path, current_path: &Path) -> Result<()> {
        let archive_path = selected_path
            .strip_prefix(current_path)?
            .to_path_buf()
            .clean();

        if self.current_archive.is_none() {
            self.current_archive = Some(create_archive(
                self.archive_type,
                self.compression_strength,
            )?);
        }

        if self.archive_names.contains(&archive_path) {
            self.remove_file(&archive_path)?;
            return Ok(());
        }

        if let Some(archive) = self.current_archive.as_mut() {
            archive.add_file_to_archive(selected_path, &archive_path)?;
        }

        self.archive_names.insert(archive_path);

        Ok(())
    }

    /// Save the currently built/loaded archive.
    pub fn save_archive(&mut self) -> Result<String> {
        let mut archive = self
            .current_archive
            .take()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?;

        archive.save_archive(&self.save_filename)?;

        let msg = archive.save_msg(&self.save_filename);

        Ok(msg)
    }

    /// Removes a file from the current archive builder.
    pub fn remove_file(&mut self, file_to_remove: &Path) -> Result<()> {
        if let Some(archive) = self.current_archive.as_mut() {
            archive.remove_from_archive(file_to_remove)?;
        }

        self.archive_names.remove(file_to_remove);
        self.archive_idx = self
            .archive_idx
            .clamp(0, self.archive_names.len().saturating_sub(1));

        Ok(())
    }

    /// Takes the archive at archive_path and extracts it
    /// in a subfolder in the current browser directory.
    pub fn extract_archive_file(&mut self, archive_path: &Path, destination: &Path) -> Result<()> {
        let (new_arch, paths) = load_archive_from_file(archive_path, self.compression_strength)?;
        self.current_archive = Some(new_arch);
        self.archive_names = paths.into_iter().collect();
        self.current_archive
            .as_mut()
            .ok_or_else(|| anyhow!("Error loading archive"))?
            .extract_archive_file(archive_path, destination)?;

        Ok(())
    }

    /// Loads the archive at archive_path into the archive pane
    /// with a new builder.
    pub fn load(&mut self, archive_path: &Path) -> Result<()> {
        let (archive, entry_names) =
            load_archive_from_file(archive_path, self.compression_strength)?;

        self.current_archive = Some(archive);
        self.archive_names = entry_names.into_iter().collect();
        self.archive_idx = 0;

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

    /// Remove the current archive
    /// and reset the names.
    pub fn clear(&mut self) {
        self.current_archive = None;
        self.archive_names = HashSet::new();
    }

    /// Append a character to the archive
    /// filename before saving it.
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
}
