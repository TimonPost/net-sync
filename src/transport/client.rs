use std::{net::SocketAddr, time::Instant};

use crate::{
    synchronisation::{CommandFrame, NetworkCommand, NetworkMessage, ServerCommandBuffer},
    transport::{message, PostBox},
};

pub type ClientId = u16;

pub struct Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>
where
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand,
{
    client_id: ClientId,
    addr: SocketAddr,
    message_postbox:
        PostBox<ClientToServerMessage, message::ServerToClientMessage<ServerToClientMessage>>,
    pub(crate) command_postbox: ServerCommandBuffer<ClientToServerCommand>,

    last_packet: Instant,
}

impl<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>
    Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>
where
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand,
{
    pub fn new(addr: SocketAddr, connection_id: ClientId) -> Self {
        Client {
            client_id: connection_id,
            addr,
            message_postbox: PostBox::new(),
            command_postbox: ServerCommandBuffer::new(),

            last_packet: Instant::now(),
        }
    }

    pub fn add_received_message(
        &mut self,
        message: message::ClientToServerMessage<ClientToServerMessage, ClientToServerCommand>,
        server_command_frame: CommandFrame,
    ) {
        self.last_packet = Instant::now();

        match message {
            message::ClientToServerMessage::Message(message) => {
                self.message_postbox.add_to_inbox(message);
            }
            message::ClientToServerMessage::Command(client_command_frame, command) => {
                self.command_postbox
                    .push(command, client_command_frame, server_command_frame);
            }
            message::ClientToServerMessage::TimeSync => {}
        };
    }

    pub fn last_packet(&self) -> Instant {
        self.last_packet
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn postbox_mut(
        &mut self,
    ) -> &mut PostBox<ClientToServerMessage, message::ServerToClientMessage<ServerToClientMessage>>
    {
        &mut self.message_postbox
    }

    pub fn postbox(
        &self,
    ) -> &PostBox<ClientToServerMessage, message::ServerToClientMessage<ServerToClientMessage>>
    {
        &self.message_postbox
    }

    pub fn command_postbox_mut(&mut self) -> &mut ServerCommandBuffer<ClientToServerCommand> {
        &mut self.command_postbox
    }

    pub fn command_postbox(&self) -> &ServerCommandBuffer<ClientToServerCommand> {
        &self.command_postbox
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::{Client, ClientToServerMessage, ServerToClientMessage};

    #[test]
    fn command_message_is_added_to_command_inbox() {
        let mut client = Client::<u32, u32, u32>::new("127.0.0.1:0".parse().unwrap(), 0);

        client.add_received_message(ClientToServerMessage::Command(1, 1), 1);

        let mut postbox = client.postbox_mut();
        assert_eq!(
            client.command_postbox_mut().drain_frame(1).unwrap().len(),
            1
        );
    }

    #[test]
    fn normal_message_is_added_to_postbox_inbox() {
        let mut client = Client::<u32, u32, u32>::new("127.0.0.1:0".parse().unwrap(), 0);

        client.add_received_message(ClientToServerMessage::Message(1), 1);

        let mut postbox = client.postbox_mut();
        assert_eq!(postbox.drain_inbox(|_| true).len(), 1);
    }
}
