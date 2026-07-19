use anyhow::Result;
use std::path::Path;

pub trait AppArchive {

    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()>;

    fn save_archive(&mut self, save_filename: &str) -> Result<()>;

    fn save_msg(&self, name: &str) -> String;

    fn extract_archive_file(&self, archive_path: &Path, destination: &Path) -> Result<()>;

    fn extract_current(&mut self, destination: &Path) -> Result<()>;

    fn remove_from_archive(&mut self, file_to_remove: &Path) -> Result<()>;

}
