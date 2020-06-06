//! This module provides code for identifying entities.

use std::{collections::HashMap, hash::Hash, u32};

pub type Uid = u32;

/// This allocator can be used to generate identifiers for an given generic type.
/// The allocator uses an underlying hashmap for quick look ups,
/// were the `Key` is `T` and the `Value` is `u32`.
///
/// Identifiers are generated incremental. This might change in the future to reuse space.
pub struct UidAllocator<T: Hash + Eq> {
    // An incremental counter used for the identifier.
    current_id: u32,
    // A hasmap where the generic types and assigned identifiers are stored.
    mapping: HashMap<T, u32>,
}

impl<T: Hash + Eq> UidAllocator<T> {
    /// Returns a new instance of the `UidAllocator`.n
    pub fn new() -> Self {
        Self {
            current_id: 1,
            mapping: HashMap::new(),
        }
    }

    /// Returns the assigned `Uid` for the given key `T`.
    pub fn get(&self, key: &T) -> Uid {
        *self.mapping.get(key).expect("Uid should exist!")
    }

    pub fn get_mut(&mut self, key: &T) -> Option<&mut Uid> {
        self.mapping.get_mut(key)
    }

    pub fn get_by_val(&self, key: &u32) -> &T {
        self.mapping
            .iter()
            .find(|(_k, v)| key == *v)
            .expect("Uid should exist!")
            .0
    }

    pub fn replace_val(&mut self, original: u32, new: u32) {
        *self
            .mapping
            .iter_mut()
            .find(|(_k, v)| original == **v)
            .expect("Uid should exist!")
            .1 = new;
    }

    // Useful for when a single entity is deleted because it doesn't reconstruct the
    // entire hashmap
    pub fn deallocate(&mut self, id: T) -> Option<u32> {
        self.mapping.remove(&id)
    }

    pub fn allocate(&mut self, entity: T, id: Option<u32>) -> Uid {
        let id = id.unwrap_or_else(|| self.get_and_increment());

        self.mapping.insert(entity, id);
        id
    }

    pub fn get_and_increment(&mut self) -> u32 {
        let id = self.current_id;
        self.current_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {}
