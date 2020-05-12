use std::net::SocketAddr;
use crate::transport;
use log::debug;
use crate::transport::{NetworkMessage, NetworkCommand, ClientId, Client};
use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut};
use std::iter::Filter;

pub struct PostOffice<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>
where
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand,
{
    clients: HashMap<
        ClientId,
        Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>,
    >,
}

impl<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>
    PostOffice<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>
where
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand,
{
    pub fn new() -> PostOffice<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>
    {
        PostOffice {
            clients: HashMap::new()
        }
    }

    pub fn clients(
        &self,
    ) -> Iter<ClientId, Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>> {
        self.clients.iter()
    }

    pub fn clients_mut(
        &mut self,
    ) -> IterMut<ClientId, Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>> {
        self.clients.iter_mut()
    }

    pub fn add_client(&mut self, addr: SocketAddr) -> Option<ClientId> {
        let new_client_id = self.client_count() as u16;
        if !self.client_exists(addr) {
            self.clients.insert(new_client_id, Client::new(addr, new_client_id));
            return Some(new_client_id);
        } else {
            None
        }
    }

    pub fn remove_client(&mut self, client_id: &ClientId) {
        if !self.clients.contains_key(client_id) {
            self.clients.remove(client_id);
        } else {
            panic!("Tried to remove client, but it doesn't exist.");
        }
    }

    pub fn client_exists(&self, addr: SocketAddr) -> bool {
        self.clients.values().any(|v| v.addr() == addr)
    }

    pub fn client_by_addr_mut(
        &mut self,
        addr: &SocketAddr,
    ) -> Option<&mut Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>>
    {
        self.clients
            .iter_mut()
            .find(|(_, client)| client.addr() == *addr)
            .map(|(_, v)| v)
    }

    pub fn client_by_id_mut(
        &mut self,
        id: &ClientId,
    ) -> Option<&mut Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>>
    {
        self.clients.get_mut(id)
    }

    pub fn clients_with_inbox(
        &mut self,
    ) -> Filter<
        IterMut<u16, Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>>,
        fn(
            &(
                &u16,
                &mut Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>,
            ),
        ) -> bool,
    > {
        self.clients_mut().filter(|f| !f.1.postbox().empty_inbox())
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn broadcast(&mut self, message: transport::ServerToClientMessage<ServerToClientMessage>) {
        debug!("Broadcast Message");
        for client in self.clients_mut() {
            let mut message = message.clone();
            let client_highest_seen = client.1
                .command_postbox()
                .highest_seen();

            // Calculate how much command frames the client offset from the server command frame.
            // The client uses this value to adjust his local synchronisation speed.
            if let transport::ServerToClientMessage::StateUpdate(ref mut world_state) = message {
                world_state.command_frame_offset = client_highest_seen as i32 - world_state.command_frame as i32;
            }

            let postbox = client.1.postbox_mut();

            postbox.send(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use crate::{ComponentRecord, SentPacket};

    use super::*;

    #[test]
    fn test_send_with_default_requirements() {
        let mut resource = create_test_resource();

        resource.send(remove_event());

        let packet = &resource.messages[0];

        assert_eq!(resource.messages.len(), 1);
        assert_eq!(packet.urgency(), UrgencyRequirement::OnTick);
    }

    #[test]
    fn test_send_immediate_message() {
        let mut resource = create_test_resource();

        resource.send_immediate(modify_event());

        let packet = &resource.messages[0];

        assert_eq!(resource.messages.len(), 1);
        assert_eq!(packet.urgency(), UrgencyRequirement::Immediate);
    }

    #[test]
    fn test_has_messages() {
        let mut resource = create_test_resource();
        assert_eq!(resource.has_messages(), false);
        resource.send_immediate(modify_event());
        assert_eq!(resource.has_messages(), true);
    }

    #[test]
    fn test_drain_only_immediate_messages() {
        let mut resource = create_test_resource();

        let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
        resource.send_immediate(modify_event());
        resource.send_immediate(modify_event());
        resource.send(remove_event());
        resource.send(remove_event());
        resource.send_immediate(modify_event());

        assert_eq!(resource.drain_messages_to_send(|_| false).len(), 3);
        assert_eq!(resource.drain_messages_to_send(|_| false).len(), 0);
    }

    #[test]
    fn drain_removed_events() {
        let mut buffer = ReceiveBufferResource::default();
        packets().into_iter().for_each(|f| buffer.push(f));

        assert_eq!(buffer.drain_removed().len(), 1);
        assert_eq!(buffer.drain_removed().len(), 0);
    }

    #[test]
    fn drain_inserted_events() {
        let mut buffer = ReceiveBufferResource::default();
        packets().into_iter().for_each(|f| buffer.push(f));

        assert_eq!(buffer.drain_inserted().len(), 2);
        assert_eq!(buffer.drain_inserted().len(), 0);
    }

    #[test]
    fn drain_modified_events() {
        let mut buffer = ReceiveBufferResource::default();
        packets().into_iter().for_each(|f| buffer.push(f));

        // There are three modification events.
        assert_eq!(buffer.drain_modified(Uid(0), Uid(2)).len(), 2);
        assert_eq!(buffer.drain_modified(Uid(0), Uid(1)).len(), 1);

        // Everything should be drained.
        assert_eq!(buffer.drain_modified(Uid(0), Uid(1)).len(), 0);
        assert_eq!(buffer.drain_modified(Uid(0), Uid(2)).len(), 0);
    }

    fn packets() -> Vec<ReceivedPacket> {
        let addr = "127.0.0.1:1234".parse().unwrap();
        let id = Uid(0);

        vec![
            ReceivedPacket::new(addr, SentPacket::new(ClientMessage::EntityRemoved(id))),
            ReceivedPacket::new(
                addr,
                SentPacket::new(ClientMessage::EntityInserted(id, vec![])),
            ),
            ReceivedPacket::new(
                addr,
                SentPacket::new(ClientMessage::EntityInserted(id, vec![])),
            ),
            ReceivedPacket::new(
                addr,
                SentPacket::new(ClientMessage::ComponentModified(
                    id,
                    ComponentRecord::new(1, vec![]),
                )),
            ),
            ReceivedPacket::new(
                addr,
                SentPacket::new(ClientMessage::ComponentModified(
                    id,
                    ComponentRecord::new(2, vec![]),
                )),
            ),
            ReceivedPacket::new(
                addr,
                SentPacket::new(ClientMessage::ComponentModified(
                    id,
                    ComponentRecord::new(2, vec![]),
                )),
            ),
        ]
    }

    fn modify_event() -> ClientMessage {
        ClientMessage::ComponentModified(Uid(0), ComponentRecord::new(0, test_payload().to_vec()))
    }

    fn remove_event() -> ClientMessage {
        ClientMessage::EntityRemoved(Uid(0))
    }

    fn test_payload() -> &'static [u8] {
        b"test"
    }

    fn create_test_resource() -> SentBufferResource {
        SentBufferResource::new()
    }
}
