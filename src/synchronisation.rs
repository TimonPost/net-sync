//! This module provides code for world synchronisation.

pub use self::{
    client_command_buffer::{ClientCommandBuffer, ClientCommandBufferEntry},
    command_frame_ticker::CommandFrameTicker,
    modified_components_buffer::ModifiedComponentsBuffer,
    resimmulation_buffer::{ResimulationBuffer, ResimulationBufferEntry},
    server_command_buffer::{PushResult, ServerCommandBuffer},
};
use crate::uid::Uid;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt::{Debug, Error, Formatter},
    hash::Hash,
};

mod client_command_buffer;
mod command_frame_ticker;
mod modified_components_buffer;
mod resimmulation_buffer;
mod server_command_buffer;

pub type CommandFrame = u32;

/// A type that contains the world state of a game.
#[derive(Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// The command frame on which this world state was generated.
    pub command_frame: CommandFrame,
    /// The command frame offset that the client has with respect to the server.
    /// The client sends a command with the current `command frame`.
    /// When the server receives the client `command frame` it calculates with its own `command frame` the offset with respect to the server.
    /// With this calculated value the client can see if the simulation should be faster or slower based on the current simulation.
    pub command_frame_offset: i32,
    /// The removed entity ids.
    pub removed: HashSet<EntityId>,
    /// The inserted entities.
    pub inserted: HashSet<EntityInsert>,
    /// The changed components and their differences.
    pub changed: HashSet<ComponentChanged>,
    /// The added components.
    pub component_added: HashSet<ComponentAdded>,
    /// The removed components.
    pub component_removed: HashSet<ComponentRemoved>,
}

impl WorldState {
    /// Returns a new empty `WorldState` instance with the given command frame.
    pub fn new(command_frame: CommandFrame) -> WorldState {
        WorldState {
            removed: HashSet::new(),
            inserted: HashSet::new(),
            changed: HashSet::new(),
            component_added: HashSet::new(),
            component_removed: HashSet::new(),
            command_frame,
            // The client command_frame offset is different for each client.
            // This offset will be set when the state is sent to a certain client.
            command_frame_offset: 0,
        }
    }

    /// Marks an entity as 'removed'.
    /// The entity will be removed from the insert buffer if it was previously marked as 'insert'.
    pub fn remove_entity(&mut self, entity_id: Uid) {
        self.inserted.retain(|x| x.0 == entity_id);
        self.removed.insert(entity_id);
    }

    /// Marks an entity as 'inserted'.
    pub fn insert_entity(&mut self, entity_id: Uid, components: Vec<ComponentData>) {
        self.inserted.insert(EntityInsert(entity_id, components));
    }

    /// Marks an entity as 'changed', previous entries of the same entity are overwritten.
    pub fn change(&mut self, entity_id: Uid, component: ComponentData) {
        // we only need the newest change of an certain entity.
        self.changed.retain(|x| x.0 == entity_id);

        self.changed.insert(ComponentChanged(entity_id, component));
    }

    /// Marks an entity as having a new component.
    pub fn add_component(&mut self, entity_id: Uid, component: ComponentData) {
        self.component_added
            .insert(ComponentAdded(entity_id, component));
    }

    /// Marks an entity as having a removed component.
    pub fn remove_component(&mut self, entity_id: Uid, component_id: ComponentId) {
        // TODO: remote `cloned`.
        self.component_added.retain(|x| x.0 == entity_id);

        self.component_removed
            .insert(ComponentRemoved(entity_id, component_id));
    }

    /// Empties all underlying buffers for the failed entities.
    pub fn reset(&mut self) {
        self.removed.clear();
        self.inserted.clear();
        self.component_removed.clear();
        self.component_added.clear();
    }

    /// Returns if the world state contains any data.
    pub fn is_empty(&self) -> bool {
        self.inserted.is_empty()
            && self.removed.is_empty()
            && self.changed.is_empty()
            && self.component_added.is_empty()
            && self.component_removed.is_empty()
    }
}

impl Debug for WorldState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Inserted: {:?}\t Removed: {:?}\t Changed: {:?}\t Added: {:?}\t Removed: {:?}",
            self.inserted.len(),
            self.removed.len(),
            self.changed.len(),
            self.component_added.len(),
            self.component_removed.len()
        )
    }
}

/// Type that is used to identify an entity.
pub type EntityId = u32;
/// Type that is used to identify an component.
pub type ComponentId = u32;

/// Type that represents the raw component data. The first entry is the 'id'
#[derive(Clone, Hash, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentData(ComponentId, Vec<u8>);

impl ComponentData {
    pub fn new(register_id: u32, data: Vec<u8>) -> ComponentData {
        ComponentData(register_id, data)
    }

    /// Returns the component id.
    pub fn component_id(&self) -> ComponentId {
        self.0
    }

    /// Returns the component data.
    pub fn data(&self) -> &Vec<u8> {
        &self.1
    }
}

/// Type used to store changed component data, such as the difference bytes, entity id and component id referring to the changed component.
#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentChanged(pub EntityId, pub ComponentData);

impl ComponentChanged {
    pub fn entity_id(&self) -> EntityId {
        self.0
    }

    pub fn component_data(&self) -> &ComponentData {
        &self.1
    }
}

/// Type used to store information for an component that is added to an entity, such as the component bytes, entity id and component id referring to the added component.
#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentAdded(pub EntityId, pub ComponentData);

impl ComponentAdded {
    pub fn entity_id(&self) -> EntityId {
        self.0
    }

    pub fn component_data(&self) -> &ComponentData {
        &self.1
    }
}

/// Type used to store information for an component that is removed from an entity, such as the entity id and component id referring to the removed component.
#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRemoved(EntityId, ComponentId);

impl ComponentRemoved {
    pub fn entity_id(&self) -> EntityId {
        self.0
    }

    pub fn component_id(&self) -> &ComponentId {
        &self.1
    }
}

/// Type used to store information for an entity that is inserted into the world, such as the entity id and component id referring to the removed component and an vector of `ComponentData`.
#[derive(Debug, Clone, PartialOrd, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityInsert(EntityId, Vec<ComponentData>);

impl EntityInsert {
    pub fn entity_id(&self) -> EntityId {
        self.0
    }

    pub fn components(&self) -> &Vec<ComponentData> {
        &self.1
    }
}

/// Marker interface for messages that can be sent by the transport layer.
pub trait NetworkMessage:
    Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static
{
}

/// Marker interface for commands that can be sent by the transport layer.
pub trait NetworkCommand: Clone + NetworkMessage + Hash + Eq + PartialEq + 'static {}

#[cfg(tests)]
mod tests {
    use net_sync::{uid::Uid, ComponentData};

    use crate::{state::WorldState, ComponentData};

    #[test]
    fn insert_remove_while_inserted_should_clear_insert() {
        let mut state = WorldState::new();
        let comp = fake_component();

        state.insert_entity(comp.0, comp.1);
        state.remove_entity(comp.0);

        assert_eq!(!state.inserted.contains_key(*comp.0));
        assert_eq!(!state.removed.contains_key(comp.0));
    }

    fn fake_component() -> (Uid, Vec<ComponentData>) {
        (1, vec![ComponentData::new(0, vec![0, 1, 2, 3])])
    }
}
