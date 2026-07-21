use crate::archive::traits::AppArchive;
use anyhow::{anyhow, Result};
use sevenz_rust2::{
    decompress_file,
    encoder_options::Lzma2Options,
    ArchiveEntry, ArchiveReader, ArchiveWriter, EncoderConfiguration, Password,
};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use walkdir::WalkDir;

pub struct SevenZ {
    writer: Option<ArchiveWriter<NamedTempFile>>,
    config: EncoderConfiguration,
}

impl SevenZ {
    pub fn new(compression_strength: u32) -> Result<Self> {
        let file = NamedTempFile::new_in(".")?;
        let mut writer = ArchiveWriter::new(file)?;

        let config: EncoderConfiguration =
            Lzma2Options::from_level(compression_strength).into();

        writer.set_content_methods(vec![
            config.clone(),
        ]);

        Ok(Self {
            writer: Some(writer),
            config,
        })
    }

    pub fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)> {
        todo!()
    }
}

impl AppArchive for SevenZ {
    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?;

        for item in WalkDir::new(full_path) {
            let item = item?;
            let item_full_path = item.path();

            // Skip directories. Their paths will be implied by contained files
            if !item.file_type().is_file() {
                continue;
            }

            let relative_path = if full_path.is_dir() {
                item_full_path.strip_prefix(full_path)?
            } else {
                Path::new("")
            };

            let item_archive_path = if full_path.is_dir() {
                archive_path.join(relative_path)
            } else {
                archive_path.to_path_buf()
            };

            let entry_name = item_archive_path.to_string_lossy().to_string();

            let entry = ArchiveEntry::from_path(item_full_path, entry_name);

            writer.push_archive_entry(entry, Some(File::open(item_full_path)?))?;
        }

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
        format!("Saved {name}.7z!")
    }

    fn extract_archive_file(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        decompress_file(archive_path, destination)?;
        Ok(())
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        todo!()
    }

    fn remove_from_archive(&mut self, file_to_remove: &Path) -> Result<()> {
        let inner_file = self
            .writer
            .take()
            .ok_or_else(|| anyhow!("Current archive does not exist"))?
            .finish()?;

        let password = Password::empty();
        let mut reader = ArchiveReader::new(inner_file, password)?;

        let mut new_writer = ArchiveWriter::new(NamedTempFile::new_in(".")?)?;
        new_writer.set_content_methods(vec![ // Keep same level of compression
            self.config.clone()
        ]);

        let file_to_remove = file_to_remove.to_string_lossy().replace('\\', "/");

        reader.for_each_entries(|entry, entry_reader| {
            let entry_name = entry.name().replace('\\', "/");

            let should_remove = entry_name == file_to_remove
                || entry_name.starts_with(&format!("{file_to_remove}/"));

            if should_remove {
                // The reader still needs to advance through this entry,
                // particularly when reading a solid archive.
                std::io::copy(entry_reader, &mut std::io::sink())?;
                return Ok(true);
            }

            if entry.is_directory {
                let directory_entry = sevenz_rust2::ArchiveEntry::new_directory(entry.name());

                new_writer.push_archive_entry(directory_entry, None::<std::io::Empty>)?;
            } else {
                let mut data = Vec::new();
                entry_reader.read_to_end(&mut data)?;

                let new_entry = entry.clone();

                new_writer.push_archive_entry(new_entry, Some(data.as_slice()))?;
            }

            Ok(true)
        })?;

        self.writer = Some(new_writer);

        Ok(())
    }
}
