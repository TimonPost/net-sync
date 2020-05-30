use std::{
    any::TypeId,
    collections::{
        vec_deque::{Iter, IterMut},
        VecDeque,
    },
};

use crate::{
    synchronisation::CommandFrame, tracker::ClientChangeTracker, transport::NetworkCommand,
    uid::Uid,
};

#[derive(Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct ClientCommandBufferEntry<ClientToServerCommand: NetworkCommand> {
    pub command_frame: CommandFrame,
    pub command: ClientToServerCommand,
    pub unchanged_data: Vec<u8>,
    pub changed_data: Vec<u8>,
    pub entity_id: Uid,
    pub component_type: TypeId,
    pub is_sent: bool,
}

impl<ClientToServerCommand: NetworkCommand> ClientCommandBufferEntry<ClientToServerCommand> {
    pub fn new(
        command: ClientToServerCommand,
        command_frame: CommandFrame,
        unchanged_data: Vec<u8>,
        changed_data: Vec<u8>,
        entity_id: Uid,
        component_type: TypeId,
    ) -> ClientCommandBufferEntry<ClientToServerCommand> {
        ClientCommandBufferEntry {
            command,
            command_frame,
            unchanged_data,
            changed_data,
            is_sent: false,
            entity_id,
            component_type,
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
                self.clear_old(command_frame);
            }
        }
    }

    pub fn push(
        &mut self,
        command: ClientToServerCommand,
        command_frame: CommandFrame,
        unchanged_data: Vec<u8>,
        changed_data: Vec<u8>,
        entity_id: Uid,
        component_type: TypeId,
    ) {
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

                self.clear_old(removed_command.command_frame);

                if let Some(oldest) = self.commands.get(self.commands.len() - 1) {
                    self.oldest_seen_command_frame = oldest.command_frame;
                }
            }
        }

        self.commands.push_front(ClientCommandBufferEntry::new(
            command,
            command_frame,
            unchanged_data,
            changed_data,
            entity_id,
            component_type,
        ))
    }

    fn clear_old(&mut self, command_frame: CommandFrame) {
        // pop all commands with the same synchronisation frame as the removed synchronisation.
        while let Some(command) = self.commands.get(self.commands.len() - 1) {
            if command_frame == command.command_frame {
                self.commands
                    .pop_back()
                    .expect("Should have command frame here");
            } else {
                break;
            }
        }
    }

    pub fn iter(&self) -> Iter<ClientCommandBufferEntry<ClientToServerCommand>> {
        self.commands.iter()
    }

    pub fn iter_history(
        &mut self,
        frames_in_history: u32,
    ) -> CommandIterMut<'_, ClientToServerCommand> {
        let mut frames_in_history = frames_in_history;
        if frames_in_history > self.last_seen_command_frame {
            frames_in_history = self.last_seen_command_frame;
        }

        CommandIterMut {
            items: self.commands.iter_mut(),
            iter_down_to_frame: self.last_seen_command_frame - frames_in_history,
        }
    }
}

impl<C: NetworkCommand> ClientChangeTracker<C> for ClientCommandBuffer<C> {
    fn push(
        &mut self,
        command: C,
        command_frame: CommandFrame,
        entity_id: Uid,
        unchanged_serialized: Vec<u8>,
        changed_serialized: Vec<u8>,
        component_type: TypeId,
    ) {
        self.push(
            command,
            command_frame,
            unchanged_serialized,
            changed_serialized,
            entity_id,
            component_type,
        );
    }
}

pub struct CommandIterMut<'a, ClientToServerCommand: NetworkCommand> {
    items: IterMut<'a, ClientCommandBufferEntry<ClientToServerCommand>>,
    iter_down_to_frame: CommandFrame,
}

impl<'a, ClientToServerCommand: NetworkCommand> Iterator
    for CommandIterMut<'a, ClientToServerCommand>
{
    type Item = &'a mut ClientCommandBufferEntry<ClientToServerCommand>;

    /// Returns `Some` when there is an item in our cache matching the `expected_index`.
    /// Returns `None` if there are no times matching our `expected` index.
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(command) = self.items.next() {
            if command.command_frame >= self.iter_down_to_frame && !command.is_sent {
                return Some(command);
            }
        }

        return None;
    }
}

#[cfg(test)]
mod test {
    use crate::{
        synchronisation::{client_command_buffer::ClientCommandBuffer, ClientCommandBufferEntry},
        transport::{NetworkCommand, NetworkMessage},
    };
    use std::any::TypeId;

    #[test]
    fn should_not_size_over_capacity() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        push_command(&mut buffer, 1, 1);
        push_command(&mut buffer, 1, 2);
        push_command(&mut buffer, 3, 3);
        push_command(&mut buffer, 3, 4);

        assert_eq!(buffer.commands.len(), 3);
    }

    #[test]
    fn should_delete_all_frames_out_history_scope() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        push_command(&mut buffer, 1, 1); // -|
        push_command(&mut buffer, 1, 1); // -- second synchronisation at this frame
        push_command(&mut buffer, 3, 3);
        push_command(&mut buffer, 3, 4);

        // The buffer wil retain 3 frame ticks,
        // when the forth is pushed all commands on 1 will be dropped.
        assert_eq!(buffer.commands.len(), 2);
    }

    #[test]
    fn should_grow_capacity() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        push_command(&mut buffer, 1, 1); // -|
        push_command(&mut buffer, 1, 1); // -- second synchronisation at this frame
        push_command(&mut buffer, 3, 3);

        buffer.grow(1); // buffer is full, grow with 1

        push_command(&mut buffer, 3, 4);

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
        push_command(&mut buffer, 1, 1);
        push_command(&mut buffer, 1, 2);
        push_command(&mut buffer, 3, 3); // will be deleted by shrink

        buffer.shrink(1);

        assert_eq!(buffer.max_command_frame_capacity, 2);
    }

    #[test]
    fn should_shrink_two_and_delete_elements_above_schrinked_capacity() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        push_command(&mut buffer, 1, 1);
        push_command(&mut buffer, 1, 2);
        push_command(&mut buffer, 3, 3); // will be deleted by shrink

        buffer.shrink(2);

        assert_eq!(buffer.max_command_frame_capacity, 1);
    }

    #[test]
    fn should_iterate_until_history_command_frame() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        push_command(&mut buffer, 1, 1);
        push_command(&mut buffer, 1, 2);
        push_command(&mut buffer, 1, 2);
        push_command(&mut buffer, 3, 3);
        push_command(&mut buffer, 3, 4);

        let collected_frames: Vec<u32> = buffer.iter_history(3).map(|v| v.command_frame).collect();

        assert_eq!(collected_frames, vec![4, 3, 2, 2]);
    }

    #[test]
    fn should_iterate_all_frames() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        push_command(&mut buffer, 1, 1);
        push_command(&mut buffer, 1, 2);
        push_command(&mut buffer, 1, 2);
        push_command(&mut buffer, 3, 3);

        let collected_frames: Vec<u32> = buffer.iter_history(3).map(|v| v.command_frame).collect();

        assert_eq!(collected_frames, vec![3, 2, 2, 1]);
    }

    #[test]
    fn clear_old_command_frame() {
        let mut buffer = ClientCommandBuffer::<u32>::with_capacity(3);
        push_command(&mut buffer, 1, 1);
        push_command(&mut buffer, 1, 2);
        push_command(&mut buffer, 1, 2);

        buffer.clear_old(1);

        let collected_frames: Vec<u32> = buffer.iter_history(1).map(|v| v.command_frame).collect();

        assert_eq!(collected_frames, vec![2, 2]);
    }

    fn push_command(buffer: &mut ClientCommandBuffer<u32>, command: u32, command_frame: u32) {
        buffer.push(
            command,
            command_frame,
            vec![],
            vec![],
            1,
            TypeId::of::<String>(),
        );
    }

    impl NetworkMessage for u32 {}

    impl NetworkCommand for u32 {}
}
