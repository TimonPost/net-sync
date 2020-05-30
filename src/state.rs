use std::{
    collections::HashSet,
    fmt::{Debug, Error, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{synchronisation::CommandFrame, uid::Uid, ComponentData, ComponentId, EntityId};

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub command_frame: CommandFrame,
    pub command_frame_offset: i32,
    pub removed: HashSet<EntityId>,
    pub inserted: HashSet<EntityInsert>,
    pub changed: HashSet<ComponentChanged>,
    pub component_added: HashSet<ComponentAdded>,
    pub component_removed: HashSet<ComponentRemoved>,
}

impl WorldState {
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

    pub fn remove_entity(&mut self, entity_id: Uid) {
        self.inserted.retain(|x| x.0 == entity_id);
        self.removed.insert(entity_id);
    }

    pub fn insert_entity(&mut self, entity_id: Uid, components: Vec<ComponentData>) {
        self.inserted.insert(EntityInsert(entity_id, components));
    }

    pub fn change(&mut self, entity_id: Uid, component: ComponentData) {
        // we only need the newest change of an certain entity.
        self.changed.retain(|x| x.0 == entity_id);

        self.changed.insert(ComponentChanged(entity_id, component));
    }

    pub fn add_component(&mut self, entity_id: Uid, component: ComponentData) {
        self.component_added
            .insert(ComponentAdded(entity_id, component));
    }

    pub fn remove_component(&mut self, entity_id: Uid, component_id: ComponentId) {
        // TODO: remote `cloned`.
        self.component_added.retain(|x| x.0 == entity_id);

        self.component_removed
            .insert(ComponentRemoved(entity_id, component_id));
    }

    pub fn reset(&mut self) {
        self.removed.clear();
        self.inserted.clear();
        self.component_removed.clear();
        self.component_added.clear();
    }

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
