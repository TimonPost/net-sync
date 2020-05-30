use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;

use crate::{error::ErrorKind, serialization::SerializationStrategy};

#[derive(Clone, Debug)]
/// Serialization strategy using bincode.
pub struct Bincode;

impl SerializationStrategy for Bincode {
    fn serialize<I: Serialize>(&self, input: &I) -> Result<Vec<u8>, ErrorKind> {
        Ok(bincode::serialize(&input).map_err(|e| ErrorKind::SerializationError(e.to_string()))?)
    }

    fn deserialize<'a, T: Deserialize<'a>>(&self, buffer: &'a [u8]) -> Result<T, ErrorKind> {
        Ok(bincode::deserialize::<T>(buffer)
            .map_err(|e| ErrorKind::SerializationError(e.to_string()))?)
    }

    fn apply_to<C: SerdeDiff>(&self, component: &mut C, data: &[u8]) -> Result<(), ErrorKind> {
        bincode::config()
            .deserialize_seed(serde_diff::Apply::deserializable(component), data)
            .map_err(|e| ErrorKind::SerializationError(e.to_string()))?;

        Ok(())
    }
}

impl Default for Bincode {
    fn default() -> Self {
        Bincode
    }
}
