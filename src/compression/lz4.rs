use crate::{compression::CompressionStrategy, error::ErrorKind};

/// An compression strategy using the Lz4 compression.
#[derive(Clone)]
pub struct Lz4;

impl CompressionStrategy for Lz4 {
    fn compress(&self, buffer: &[u8]) -> Vec<u8> {
        lz4_compress::compress(buffer)
    }

    fn decompress(&self, buffer: &[u8]) -> Result<Vec<u8>, ErrorKind> {
        lz4_compress::decompress(&buffer).map_err(|e| ErrorKind::CompressionError(e.to_string()))
    }
}

impl Default for Lz4 {
    fn default() -> Self {
        Lz4
    }
}
