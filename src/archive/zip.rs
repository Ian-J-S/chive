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

impl Zip {
    pub fn new(compression_strength: u32) -> Result<Self> {
        let file_options = match compression_strength {
            0 => SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            _ => SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(compression_strength.into()))
        };
        Ok(Zip {
            writer: Some(ZipWriter::new(NamedTempFile::new_in(".")?)),
            file_options
        })
    }

    pub fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)> {
        let source = File::open(path)?;
        let mut archive = ZipArchive::new(source)?;

        let mut entry_names = Vec::with_capacity(archive.len());

        for index in 0..archive.len() {
            let entry = archive.by_index(index)?;

            let entry_path = entry
                .enclosed_name()
                .ok_or_else(|| anyhow!("Unsafe ZIP entry path: {}", entry.name()))?;

            entry_names.push(entry_path);
        }

        let mut temp_file = NamedTempFile::new_in(".")?;
        let mut source = File::open(path)?;

        std::io::copy(&mut source, &mut temp_file)?;
        temp_file.rewind()?;

        let writer = ZipWriter::new_append(temp_file)?;

        let file_options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(compression_strength.into()));

        Ok((
            Zip {
                writer: Some(writer),
                file_options,
            },
            entry_names,
        ))
    }
}

impl AppArchive for Zip {
    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        if let Some(writer) = self.writer.as_mut() {
            let file_name = archive_path.to_string_lossy();
            writer.start_file(file_name, self.file_options)?;

            let mut source = File::open(full_path)?;

            std::io::copy(&mut source, writer)?;
        }

        Ok(())
    }

    fn save_archive(&mut self, save_filename: &str) -> Result<()> {
        let temp_file = self
            .writer
            .take()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?
            .finish()?;

        temp_file.persist(format!("{save_filename}.zip"))?;

        Ok(())
    }

    fn save_msg(&self, name: &str) -> String {
        format!("Saved {name}.zip!")
    }

    fn extract_archive_file(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        archive.extract(destination)?;

        Ok(())
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        let writer = self
            .writer
            .take()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?;

        let file = writer.finish()?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(destination)?;

        // Reconvert arhive back into usable writer
        let file = archive.into_inner();
        self.writer = Some(ZipWriter::new_append(file)?);

        Ok(())
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
}
