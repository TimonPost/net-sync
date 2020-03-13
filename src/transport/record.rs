use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRecord {
    register_id: u32,
    data: Vec<u8>,
}

impl ComponentRecord {
    pub fn new(register_id: u32, data: Vec<u8>) -> Self {
        ComponentRecord { register_id, data }
    }

    pub fn register_id(&self) -> u32 {
        self.register_id
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
