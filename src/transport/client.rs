use std::net::SocketAddr;

use crate::synchronisation::ServerCommandBuffer;
use crate::transport::{message, PostBox, NetworkCommand, NetworkMessage};
use std::time::Instant;

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
    command_postbox: ServerCommandBuffer<ClientToServerCommand>,

    last_packet: Instant,
    command_frame_offset: usize
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
            command_frame_offset: 7
        }
    }

    pub fn add_received_message(
        &mut self,
        message: message::ClientToServerMessage<ClientToServerMessage, ClientToServerCommand>,
    ) {
        self.last_packet = Instant::now();

        match message {
            message::ClientToServerMessage::Message(message) => {
                self.message_postbox.add_to_inbox(message);
            }
            message::ClientToServerMessage::Command(client_command_frame, command) => {
                self.command_postbox.push(command, client_command_frame);

                // is it the newest one
                // calculate offset



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

