use anyhow::Result;
use std::path::{Path, PathBuf};
use tar::Builder;
use tempfile::NamedTempFile;
use xz2::write::XzEncoder;

use crate::archive::traits::AppArchive;

struct TarXz {
    builder: Option<Builder<XzEncoder<NamedTempFile>>>,
}

impl TarXz {
    pub fn new(compression_strength: u32) -> Result<Self> {
        todo!()
    }

    pub fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)> {
        todo!()
    }
}

impl AppArchive for TarXz {
    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        todo!()
    }

    fn save_archive(&mut self, save_filename: &str) -> Result<()> {
        todo!()
    }

    fn save_msg(&self, name: &str) -> String {
        todo!()
    }

    fn extract_archive_file(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        todo!()
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        todo!()
    }

    fn remove_from_archive(&mut self, file_to_remove: &Path) -> Result<()> {
        todo!()
    }
}
