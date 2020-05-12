use std::collections::VecDeque;
use std::collections::vec_deque::Iter;
use crate::synchronisation::{ClientCommandBufferEntry, CommandFrame};
use crate::transport::NetworkCommand;

pub struct ResimulationBufferEntry<ClientToServerCommand: NetworkCommand> {
    pub start_state: Vec<u8>,
    pub to_resimmulate: Vec<ClientCommandBufferEntry<ClientToServerCommand>>,
    pub start_command_frame: CommandFrame,
    pub end_command_frame: CommandFrame,
}

pub struct ResimulationBuffer<ClientToServerCommand: NetworkCommand> {
    pub entries: VecDeque<ResimulationBufferEntry<ClientToServerCommand>>
}

impl<ClientToServerCommand: NetworkCommand> ResimulationBuffer<ClientToServerCommand> {
    pub fn new() -> ResimulationBuffer<ClientToServerCommand> {
        ResimulationBuffer {
            entries: VecDeque::new()
        }
    }
}

impl<ClientToServerCommand: NetworkCommand> ResimulationBuffer<ClientToServerCommand> {
    pub fn push(&mut self, start_command_frame: CommandFrame, end_command_frame: CommandFrame, start_state: Vec<u8>, to_resimmulate: Vec<ClientCommandBufferEntry<ClientToServerCommand>>) {
        self.entries.push_front(ResimulationBufferEntry {
            start_command_frame,
            end_command_frame,
            start_state,
            to_resimmulate
        })
    }

    pub fn iter(&self) -> Iter<ResimulationBufferEntry<ClientToServerCommand>> {
        self.entries.iter()
    }
}