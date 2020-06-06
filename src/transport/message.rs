use serde::{Deserialize, Serialize};

use crate::synchronisation::{CommandFrame, NetworkMessage, WorldState};

#[derive(Clone, Serialize, Deserialize)]
pub enum ClientToServerMessage<Message, Command> {
    Message(Message),
    Command(CommandFrame, Command),
    TimeSync,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerToClientMessage<Message> {
    // The server state update.
    StateUpdate(WorldState),
    Message(Message),
}

impl<Message: Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static> NetworkMessage
    for ServerToClientMessage<Message>
{
}

impl<
        Message: Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static,
        Command: Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static,
    > NetworkMessage for ClientToServerMessage<Message, Command>
{
}
