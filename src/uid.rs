use serde::{Deserialize, Serialize};
use std::{fmt, u32};
use track::Identifier;
use std::collections::{HashMap};
use std::hash::Hash;

#[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct Uid(pub u32);

impl Uid {
    pub fn id(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Uid {
    fn from(uid: u32) -> Self { Self(uid) }
}

impl Identifier for Uid { }

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

pub struct UidAllocator<T: Hash + Eq> {
    index: u32,
    mapping: HashMap<T, u32>,
}

impl<T: Hash + Eq> UidAllocator<T> {
    pub fn new() -> Self {
        Self {
            index: 1,
            mapping: HashMap::new(),
        }
    }

    pub fn get(&self, key: &T) -> Uid {
        Uid(*self.mapping.get(key).expect("Uid should exist!"))
    }

    pub fn get_by_val(&self, key: &u32) -> &T {
        self.mapping.iter().find(|(k, v)| key == *v).expect("Uid should exist!").0
    }

    // Useful for when a single entity is deleted because it doesn't reconstruct the
    // entire hashmap
    pub fn deallocate(&mut self, id: T) -> Option<u32> { self.mapping.remove(&id) }

    pub fn allocate(&mut self, entity: T, id: Option<u32>) -> Uid {
        let id = id.unwrap_or_else(|| {
            let id = self.index;
            self.index += 1;
            id
        });
        self.mapping.insert(entity, id);
        Uid(id)
    }
}

