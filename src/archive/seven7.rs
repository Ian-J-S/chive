use crate::archive::traits::AppArchive;
use anyhow::{Result, anyhow};
use sevenz_rust::{SevenZArchiveEntry, SevenZWriter, lzma::LZMA2Options};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub struct SevenZ {
    writer: Option<SevenZWriter<NamedTempFile>>,
}

impl SevenZ {
    fn new(compression_strength: u32) -> Result<Self> {
        let file = NamedTempFile::new_in(".")?;
        let mut writer = SevenZWriter::new(file)?;
        writer.set_content_methods(vec![LZMA2Options::with_preset(compression_strength).into()]);
        Ok(SevenZ {
            writer: Some(writer),
        })
    }

    fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)> {
        Ok((Self { writer: None }, vec![]))
    }
}

impl AppArchive for SevenZ {

    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?;

        let entry_name = archive_path.to_string_lossy().to_string();

        let entry = SevenZArchiveEntry::from_path(full_path, entry_name);

        let source = File::open(full_path)?;

        writer.push_archive_entry(entry, Some(source))?;

        Ok(())
    }

    fn save_archive(&mut self, save_filename: &str) -> Result<()> {
        let inner_file = self
            .writer
            .take()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?
            .finish()?;

        inner_file.persist(format!("{save_filename}.7z"))?;

        Ok(())
    }

    fn save_msg(&self, name: &str) -> String {
        String::default()
    }

    fn extract_archive_file(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        Ok(())
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        todo!()
    }

    fn remove_from_archive(&mut self, file_to_remove: &Path) -> Result<()> {
        todo!()
    }
}
