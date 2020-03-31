use serde::{Serialize, Deserialize};

pub mod compression;
pub mod error;
pub mod transport;
pub mod uid;
pub mod state;

mod event;

pub use event::{ServerMessage, ClientMessage};

pub type EntityId = u32;
pub type ComponentId = u32;

#[derive(Clone, Hash, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentData(ComponentId, Vec<u8>);

impl ComponentData {
    pub fn new(register_id: u32, data: Vec<u8>) -> ComponentData {
        ComponentData(register_id, data)
    }

    pub fn component_id(&self) -> ComponentId {
        self.0
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.1
    }
}