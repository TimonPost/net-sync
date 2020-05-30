use std::{
    any::TypeId,
    collections::{hash_map::Drain, HashMap},
};

use crate::{synchronisation::CommandFrame, tracker::ServerChangeTracker, uid::Uid};

type EntryIdentifier = (Uid, TypeId);

pub struct ModifiedComponentsBuffer {
    pub entries: HashMap<CommandFrame, HashMap<EntryIdentifier, Vec<u8>>>,
}

impl ModifiedComponentsBuffer {
    pub fn new() -> ModifiedComponentsBuffer {
        ModifiedComponentsBuffer {
            entries: HashMap::new(),
        }
    }

    pub fn push(
        &mut self,
        frame: CommandFrame,
        unchanged_serialized: Vec<u8>,
        entity_identifier: Uid,
        component_type: TypeId,
    ) {
        let entry = self.entries.entry(frame).or_insert_with(|| HashMap::new());

        if !entry.contains_key(&(entity_identifier, component_type)) {
            let mut hashmap = HashMap::new();
            hashmap.insert((entity_identifier, component_type), unchanged_serialized);

            self.entries.insert(frame, hashmap);
        }
    }

    pub fn drain_entries(&mut self) -> Drain<CommandFrame, HashMap<EntryIdentifier, Vec<u8>>> {
        self.entries.drain()
    }
}

impl ServerChangeTracker for ModifiedComponentsBuffer {
    fn push(
        &mut self,
        command_frame: CommandFrame,
        entity_identifier: Uid,
        unchanged_serialized: Vec<u8>,
        component_type: TypeId,
    ) {
        self.push(
            command_frame,
            unchanged_serialized,
            entity_identifier,
            component_type,
        );
    }
}
