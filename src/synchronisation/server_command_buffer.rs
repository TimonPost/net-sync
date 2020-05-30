use std::{collections::HashMap, hash::Hash};

use crate::synchronisation::CommandFrame;

#[derive(PartialOrd, PartialEq, Eq, Hash)]
pub struct ServerCommandBufferEntry<C> {
    pub command_frame: CommandFrame,
    pub command: C,
}

impl<C> ServerCommandBufferEntry<C> {
    pub fn new(command: C, command_frame: CommandFrame) -> ServerCommandBufferEntry<C> {
        ServerCommandBufferEntry {
            command,
            command_frame,
        }
    }
}

pub struct CommandBufferConfig {
    ignore_older_then: usize,
    ignore_newer_then: usize,
}

impl CommandBufferConfig {
    pub fn new(ignore_older_then: usize, ignore_newer_then: usize) -> CommandBufferConfig {
        CommandBufferConfig {
            ignore_newer_then,
            ignore_older_then,
        }
    }
}

impl Default for CommandBufferConfig {
    fn default() -> Self {
        CommandBufferConfig {
            ignore_newer_then: 10,
            ignore_older_then: 10,
        }
    }
}

#[derive(PartialOrd, PartialEq)]
pub enum PushResult<Command: Hash + Eq> {
    ToOld(Command),
    ToNew(Command),
    Accepted,
}

pub struct ServerCommandBuffer<Command: Hash + Eq> {
    commands: HashMap<CommandFrame, Vec<ServerCommandBufferEntry<Command>>>,
    last_seen_command_frame: CommandFrame,
    highest_seen_command_frame: CommandFrame,
    config: CommandBufferConfig,
    pub(crate) command_frame_offset: i32,
}

impl<Command> ServerCommandBuffer<Command>
where
    Command: Hash + Eq,
{
    pub fn new() -> ServerCommandBuffer<Command> {
        Self::with_config(CommandBufferConfig::default())
    }

    pub fn with_config(config: CommandBufferConfig) -> ServerCommandBuffer<Command> {
        ServerCommandBuffer {
            commands: HashMap::new(),
            last_seen_command_frame: 0,
            highest_seen_command_frame: 0,
            command_frame_offset: 0,
            config,
        }
    }

    pub fn push(
        &mut self,
        command: Command,
        client_command_frame: CommandFrame,
        server_command_frame: CommandFrame,
    ) -> PushResult<Command> {
        self.last_seen_command_frame = client_command_frame;

        // if this frame is higher then the highest last seen frame then update.
        if self.last_seen_command_frame > self.highest_seen_command_frame {
            self.highest_seen_command_frame = self.last_seen_command_frame;
        }

        // client synchronisation frame should be ahead of server synchronisation frame.
        self.command_frame_offset = client_command_frame as i32 - server_command_frame as i32;

        if self.command_frame_offset < 0 {
            if self.command_frame_offset.abs() > self.config.ignore_older_then as i32 {
                return PushResult::ToOld(command);
            }
        } else {
            if self.command_frame_offset > self.config.ignore_newer_then as i32 {
                return PushResult::ToNew(command);
            }
        }

        if self.commands.contains_key(&client_command_frame) {
            self.commands
                .get_mut(&client_command_frame)
                .unwrap()
                .push(ServerCommandBufferEntry::new(command, client_command_frame));
        } else {
            let message = vec![ServerCommandBufferEntry::new(command, client_command_frame)];
            self.commands.insert(client_command_frame, message);
        }

        PushResult::Accepted
    }

    pub fn drain_frame(
        &mut self,
        command_frame: CommandFrame,
    ) -> Option<Vec<ServerCommandBufferEntry<Command>>> {
        self.commands.remove(&command_frame)
    }

    pub fn iter_frame(
        &mut self,
        command_frame: CommandFrame,
    ) -> Option<&Vec<ServerCommandBufferEntry<Command>>> {
        self.commands.get(&command_frame)
    }

    pub fn command_frame_span(&self) -> usize {
        self.commands.len()
    }

    pub fn last_seen(&self) -> CommandFrame {
        self.last_seen_command_frame
    }

    pub fn highest_seen(&self) -> CommandFrame {
        self.highest_seen_command_frame
    }

    pub fn command_frame_offset(&self) -> i32 {
        self.command_frame_offset
    }
}

#[cfg(test)]
mod test {
    use crate::synchronisation::{
        server_command_buffer::{
            CommandBufferConfig, ServerCommandBuffer, ServerCommandBufferEntry,
        },
        PushResult,
    };

    #[test]
    fn should_add_and_drain_commands_from_frame() {
        let mut buffer = ServerCommandBuffer::with_config(CommandBufferConfig::new(3, 3));
        buffer.push(1, 1, 0);
        buffer.push(2, 1, 0);

        buffer.push(1, 2, 0);

        assert_eq!(buffer.drain_frame(1).unwrap().len(), 2);
        assert_eq!(buffer.drain_frame(2).unwrap().len(), 1);
    }

    #[test]
    fn should_set_last_seen_commands_frame() {
        let mut buffer = ServerCommandBuffer::with_config(CommandBufferConfig::new(3, 3));
        buffer.push(1, 1, 0);
        assert_eq!(buffer.last_seen_command_frame, 1);

        buffer.push(2, 2, 0);

        assert_eq!(buffer.last_seen_command_frame, 2);
    }

    #[test]
    fn should_set_highest_seen_commands_frame() {
        let mut buffer = ServerCommandBuffer::with_config(CommandBufferConfig::new(3, 3));
        buffer.push(1, 1, 0);
        assert_eq!(buffer.highest_seen_command_frame, 1);

        buffer.push(2, 2, 0);

        assert_eq!(buffer.highest_seen_command_frame, 2);

        buffer.push(2, 1, 0);

        assert_eq!(buffer.highest_seen_command_frame, 2);
    }

    #[test]
    fn should_ignore_older_command_frame() {
        let mut buffer = ServerCommandBuffer::with_config(CommandBufferConfig::new(3, 3));
        buffer.push(1, 1, 0);

        let result = buffer.push(1, 5, 0);

        match result {
            PushResult::ToNew(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_ignore_future_command_frame() {
        let mut buffer = ServerCommandBuffer::with_config(CommandBufferConfig::new(3, 3));
        buffer.push(1, 1, 0);
        buffer.push(1, 2, 0);
        buffer.push(1, 3, 0);
        buffer.push(1, 4, 0);
        buffer.push(1, 5, 0);

        let result = buffer.push(1, 1, 0);

        match result {
            PushResult::ToOld(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_buffer_on_same_command_frame() {
        let mut buffer = ServerCommandBuffer::with_config(CommandBufferConfig::new(3, 3));
        buffer.push(1, 1, 0);
        buffer.push(2, 1, 0);

        assert_eq!(
            buffer
                .drain_frame(1)
                .unwrap()
                .iter()
                .map(|(v)| v.command_frame)
                .collect::<Vec<u32>>(),
            vec![1, 2]
        )
    }

    #[test]
    fn push_updates_command_frame_offset() {
        let mut buffer = ServerCommandBuffer::with_config(CommandBufferConfig::new(3, 3));

        buffer.push(1, 2, 1);
        assert_eq!(buffer.command_frame_offset, 1);
        buffer.push(2, 4, 3);
        assert_eq!(buffer.command_frame_offset, 1);
    }
}
