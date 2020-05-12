use std::collections::HashMap;
use std::hash::Hash;
use std::u32;

pub type Uid = u32;

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
        let id = self.index;
        self.index += 1;
        id
    }
}

#[cfg(test)]
mod tests {

}
