use serde::{Deserialize, Serialize};
use std::{fmt, u32};
use track::Identifier;
use std::collections::HashSet;

#[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct Uid(pub u32);

impl From<u32> for Uid {
    fn from(uid: u32) -> Self { Self(uid) }
}

impl Identifier for Uid { }

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

pub struct UidAllocator {
    index: u32,
    mapping: HashSet<u32>,
}

impl UidAllocator {
    pub fn new() -> Self {
        Self {
            index: 0,
            mapping: HashSet::new(),
        }
    }

    // Useful for when a single entity is deleted because it doesn't reconstruct the
    // entire hashmap
    pub fn deallocate(&mut self, id: u32) { self.mapping.remove(&id); }

    pub fn allocate(&mut self, id: Option<u32>) -> Uid {
        let id = id.unwrap_or_else(|| {
            let id = self.index;
            self.index += 1;
            id
        });
        self.mapping.insert(id);
        Uid(id)
    }
}

impl Default for UidAllocator {
    fn default() -> Self { Self::new() }
}
