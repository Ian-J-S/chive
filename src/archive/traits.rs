use anyhow::Result;
use std::path::{Path, PathBuf};

pub trait AppArchive {
    type Output;

    fn new(compression_strength: u32) -> Result<Self>
    where
        Self: Sized;

    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()>;

    fn save_archive(self, save_filename: &str) -> Result<()>;

    fn save_msg(&self, name: &str) -> String;

    fn extract_archive_file(archive_path: &Path, destination: &Path) -> Result<()>;

    fn extract_current(&mut self, destination: &Path) -> Result<()>;

    fn remove_from_archive(&mut self, file_to_remove: &Path) -> Result<()>;

    fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)>
    where
        Self: Sized;
}
