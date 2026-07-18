use crate::archive::traits::AppArchive;
use anyhow::{Result, anyhow};
use std::{
    fs::File,
    io::Seek,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

pub struct Zip {
    writer: Option<ZipWriter<NamedTempFile>>,
    file_options: SimpleFileOptions,
}

impl AppArchive for Zip {
    fn new(compression_strength: u32) -> Result<Self> {
        Ok(Zip {
            writer: Some(ZipWriter::new(NamedTempFile::new_in(".")?)),
            file_options: SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(compression_strength.into())), // TODO
        })
    }

    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        if let Some(writer) = self.writer.as_mut() {
            let file_name = archive_path.to_string_lossy();
            writer.start_file(file_name, self.file_options)?;

            let mut source = File::open(full_path)?;

            std::io::copy(&mut source, writer)?;
        }

        Ok(())
    }

    fn save_archive(self, save_filename: &str) -> Result<()> {
        let temp_file = self
            .writer
            .ok_or_else(|| anyhow!("Current archive does not exist"))?
            .finish()?;

        temp_file.persist(format!("{save_filename}.zip"))?;

        Ok(())
    }

    fn save_msg(&self, name: &str) -> String {
        format!("Saved {name}.zip!")
    }

    fn extract_archive_file(archive_path: &Path, destination: &Path) -> Result<()> {
        todo!();
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        todo!();
    }

    fn remove_from_archive(&mut self, file_to_remove: &Path) -> Result<()> {
        let mut inner_file = self
            .writer
            .take()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?
            .finish()?;

        inner_file.rewind()?;

        let mut archive = ZipArchive::new(inner_file)?;

        let new_file = NamedTempFile::new_in(".")?;
        let mut new_writer = ZipWriter::new(new_file);

        // Loop over existing files, copy all except file_to_remove
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            let name = file.name().to_string();

            if name == file_to_remove.to_string_lossy() {
                continue;
            }

            new_writer.start_file(name, SimpleFileOptions::default())?;
            std::io::copy(&mut file, &mut new_writer)?;
        }

        self.writer = Some(new_writer);

        Ok(())
    }

    fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)> {
        todo!();
    }
}
