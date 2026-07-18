use crate::archive::traits::AppArchive;
use anyhow::{Result, anyhow};
use flate2::Compression;
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tempfile::tempfile;

pub struct TarGz {
    pub builder: Option<Builder<GzEncoder<File>>>,
}

impl AppArchive for TarGz {
    type Output = GzEncoder<File>;

    fn new(compression_strength: u32) -> Result<Self> {
        let file = tempfile()?;
        let gz = GzEncoder::new(file, Compression::new(compression_strength));
        let builder = Builder::new(gz);
        Ok(TarGz {
            builder: Some(builder),
        })
    }

    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()> {
        self.builder
            .as_mut()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?
            .append_path_with_name(full_path, archive_path)?;
        Ok(())
    }

    fn save_archive(self, save_filename: &str) -> Result<()> {
        let builder = self
            .builder
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?;
        let gz = builder.into_inner()?;

        let mut temp_file = gz.finish()?;

        temp_file.seek(SeekFrom::Start(0))?;

        let filename = PathBuf::from(format!("{}.tar.gz", save_filename));
        let mut out = File::create(filename)?;
        io::copy(&mut temp_file, &mut out)?;

        Ok(())
    }

    fn save_msg(&self, name: &str) -> String {
        format!("Saved to {}.tar.gz", name)
    }

    fn extract_archive_file(archive_path: &Path, destination: &Path) -> Result<()> {
        let file = File::open(archive_path)?;
        let decoder = MultiGzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive.unpack(destination)?;

        Ok(())
    }

    fn extract_current(&mut self, destination: &Path) -> Result<()> {
        let builder = self
            .builder
            .take()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?;
        let gz = builder.into_inner()?;
        let mut temp_file = gz.finish()?;
        temp_file.seek(SeekFrom::Start(0))?;

        let decoder = MultiGzDecoder::new(temp_file);
        let mut archive = Archive::new(decoder);
        archive.unpack(destination)?;

        Ok(())
    }

    /// Removes selected file from the archive
    fn remove_from_archive(&mut self, entry_path: &Path) -> Result<()> {
        let builder = self
            .builder
            .take()
            .ok_or_else(|| anyhow!("archive builder is not initialized"))?;

        let gz = builder.into_inner()?;
        let mut temp_file = gz.finish()?;
        temp_file.seek(SeekFrom::Start(0))?;

        let decoder = MultiGzDecoder::new(temp_file);
        let mut archive = Archive::new(decoder);

        let new_file = tempfile()?;
        let gz = GzEncoder::new(new_file, Compression::new(6));
        let mut new_builder = Builder::new(gz);

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

    fn load_from_file(path: &Path, compression_strength: u32) -> Result<(Self, Vec<PathBuf>)> {
        let file = File::open(path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let new_file = tempfile()?;
        let gz = GzEncoder::new(new_file, Compression::new(compression_strength));
        let mut new_builder = Builder::new(gz);

        let mut entry_names = Vec::new();
        for entry_res in archive.entries()? {
            let entry = entry_res?;
            let entry_path = entry.path()?.to_path_buf();
            entry_names.push(entry_path.clone());
            let header = entry.header().clone();
            new_builder.append(&header, entry)?;
        }

        Ok((
            TarGz {
                builder: Some(new_builder),
            },
            entry_names,
        ))
    }
}
