use anyhow::Result;
use std::io::{BufReader, Read};
use tempfile::NamedTempFile;
use zstd::{Decoder, Encoder};

use crate::archive::tar_shared::{Tar, TarCompression, TarEncoder};

pub struct Zst;

impl TarEncoder for Encoder<'_, NamedTempFile> {
    fn finish(self) -> Result<NamedTempFile> {
        Ok(Encoder::finish(self)?)
    }
}

impl TarCompression for Zst {
    type Encoder = Encoder<'static, NamedTempFile>;
    type Decoder<R: Read> = Decoder<'static, BufReader<R>>;

    const EXTENSION: &'static str = ".tar.zst";

    fn encoder(output: NamedTempFile, compression_strength: u32) -> Result<Self::Encoder> {
        Ok(Encoder::new(output, compression_strength as i32)?)
    }

    fn decoder<R>(reader: R) -> Result<Self::Decoder<R>>
    where
        R: Read + 'static,
    {
        Ok(Decoder::new(reader)?)
    }
}

pub type TarZst = Tar<Zst>;
