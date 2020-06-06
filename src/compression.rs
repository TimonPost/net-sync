//! Module that provides different compressors to be used for compressing data.
//!
//! By default, a number of compressions are supplied that can be used by turning on the feature flag.
//! You might want to create your own compression strategy by implementing: [CompressionStrategy](LINK).
//!
//! | Feature | Description |
//! | :-- | :-- |
//! | `lz4-compression` | compression using [lz4](LINK) (enabled by default) .|

use crate::error::ErrorKind;

#[cfg(feature = "lz4-compresion")]
pub mod lz4;

/// An adapter interface with extension methods for compression purposes in this crate.
pub trait CompressionStrategy: Clone + Default + Send + Sync {
    /// Compresses the given buffer and returns the compression result.
    fn compress(&self, buffer: &[u8]) -> Vec<u8>;

    /// Decompresses the given buffer and returns the uncompressed result.
    fn decompress(&self, buffer: &[u8]) -> Result<Vec<u8>, ErrorKind>;
}

/// A wrapper type over an implementation of CompressionStrategy.
#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub struct ModificationCompressor<S: CompressionStrategy> {
    strategy: S,
}

impl<S: CompressionStrategy> ModificationCompressor<S> {
    pub fn new(strategy: S) -> ModificationCompressor<S> {
        ModificationCompressor { strategy }
    }

    /// Compresses the given buffer with the generic compression strategy.
    pub fn compress(&self, buffer: &[u8]) -> Vec<u8> {
        self.strategy.compress(buffer)
    }

    /// Decompresses the given buffer with the generic compression strategy.
    pub fn decompress(&self, buffer: &[u8]) -> Result<Vec<u8>, ErrorKind> {
        self.strategy.decompress(buffer)
    }
}

impl<S: CompressionStrategy> Default for ModificationCompressor<S> {
    fn default() -> Self {
        ModificationCompressor::new(Default::default())
    }
}
