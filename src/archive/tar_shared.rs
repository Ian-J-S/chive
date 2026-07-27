use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use tar::{Archive, Builder};
use tempfile::NamedTempFile;

use crate::archive::AppArchive;

pub trait TarEncoder: io::Write {
    fn finish(self) -> Result<NamedTempFile>;
}

pub trait TarCompression {
    type Encoder: TarEncoder;
    type Decoder<R: Read>: Read;

    const EXTENSION: &'static str;

    fn encoder(output: NamedTempFile, compression_strength: u32) -> Result<Self::Encoder>;

    fn decoder<R>(reader: R) -> Result<Self::Decoder<R>>
    where
        R: Read + 'static;
}

pub struct Tar<C: TarCompression> {
    builder: Option<Builder<C::Encoder>>,
    compression_strength: u32,
}

impl<C: TarCompression> Tar<C> {
    pub fn new(compression_strength: u32) -> Result<Self> {
        let file = NamedTempFile::new()?;
        let encoder = C::encoder(file, compression_strength)?;

        Ok(Self {
            builder: Some(Builder::new(encoder)),
            compression_strength,
        })
    }

    pub fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)> {
        let file = File::open(path)?;
        let decoder = C::decoder(file)?;
        let mut archive = Archive::new(decoder);

        let new_file = NamedTempFile::new()?;
        let encoder = C::encoder(new_file, compression_strength)?;
        let mut new_builder = Builder::new(encoder);

        let mut entry_names = Vec::new();

        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let entry_path = entry.path()?.into_owned();

            entry_names.push(entry_path);

            let header = entry.header().clone();
            new_builder.append(&header, &mut entry)?;
        }

        Ok((
            Self {
                builder: Some(new_builder),
                compression_strength,
            },
            entry_names,
        ))
    }

    fn finish_builder(&mut self) -> Result<NamedTempFile> {
        let builder = self
            .builder
            .take()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?;

        let encoder = builder.into_inner()?;
        encoder.finish()
    }

    fn create_builder(&self) -> Result<Builder<C::Encoder>> {
        let file = NamedTempFile::new()?;
        let encoder = C::encoder(file, self.compression_strength)?;

        Ok(Builder::new(encoder))
    }
}

impl<C: TarCompression> AppArchive for Tar<C> {
    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        let builder = self
            .builder
            .as_mut()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?;

        if full_path.is_dir() {
            builder.append_dir_all(archive_path, full_path)?;
        } else {
            builder.append_path_with_name(full_path, archive_path)?;
        }

        Ok(())
    }

    fn save_archive(&mut self, save_filename: &str) -> Result<()> {
        let mut temp_file = self.finish_builder()?;
        temp_file.seek(SeekFrom::Start(0))?;

        let filename = format!("{save_filename}{}", C::EXTENSION);
        let mut output = File::create(filename)?;

        io::copy(&mut temp_file, &mut output)?;

        Ok(())
    }

    fn save_msg(&self, name: &str) -> String {
        format!("Saved to {name}{}", C::EXTENSION)
    }

    fn extract_archive_file(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        let file = File::open(archive_path)?;
        let decoder = C::decoder(file)?;
        let mut archive = Archive::new(decoder);

        archive.unpack(destination)?;

        Ok(())
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        let mut temp_file = self.finish_builder()?;
        temp_file.seek(SeekFrom::Start(0))?;

        let decoder = C::decoder(temp_file)?;
        let mut archive = Archive::new(decoder);

        archive.unpack(destination)?;

        Ok(())
    }

    fn remove_from_archive(&mut self, entry_path: &Path) -> Result<()> {
        let mut old_file = self.finish_builder()?;
        old_file.seek(SeekFrom::Start(0))?;

        let decoder = C::decoder(old_file)?;
        let mut old_archive = Archive::new(decoder);
        let mut new_builder = self.create_builder()?;

        for entry_result in old_archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?;

            if path == entry_path || path.starts_with(entry_path) {
                continue;
            }

            let header = entry.header().clone();
            new_builder.append(&header, &mut entry)?;
        }

        self.builder = Some(new_builder);

        Ok(())
    }
}
