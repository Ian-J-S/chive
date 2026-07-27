use anyhow::Result;
use flate2::{Compression, read::MultiGzDecoder, write::GzEncoder};
use std::io::Read;
use tempfile::NamedTempFile;

use crate::archive::tar_shared::{Tar, TarCompression, TarEncoder};

pub struct Gzip;

impl TarEncoder for GzEncoder<NamedTempFile> {
    fn finish(self) -> Result<NamedTempFile> {
        Ok(GzEncoder::finish(self)?)
    }
}

impl TarCompression for Gzip {
    type Encoder = GzEncoder<NamedTempFile>;
    type Decoder<R: Read> = MultiGzDecoder<R>;

    const EXTENSION: &'static str = ".tar.gz";

    fn encoder(output: NamedTempFile, compression_strength: u32) -> Result<Self::Encoder> {
        Ok(GzEncoder::new(
            output,
            Compression::new(compression_strength),
        ))
    }

    fn decoder<R>(reader: R) -> Result<Self::Decoder<R>>
    where
        R: Read + 'static,
    {
        Ok(MultiGzDecoder::new(reader))
    }
}

pub type TarGz = Tar<Gzip>;
