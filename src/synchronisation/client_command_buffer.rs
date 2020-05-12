use std::collections::VecDeque;
use crate::transport::NetworkCommand;
use crate::synchronisation::CommandFrame;

#[derive(Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct ClientCommandBufferEntry<ClientToServerCommand: NetworkCommand> {
    pub command_frame: CommandFrame,
    pub command: ClientToServerCommand,
    pub changed_data: Vec<u8>
}

impl<ClientToServerCommand: NetworkCommand> ClientCommandBufferEntry<ClientToServerCommand> {
    pub fn new(command: ClientToServerCommand, command_frame: CommandFrame, changed_data: Vec<u8>) -> ClientCommandBufferEntry<ClientToServerCommand> {
        ClientCommandBufferEntry {
            command,
            command_frame,
            changed_data
        }
    }
}

pub struct ClientCommandBuffer<ClientToServerCommand: NetworkCommand> {
    commands: VecDeque<ClientCommandBufferEntry<ClientToServerCommand>>,
    max_command_frame_capacity: u32,
    last_seen_command_frame: CommandFrame,
    oldest_seen_command_frame: CommandFrame,
}

impl<ClientToServerCommand: NetworkCommand> ClientCommandBuffer<ClientToServerCommand> {
    pub fn with_capacity(capacity: u32) -> ClientCommandBuffer<ClientToServerCommand> {
        ClientCommandBuffer {
            commands: VecDeque::new(),
            max_command_frame_capacity: capacity,
            last_seen_command_frame: 0,
            oldest_seen_command_frame: 0,
        }
    }

    pub fn grow(&mut self, size: u32) {
        self.max_command_frame_capacity += size;
    }

    pub fn shrink(&mut self, size: u32) {
        self.max_command_frame_capacity -= size;

        if self.last_seen_command_frame > self.max_command_frame_capacity {
            let shrinked = self.last_seen_command_frame - self.max_command_frame_capacity;

            for command_frame in (0..shrinked).rev() {
                self.clear_where(command_frame);
            }
        }
    }

    pub fn push(&mut self, command: ClientToServerCommand, command_frame: CommandFrame, changed_data: Vec<u8>) {
        assert!(command_frame >= self.last_seen_command_frame);

        self.last_seen_command_frame = command_frame;

        if self.oldest_seen_command_frame == 0 {
            self.oldest_seen_command_frame = self.last_seen_command_frame;
        }

        if (self.last_seen_command_frame - self.oldest_seen_command_frame)
            == self.max_command_frame_capacity
        {
            let removed_command = self.commands.pop_back();

            if let Some(removed_command) = removed_command {
                if self.commands.len() == 0 {
                    return;
                }

                self.clear_where(removed_command.command_frame);

                if let Some(oldest) = self.commands.get(self.commands.len() - 1) {
                    self.oldest_seen_command_frame = oldest.command_frame;
                }
            }
        }

        self.commands
            .push_front(ClientCommandBufferEntry::new(command, command_frame, changed_data))
    }

    fn clear_where(&mut self, command_frame: CommandFrame) {
        // pop all commands with the same synchronisation frame as the removed synchronisation.
        while let Some(command) = self.commands.get(self.commands.len() - 1) {
            if command_frame == command.command_frame {
                self.commands.pop_back();
            } else {
                break;
            }
        }
    }

    pub fn frame(&self, command_frame: CommandFrame) -> Option<&ClientCommandBufferEntry<ClientToServerCommand>> {
        self.commands.iter().filter(|x| x.command_frame == command_frame).last()
    }

    pub fn iterate_frames(&mut self, frames_in_history: u32) -> CommandIter<'_, ClientToServerCommand> {
        let mut frames_in_history = frames_in_history;
        if frames_in_history > self.last_seen_command_frame {
            frames_in_history = self.last_seen_command_frame;
        }

        CommandIter {
            items: &self.commands,
            inter_down_to_frame: self.last_seen_command_frame - frames_in_history,
            current_index: 0,
        }
    }
}

pub struct CommandIter<'a, ClientToServerCommand: NetworkCommand> {
    items: &'a VecDeque<ClientCommandBufferEntry<ClientToServerCommand>>,
    inter_down_to_frame: CommandFrame,
    current_index: usize,
}

impl<'a, ClientToServerCommand: NetworkCommand> Iterator for CommandIter<'a, ClientToServerCommand> {
    type Item = &'a ClientCommandBufferEntry<ClientToServerCommand>;

    /// Returns `Some` when there is an item in our cache matching the `expected_index`.
    /// Returns `None` if there are no times matching our `expected` index.
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(command) = self.items.get(self.current_index) {
            if command.command_frame >= self.inter_down_to_frame {
                self.current_index += 1;
                return Some(command);
            }
        }

        return None;
    }
}

#[cfg(test)]
mod test {
    use crate::synchronisation::client_command_buffer::ClientCommandBuffer;

    #[test]
    fn should_not_size_over_capacity() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        buffer.push(1, 1);
        buffer.push(1, 2);
        buffer.push(3, 3);
        buffer.push(3, 4);

        assert_eq!(buffer.commands.len(), 3);
    }

    #[test]
    fn should_delete_all_frames_out_history_scope() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        buffer.push(1, 1); // -|
        buffer.push(1, 1); // -- second synchronisation at this frame
        buffer.push(3, 3);
        buffer.push(3, 4);

        // The buffer wil retain 3 frame ticks,
        // when the forth is pushed all commands on 1 will be dropped.
        assert_eq!(buffer.commands.len(), 2);
    }

    #[test]
    fn should_grow_capacity() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        buffer.push(1, 1); // -|
        buffer.push(1, 1); // -- second synchronisation at this frame
        buffer.push(3, 3);

        buffer.grow(1); // buffer is full, grow with 1

        buffer.push(3, 4);

        assert_eq!(buffer.commands.len(), 4);
    }

    #[test]
    fn should_shrinked_capacity() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        buffer.grow(1);

        assert_eq!(buffer.max_command_frame_capacity, 4);
    }

    #[test]
    fn should_shrink_one_and_delete_elements_above_schrinked_capacity() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        buffer.push(1, 1);
        buffer.push(1, 2);
        buffer.push(3, 3); // will be deleted by shrink

        buffer.shrink(1);

        assert_eq!(buffer.max_command_frame_capacity, 2);
    }

    #[test]
    fn should_shrink_two_and_delete_elements_above_schrinked_capacity() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        buffer.push(1, 1);
        buffer.push(1, 2);
        buffer.push(3, 3); // will be deleted by shrink

        buffer.shrink(2);

        assert_eq!(buffer.max_command_frame_capacity, 1);
    }

    #[test]
    fn should_iterate_until_history_command_frame() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        buffer.push(1, 1);
        buffer.push(1, 2);
        buffer.push(1, 2);
        buffer.push(3, 3);
        buffer.push(3, 4);

        let collected_frames: Vec<usize> =
            buffer.iterate_frames(3).map(|v| v.command_frame).collect();

        assert_eq!(collected_frames, vec![4, 3, 2, 2]);
    }

    #[test]
    fn should_iterate_all_frames() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        buffer.push(1, 1);
        buffer.push(1, 2);
        buffer.push(1, 2);
        buffer.push(3, 3);

        let collected_frames: Vec<usize> =
            buffer.iterate_frames(3).map(|v| v.command_frame).collect();

        assert_eq!(collected_frames, vec![3, 2, 2, 1]);
    }
}
