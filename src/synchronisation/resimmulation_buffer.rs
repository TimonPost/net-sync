use std::collections::{vec_deque::Iter, VecDeque};

use crate::{
    synchronisation::{ClientCommandBufferEntry, CommandFrame},
    transport::NetworkCommand,
};

// TODO: make resim buffer empty when iterated

pub struct ResimulationBufferEntry<ClientToServerCommand: NetworkCommand> {
    pub to_resimmulate: Vec<ClientCommandBufferEntry<ClientToServerCommand>>,
    pub start_command_frame: CommandFrame,
    pub end_command_frame: CommandFrame,
}

pub struct ResimulationBuffer<ClientToServerCommand: NetworkCommand> {
    pub entries: VecDeque<ResimulationBufferEntry<ClientToServerCommand>>,
}

impl<ClientToServerCommand: NetworkCommand> ResimulationBuffer<ClientToServerCommand> {
    pub fn new() -> ResimulationBuffer<ClientToServerCommand> {
        ResimulationBuffer {
            entries: VecDeque::new(),
        }
    }

    pub fn push(
        &mut self,
        start_command_frame: CommandFrame,
        end_command_frame: CommandFrame,
        to_resimmulate: Vec<ClientCommandBufferEntry<ClientToServerCommand>>,
    ) {
        self.entries.push_front(ResimulationBufferEntry {
            start_command_frame,
            end_command_frame,
            to_resimmulate,
        })
    }

    pub fn iter(&self) -> Iter<ResimulationBufferEntry<ClientToServerCommand>> {
        self.entries.iter()
    }
}
