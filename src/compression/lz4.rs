use crate::compression::CompressionStrategy;
use std::result;

/// An compression strategy using the Lz4 compression.
#[derive(Clone)]
pub struct Lz4;

impl CompressionStrategy for Lz4 {
    fn compress(&self, buffer: &[u8]) -> Vec<u8> {
        lz4_compress::compress(buffer)
    }

    fn decompress(&self, buffer: Vec<u8>) -> result::Result<Vec<u8>, ()> {
        lz4_compress::decompress(&buffer).map_err(|_| ())
    }
}

impl Default for Lz4 {
    fn default() -> Self {
        Lz4
    }
}
