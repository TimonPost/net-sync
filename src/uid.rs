use std::{u32};
use std::collections::{HashMap};
use std::hash::Hash;
use std::collections::hash_map::Entry;

pub type Uid = u32;

pub struct UidAllocator<T: Hash + Eq> {
    index: u32,
    mapping: HashMap<T, u32>,
    /// Imagine that in 1 frame, both an insert and modify event of a client arrive on the server.
    /// In this case we need to replace the client generated id and use it for the insert and modify.
    /// We can only assign an entity id when the entity is inserted, because we need `T` as key where `T` represents the ECS entity.
    /// This is a problem because this can be done in a different system or at a different time.
    ///
    /// So what we do in this case is reserve an id which we can link to `T` later,
    /// when the entity is generated.
    /// A reservation contains the server generated identifier and the client identifier.
    reservated: HashMap<u32, u32>
}

impl<T: Hash + Eq> UidAllocator<T> {
    pub fn new() -> Self {
        Self {
            index: 1,
            mapping: HashMap::new(),
            reservated: HashMap::new()
        }
    }

    pub fn get(&self, key: &T) -> Uid {
        *self.mapping.get(key).expect("Uid should exist!")
    }

    pub fn get_mut(&mut self, key: &T) -> Option<&mut Uid> {
        self.mapping.get_mut(key)
    }

    pub fn get_by_val(&self, key: &u32) -> &T {
        self.mapping.iter().find(|(_k, v)| key == *v).expect("Uid should exist!").0
    }

    pub fn  replace_val(&mut self, original: u32, new: u32) {
        *self.mapping.iter_mut().find(|(_k, v)| original == **v).expect("Uid should exist!").1 = new;
    }


    // Useful for when a single entity is deleted because it doesn't reconstruct the
    // entire hashmap
    pub fn deallocate(&mut self, id: T) -> Option<u32> { self.mapping.remove(&id) }

    pub fn reserve_for(&mut self, client_id: u32) -> u32 {
        let server_id =  self.get_and_increment();
        self.reservated.insert(client_id, server_id);
        server_id
    }

    pub fn reserved(&self, client_id: u32) -> Option<&u32> {
        self.reservated.get(&client_id)
    }

    pub fn allocate(&mut self, entity: T, id: Option<u32>) -> Uid {
        let mut id = id.unwrap_or_else(|| { self.get_and_increment() });

        if self.reservated.contains_key(&id) {
            id = self.reservated.remove(&id).expect("Should exist.");
        }

        self.mapping.insert(entity, id);
        id
    }

    pub fn get_and_increment(&mut self) -> u32 {
        let id = self.index;
        self.index += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use crate::uid::UidAllocator;
    use std::intrinsics::atomic_load;

    #[test]
    fn reserve_uid() {
        let mut allocator = UidAllocator::<u32>::new();
        allocator.allocate(100, Some(1));
        allocator.allocate(100, Some(2));

        let server_id1 = allocator.reserve_for(1); // this should reserve index 3
        let server_id2 = allocator.reserve_for(2); // this should reserve index 4

        assert_eq!(server_id1 == 3);
        assert_eq!(server_id1 == 4);
    }
}

