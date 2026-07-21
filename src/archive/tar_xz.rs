use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tempfile::NamedTempFile;
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

use crate::archive::traits::AppArchive;

pub struct TarXz {
    builder: Option<Builder<XzEncoder<NamedTempFile>>>,
    compression_strength: u32,
}

impl TarXz {
    pub fn new(compression_strength: u32) -> Result<Self> {
        let file = NamedTempFile::new_in(".")?;
        let xz = XzEncoder::new(file, compression_strength);
        let builder = Some(Builder::new(xz));

        Ok(TarXz {
            builder,
            compression_strength,
        })
    }

    pub fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)> {
        todo!()
    }
}

impl AppArchive for TarXz {
    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        self.builder
            .as_mut()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?
            .append_path_with_name(full_path, archive_path)?;
        Ok(())
    }

    fn save_archive(&mut self, save_filename: &str) -> Result<()> {
        let builder = self
            .builder
            .take()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?;
        let xz = builder.into_inner()?;

        let mut temp_file = xz.finish()?;

        temp_file.seek(SeekFrom::Start(0))?;

        let filename = PathBuf::from(format!("{}.tar.xz", save_filename));
        let mut out = File::create(filename)?;
        io::copy(&mut temp_file, &mut out)?;

        Ok(())
    }

    fn save_msg(&self, name: &str) -> String {
        format!("Saved to {}.tar.xz", name)
    }

    fn extract_archive_file(&self, archive_path: &Path, destination: &Path) -> Result<()> {
        let archive_file = File::open(archive_path)?;
        let decoder = XzDecoder::new_multi_decoder(archive_file);
        let mut archive = Archive::new(decoder);
        archive.unpack(destination)?;

        Ok(())
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        let builder = self
            .builder
            .take()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?;
        let xz = builder.into_inner()?;
        let mut temp_file = xz.finish()?;
        temp_file.seek(SeekFrom::Start(0))?;

        let decoder = XzDecoder::new_multi_decoder(temp_file);
        let mut archive = Archive::new(decoder);
        archive.unpack(destination)?;

        Ok(())
    }

    fn remove_from_archive(&mut self, entry_path: &Path) -> Result<()> {
        let builder = self
            .builder
            .take()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?;

        let xz = builder.into_inner()?;
        let mut temp_file = xz.finish()?;
        temp_file.seek(SeekFrom::Start(0))?;

        let decoder = XzDecoder::new_multi_decoder(temp_file);
        let mut archive = Archive::new(decoder);

        let new_file = NamedTempFile::new_in(".")?;
        let xz = XzEncoder::new(new_file, self.compression_strength);
        let mut new_builder = Builder::new(xz);

        for entry_res in archive.entries()? {
            let entry = entry_res?;
            let entry_path_in_archive = entry.path()?;

            if entry_path_in_archive == entry_path || entry_path_in_archive.starts_with(entry_path)
            {
                continue;
            }

            let header = entry.header().clone();
            new_builder.append(&header, entry)?;
        }

        self.builder = Some(new_builder);
        Ok(())
    }
}
