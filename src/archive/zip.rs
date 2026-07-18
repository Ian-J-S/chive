use crate::archive::traits::AppArchive;
use anyhow::Result;
use std::{fs::File, path::{Path, PathBuf}};
use zip::ZipWriter;

struct ZipArchive {
    writer: Option<ZipWriter<File>>
}

impl AppArchive for ZipArchive {
    fn new(compression_strength: u32) -> Result<Self> {
        Ok(ZipArchive { writer: None, })
    }

    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        Ok(())
    }

    fn save_archive(self, save_filename: &str) -> Result<()> {
        Ok(())
    }

    fn save_msg(&self, name: &str) -> String {
        format!("Saved {name}.zip!")
    }

    fn extract_archive_file(archive_path: &Path, destination: &Path) -> Result<()> {
        Ok(())
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_from_archive(&mut self, file_to_remove: &Path) -> Result<()> {
        Ok(())
    }

    fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)>  {
        Ok((
            ZipArchive { writer: None },
            vec![],
        ))
    }
}
