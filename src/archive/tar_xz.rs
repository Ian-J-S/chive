use anyhow::Result;
use std::io::Read;
use tempfile::NamedTempFile;
use xz2::{read::XzDecoder, write::XzEncoder};

use crate::archive::tar_shared::{Tar, TarCompression, TarEncoder};

pub struct Xz;

impl TarEncoder for XzEncoder<NamedTempFile> {
    fn finish(self) -> Result<NamedTempFile> {
        Ok(XzEncoder::finish(self)?)
    }
}

impl TarCompression for Xz {
    type Encoder = XzEncoder<NamedTempFile>;
    type Decoder<R: Read> = XzDecoder<R>;

    const EXTENSION: &'static str = ".tar.xz";

    fn encoder(output: NamedTempFile, compression_strength: u32) -> Result<Self::Encoder> {
        Ok(XzEncoder::new(output, compression_strength))
    }

    fn decoder<R>(reader: R) -> Result<Self::Decoder<R>>
    where
        R: Read + 'static,
    {
        Ok(XzDecoder::new_multi_decoder(reader))
    }
}

pub type TarXz = Tar<Xz>;
