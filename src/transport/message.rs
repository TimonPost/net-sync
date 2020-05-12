use serde::{Deserialize, Serialize};

use crate::state::WorldState;
use crate::transport::NetworkMessage;
use crate::synchronisation::CommandFrame;

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

impl<Message:  Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static>  NetworkMessage for ServerToClientMessage<Message>  {

}

impl<Message:  Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static, Comand:  Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static>  NetworkMessage for ClientToServerMessage<Message, Comand>  {

}


#[cfg(test)]
pub mod test {
    use crate::transport::message::MessageType;
    use crate::transport::{Message, UrgencyRequirement};
    use crate::uid::Uid;

    #[test]
    fn create_message_test() {
        let id = Uid(0);
        let event = ClientMessage::EntityRemoved(Uid(1));
        let requirement = UrgencyRequirement::Immediate;

        let message = Message::new(event.clone(), requirement.clone(), MessageType::Message);
        assert_eq!(message.message_ref(), &event);
        assert_eq!(message.urgency(), requirement);
    }
}
