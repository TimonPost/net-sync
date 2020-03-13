use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use crate::uid::Uid;
use crate::transport::{Message, UrgencyRequirement, PostBox};
use crate::Event;
use std::collections::hash_map::{Iter, IterMut};
use std::iter::Filter;

type ClientId = u16;

pub struct Client {
    generated_entity_ids: HashMap<Uid, Uid>,
    connection_id: ClientId,
    post_box: PostBox
}

impl Client {
    pub fn new(addr: SocketAddr, connection_id: ClientId) -> Self {
        Client {
            generated_entity_ids: HashMap::new(),
            connection_id,
            post_box: PostBox::new(addr)
        }
    }

    pub fn postbox_mut(&mut self) -> &mut PostBox {
        &mut self.post_box
    }

    pub fn postbox(&self) -> &PostBox {
        &self.post_box
    }

    pub fn add_id_mapping(&mut self, client_id: Uid, server_id: Uid) {
        self.generated_entity_ids.insert(client_id, server_id);
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
        }else {
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

