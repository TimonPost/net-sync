use std::collections::{HashMap};
use std::net::SocketAddr;
use crate::uid::Uid;
use crate::transport::{PostBox};
use crate::{ClientMessage, ServerMessage, EntityId};
use std::collections::hash_map::{Iter, IterMut};
use std::iter::Filter;

pub type ClientId = u16;

pub struct Client {
    pub generated_entity_ids: HashMap<Uid, Uid>,
    connection_id: ClientId,
    post_box: PostBox<ClientMessage, ServerMessage>
}

impl Client {
    pub fn new(addr: SocketAddr, connection_id: ClientId) -> Self {
        Client {
            generated_entity_ids: HashMap::new(),
            connection_id,
            post_box: PostBox::new(addr)
        }
    }

    pub fn postbox_mut(&mut self) -> &mut PostBox<ClientMessage, ServerMessage> {
        &mut self.post_box
    }

    pub fn postbox(&self) -> &PostBox<ClientMessage, ServerMessage> {
        &self.post_box
    }

    pub fn add_id_mapping(&mut self, client_id: Uid, server_id: Uid) {
        self.generated_entity_ids.insert(client_id, server_id);
    }

    pub fn get_server_entity_id(&self, client_id: Uid) -> Option<Uid> {
        self.generated_entity_ids.iter().find(|(c_id, server_id)| **c_id == client_id).map(|v| *v.1)
    }

    /// The server replaces the client generated entity identifier.
    /// The client will keep using its own identifier until it has processed the
    /// server acknowledgment which contains the new server generated identifier.
    /// Until that time the entity id should be mapped, a check has to be done to figure out if the given identifier was already mapped by the server.
    /// If that is the case we will return the server identifier.
    pub fn is_accepted(&self, entity_id: Uid) -> Option<Uid> {
        self.get_server_entity_id(entity_id)
    }
}

pub struct Clients {
    clients: HashMap<ClientId, Client>
}

impl Clients {
    pub fn new() -> Clients {
        Clients {
            clients: HashMap::new()
        }
    }


    pub fn add(&mut self, addr: SocketAddr, client_id: ClientId) {
        if !self.clients.contains_key(&client_id) {
            self.clients.insert(client_id, Client::new(addr, client_id));
        } else {
            panic!("Tried to add client, but it already exists.");
        }
    }

    pub fn remove(&mut self, client_id: &ClientId) {
        if !self.clients.contains_key(client_id) {
            self.clients.remove(client_id);
        }else {
            panic!("Tried to remove client, but it doesn't exist.");
        }
    }

    pub fn by_addr_mut(&mut self, addr: &SocketAddr) -> Option<&mut Client> {
        self.clients.iter_mut().find(|(_, v)| v.postbox().addr() == *addr).map(|(_, v)| v)
    }

    pub fn by_id_mut(&mut self, id: &ClientId) -> Option<&mut Client> {
        self.clients.get_mut(id)
    }

    pub fn iter(&self) -> Iter<ClientId, Client> {
        self.clients.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<ClientId, Client> {
        self.clients.iter_mut()
    }

    pub fn with_inbox(&mut self) -> Filter<IterMut<u16, Client>, fn(&(&u16, &mut Client)) -> bool> {
      self.iter_mut().filter(|f| !f.1.postbox().empty_inbox())
    }
}

