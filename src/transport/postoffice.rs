use std::{
    collections::{
        hash_map::{Iter, IterMut},
        HashMap,
    },
    iter::Filter,
    net::SocketAddr,
};

use log::debug;

use crate::{
    transport,
    transport::{Client, ClientId, NetworkCommand, NetworkMessage},
};

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
            clients: HashMap::new(),
        }
    }

    pub fn clients(
        &self,
    ) -> Iter<ClientId, Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>>
    {
        self.clients.iter()
    }

    pub fn clients_mut(
        &mut self,
    ) -> IterMut<
        ClientId,
        Client<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>,
    > {
        self.clients.iter_mut()
    }

    pub fn add_client(&mut self, addr: SocketAddr) -> Option<ClientId> {
        let new_client_id = self.client_count() as u16;
        if !self.client_exists(addr) {
            self.clients
                .insert(new_client_id, Client::new(addr, new_client_id));
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
            let client_offset_from_server = client.1.command_postbox().command_frame_offset();

            // Calculate how much command frames the client offset from the server command frame.
            // The client uses this value to adjust his local synchronisation speed.
            if let transport::ServerToClientMessage::StateUpdate(ref mut world_state) = message {
                world_state.command_frame_offset = client_offset_from_server;
            }

            let postbox = client.1.postbox_mut();

            postbox.send(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use crate::{
        state::WorldState,
        transport::{
            Client, ClientId, ClientToServerMessage, PostOffice, ServerToClientMessage,
            ServerToClientMessage::StateUpdate,
        },
    };

    fn test_payload() -> &'static [u8] {
        b"test"
    }

    #[test]
    fn broadcast_should_broadcast() {
        let mut postoffice = PostOffice::<u32, u32, u32>::new();

        postoffice
            .add_client("127.0.0.1:10".parse().unwrap())
            .unwrap();
        postoffice
            .add_client("127.0.0.1:22".parse().unwrap())
            .unwrap();

        postoffice.broadcast(ServerToClientMessage::Message(1));

        let client_count = postoffice.client_count();

        let mut client1 = postoffice.client_by_id_mut(&0).unwrap();
        assert_eq!(client1.postbox_mut().drain_outgoing(|_| true).len(), 1);

        let mut client2 = postoffice.client_by_id_mut(&1).unwrap();
        assert_eq!(client2.postbox_mut().drain_outgoing(|_| true).len(), 1);
    }

    #[test]
    fn broadcast_should_update_offset_in_world_state() {
        let mut postoffice = PostOffice::<u32, u32, u32>::new();

        let client_id_1 = postoffice
            .add_client("127.0.0.1:10".parse().unwrap())
            .unwrap();
        let client_id_2 = postoffice
            .add_client("127.0.0.1:20".parse().unwrap())
            .unwrap();

        {
            let mut client1 = postoffice.client_by_id_mut(&0).unwrap();
            client1.command_postbox.command_frame_offset = 4;

            let mut client2 = postoffice.client_by_id_mut(&1).unwrap();
            client2.command_postbox.command_frame_offset = 6;
        }

        postoffice.broadcast(ServerToClientMessage::StateUpdate(WorldState::new(1)));

        let mut client1 = postoffice.client_by_id_mut(&0).unwrap();
        let client1_outgoing = client1.postbox_mut().drain_outgoing(|a| true);

        let mut client2 = postoffice.client_by_id_mut(&1).unwrap();
        let client_2_outgoing = client2.postbox_mut().drain_outgoing(|a| true);

        match client1_outgoing.first().unwrap() {
            ServerToClientMessage::StateUpdate(WorldState {
                command_frame_offset: 4,
                ..
            }) => assert!(true),
            _ => assert!(false),
        };

        match client_2_outgoing.first().unwrap() {
            ServerToClientMessage::StateUpdate(WorldState {
                command_frame_offset: 6,
                ..
            }) => assert!(true),
            _ => assert!(false),
        };
    }

    #[test]
    fn get_client_by_address_should_return() {
        let mut postoffice = PostOffice::<u32, u32, u32>::new();

        postoffice
            .add_client("127.0.0.1:10".parse().unwrap())
            .unwrap();

        assert!(postoffice
            .client_by_addr_mut(&("127.0.0.1:10".parse::<SocketAddr>().unwrap()))
            .is_some())
    }

    #[test]
    fn inserting_client_increases_client_id() {
        let mut postoffice = PostOffice::<u32, u32, u32>::new();

        assert_eq!(
            postoffice
                .add_client("127.0.0.1:10".parse().unwrap())
                .unwrap(),
            0
        );
        assert_eq!(
            postoffice
                .add_client("127.0.0.1:11".parse().unwrap())
                .unwrap(),
            1
        );
        assert_eq!(
            postoffice
                .add_client("127.0.0.1:12".parse().unwrap())
                .unwrap(),
            2
        );
        assert_eq!(
            postoffice
                .add_client("127.0.0.1:13".parse().unwrap())
                .unwrap(),
            3
        );
    }

    #[test]
    fn add_twice_the_same_client_returns_none() {
        let mut postoffice = PostOffice::<u32, u32, u32>::new();

        assert!(postoffice
            .add_client("127.0.0.1:19".parse().unwrap())
            .is_some());
        assert!(postoffice
            .add_client("127.0.0.1:19".parse().unwrap())
            .is_none());
    }

    #[test]
    fn returns_only_clients_with_inbox() {
        let mut postoffice = PostOffice::<u32, u32, u32>::new();

        let id_1 = postoffice
            .add_client("127.0.0.1:10".parse().unwrap())
            .unwrap();
        let id_2 = postoffice
            .add_client("127.0.0.1:11".parse().unwrap())
            .unwrap();

        let client = postoffice.client_by_id_mut(&id_1).unwrap();
        client.add_received_message(ClientToServerMessage::Message(1), 0);

        let clients = postoffice
            .clients_with_inbox()
            .into_iter()
            .collect::<Vec<(&ClientId, &mut Client<u32, u32, u32>)>>();
        assert_eq!(clients.len(), 1);
    }
}
