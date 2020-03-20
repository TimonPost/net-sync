use std::fmt::{Debug, Formatter, Error};
use serde::{Serialize, Deserialize};
use std::collections::{HashSet, HashMap};
use crate::{ComponentId, ComponentData, EntityId};
use crate::uid::Uid;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    pub(crate) removed: HashSet<EntityId>,
    pub(crate) inserted: HashMap<EntityId, EntityInsert>,
    pub(crate) changed: HashMap<EntityId, ComponentChanged>,
    pub(crate) component_added: HashMap<EntityId, ComponentAdded>,
    pub(crate) component_removed: HashMap<EntityId, ComponentRemoved>
}

impl WorldState {
    pub fn new() -> WorldState {
        WorldState {
            removed: HashSet::new(),
            inserted: HashMap::new(),
            changed: HashMap::new(),
            component_added: HashMap::new(),
            component_removed: HashMap::new()
        }
    }

    pub fn remove_entity(&mut self, id: Uid) {
        if self.inserted.contains_key(&id) {
            self.inserted.remove(&id);
        }

        self.removed.insert(id);
    }

    pub fn insert_entity(&mut self, id: Uid, components: Vec<ComponentData>) {
        self.inserted.insert(id, EntityInsert(id, components));
    }

    pub fn change(&mut self, entity_id: Uid, component: ComponentData) {
        self.changed.insert(entity_id, ComponentChanged(entity_id, component));
    }

    pub fn add_component(&mut self, entity_id: Uid, component: ComponentData) {
        self.component_added.insert(component.component_id(),  ComponentAdded(entity_id, component));
    }

    pub fn remove_component(&mut self, entity_id: Uid, component_id: ComponentId) {
        if self.component_added.contains_key(&entity_id) {
            self.component_added.remove(&entity_id);
        }

        self.component_removed.insert(entity_id, ComponentRemoved(entity_id, component_id));
    }

    pub fn reset(&mut self) {
        self.removed.clear();
        self.inserted.clear();
        self.component_removed.clear();
        self.component_added.clear();
    }
}

impl Debug for WorldState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Inserted: {:?}\t Removed: {:?}\t Changed: {:?}\t Added: {:?}\t Removed: {:?}", self.inserted.len(), self.removed.len(), self.changed.len(), self.component_added.len(), self.component_removed.len())
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentChanged(EntityId, ComponentData);

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentAdded(EntityId, ComponentData);

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRemoved(EntityId, ComponentId);

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityInsert(EntityId, Vec<ComponentData>);

#[cfg(tests)]
mod tests {
    use crate::state::WorldState;
    use net_sync::ComponentData;
    use net_sync::uid::Uid;
    use crate::ComponentData;

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
        (1, vec![ComponentData::new(0, vec![0,1,2,3])])
    }
}