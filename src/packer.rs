//! This module provides an `Packer` utility that is able to both (de)serialize and (de)compress data.

use crate::{
    compression::{CompressionStrategy, ModificationCompressor},
    serialization::{ModificationSerializer, SerializationStrategy},
};

/// The `Packer` utility can (de)serialize as well as (de)compress data.
/// The given `SerialializationStrategy` and the `CompressionStrategy` determine how this process runs.
pub struct Packer<S: SerializationStrategy, C: CompressionStrategy> {
    compression: ModificationCompressor<C>,
    serialization: ModificationSerializer<S>,
}

impl<S: SerializationStrategy, C: CompressionStrategy> Packer<S, C> {
    /// Create a new `Packer` instance, the given `SerialializationStrategy` and the `CompressionStrategy` determine how this process goes.
    pub fn new(serialization: S, compression: C) -> Packer<S, C> {
        Packer {
            serialization: ModificationSerializer::new(serialization),
            compression: ModificationCompressor::new(compression),
        }
    }

    /// Returns a reference to the `ModificationCompressor`.
    pub fn compression(&self) -> &ModificationCompressor<C> {
        &self.compression
    }

    /// Returns a reference to the `ModificationSerializer`.
    pub fn serialization(&self) -> &ModificationSerializer<S> {
        &self.serialization
    }
}

impl<S: SerializationStrategy, C: CompressionStrategy> Default for Packer<S, C> {
    fn default() -> Self {
        Packer {
            serialization: Default::default(),
            compression: Default::default(),
        }
    }
}
